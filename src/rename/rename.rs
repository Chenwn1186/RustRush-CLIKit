use anyhow::{Ok, Result, anyhow};
use fancy_regex::Regex;
use id3::{Tag, TagLike};
use nom_exif;
use rand::Rng;
use rand::distr::Alphanumeric;
use std::collections::HashMap;
use std::path::Path;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::default::get_probe;

/// # 参数
/// - ` source`: 原始名称
/// - `target`: 目标名称
/// - `directory`: 目录
/// - `regex`: 是否使用正则表达式
/// - `pattern`: 是否使用模板
/// - `wildcard`: 是否使用通配符
pub fn rename_command(
    source: String,
    target: String,
    directory: String,
    regex: bool,
    pattern: bool,
    wildcard: bool,
) {
    // println!("Renaming '{}' to '{}'...", source, target);
    // 处理流程：
    // 将变量存储到列表中
    // 将source中的变量替换成组，作为最终的正则表达式
    // 匹配字符串并将组赋值到变量中
    // 将target中的字符串分割成变量和字符的列表
    // 将列表中的变量替换成具体的值，这个值如果不在组里，那就尝试从通配符或者元数据中获取l；并且处理特殊字符
    // 将字符和变量拼接成字符串
}

/// 处理最终的批量重命名过程
/// # 参数
/// - `paths`: 路径
/// - `value_map`: Vec<HashMap<String, String>>, 包含模板变量和值的映射
/// - `target`
/// - `wildcard`: bool, 是否使用通配符
/// # 返回值
/// 返回操作是否成功
pub fn rename_batch(
    paths: Vec<String>,
    value_map: Vec<HashMap<String, String>>,
    target: String,
    wildcard: bool,
) -> Result<bool> {
    let mut final_paths: Vec<String> = Vec::new();
    let mut target_parser: Vec<String> = Vec::new();
    let mut start = 0;
    let mut is_pattern = false;
    for (i, a) in target.as_str().chars().enumerate() {
        if a == '{' {
            let is_start = if i > 0 {
                target.as_str().chars().nth(i - 1)
            } else {
                None
            };
            if is_start != Some('\\') {
                if is_pattern {
                    return Err(anyhow!("Invalid target: {}, error \"{{\"", target));
                } else {
                    if start < i {
                        target_parser.push(target.as_str()[start..i].to_string());
                    }
                    start = i;
                    is_pattern = true;
                }
            }
        }
        if a == '}' {
            let is_end = if i > 0 {
                target.as_str().chars().nth(i - 1)
            } else {
                None
            };
            if is_end != Some('\\') {
                if !is_pattern {
                    return Err(anyhow!("Invalid target: {}, error \"}}\"", target));
                } else {
                    target_parser.push(target.as_str()[start..i + 1].to_string());
                    is_pattern = false;
                    start = i + 1;
                }
            }
        }
    }
    // 添加剩余部分
    if start < target.len() {
        target_parser.push(target.as_str()[start..].to_string());
    }
    Err(anyhow!("Invalid target: {}", target))
}

/// 使用命名捕获组批量处理扩展正则表达式
/// # 参数
/// - `inputs`: 待匹配的字符串列表
/// - `ext_regex`: 扩展正则表达式，包含 {varname} 标记
/// # 返回值
/// 包含捕获键值对的 HashMap 列表，每个元素对应一个输入
pub fn extract_named_groups(
    inputs: &Vec<String>,
    ext_regex: &String,
) -> Option<Vec<HashMap<String, String>>> {
    // 预编译正则表达式（只执行一次）
    let (re, var_names) = {
        let re_var_name = Regex::new(r"\{([a-zA-Z]+)\}").unwrap();
        let mut var_names = Vec::new();

        // 构建最终正则表达式
        let final_regex = re_var_name
            .replace_all(ext_regex.as_str(), |caps: &fancy_regex::Captures| {
                let var_name = caps.get(1).unwrap().as_str();
                var_names.push(var_name.to_string());
                format!("(?P<{}>.*)", var_name)
            })
            .to_string();

        // 编译正则表达式
        let re = Regex::new(&final_regex).ok()?;
        (re, var_names)
    };

    // 批量处理输入
    let mut results = Vec::with_capacity(inputs.len());
    for input in inputs {
        let mut groups = HashMap::new();
        if let std::result::Result::Ok(Some(caps)) = re.captures(input.as_str()) {
            for name in &var_names {
                if let Some(value) = caps.name(name) {
                    groups.insert(name.clone(), value.as_str().to_string());
                }
            }
        }
        results.push(groups);
    }

    Some(results)
}

