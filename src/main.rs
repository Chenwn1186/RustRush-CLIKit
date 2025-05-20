use chrono::DateTime;
use chrono::Local;
use clap::Parser;
use colored::{Color, Colorize};
use lazy_static::lazy_static;
// use regex::Regex;
use fancy_regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{LinesWithEndings, as_24_bit_terminal_escaped};

/// **功能**：
/// 1. 列出目录内容：支持彩色突出显示不同文件/文件夹类型，支持按大小、修改时间等排序
/// 2. 搜索文件、文件夹和文件内容：支持正则表达式
/// 3. 合并文本文件
/// 4. 打开文本文件并高亮显示前 n 行
/// 5. 批量重命名：支持正则表达式、多种高级模板匹配
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 要列出内容的目录路径，默认为当前目录
    #[arg(default_value = ".")]
    directory: PathBuf,

    /// 显示详细信息
    #[arg(short, long)]
    long: bool,

    /// 显示隐藏文件
    #[arg(short, long)]
    all: bool,

    /// 递归列出子目录内容
    #[arg(short = 'R', long)]
    recursive: bool,

    /// 人类可读的文件大小
    #[arg(short = 'H', long)]
    human_readable: bool,

    /// 按修改时间排序
    #[arg(short, long)]
    time_sort: bool,

    /// 反转排序顺序
    #[arg(short, long)]
    reverse: bool,

    /// 按扩展名字母顺序排序
    #[arg(short = 'X', long)]
    ext_sort: bool,

    /// 按文件大小降序排序
    #[arg(short = 'S', long)]
    size_sort: bool,

    /// 仅显示目录
    #[arg(short = 'd', long)]
    directories_only: bool,

    /// 在文件名后添加类型标识符
    #[arg(short = 'F', long)]
    file_type: bool,

    // /// 显示 inode 信息（Windows 不支持，显示占位符）
    // #[arg(short = 'i', long)]
    // inode: bool,

    /// 高亮显示不同类型文件
    #[arg(short, long, default_value_t = false)]
    color: bool,

    #[command(subcommand)]
    sub: Option<SubCommands>,
}

#[derive(Parser, Debug)]
enum SubCommands {
    /// 打开某个文件并高亮显示前 n 行
    Show {
        /// 文件路径
        file_path: String,
        /// 要显示的行数，默认 10 行
        #[arg(short, long, default_value_t = 10)]
        lines: usize,
    },
    /// 合并若干个文件，并提供选项来输出到文件；提供选项输出n行结果（默认为0，就是不输出）
    Merge {
        /// 要合并的文件路径列表
        file_paths: Vec<String>,

        /// 输出到文件的路径
        #[arg(short, long, default_value = "None")]
        output: Option<String>,

        /// 输出前 n 行结果（默认为0，就是不输出）
        #[arg(short, long, default_value_t = 20)]
        lines: usize,
    },
    /// 搜索文件名或文件内容
    Search {
        /// 要搜索的文件路径列表
        #[arg(short, long, default_value = ".")]
        paths: Vec<String>,
        /// 要搜索的关键字
        keyword: String,
        /// 是否搜索文件内容
        #[arg(short, long, default_value_t = false)]
        search_content: bool,
        /// 搜索模式，默认为普通搜索
        #[arg(short, long, default_value_t = false)]
        regex: bool,
        /// 忽略大小写
        #[arg(short, long, default_value_t = false)]
        ignore_case: bool,
        /// 是否递归搜索
        #[arg(short = 'R', long, default_value_t = false)]
        recursive: bool,
    },
    /// 批量重命名
    Rename {
        /// 要重命名的文件路径列表
        source: String,
        /// 重命名后的文件名
        target: String,
        /// 指定文件夹下面的文件进行重命名
        #[arg(short, long, default_value = ".")]
        directory: String,
        /// 是否开启正则表达式
        #[arg(short, long, default_value_t = false)]
        regex: bool,
        /// 是否开启模式匹配模式，开启后会覆盖正则表达式
        /// 默认通配符:
        /// - {source}: 整个文件名，包含前缀和后缀
        /// - {prefix}: 文件名前缀，比如 "example" 中的 "example"
        /// - {suffix}: 文件名后缀，比如 "example.txt" 中的 "txt"，"abc.c.d"中的"c.d"
        /// - {date_time:%format%}: 文件修改日期, %format%为格式控制字符，默认为%Y-%m-%d %H:%M:%S
        /// - {n:x}: 序号，从x开始，默认（{n}）从0开始；在x前补0如0x，可以控制对齐宽度：0->01，3->03
        /// - {+p}: 将p指定的内容转换成大写，如{+source}->ABC.TXT
        /// - {-p}: 将p指定的内容转换成小写，如{-source}->abc.txt
        /// - {p:y}: 对p指定的内容进行截取，y为截取的长度，如{source:3}->abc；只对{source}、{prefix}、{suffix}有效
        /// - {p:y:z}: 对p指定的内容进行截取，y为截取的长度，z为起始位置，如{source:3:1}->bca；只对{source}、{prefix}、{suffix}有效
        /// - {p:y-z}: 对p指定的内容进行截取，y为起始位置，z为结束位置，如{source:1-3}->bca；只对{source}、{prefix}、{suffix}有效
        /// 
        #[arg(short, long, default_value_t = false,)]
        pattern: bool,
    },
    /// where
    Where {},
}

