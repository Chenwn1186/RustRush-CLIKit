use anyhow::Result;
use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use fancy_regex::Regex;
// use std::fmt::format;
use std::path::Path;
use std::time::SystemTime;
use std::{fs};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{LinesWithEndings, as_24_bit_terminal_escaped};
/// 获取文件的扩展名
pub fn get_extension(path: &Path) -> String {
    path.extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string()
}

pub fn print_highlighted_lines(file_path: &str, lines_to_show: usize) -> Result<()> {
    // 读取文件内容
    let code = fs::read_to_string(file_path)?;
    // 获取文件扩展名
    let extension = file_path
        .split('.')
        .last()
        .ok_or("File has no extension")
        .unwrap_or("");

    // 加载语法和主题
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    // 查找对应语法
    let syntax = ps
        .find_syntax_by_extension(extension)
        .ok_or_else(|| format!("Unsupported file type: {}", extension));

    match syntax {
        Ok(s) => {
            // 创建 HighlightLines 实例
            let mut highlighter = HighlightLines::new(s, &ts.themes["base16-ocean.dark"]);
            // 按行分割代码
            let lines = LinesWithEndings::from(&code);
            // 遍历前 n 行
            for (i, line) in lines.take(lines_to_show).enumerate() {
                let ranges: Vec<(Style, &str)> =
                    highlighter.highlight_line(line, &ps).unwrap_or_default();
                let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
                print!("{}: {}", i + 1, escaped);
            }
        }
        Err(_) => {
            // 直接输出原始内容
            println!("{}", code);
        }
    }
    Ok(())
}

/// 文件大小解析器
/// 筛选条件：文件大小
/// 格式：xx-yyZ、xxZ、-yyZ（需要用双引号包含）
/// xx: 起始大小，数字
/// yy: 结束大小，数字
/// Z: 可以是k、m、g、t、p，表示KB、MB、GB、TB、PB
/// 例如：100k-200m表示100KB到200MB之间的文件
/// 范围可以叠加，用逗号分隔，例如：100k-200m,10g表示100KB到200MB之间的文件，或者10GB以上的文件
fn parse_size_condition(size_str: &str) -> Result<Vec<(Option<u64>, Option<u64>)>> {
    let mut conditions = vec![];
    // 按逗号分割条件
    for condition in size_str.split(',') {
        let trimmed = condition.trim();
        if trimmed.is_empty() {
            continue;
        }

        // 处理三种格式
        if let Some((min_str, max_str)) = trimmed.split_once('-') {
            // 格式: xx-yyZ
            conditions.push((
                parse_size_unit(min_str.trim())?,
                parse_size_unit(max_str.trim())?,
            ));
        } else if trimmed.starts_with('-') {
            // 格式: -yyZ
            conditions.push((None, parse_size_unit(&trimmed[1..].trim())?));
        } else {
            // 格式: xxZ
            conditions.push((parse_size_unit(trimmed)?, None));
        }
    }

    Ok(conditions)
}

/// 解析带单位的文件大小字符串
/// 示例: "100k" -> Some(102400)
/// 支持单位: k(KB), m(MB), g(GB), t(TB), p(PB)
fn parse_size_unit(size_str: &str) -> Result<Option<u64>> {
    if size_str.is_empty() {
        return Ok(None);
    }

    let re = Regex::new(r"^(\d+)([kmgtp]?)$")?;
    let caps = re
        .captures(size_str)
        .or_else(|_| Err(anyhow::anyhow!("Invalid size format: {}", size_str)))?;

    let num = caps
        .as_ref()
        .and_then(|c| c.get(1))
        .unwrap()
        .as_str()
        .parse::<u64>()?;
    let unit = caps
        .as_ref()
        .and_then(|c| c.get(2))
        .map(|m| m.as_str())
        .unwrap_or("");

    let multiplier = match unit.to_lowercase().as_str() {
        "k" => 1024,
        "m" => 1024 * 1024,
        "g" => 1024 * 1024 * 1024,
        "t" => 1024 * 1024 * 1024 * 1024,
        "p" => 1024 * 1024 * 1024 * 1024 * 1024,
        _ => 1, // 无单位默认为字节
    };

    Ok(Some(num * multiplier))
}