/// 将通配符转换成目标字符串
/// # 参数
/// - ` paths`: 路径字符串的列表，表示所有要重命名的文件路径
/// - `source`: 源字符串
/// # 返回值
/// 返回转换后的字符串
/// 默认通配符:
/// - {source}: 整个文件名，包含前缀和后缀
/// - {prefix}: 文件名前缀，比如 "example.txt" 中的 "example"
/// - {suffix}: 文件名后缀，比如 "example.txt" 中的 "txt"，"abc.c.d"中的"c.d"
/// - { n }: 序号，从0开始，如0, 1, 2...
///     - {n:start=1}: 起始值为1, 如1, 2...**默认起始值为1**
///     - {n:width=2}: 宽度为2，不足2位用0填充, 如001, 002...**默认宽度为0** <!-- 十六进制需要在0x后面补0-->
///     - {n:step=2}: 步长为2, 如1, 3, 5...**默认步长为1；步长只能是正数**
///     - {n:radix=16}: 进制为16, 如0x01, 0x02...**默认进制为10**
///     - {n:reverse}: 将生成的列表反向, **默认不反向, 并且不是十进制时无效**
/// - {rand:n}: 生成随机数，n为生成的随机数的长度，如{rand:3}->123 <!-- 需要保证不重复-->
pub fn wildcard_to_target(paths: &Vec<String>, pattern: &String) -> Result<Vec<String>> {
    let cont = pattern.trim_start_matches("{").trim_end_matches("}");
    if cont == "source" {
        return Ok(paths.clone());
    }
    if cont == "prefix" {
        let mut new_paths = Vec::new();
        for path in paths {
            let path = Path::new(path);
            let file_name = path.file_name().unwrap().to_str().unwrap();
            // 使用 rfind 查找最后一个点
            let (prefix, _) = match file_name.rfind('.') {
                Some(pos) => (
                    file_name[..pos].to_string(),
                    file_name[pos + 1..].to_string(),
                ),
                None => (file_name.to_string(), String::new()),
            };
            new_paths.push(prefix);
        }
        return Ok(new_paths);
    }
    if cont == "suffix" {
        let mut new_paths = Vec::new();
        for path in paths {
            let path = Path::new(path);
            let ext = path.extension().unwrap().to_str().unwrap_or("").to_string();
            new_paths.push(ext);
        }
    }

    if cont == "n" {
        let res: Vec<String> = (0..paths.len()).map(|i| i.to_string()).collect();
        return Ok(res);
    }
    // 序号
    if let Some(n) = cont.strip_prefix("n:") {
        let re_start = Regex::new(r"start=(\d+)")?;
        let re_width = Regex::new(r"width=(\d+)")?;
        let re_step = Regex::new(r"step=(\d+)")?;
        let re_radix = Regex::new(r"radix=(\d+)")?;
        let re_reverse = Regex::new(r"reverse")?;
        let mut start = 0;
        let mut width = 0;
        let mut step = 1;
        let mut radix = 10;
        let mut reverse = false;
        if let Some(cap) = re_start.captures(n)? {
            start = cap[1].parse::<usize>()?;
        }
        if let Some(cap) = re_width.captures(n)? {
            width = cap[1].parse::<usize>()?;
        }
        if let Some(cap) = re_step.captures(n)? {
            step = cap[1].parse::<usize>()?;
        }
        if let Some(_) = re_reverse.captures(n)? {
            reverse = true;
        }
        if let Some(cap) = re_radix.captures(n)? {
            radix = cap[1].parse::<usize>()?;
            if radix != 10 {
                reverse = false;
            }
        }
        let mut res: Vec<String> = (start..paths.len())
            .step_by(step)
            .map(|i| i.to_string())
            .collect();
        if reverse {
            res.reverse();
        }
        if radix != 10 {
            res = res
                .iter()
                .map(|i| {
                    let num = i.parse::<usize>().unwrap();
                    let hex_prefix = match radix {
                        16 => "0x",
                        _ => "",
                    };
                    let radix_str = format!("{:x}", num);
                    let padded = format!("{}{:0>width$}", hex_prefix, radix_str, width = width);
                    padded
                })
                .collect();
        } else if width != 0 {
            res = res
                .iter()
                .map(|i| format!("{:0width$}", i, width = width))
                .collect();
        }
        return Ok(res);
    }

    // 随机数
    if let Some(rand_part) = cont.strip_prefix("rand:") {
        let len = rand_part.parse::<usize>().unwrap_or(6);
        let mut rng = rand::rng();
        return Ok((0..paths.len())
            .map(|_| {
                (0..len)
                    .map(|_| rng.sample(Alphanumeric) as char)
                    .collect::<String>()
            })
            .collect());
    }

    Ok(vec![])
}