/// 定义颜色配置结构体
#[derive(Serialize, Deserialize, Debug)]
struct ColorConfig {
    file_ext_colors: Vec<(String, String)>,
    special_dir_colors: Vec<(String, String)>,
}

impl ColorConfig {
    fn new() -> Self {
        ColorConfig {
            file_ext_colors: vec![
                ("rs".to_string(), "#FF00FF".to_string()),     // 品红色
                ("txt".to_string(), "#FFFF00".to_string()),    // 黄色
                ("exe".to_string(), "#00FF00".to_string()),    // 绿色
                ("md".to_string(), "#00FFFF".to_string()),     // 青色
                ("py".to_string(), "#FF4500".to_string()),     // 橙红色
                ("json".to_string(), "#0000FF".to_string()),   // 蓝色
                ("toml".to_string(), "#4B0082".to_string()),   // 靛蓝色
                ("yml".to_string(), "#8A2BE2".to_string()),    // 蓝色紫罗兰色
                ("yaml".to_string(), "#8A2BE2".to_string()),   // 蓝色紫罗兰色
                ("lock".to_string(), "#008080".to_string()),   // 水鸭色
                ("c".to_string(), "#FF69B4".to_string()),      // 深粉色
                ("cpp".to_string(), "#FF69B4".to_string()),    // 深粉色
                ("h".to_string(), "#FF69B4".to_string()),      // 深粉色
                ("hpp".to_string(), "#FF69B4".to_string()),    // 深粉色
                ("cs".to_string(), "#DA70D6".to_string()),     // 淡紫色
                ("java".to_string(), "#FFA500".to_string()),   // 橙色
                ("js".to_string(), "#FFD700".to_string()),     // 金色
                ("ts".to_string(), "#4169E1".to_string()),     // 皇家蓝色
                ("html".to_string(), "#E34C26".to_string()),   // HTML 官方橙色
                ("css".to_string(), "#264DE4".to_string()),    // CSS 官方蓝色
                ("php".to_string(), "#777BB4".to_string()),    // PHP 官方紫色
                ("rb".to_string(), "#CC342D".to_string()),     // Ruby 官方红色
                ("go".to_string(), "#00ADD8".to_string()),     // Go 语言官方蓝色
                ("swift".to_string(), "#F05138".to_string()),  // Swift 官方橙色
                ("kt".to_string(), "#F88900".to_string()),     // Kotlin 官方橙色
                ("kts".to_string(), "#F88900".to_string()),    // Kotlin 官方橙色
                ("sh".to_string(), "#4EAA25".to_string()),     // 绿色
                ("bat".to_string(), "#4EAA25".to_string()),    // 绿色
                ("ps1".to_string(), "#012456".to_string()),    // PowerShell 官方蓝色
                ("psm1".to_string(), "#012456".to_string()),   // PowerShell 官方蓝色
                ("psd1".to_string(), "#012456".to_string()),   // PowerShell 官方蓝色
                ("ps1xml".to_string(), "#012456".to_string()), // PowerShell 官方蓝色
            ],
            special_dir_colors: vec![
                ("target".to_string(), "#0000FF".to_string()),   // 蓝色
                ("src".to_string(), "#00FFFF".to_string()),      // 青色
                ("bin".to_string(), "#00FF00".to_string()),      // 绿色
                ("lib".to_string(), "#FF00FF".to_string()),      // 品红色
                ("include".to_string(), "#FFFF00".to_string()),  // 黄色
                ("docs".to_string(), "#FF4500".to_string()),     // 橙红色
                ("examples".to_string(), "#FF4500".to_string()), // 橙红色
                ("test".to_string(), "#FF6347".to_string()),     // 番茄红色
                ("vendor".to_string(), "#FF6347".to_string()),   // 番茄红色
                ("build".to_string(), "#FF6347".to_string()),    // 番茄红色
                ("out".to_string(), "#FF6347".to_string()),      // 番茄红色
                ("dist".to_string(), "#FF6347".to_string()),     // 番茄红色
                ("node_modules".to_string(), "#3C873A".to_string()), // Node.js 官方绿色
                ("public".to_string(), "#FFA500".to_string()),   // 橙色
                ("assets".to_string(), "#FFA500".to_string()),   // 橙色
                ("styles".to_string(), "#264DE4".to_string()),   // CSS 官方蓝色
                ("scripts".to_string(), "#FFD700".to_string()),  // 金色
                ("images".to_string(), "#87CEEB".to_string()),   // 天蓝色
                ("fonts".to_string(), "#808080".to_string()),    // 灰色
                ("data".to_string(), "#9370DB".to_string()),     // 暗紫色
                ("config".to_string(), "#4B0082".to_string()),   // 靛蓝色
                ("logs".to_string(), "#A52A2A".to_string()),     // 棕色
                ("tmp".to_string(), "#A52A2A".to_string()),      // 棕色
                ("cache".to_string(), "#A52A2A".to_string()),    // 棕色
                ("backup".to_string(), "#A52A2A".to_string()),   // 棕色
                ("old".to_string(), "#A52A2A".to_string()),      // 棕色
                ("temp".to_string(), "#A52A2A".to_string()),     // 棕色
                ("draft".to_string(), "#A52A2A".to_string()),    // 棕色
                ("unfinished".to_string(), "#A52A2A".to_string()), // 棕色
            ],
        }
    }
    fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path
            .parent()
            .ok_or("Failed to get executable directory")?;
        Ok(exe_dir.join("color_config.json"))
    }
    fn load_from_file() -> Self {
        match Self::get_config_path() {
            Ok(config_path) => {
                if let Ok(contents) = fs::read_to_string(&config_path) {
                    if let Ok(config) = serde_json::from_str(&contents) {
                        return config;
                    }
                }
            }
            Err(_) => {}
        }
        Self::new()
    }

    fn save_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path()?;
        let json = serde_json::to_string_pretty(self)?;
        fs::write(config_path, json)?;
        Ok(())
    }

    fn parse_color(color_str: &str) -> Option<Color> {
        // 尝试解析为预定义颜色
        if let Ok(color) = color_str.parse::<Color>() {
            return Some(color);
        }
        // 尝试解析为 RGB 值，格式如 "#RRGGBB"
        if color_str.starts_with('#') && color_str.len() == 7 {
            if let Ok(r) = u8::from_str_radix(&color_str[1..3], 16) {
                if let Ok(g) = u8::from_str_radix(&color_str[3..5], 16) {
                    if let Ok(b) = u8::from_str_radix(&color_str[5..7], 16) {
                        return Some(Color::TrueColor { r, g, b });
                    }
                }
            }
        }
        None
    }

    fn get_file_color(&self, ext: &str) -> Option<Color> {
        self.file_ext_colors
            .iter()
            .find(|(e, _)| e == ext)
            .and_then(|(_, c)| Self::parse_color(c))
    }

    fn get_dir_color(&self, dir_name: &str) -> Option<Color> {
        self.special_dir_colors
            .iter()
            .find(|(d, _)| d == dir_name)
            .and_then(|(_, c)| Self::parse_color(c))
    }
}

