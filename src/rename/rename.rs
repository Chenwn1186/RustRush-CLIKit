use anyhow::{Ok, Result, anyhow};
use colored::Colorize;
use fancy_regex::Regex;
use id3::{Tag, TagLike};
use nom_exif;
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::io::{self, Write};
use std::path::Path;
use std::vec;
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
) -> Result<bool> {
    // println!("Renaming '{}' to '{}'...", source, target);
    // 处理流程：
    // 将变量存储到列表中
    // 将source中的变量替换成组，作为最终的正则表达式
    // 匹配字符串并将组赋值到变量中
    // 将target中的字符串分割成变量和字符的列表
    // 将列表中的变量替换成具体的值，这个值如果不在组里，那就尝试从通配符或者元数据中获取l；并且处理特殊字符
    // 将字符和变量拼接成字符串
    let path_entries = Path::new(directory.as_str()).read_dir()?;
    let mut paths = Path::new(directory.as_str())
        .read_dir()?
        .map(|entry| {
            entry
                .unwrap()
                .path()
                .into_os_string()
                .into_string()
                .unwrap()
                .trim_start_matches("./")
                .to_string()
        })
        .collect::<Vec<String>>();
    if !pattern && !wildcard {
        if !regex {
            let path_entry = Path::new(source.as_str());
            if path_entry.exists() {
                for p in path_entries {
                    if let std::result::Result::Ok(p) = p {
                        if p.file_name().to_string_lossy().contains(source.as_str()) {
                            return rename_single_file(&p.path(), &target);
                        }
                    }
                }
            }
        } else {
            let re = Regex::new(&source)?;
            for p in path_entries {
                if let std::result::Result::Ok(p) = p {
                    if re.is_match(p.file_name().as_os_str().to_str().unwrap())? {
                        return rename_single_file(&p.path(), &target);
                    }
                }
            }
        }
    }
    let value_map = if pattern {
        extract_named_groups(&mut paths, &source).unwrap_or(vec![HashMap::new(); paths.len()])
    } else {
        let path_entries = Path::new(directory.as_str()).read_dir()?;
        let path_entry = Path::new(source.as_str());
            if path_entry.exists() {
                for p in path_entries {
                    if let std::result::Result::Ok(p) = p {
                        if p.file_name().to_string_lossy().contains(source.as_str()) {
                            paths = vec![p.file_name().to_string_lossy().to_string()];
                        }
                    }
                }
            }
        vec![HashMap::new()]
        
    };
    println!("value_map: {:?}", value_map);
    println!("paths: {:?}", paths);
    let res = rename_batch(paths, value_map, target, wildcard);
    match res {
        std::result::Result::Ok(_) => {
            // println!("Rename success");
            return Ok(true);
        }
        std::result::Result::Err(e) => {
            println!("Rename failed: {}", e);
            return Err(e);
        }
    }
}

fn wait_for_yes_no() -> bool {
    loop {
        // 提示用户确认操作
        print!("确认执行重命名操作吗？(y 确认 / c 取消): ");
        io::stdout().flush().unwrap(); // 确保立即输出（print! 不会自动刷新）

        // 读取用户输入
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("无法读取输入");

        // 去除换行符并转换为小写
        let input = input.trim().to_lowercase();

        match input.as_str() {
            "y" => {
                return true;
            }
            "c" => {
                return false;
            }
            _ => {
                println!("无效输入，请输入 y 或 c");
            }
        }
    }
}

fn rename_single_file(path: &Path, target: &str) -> Result<bool> {
    println!(
        "将 '{}' 重命名为 '{}'",
        path.file_name().unwrap().to_string_lossy(),
        target.green()
    );
    let yes_no = wait_for_yes_no();
    if yes_no {
        let new_path = path.with_file_name(target);
        std::fs::rename(path, new_path)?;
        return Ok(true);
    }
    return Ok(false);
}

fn rename_batch_files(paths: &Vec<String>, target: &Vec<String>) -> Result<bool> {
    let mut success = true;
    println!("重命名:");
    for (i, path) in paths.iter().enumerate() {
        println!("{} -> {}", path, target[i].green());
    }
    let yes_no = wait_for_yes_no();
    if yes_no {
        for (i, path) in paths.iter().enumerate() {
            let new_path = Path::new(path).with_file_name(&target[i]);
            let res = std::fs::rename(path, new_path);
            match res {
                std::result::Result::Ok(_) => {}
                std::result::Result::Err(e) => {
                    println!("重命名失败: {}", e);
                    success = false;
                }
            }
        }
    }
    return Ok(success);
}