/// 处理特殊符号：
/// - {+p}: 将p指定的内容转换成大写，如{+source}->ABC.TXT
/// - {-p}: 将p指定的内容转换成小写，如{-source}->abc.txt
/// - {p:l}: 对p指定的内容进行截取，l为截取的长度，如{source:3}->abc
/// - {p:s:l}: 对p指定的内容进行截取，l为截取的长度，s为起始位置，如{source:1:3}->bca
/// - {p:s-e}: 对p指定的内容进行截取，s为起始位置，e为结束位置，如{source:1-3}->bca
fn process_special_symbols(sources: &Vec<String>, pattern: &String) -> Result<Vec<String>> {
    let mut results = sources.clone();

    // 处理大小写转换
    if let Some(_case_target) = pattern.strip_prefix('+') {
        results = results.iter().map(|s| s.to_uppercase()).collect();
    } else if let Some(_case_target) = pattern.strip_prefix('-') {
        results = results.iter().map(|s| s.to_lowercase()).collect();
    }

    // 处理子字符串截取
    if let Some((_, params)) = pattern.split_once(':') {
        results = results
            .iter()
            .map(|s| {
                match params {
                    p if p.contains(':') => {
                        // {p:s:l} 格式
                        let parts: Vec<&str> = p.split(':').collect();
                        let start = parts[0].parse().unwrap_or(0);
                        let length = parts[1].parse().unwrap_or(0);
                        s.chars().skip(start).take(length).collect()
                    }
                    p if p.contains('-') => {
                        // {p:s-e} 格式
                        let parts: Vec<&str> = p.split('-').collect();
                        let start = parts[0].parse().unwrap_or(0);
                        let end = parts[1].parse().unwrap_or(0);
                        s.chars().skip(start).take(end - start).collect()
                    }
                    p => {
                        // {p:l} 格式
                        let length = p.parse().unwrap_or(0);
                        s.chars().take(length).collect()
                    }
                }
            })
            .collect();
    }

    Ok(results)
}

/// 获取音乐元数据
/// # 参数
/// - `path`: 文件路径
/// - `key`: 元数据键，格式为 `{audio:key_name}`
/// # 返回值
/// 返回 `Result<String, Error>`，包含请求的元数据值或错误
fn get_audio_metadata(path: &Path, key: &str) -> Result<String> {
    let tag = Tag::read_from_path(path)?;
    let re = Regex::new(r"^\{audio:(.+)\}$")?;
    let cap = re.captures(key)?.unwrap();
    let key_name = cap.get(1).unwrap().as_str();
    match key_name {
        "title" => Ok(tag.title().unwrap_or_default().to_string()),
        "artist" => Ok(tag.artist().unwrap_or_default().to_string()),
        "album" => Ok(tag.album().unwrap_or_default().to_string()),
        "year" => Ok(tag.year().unwrap_or_default().to_string()),
        "genre" => Ok(tag.genre().unwrap_or_default().to_string()),
        "track" => Ok(tag.track().unwrap_or_default().to_string()),
        "disc" => Ok(tag.disc().unwrap_or_default().to_string()),
        "date_recorded" => Ok(tag.date_recorded().unwrap_or_default().to_string()),
        "date_released" => Ok(tag.date_released().unwrap_or_default().to_string()),
        "duration" => Ok(tag.duration().unwrap_or_default().to_string()),
        _ => Err(anyhow::anyhow!("Invalid key name")),
    }
}