/// 将字节数转换为人类可读的格式
fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.1}{}", size, UNITS[unit_index])
}

/// 获取文件的扩展名
fn get_extension(path: &Path) -> String {
    path.extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string()
}

/// 应用颜色高亮
fn apply_color(
    name: &str,
    is_dir: bool,
    ext: &str,
    dir_name: &str,
    color_config: &ColorConfig,
    color_enabled: bool,
) -> String {
    if !color_enabled {
        return name.to_string();
    }

    if is_dir {
        if let Some(color) = color_config.get_dir_color(dir_name) {
            return name.color(color).to_string();
        }
        return name.blue().to_string();
    }

    if let Some(color) = color_config.get_file_color(ext) {
        return name.color(color).to_string();
    }

    name.to_string()
}
/// 计算相对路径
fn get_relative_path(base: &Path, target: &Path) -> PathBuf {
    let base_components: Vec<_> = base.components().collect();
    let target_components: Vec<_> = target.components().collect();

    let mut common_len = 0;
    for (i, (base_comp, target_comp)) in base_components
        .iter()
        .zip(target_components.iter())
        .enumerate()
    {
        if base_comp == target_comp {
            common_len = i + 1;
        } else {
            break;
        }
    }

    let mut relative_path = PathBuf::new();
    for _ in common_len..base_components.len() {
        relative_path.push("..");
    }
    for comp in &target_components[common_len..] {
        relative_path.push(comp);
    }

    if relative_path.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        relative_path
    }
}
/// 过滤 ANSI 转义序列的函数
fn strip_ansi_escapes(s: &str) -> String {
    lazy_static! {
        static ref ANSI_ESCAPE: Regex =
            Regex::new(r"\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])").unwrap();
    }
    ANSI_ESCAPE.replace_all(s, "").to_string()
}