/// 处理最终的批量重命名过程，需要开启pattern模式，并且会强制使用regex
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
    if value_map.len() != paths.len() {
        return Err(anyhow!("Value map length does not match paths length"));
    }
    // println!("wildcard: {}", wildcard);
    // if value_map.is_empty() {
    //     return Err(anyhow!("Value map is empty"));
    // }
    // if value_map[0].is_empty() {
    //     return Err(anyhow!("Value map is empty"));
    // }
    let mut final_paths: Vec<String> = vec![String::new(); paths.len()];
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
    for part in target_parser.iter() {
        let mut part_type = 0; // 0: 普通字符串 1: 变量 2: 元数据 3: 通配符
        let mut var_name = "";
        let mut key_name = "";
        if part.starts_with('{') && part.ends_with('}') {
            key_name = &part[1..part.len() - 1];
            let var_name_re = Regex::new(r"\{(\+)?(\w+)(:.+)?\}").unwrap();
            let var_name_caps = var_name_re.captures(part).unwrap().unwrap();
            var_name = var_name_caps.get(2).unwrap().as_str();
            if !value_map.is_empty() && value_map[0].contains_key(var_name) {
                part_type = 1;
            } else if get_metadata(Path::new(paths[0].as_str()), part.as_str()).is_some() {
                part_type = 2;
            } else if wildcard {
                part_type = 3;
            } else {
                return Err(anyhow!("Invalid target: {}, error \"{}\"", target, part));
            }
        }
        match part_type {
            0 => {
                for (i, _path) in paths.iter().enumerate() {
                    final_paths[i].push_str(&part);
                }
            }
            1 => {
                let mut value_vec = vec![];
                for (i, _path) in paths.iter().enumerate() {
                    let value = value_map[i].get(var_name).unwrap();
                    // final_paths[i].push_str(&value);
                    value_vec.push(value.clone());
                }
                let final_value_vec = process_special_symbols(&value_vec, &key_name.to_string())?;
                for (i, _path) in paths.iter().enumerate() {
                    final_paths[i].push_str(&final_value_vec[i]);
                }
            }
            2 => {
                let mut value_vec = vec![];
                for (_i, path) in paths.iter().enumerate() {
                    let value = get_metadata(Path::new(path.as_str()), part.as_str()).unwrap();
                    // final_paths[i].push_str(&value);
                    value_vec.push(value.clone());
                }
                let final_value_vec = process_special_symbols(&value_vec, &key_name.to_string())?;
                for (i, _path) in paths.iter().enumerate() {
                    final_paths[i].push_str(&final_value_vec[i]);
                }
            }
            3 => {
                let wildcards = wildcard_to_target(&paths, part)?;
                // println!("wildcards: {:?}", wildcards);
                let final_wildcards = if part.contains("{n:") {
                    wildcards
                } else {
                    process_special_symbols(&wildcards, &key_name.to_string())?
                };
                for (i, _path) in paths.iter().enumerate() {
                    final_paths[i].push_str(&final_wildcards[i]);
                }
            }
            _ => {
                return Err(anyhow!("Invalid target: {}, error \"{}\"", target, part));
            }
        }
        part_type = 0;
        _ = part_type;
    }
    // for (path, final_path) in paths.iter().zip(final_paths.iter()) {
    //     println!("Renaming '{}' to '{}'", path, final_path);
    // }
    rename_batch_files(&paths, &final_paths)
}