/// 最终检查函数
pub fn check_size_condition(path: &Path, size_str: &str) -> Result<bool> {
    let conditions = parse_size_condition(size_str)?;
    let file_size = path.metadata()?.len();

    Ok(conditions.iter().any(|(min, max)| {
        let min_val = min.unwrap_or(0);
        let max_val = max.unwrap_or(u64::MAX);
        file_size >= min_val && file_size <= max_val
    }))
}

/// 筛选条件：文件修改时间
/// 格式：xx:yyZ、xxZ:、:yyZ、xxZ、special_datetime
/// xx: 起始时间，数字
/// yy: 结束时间，数字
/// Z: 可以是y、m、d、h、M、s，表示年、月、日、时、分、秒
/// xx:yyZ 表示在Z的单位内，从xx开始到yy结束的时间范围
/// xxZ: 表示从xx开始到当前时间的时间范围
/// :yyZ 表示yyZ及往前的所有时间范围
/// xxZ 表示xxZ表示的时间范围，时间跨度与Z的单位相同
/// special_datetime: 特殊时间，如today、yesterday、this_month、last_month、this_year、last_year
/// 范围可以用逗号分隔以取并集，例如：2021:2022y,10m表示在2021年到2022年或者在10月份
/// 单个时间范围内的不同时间单位用“-”分隔，例如：2021y-7:8m-10:20d-:10h表示在2021年7月或8月的10日到20日，并且在00:00到10:00之间的时间范围
/// 可以用括号来约定时间点，例如: (2021y-7m-10d-0h):(2021y-8m-20d-10h)表示在2021年7月10日00:00到2021年8月20日10:00之间的时间范围
pub fn check_datetime_condition(path: &Path, datetime_str: &str, datetime_type: &str) -> Result<bool> {
    let datetime_str = datetime_str.trim();
    if datetime_str.is_empty() {
        return Ok(true);
    }
    let intersection = datetime_str.split(',').collect::<Vec<&str>>();
    for datetime in intersection {
        let datetime = datetime.trim();
        if datetime.is_empty() {
            continue;
        }
        let time_range = TimeRange::parser(datetime)?;
        return Ok(time_range.check_datatime(path, datetime_type));
    }
    Err(anyhow::anyhow!("Invalid datetime format: {}", datetime_str))
}

struct TimeRange {
    year_start: Option<u32>,
    year_end: Option<u32>,
    month_start: Option<u32>, //m
    month_end: Option<u32>,
    day_start: Option<u32>,
    day_end: Option<u32>,
    hour_start: Option<u32>,
    hour_end: Option<u32>,
    minute_start: Option<u32>, //M
    minute_end: Option<u32>,
    second_start: Option<u32>,
    second_end: Option<u32>,
    millisecond_start: Option<u32>, //ms
    millisecond_end: Option<u32>,
    nanosecond_start: Option<u32>,
    nanosecond_end: Option<u32>,
    fragment: bool,
}