/// 列出目录内容
fn list_directory(args: &Args, path: &Path) {
    let color_config = ColorConfig::load_from_file();
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    if args.directories_only && !path.is_dir() {
        return;
    }

    let mut entries_info: Vec<(String, String, String, bool, String)> = Vec::new();
    // (size, 原始文件名, modified, is_dir, ext)

    if let Ok(entries) = fs::read_dir(path) {
        let mut entries: Vec<fs::DirEntry> = entries.filter_map(Result::ok).collect();

        // 过滤隐藏文件
        if !args.all {
            entries.retain(|entry| {
                if let Some(name) = entry.file_name().to_str() {
                    !name.starts_with('.')
                } else {
                    true
                }
            });
        }

        // 先按是否为目录排序，目录在前，文件在后，再按原有规则排序
        entries.sort_by(|a, b| {
            let a_is_dir = a.path().is_dir();
            let b_is_dir = b.path().is_dir();
            if a_is_dir != b_is_dir {
                return b_is_dir.cmp(&a_is_dir);
            }

            if args.time_sort {
                let a_time = a
                    .metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                let b_time = b
                    .metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                a_time.cmp(&b_time)
            } else if args.ext_sort {
                let a_ext = get_extension(&a.path());
                let b_ext = get_extension(&b.path());
                a_ext.cmp(&b_ext)
            } else if args.size_sort {
                let a_size = a.metadata().ok().map(|m| m.len()).unwrap_or(0);
                let b_size = b.metadata().ok().map(|m| m.len()).unwrap_or(0);
                a_size.cmp(&b_size)
            } else {
                a.file_name().cmp(&b.file_name())
            }
        });

        if args.reverse {
            entries.reverse();
        }

        for entry in entries.iter() {
            let full_path = entry.path();
            let relative_path = get_relative_path(&current_dir, &full_path);
            let file_name = relative_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let ext = get_extension(&relative_path);

            if args.directories_only && !full_path.is_dir() {
                continue;
            }

            if let Ok(metadata) = entry.metadata() {
                let size = if args.human_readable {
                    format_size(metadata.len())
                } else {
                    format!("{} B", metadata.len())
                };
                let modified = metadata.modified().ok().map_or_else(
                    || "unknown".to_string(),
                    |t| {
                        let dt: DateTime<Local> = t.into();
                        // 修改时间格式，显示年份
                        dt.format("%b %d %Y %H:%M").to_string()
                    },
                );

                let is_dir = metadata.is_dir();
                // 删除可执行文件判断逻辑
                entries_info.push((size, file_name, modified, is_dir, ext));
            }
        }

        // 计算各列的最大宽度
        let max_size_width = entries_info
            .iter()
            .map(|(size, _, _, _, _)| size.len())
            .max()
            .unwrap_or(0);
        let max_name_width = entries_info
            .iter()
            .map(|(_, name, _, is_dir, ext)| {
                let prefix = if *is_dir { "- " } else { "* " };
                let colored_name = apply_color(name, *is_dir, ext, name, &color_config, args.color);
                let final_name = format!("{}{}", prefix, colored_name);
                strip_ansi_escapes(&final_name).len()
            })
            .max()
            .unwrap_or(0);

        if args.long {
            // 打印对齐后的信息，修改输出顺序，文件名在最右边，文件大小在中间，文件名左对齐
            for (size, raw_name, modified, is_dir, ext) in entries_info {
                let prefix = if is_dir { "- " } else { "* " };
                let colored_name = apply_color(
                    &raw_name,
                    is_dir,
                    &ext,
                    &raw_name,
                    &color_config,
                    args.color,
                );
                let final_name = format!("{}{}", prefix, colored_name);

                println!(
                    "{} {:>width_size$} {:<width_name$}",
                    modified,
                    size,
                    final_name,
                    width_size = max_size_width,
                    width_name = max_name_width
                );
            }
        } else {
            for (_, raw_name, _, is_dir, ext) in entries_info {
                let prefix = if is_dir { "- " } else { "* " };
                let colored_name = apply_color(
                    &raw_name,
                    is_dir,
                    &ext,
                    &raw_name,
                    &color_config,
                    args.color,
                );
                let final_name = format!("{}{}", prefix, colored_name);
                println!("{}", final_name);
            }
        }

        if args.recursive {
            for entry in entries.iter() {
                let full_path = entry.path();
                if full_path.is_dir() {
                    let relative_path = get_relative_path(&current_dir, &full_path);
                    println!("\n{}:", relative_path.display());
                    list_directory(args, &full_path);
                }
            }
        }
    }

    // 保存配置（如果需要更新配置，可以在这里修改 ColorConfig 后保存）
    if let Err(e) = color_config.save_to_file() {
        eprintln!("Error saving color config: {}", e);
    }
}