/// 使用命名捕获组批量处理扩展正则表达式
/// # 参数
/// - `inputs`: 待匹配的字符串列表
/// - `ext_regex`: 扩展正则表达式，包含 {varname} 标记
/// # 返回值
/// 包含捕获键值对的 HashMap 列表，每个元素对应一个输入
pub fn extract_named_groups(
    inputs: &mut Vec<String>,
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
                format!("(?P<{}>.+)", var_name)
            })
            .to_string();

        // 编译正则表达式
        let re = Regex::new(&final_regex).ok()?;
        (re, var_names)
    };
    let mut filtered_inputs: Vec<String> = Vec::new();
    // 批量处理输入
    let mut can_add = false;
    let mut results = vec![];
    for input in inputs.iter() {
        let mut groups = HashMap::new();
        if let std::result::Result::Ok(Some(caps)) = re.captures(input.as_str()) {
            for name in &var_names {
                if let Some(value) = caps.name(name) {
                    groups.insert(name.clone(), value.as_str().to_string());
                    can_add = true;
                } else {
                    filtered_inputs.push(input.clone());
                    can_add = false;
                    // println!("Invalid input: {}", input);
                    break;
                }
            }
        } else {
            filtered_inputs.push(input.clone());
            can_add = false;
        }
        if can_add {
            // println!("Input: {}, Groups: {:?}", input, groups);
            results.push(groups);
            can_add = false;
        }
    }
    inputs.retain(|x| !filtered_inputs.contains(x));
    // println!("results: {:?}", results);
    // println!("inputs: {:?}", inputs);

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
        }
        let total_items = paths.len();
        let end = start + total_items * step;
        let mut res: Vec<String> = (start..end)
            .step_by(step)
            .take(total_items)
            .map(|i| i.to_string())
            .collect();
        // let mut res: Vec<String> = (start..(paths.len() + start*step))
        //     .step_by(step)
        //     .map(|i| i.to_string())
        //     .collect();
        if reverse {
            // println!("reverse: {:?}", res);
            res.reverse();
        }
        if radix != 10 {
            res = res
                .iter()
                .map(|i| {
                    let num = i.parse::<u32>().unwrap();
                    let (prefix, base_str) = match radix {
                        16 => ("0x", format_radix(num, 16).unwrap()),
                        8 => ("0o", format_radix(num, 8).unwrap()),
                        2 => ("0b", format_radix(num, 2).unwrap()),
                        _ => ("", format_radix(num, radix.try_into().unwrap()).unwrap()),
                    };

                    let padding = width;
                    format!("{}{:0>padding$}", prefix, base_str, padding = padding)
                })
                .collect();
        } else if width != 0 {
            res = res
                .iter()
                .map(|i| format!("{:0>width$}", i, width = width))
                .collect();
        }
        return Ok(res);
    }

    // 随机数
    if let Some(rand_part) = cont.strip_prefix("rand:") {
        let len = rand_part.parse::<usize>().unwrap_or(6);
        let mut rng = rand::rng();
        let max_unique = 10usize.pow(len as u32);
        let check_uniqueness = paths.len() <= max_unique;

        let mut used_numbers = HashSet::with_capacity(paths.len());
        return Ok((0..paths.len())
            .map(|_| {
                if check_uniqueness {
                    loop {
                        let num_str: String = (0..len)
                            .map(|_| rng.random_range(0..=9).to_string())
                            .collect();
                        if used_numbers.insert(num_str.clone()) {
                            break num_str;
                        }
                    }
                } else {
                    (0..len)
                        .map(|_| rng.random_range(0..=9).to_string())
                        .collect()
                }
            })
            .collect());
    }

    Ok(vec![])
}

fn format_radix(mut x: u32, radix: u32) -> Result<String> {
    if !(2..=36).contains(&radix) {
        return Err(anyhow!("Radix {} out of range (2-36)", radix));
    }

    if x == 0 {
        return Ok(x.to_string());
    }

    let mut result = Vec::new();
    let digits: Vec<char> = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().collect();

    while x > 0 {
        let m = (x % radix) as usize;
        result.push(digits[m]);
        x /= radix;
    }

    Ok(result.into_iter().rev().collect())
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
        // println!("大写");
        results = results.iter().map(|s| s.to_uppercase()).collect();
    } else if let Some(_case_target) = pattern.strip_prefix('-') {
        // println!("小写");
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

/// 获取元数据
/// # 参数
/// - `path`: 文件路径
/// - `key`: 元数据键，格式为 `{key_name}`
/// # 返回值
/// 返回 `Result<String, Error>`，包含请求的元数据值或错误
fn get_metadata(path: &Path, key: &str) -> Option<String> {
    let re = Regex::new(r"^\{(.+):.+\}$").unwrap();
    let cap = re.captures(key);
    match cap {
        std::result::Result::Ok(Some(cap)) => {
            let key_name = cap.get(1).unwrap().as_str();
            match key_name {
                "audio" => {
                    let audio_md = get_audio_metadata(path, key);
                    match audio_md {
                        std::result::Result::Ok(md) => {
                            return Some(md);
                        }
                        std::result::Result::Err(_) => {
                            return None;
                        }
                    }
                }
                "video" => {
                    let video_md = get_video_metadata(path, key);
                    match video_md {
                        std::result::Result::Ok(md) => {
                            return Some(md);
                        }
                        std::result::Result::Err(_) => {
                            return None;
                        }
                    }
                }
                "image" => {
                    let image_md = get_image_metadata(path, key);
                    match image_md {
                        std::result::Result::Ok(md) => {
                            return Some(md);
                        }
                        std::result::Result::Err(_) => {
                            return None;
                        }
                    }
                }
                _ => {
                    return None;
                }
            }
        }
        std::result::Result::Ok(None) => {
            return None;
        }
        std::result::Result::Err(_) => {
            return None;
        }
    }
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