impl TimeRange {
    fn new() -> Self {
        TimeRange {
            year_start: None,
            year_end: None,
            month_start: None,
            month_end: None,
            day_start: None,
            day_end: None,
            hour_start: None,
            hour_end: None,
            minute_start: None,
            minute_end: None,
            second_start: None,
            second_end: None,
            millisecond_start: None,
            millisecond_end: None,
            nanosecond_start: None,
            nanosecond_end: None,
            fragment: true,
        }
    }
    fn parser(time_str: &str) -> Result<Self> {
        let intersection = time_str.split('-').collect::<Vec<&str>>();
        let mut time_range = TimeRange::new();
        for time in intersection {
            let time = time.trim();
            if time.is_empty() {
                continue;
            }
            match time_range.parser_unit(time) {
                Ok(true) => {}
                Ok(false) => {}
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(time_range)
    }

    fn set_datatime_unit(&mut self, num: &str, unit: &str, start: bool) {
        let num = num.parse::<u32>().unwrap_or(0);
        if unit == "y" {
            if start {
                self.year_start = Some(num);
            } else {
                self.year_end = Some(num);
            }
        } else if unit == "m" {
            if start {
                self.month_start = Some(num);
            } else {
                self.month_end = Some(num);
            }
        } else if unit == "d" {
            if start {
                self.day_start = Some(num);
            } else {
                self.day_end = Some(num);
            }
        } else if unit == "h" {
            if start {
                self.hour_start = Some(num);
            } else {
                self.hour_end = Some(num);
            }
        } else if unit == "M" {
            if start {
                self.minute_start = Some(num);
            } else {
                self.minute_end = Some(num);
            }
        } else if unit == "s" {
            if start {
                self.second_start = Some(num);
            } else {
                self.second_end = Some(num);
            }
        } else if unit == "ms" {
            if start {
                self.millisecond_start = Some(num);
            } else {
                self.millisecond_end = Some(num);
            }
        } else if unit == "ns" {
            if start {
                self.nanosecond_start = Some(num);
            } else {
                self.nanosecond_end = Some(num);
            }
        }
    }

    fn parser_unit(&mut self, time_str: &str) -> Result<bool> {
        // 2025y:
        let re1 =
            Regex::new(r"^(\d+)([a-zA-Z]{1,2}):$").unwrap_or_else(|_| Regex::new("").unwrap());
        // :2025y
        let re2 =
            Regex::new(r"^:(\d+)([a-zA-Z]{1,2})$").unwrap_or_else(|_| Regex::new("").unwrap());
        // 2022:2025y
        let re3 =
            Regex::new(r"^(\d+):(\d+)([a-zA-Z]{1,2})$").unwrap_or_else(|_| Regex::new("").unwrap());
        // (a):(b)
        let re4 = Regex::new(r"^\((.+)\):\((.+)\)$").unwrap_or_else(|_| Regex::new("").unwrap());
        // 2025y
        let re5 = Regex::new(r"^(\d+)([a-zA-Z]{1,2})$").unwrap_or_else(|_| Regex::new("").unwrap());
        match re1.captures(time_str) {
            Ok(caps) => match caps {
                Some(caps) => {
                    let num = caps.get(1).unwrap().as_str();
                    let unit = caps.get(2).unwrap().as_str();
                    self.set_datatime_unit(num, unit, true);
                    return Ok(true);
                }
                None => {}
            },
            Err(_) => {}
        }
        match re2.captures(time_str) {
            Ok(caps) => match caps {
                Some(caps) => {
                    let num = caps.get(1).unwrap().as_str();
                    let unit = caps.get(2).unwrap().as_str();
                    self.set_datatime_unit(num, unit, false);
                    return Ok(true);
                }
                None => {}
            },
            Err(_) => {}
        }
        match re3.captures(time_str) {
            Ok(caps) => match caps {
                Some(caps) => {
                    let num1 = caps.get(1).unwrap().as_str();
                    let num2 = caps.get(3).unwrap().as_str();
                    let unit = caps.get(4).unwrap().as_str();
                    self.set_datatime_unit(num1, unit, true);
                    self.set_datatime_unit(num2, unit, false);
                    return Ok(true);
                }
                None => {}
            },
            Err(_) => {}
        }
        match re4.captures(time_str) {
            Ok(caps) => match caps {
                Some(caps) => {
                    let time1 = caps.get(1).unwrap().as_str();
                    let time2 = caps.get(2).unwrap().as_str();
                    let time1_split = time1.split('-').collect::<Vec<&str>>();
                    let time2_split = time2.split('-').collect::<Vec<&str>>();
                    for time in time1_split {
                        let time_with_colon = format!("{}:", time); // 在time1 split后添加冒号
                        let time_range = self.parser_unit(time_with_colon.as_str());
                        match time_range {
                            Ok(_) => {}
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    }
                    for time in time2_split {
                        let time_with_colon = format!(":{}", time);
                        let time_range = self.parser_unit(time_with_colon.as_str());
                        match time_range {
                            Ok(_) => {}
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    }
                    self.fragment = false;
                    return Ok(true);
                }
                None => {}
            },
            Err(_) => {}
        }
        match re5.captures(time_str) {
            Ok(caps) => match caps {
                Some(caps) => {
                    let num = caps.get(1).unwrap().as_str();
                    let unit = caps.get(2).unwrap().as_str();
                    self.set_datatime_unit(num, unit, true);
                    self.set_datatime_unit(num, unit, false);
                }
                None => {}
            },
            Err(_) => {}
        }
        Err(anyhow::Error::msg("Invalid time format"))
    }
    /// 检查Path指定的File的指定时间是否在范围内
    /// # 参数
    /// * `path` - File的路径
    /// * `time_type` - 时间类型，可选值为：mtime、ctime、atime
    /// # 返回值
    /// * `bool` - 如果在范围内返回true，否则返回false
    /// # 示例
    /// ```
    fn check_datatime(&self, path: &Path, datetime_type: &str) -> bool {
        let metadata = path.metadata().unwrap();
        let modified = metadata.modified();
        let created = metadata.created();
        let accessed = metadata.accessed();
        let mut time = SystemTime::now();
        if datetime_type == "mtime" {
            time = modified.unwrap();
        }
        if datetime_type == "ctime" {
            time = created.unwrap();
        }
        if datetime_type == "atime" {
            time = accessed.unwrap();
        }
        let flex_time = chrono::DateTime::<Utc>::from(time);
        let year = flex_time.year() as u32;
        let month = flex_time.month();
        let day = flex_time.day();
        let hour = flex_time.hour();
        let minute = flex_time.minute();
        let second = flex_time.second();
        let millisecond = flex_time.timestamp_millis();
        let nanosecond = flex_time.timestamp_nanos_opt().unwrap_or(0);
        if !self.fragment {
            // 整体时间范围判断
            let (start_time, end_time) = self.to_datetime();
            return flex_time >= start_time && flex_time <= end_time;
        } else {
            // 分别判断每个时间字段
            return self.cmp_datetime(&self.year_start, &self.year_end, year)
                && self.cmp_datetime(&self.month_start, &self.month_end, month)
                && self.cmp_datetime(&self.day_start, &self.day_end, day)
                && self.cmp_datetime(&self.hour_start, &self.hour_end, hour)
                && self.cmp_datetime(&self.minute_start, &self.minute_end, minute)
                && self.cmp_datetime(&self.second_start, &self.second_end, second)
                && self.cmp_datetime(
                    &self.millisecond_start,
                    &self.millisecond_end,
                    millisecond.try_into().unwrap(),
                )
                && self.cmp_datetime(
                    &self.nanosecond_start,
                    &self.nanosecond_end,
                    nanosecond.try_into().unwrap(),
                );
        }
    }

    fn cmp_datetime(&self, start: &Option<u32>, end: &Option<u32>, value: u32) -> bool {
        let start = start.unwrap_or(u32::MIN);
        let end = end.unwrap_or(u32::MAX);
        value >= start && value <= end
    }
    fn to_datetime(&self) -> (DateTime<Utc>, DateTime<Utc>) {
        let start = chrono::Utc
            .with_ymd_and_hms(
                self.year_start.unwrap_or(1970) as i32,
                self.month_start.unwrap_or(1),
                self.day_start.unwrap_or(1),
                self.hour_start.unwrap_or(0),
                self.minute_start.unwrap_or(0),
                self.second_start.unwrap_or(0),
            )
            .unwrap();

        let end = chrono::Utc
            .with_ymd_and_hms(
                self.year_end.unwrap_or(9999) as i32,
                self.month_end.unwrap_or(12),
                self.day_end.unwrap_or(31),
                self.hour_end.unwrap_or(23),
                self.minute_end.unwrap_or(59),
                self.second_end.unwrap_or(59),
            )
            .unwrap();
        (start, end)
    }
}

/// 检查文件是否匹配指定类型
/// # 参数
/// * `path` - 文件路径
/// * `file_type` - 文件类型字符串，支持逗号分隔和!取反
/// # 返回值
/// * `Result<bool>` - Ok：是否匹配，Err：path错误
pub fn check_file_type(path: &Path, file_type: &str) -> Result<bool> {
    // 检查路径是否存在
    if !path.exists() {
        return Err(anyhow::Error::msg(format!(
            "Path does not exist: {}",
            path.display()
        )));
    }

    if path.is_dir() {
        return Ok(false);
    }

    let extension = match path.extension() {
        Some(ext) => ext.to_str().ok_or_else(|| {
            anyhow::Error::msg(format!(
                "Failed to convert extension to string for path: {}",
                path.display()
            ))
        })?,
        None => return Ok(false), // 无扩展名的文件默认不匹配任何类型
    }
    .to_lowercase();

    let mut is_match = false;
    let mut exclude_match = false;
    let mut is_executable_by_extension = false;
    let mut is_executable_by_metadata = false;

    // 定义文件类型与扩展名的映射
    let type_mapping: std::collections::HashMap<&str, Vec<&str>> = [
        (
            "text",
            vec![
                "txt", "md", "log", "conf", "ini", "json", "yaml", "toml", "csv", "xml", "html",
                "htm", "css", "js", "ts", "php", "py", "rb", "java", "c", "cpp", "h", "hpp", "go",
                "rs", "swift", "kt", "scala", "sh", "bash", "zsh", "ps1", "bat", "cmd", "pl",
                "lua", "sql", "r", "m", "hs", "erl", "ex", "exs",
            ],
        ),
        (
            "image",
            vec![
                "jpg", "jpeg", "png", "gif", "bmp", "svg", "webp", "tiff", "tif", "ico", "heic",
                "heif", "psd", "ai", "eps", "raw", "cr2", "nef", "orf", "sr2",
            ],
        ),
        (
            "audio",
            vec![
                "mp3", "wav", "ogg", "flac", "aac", "m4a", "wma", "aiff", "ape", "opus", "mid",
                "midi", "amr", "au", "ra", "rm", "voc", "weba",
            ],
        ),
        (
            "video",
            vec![
                "mp4", "avi", "mov", "mkv", "flv", "webm", "wmv", "mpeg", "mpg", "3gp", "m4v",
                "ogv", "ts", "m2ts", "mts", "vob", "asf", "rmvb", "divx", "f4v",
            ],
        ),
        (
            "document",
            vec![
                "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "odt", "ods", "odp", "rtf",
                "tex", "pages", "numbers", "key", "epub", "mobi", "azw", "fb2", "djvu",
            ],
        ),
        (
            "archive",
            vec![
                "zip", "tar", "gz", "rar", "7z", "bz2", "xz", "lz", "lzma", "cab", "deb", "rpm",
                "pkg", "iso", "dmg", "jar", "war", "ear", "apk", "ipa", "msi",
            ],
        ),
        (
            "executable",
            vec![
                "exe", "bin", "sh", "bat", "app", "com", "msi", "cmd", "ps1", "vbs", "js", "jar",
                "py", "rb", "pl", "php", "out", "elf", "so", "dll", "dylib",
            ],
        ),
        (
            "font",
            vec![
                "ttf", "otf", "woff", "woff2", "eot", "svg", "pfb", "afm", "pfm", "dfont",
            ],
        ),
        (
            "library",
            vec![
                "dll", "so", "a", "lib", "dylib", "jar", "whl", "gem", "npm", "deb", "rpm", "pkg",
                "msi", "apk", "ipa", "appx", "xap", "cab",
            ],
        ),
        (
            "database",
            vec![
                "db", "sqlite", "mdb", "sql", "accdb", "frm", "myd", "myi", "ibd", "dbf", "mdf",
                "ldf", "ndf", "bak", "dump", "archive", "wal", "journal",
            ],
        ),
        (
            "3d_model",
            vec![
                "obj", "fbx", "stl", "gltf", "glb", "dae", "3ds", "blend", "max", "ma", "mb",
                "lwo", "lws", "ply", "x3d", "wrl", "vrml", "b3d", "ase", "iff",
            ],
        ),
        (
            "virtual_box",
            vec![
                "vdi", "vmdk", "ova", "qcow2", "vhd", "vhdx", "hdd", "img", "iso", "vfd", "nvram",
                "vbox", "vmx", "vmxf", "vmsd", "vmtm", "vmss", "nvram",
            ],
        ),
        (
            "dump",
            vec![
                "dmp",
                "core",
                "hprof",
                "heapdump",
                "minidump",
                "crashdump",
                "memorydump",
                "stackdump",
                "threaddump",
                "javacore",
                "phd",
                "snapshot",
                "bin",
                "trc",
            ],
        ),
        (
            "config",
            vec![
                "cfg",
                "config",
                "properties",
                "yml",
                "yaml",
                "env",
                "dotenv",
                "rc",
                "profile",
                "bashrc",
                "bash_profile",
                "zshrc",
                "vimrc",
                "gitconfig",
                "npmrc",
                "dockerfile",
                "dockerignore",
                "gitignore",
                "editorconfig",
            ],
        ),
        (
            "backup",
            vec![
                "bak",
                "backup",
                "old",
                "tmp",
                "temp",
                "swp",
                "swo",
                "swn",
                "save",
                "autosave",
                "~",
                "orig",
                "rej",
                "part",
                "crdownload",
                "download",
            ],
        ),
    ]
    .iter()
    .cloned()
    .collect();
    
    // 处理每个类型条件
    for condition in file_type.split(',') {
        let condition = condition.trim();
        if condition.is_empty() {
            continue;
        }

        let (negate, type_str) = if condition.starts_with('!') {
            (true, &condition[1..])
        } else {
            (false, condition)
        };

        if type_str == "executable" {
            // 检查扩展名
            if let Some(extensions) = type_mapping.get("executable") {
                if extensions.contains(&extension.as_str()) {
                    is_executable_by_extension = true;
                }
            }

            // 检查文件元数据执行权限 (Linux/Unix系统)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(metadata) = path.metadata() {
                    let mode = metadata.permissions().mode();
                    is_executable_by_metadata = mode & 0o111 != 0; // 检查任何执行位
                }
            }

            let is_exec = is_executable_by_extension || is_executable_by_metadata;
            if is_exec {
                if negate {
                    exclude_match = true;
                } else {
                    is_match = true;
                }
            }
        } else if let Some(extensions) = type_mapping.get(type_str) {
            if extensions.contains(&extension.as_str()) {
                if negate {
                    exclude_match = true;
                } else {
                    is_match = true;
                }
            }
        } else {
            return Err(anyhow::Error::msg(format!(
                "Unsupported file type: {}",
                type_str
            )));
        }
    }

    Ok(is_match && !exclude_match)
}


/// 检查文件权限是否匹配
/// # 参数
/// * `path` - 文件路径
/// * `permission_str` - 权限字符串，格式为rwxrwxrwx
/// # 返回值
/// * `Result<bool>` - 是否匹配
pub fn check_permission(path: &Path, permission_str: &str) -> Result<bool> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = path.metadata()?;
        let mode = metadata.permissions().mode();
        
        // 将权限字符串转换为9位二进制掩码
        let mut expected = 0;
        for (i, c) in permission_str.chars().enumerate().take(9) {
            match c {
                'r' if i % 3 == 0 => expected |= 0o400 >> (i / 3 * 3),
                'w' if i % 3 == 1 => expected |= 0o200 >> (i / 3 * 3),
                'x' if i % 3 == 2 => expected |= 0o100 >> (i / 3 * 3),
                '-' => {},
                _ => return Err(anyhow::Error::msg("Invalid permission format")),
            }
        }
        
        Ok((mode & 0o777) == expected)
    }
    #[cfg(not(unix))]
    {
        Ok(true) // 非Unix系统默认返回true
    }
}