/// 根据文件路径打印代码高亮
fn print_highlighted_lines(
    file_path: &str,
    lines_to_show: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    // 读取文件内容
    let code = fs::read_to_string(file_path)?;
    // 获取文件扩展名
    let extension = file_path.split('.').last().ok_or("File has no extension")?;

    // 加载语法和主题
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    // 查找对应语法
    let syntax = ps
        .find_syntax_by_extension(extension)
        .ok_or_else(|| format!("Unsupported file type: {}", extension))?;

    // 创建 HighlightLines 实例
    let mut highlighter = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

    // 按行分割代码
    let lines = LinesWithEndings::from(&code);

    // 遍历前 n 行
    for (i, line) in lines.take(lines_to_show).enumerate() {
        let ranges: Vec<(Style, &str)> = highlighter.highlight_line(line, &ps).unwrap_or_default();
        let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
        print!("{}: {}", i + 1, escaped);
    }

    Ok(())
}

/// 高亮显示匹配的关键字，可选择是否忽略大小写，和使用正则表达式
///
/// # 参数
/// - `line`: 要搜索的文本行
/// - `keyword`: 要匹配的关键字
/// - `ignore_case`: 是否忽略大小写
/// - `ext`: 文件扩展名
/// - `regex`: 是否使用正则表达式匹配
///
/// # 返回值
/// 返回包含高亮关键字的字符串
fn highlight_keyword(
    line: &str,
    keyword: &str,
    ignore_case: bool,
    ext: &str,
    regex: bool,
) -> String {
    // 加载默认的语法集和主题集
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    // 根据文件扩展名查找对应的语法
    let syntax = ps
        .find_syntax_by_extension(ext)
        .unwrap_or_else(|| ps.find_syntax_plain_text());

    // 创建 HighlightLines 实例，使用 base16-ocean.dark 主题
    let mut highlighter = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

    // 对整行进行语法高亮
    let _ranges: Vec<(Style, &str)> = highlighter.highlight_line(line, &ps).unwrap_or_default();

    let mut highlighted = String::new();
    let mut last_index = 0;

    if regex {
        let pattern = if ignore_case {
            Regex::new(&format!("(?i){}", keyword)).unwrap_or_else(|_| Regex::new("").unwrap())
        } else {
            Regex::new(keyword).unwrap_or_else(|_| Regex::new("").unwrap())
        };

        for mat in pattern.find_iter(line) {
            // 使用 expect 方法解包 Result，如果出错则打印错误信息并 panic
            let mat = mat.expect("Failed to get match");
            let start = mat.start();
            let end = mat.end();

            // 处理未匹配部分
            let unmatched_part = &line[last_index..start];
            let unmatched_ranges: Vec<(Style, &str)> = highlighter
                .highlight_line(unmatched_part, &ps)
                .unwrap_or_default();
            highlighted.push_str(&as_24_bit_terminal_escaped(&unmatched_ranges[..], true));

            // 处理匹配部分，添加黄色背景高亮
            let matched_part = &line[start..end];
            highlighted.push_str(&matched_part.on_yellow().to_string());
            last_index = end;
        }
    } else {
        let (search_line, search_keyword) = if ignore_case {
            (line.to_lowercase(), keyword.to_lowercase())
        } else {
            (line.to_string(), keyword.to_string())
        };

        while let Some(start) = search_line[last_index..].find(&search_keyword) {
            let start = last_index + start;
            // 处理未匹配部分
            let unmatched_part = &line[last_index..start];
            let unmatched_ranges: Vec<(Style, &str)> = highlighter
                .highlight_line(unmatched_part, &ps)
                .unwrap_or_default();
            highlighted.push_str(&as_24_bit_terminal_escaped(&unmatched_ranges[..], true));

            // 处理匹配部分，添加黄色背景高亮
            let matched_part = &line[start..start + keyword.len()];
            highlighted.push_str(&matched_part.on_yellow().to_string());
            last_index = start + keyword.len();
        }
    }

    // 添加剩余未匹配的文本
    let remaining_part = &line[last_index..];
    let remaining_ranges: Vec<(Style, &str)> = highlighter
        .highlight_line(remaining_part, &ps)
        .unwrap_or_default();
    highlighted.push_str(&as_24_bit_terminal_escaped(&remaining_ranges[..], true));

    highlighted
}