/// 获取视频元数据
/// # 参数
/// - `path`: 文件路径
/// - `key`: 元数据键，格式为 `{video:key_name}`
/// # 返回值
/// 返回 `Result<String, Error>`，包含请求的元数据值或错误
fn get_video_metadata(path: &Path, key: &str) -> Result<String> {
    let file = std::fs::File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    // 探测文件格式
    let probe = get_probe();
    let format_reader = probe.format(
        Hint::new().with_extension(ext),
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;

    // 获取元数据
    let mut metadata = format_reader.metadata;

    // 解析key，去掉{video:}前缀
    let key = key.trim_start_matches("{video:").trim_end_matches('}');

    if let Some(metadata_rev) = metadata.get().unwrap().current() {
        for tag in metadata_rev.tags() {
            if tag.key == key {
                // 返回匹配的元数据值
                return Ok(tag.value.to_string());
            }
        }
    }
    // 没有找到匹配的元数据
    Err(anyhow!("Metadata not found"))
}

/// 获取图片元数据
/// # 参数
/// - `path`: 文件路径
/// - `key`: 元数据键，格式为 `{image:key_name}`
/// # 返回值
/// 返回 `Result<String, Error>`，包含请求的元数据值或错误
fn get_image_metadata(path: &Path, key: &str) -> Result<String> {
    let mut parser = nom_exif::MediaParser::new();

    let ms = nom_exif::MediaSource::file_path(path)?;

    let iter: nom_exif::ExifIter = parser.parse(ms)?;
    let exif: nom_exif::Exif = iter.into();
    // let ct = exif.get(nom_exif::ExifTag::CreateDate).unwrap();
    let key = key.trim_start_matches("{image:").trim_end_matches('}');
    match key {
        "width" => return Ok(exif.get(nom_exif::ExifTag::ImageWidth).unwrap().to_string()),
        "height" => {
            return Ok(exif
                .get(nom_exif::ExifTag::ImageHeight)
                .unwrap()
                .to_string());
        }
        "create_date" => return Ok(exif.get(nom_exif::ExifTag::CreateDate).unwrap().to_string()),
        "make" => {
            return Ok(exif.get(nom_exif::ExifTag::Make).unwrap().to_string());
        }
        "model" => return Ok(exif.get(nom_exif::ExifTag::Model).unwrap().to_string()),
        "software" => return Ok(exif.get(nom_exif::ExifTag::Software).unwrap().to_string()),
        "orientation" => {
            return Ok(exif
                .get(nom_exif::ExifTag::Orientation)
                .unwrap()
                .to_string());
        }
        "exposure_time" => {
            return Ok(exif
                .get(nom_exif::ExifTag::ExposureTime)
                .unwrap()
                .to_string());
        }
        "f_number" => return Ok(exif.get(nom_exif::ExifTag::FNumber).unwrap().to_string()),
        "iso_speed_ratings" => {
            return Ok(exif
                .get(nom_exif::ExifTag::ISOSpeedRatings)
                .unwrap()
                .to_string());
        }
        "exposure_program" => {
            return Ok(exif
                .get(nom_exif::ExifTag::ExposureProgram)
                .unwrap()
                .to_string());
        }
        "aperture_value" => {
            return Ok(exif
                .get(nom_exif::ExifTag::ApertureValue)
                .unwrap()
                .to_string());
        }
        "max_aperture_value" => {
            return Ok(exif
                .get(nom_exif::ExifTag::MaxApertureValue)
                .unwrap()
                .to_string());
        }
        "metering_mode" => {
            return Ok(exif
                .get(nom_exif::ExifTag::MeteringMode)
                .unwrap()
                .to_string());
        }
        "flash" => return Ok(exif.get(nom_exif::ExifTag::Flash).unwrap().to_string()),
        "focal_length" => {
            return Ok(exif
                .get(nom_exif::ExifTag::FocalLength)
                .unwrap()
                .to_string());
        }
        "subject_distance" => {
            return Ok(exif
                .get(nom_exif::ExifTag::SubjectDistance)
                .unwrap()
                .to_string());
        }
        "color_space" => return Ok(exif.get(nom_exif::ExifTag::ColorSpace).unwrap().to_string()),
        "datetime_original" => {
            return Ok(exif
                .get(nom_exif::ExifTag::DateTimeOriginal)
                .unwrap()
                .to_string());
        }
        "components_configuration" => {
            return Ok(exif
                .get(nom_exif::ExifTag::ComponentsConfiguration)
                .unwrap()
                .to_string());
        }
        "compression" => {
            return Ok(exif
                .get(nom_exif::ExifTag::Compression)
                .unwrap()
                .to_string());
        }
        "shutter_speed_value" => {
            return Ok(exif
                .get(nom_exif::ExifTag::ShutterSpeedValue)
                .unwrap()
                .to_string());
        }
        "brightness_value" => {
            return Ok(exif
                .get(nom_exif::ExifTag::BrightnessValue)
                .unwrap()
                .to_string());
        }
        "exposure_bias_value" => {
            return Ok(exif
                .get(nom_exif::ExifTag::ExposureBiasValue)
                .unwrap()
                .to_string());
        }
        "GPSLatitude" => {
            return Ok(exif
                .get(nom_exif::ExifTag::GPSLatitude)
                .unwrap()
                .to_string());
        }
        "GPSLongitude" => {
            return Ok(exif
                .get(nom_exif::ExifTag::GPSLongitude)
                .unwrap()
                .to_string());
        }
        "GPSAltitude" => {
            return Ok(exif
                .get(nom_exif::ExifTag::GPSAltitude)
                .unwrap()
                .to_string());
        }
        "GPSAltitudeRef" => {
            return Ok(exif
                .get(nom_exif::ExifTag::GPSAltitudeRef)
                .unwrap()
                .to_string());
        }
        "GPSTimeStamp" => {
            return Ok(exif
                .get(nom_exif::ExifTag::GPSTimeStamp)
                .unwrap()
                .to_string());
        }
        "ISO" => {
            return Ok(exif
                .get(nom_exif::ExifTag::ISOSpeedRatings)
                .unwrap()
                .to_string());
        }
        _ => return Err(anyhow!("Invalid key name")),
    }
}