/// 检查文件所有者是否匹配
/// # 参数
/// * `path` - 文件路径
/// * `owner_str` - 所有者字符串，可以是用户名或UID
/// # 返回值
/// * `Result<bool>` - 是否匹配
pub fn check_owner(path: &Path, owner_str: &str) -> Result<bool> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let metadata = path.metadata()?;
        let uid = metadata.uid();
        
        // 先尝试匹配用户名
        if let Some(user) = users::get_user_by_name(owner_str) {
            return Ok(user.uid() == uid);
        }
        
        // 再尝试匹配UID
        if let Ok(uid_num) = owner_str.parse::<u32>() {
            return Ok(uid == uid_num);
        }
        
        Err(anyhow::Error::msg("Invalid owner format"))
    }
    #[cfg(not(unix))]
    {
        Ok(true) // 非Unix系统默认返回true
    }
}

/// 检查文件所属组是否匹配
/// # 参数
/// * `path` - 文件路径
/// * `group_str` - 组字符串，可以是组名或GID
/// # 返回值
/// * `Result<bool>` - 是否匹配
pub fn check_group(path: &Path, group_str: &str) -> Result<bool> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let metadata = path.metadata()?;
        let gid = metadata.gid();
        
        // 先尝试匹配组名
        if let Some(group) = users::get_group_by_name(group_str) {
            return Ok(group.gid() == gid);
        }
        
        // 再尝试匹配GID
        if let Ok(gid_num) = group_str.parse::<u32>() {
            return Ok(gid == gid_num);
        }
        
        Err(anyhow::Error::msg("Invalid group format"))
    }
    #[cfg(not(unix))]
    {
        Ok(true) // 非Unix系统默认返回true
    }
}