fn search_file_content(
    file_path: &Path,
    keyword: &str,
    regex: bool,
    ignore_case: bool,
    match_filename: bool,
) -> Result<Vec<(usize, String)>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();

    // 匹配文件名逻辑
    if match_filename {
        let name = file_path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let name_matched = if regex {
            let pattern = if ignore_case {
                Regex::new(&format!("(?i){}", keyword)).unwrap_or_else(|_| Regex::new("").unwrap())
            } else {
                Regex::new(keyword).unwrap_or_else(|_| Regex::new("").unwrap())
            };
            pattern.is_match(name).unwrap_or(false)
        } else {
            let target = if ignore_case {
                keyword.to_lowercase()
            } else {
                keyword.to_string()
            };
            let name_to_search = if ignore_case {
                name.to_lowercase()
            } else {
                name.to_string()
            };
            name_to_search.contains(&target)
        };

        if name_matched {
            let highlighted_name = highlight_keyword(name, keyword, ignore_case, "", regex);
            let prefix = if file_path.is_dir() { "- " } else { "* " };
            results.push((0, format!("{}{}", prefix, highlighted_name)));
        }
    }

    // 匹配文件内容逻辑
    let content = fs::read_to_string(file_path)?;
    if regex {
        let pattern = if ignore_case {
            Regex::new(&format!("(?i){}", keyword))?
        } else {
            Regex::new(keyword)?
        };

        for (line_num, line) in content.lines().enumerate() {
            match pattern.is_match(line) {
                Ok(true) => {
                    let highlighted_line = highlight_keyword(
                        line,
                        keyword,
                        ignore_case,
                        &get_extension(file_path),
                        regex,
                    );
                    results.push((line_num + 1, highlighted_line));
                }
                Ok(false) => {}
                Err(e) => {
                    eprintln!("Error in regex matching: {}", e);
                }
            }
        }
    } else {
        let target = if ignore_case {
            keyword.to_lowercase()
        } else {
            keyword.to_string()
        };

        for (line_num, line) in content.lines().enumerate() {
            let line_to_search = if ignore_case {
                line.to_lowercase()
            } else {
                line.to_string()
            };

            if line_to_search.contains(&target) {
                let highlighted_line =
                    highlight_keyword(line, keyword, ignore_case, &get_extension(file_path), regex);
                results.push((line_num + 1, highlighted_line));
            }
        }
    }

    Ok(results)
}

fn search_in_directory(
    dir_path: &Path,
    keyword: &str,
    search_content: bool,
    regex: bool,
    ignore_case: bool,
    recursive: bool,
) {
    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                let name_matched = if regex {
                    let pattern = if ignore_case {
                        Regex::new(&format!("(?i){}", keyword))
                            .unwrap_or_else(|_| Regex::new("").unwrap())
                    } else {
                        Regex::new(keyword).unwrap_or_else(|_| Regex::new("").unwrap())
                    };
                    pattern.is_match(name).unwrap_or(false)
                } else {
                    let target = if ignore_case {
                        keyword.to_lowercase()
                    } else {
                        keyword.to_string()
                    };
                    let name_to_search = if ignore_case {
                        name.to_lowercase()
                    } else {
                        name.to_string()
                    };
                    name_to_search.contains(&target)
                };

                if name_matched {
                    let highlighted_name = highlight_keyword(name, keyword, ignore_case, "", regex);
                    let prefix = if path.is_dir() { "- " } else { "* " };
                    println!(
                        "{}{}",
                        prefix,
                        path.with_file_name(highlighted_name).display()
                    );
                }

                if search_content && path.is_file() {
                    if let Ok(matches) =
                        search_file_content(&path, keyword, regex, ignore_case, false)
                    {
                        if !matches.is_empty() {
                            if !name_matched {
                                let prefix = if path.is_dir() { "- " } else { "* " };
                                println!("{}{}", prefix, path.display());
                            }
                            for (line_num, line) in matches {
                                println!("{}:{} - {}", path.display(), line_num, line);
                            }
                        }
                    }
                }

                if recursive && path.is_dir() {
                    search_in_directory(
                        &path,
                        keyword,
                        search_content,
                        regex,
                        ignore_case,
                        recursive,
                    );
                }
            }
        }
    }
}

// ** D:\docs\rust\bhw\big_homework\target\debug\rt.exe **

fn main() {
    let args = Args::parse();
    match args.sub {
        Some(SubCommands::Search {
            paths,
            keyword,
            search_content,
            regex,
            ignore_case,
            recursive,
        }) => {
            println!(
                "Searching for '{}' in {}. regex: {}",
                keyword,
                paths.join(", "),
                regex
            );
            let paths_to_search = paths;
            for p in paths_to_search {
                let path = Path::new(&p);
                if path.is_file() {
                    if let Ok(matches) =
                        search_file_content(path, &keyword, regex, ignore_case, true)
                    {
                        for (line_num, line) in matches {
                            println!("{}:{} - {}", path.display(), line_num, line);
                        }
                    }
                } else if path.is_dir() {
                    search_in_directory(
                        path,
                        &keyword,
                        search_content,
                        regex,
                        ignore_case,
                        recursive,
                    );
                }
            }
        }
        Some(SubCommands::Show { file_path, lines }) => {
            if let Err(e) = print_highlighted_lines(&file_path, lines) {
                eprintln!("Error: {}", e);
            }
        }
        Some(SubCommands::Merge {
            file_paths,
            output,
            lines,
        }) => {
            let mut content = String::new();
            for file_path in file_paths {
                if let Ok(file_content) = fs::read_to_string(&file_path) {
                    content.push_str(&file_content);
                    content.push('\n');
                }
            }
            if let Some(output_path) = output {
                if let Err(e) = fs::write(&output_path, content) {
                    eprintln!("Error writing to file: {}", e);
                } else if lines > 0 {
                    if let Err(e) = print_highlighted_lines(&output_path, lines) {
                        eprintln!("Error: {}", e);
                    }
                }
            }
        }
        Some(SubCommands::Rename {
            source,
            target,
            directory,
            regex,
            pattern
        }) => {
            println!("Renaming '{}' to '{}'...", source, target);
        }
        Some(SubCommands::Where {}) => {
            let current_dir = std::env::current_dir().unwrap();
            let path_str = current_dir.to_str().unwrap();
            println!("{}", path_str);
        }
        None => {
            let path = args
                .directory
                .canonicalize()
                .unwrap_or_else(|_| PathBuf::from("."));
            // 手动去除 \\?\ 前缀
            let path_str = path.to_str().unwrap_or("");
            let clean_path = if path_str.starts_with(r"\\?\") {
                PathBuf::from(&path_str[4..])
            } else {
                path
            };
            list_directory(&args, &clean_path);
        }
    }
}
