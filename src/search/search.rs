use colored::Colorize;
// use regex::Regex;
use crate::utils::utils::get_extension;
use atty::Stream;
use fancy_regex::Regex;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;

pub fn search_command(
    paths: Vec<String>,
    keyword: String,
    search_content: bool,
    regex: bool,
    ignore_case: bool,
    recursive_depth: usize, // 修改为usize类型
    size: Option<String>,
    file_type: Option<String>,
    modified: Option<String>,
    accessed: Option<String>,
    created: Option<String>,
    permission: Option<String>,
    owner: Option<String>,
    group: Option<String>,
) {
    // 检测是否有管道输入（标准输入非终端时视为管道）
    let has_pipe_input = !atty::is(Stream::Stdin);

    if has_pipe_input {
        // 从管道读取内容并直接搜索
        let stdin = io::stdin().lock();
        for line in stdin.lines().filter_map(|line| line.ok()) {
            let (matched, highlighted) = highlight_keyword(&line, &keyword, ignore_case, "", regex);
            if matched {
                println!("{}", highlighted);
            }
        }
        return;
    }

    // 无管道时使用命令行传入的路径进行搜索
    for p in paths {
        let path = Path::new(&p);
        search_and_highlight(
            path,
            &keyword,
            search_content,
            regex,
            ignore_case,
            recursive_depth,
            true,
            &size,
            &file_type,
            &modified,
            &accessed,
            &created,
            &permission,
            &owner,
            &group,
        );
    }
}

fn search_and_highlight(
    path: &Path,
    keyword: &str,
    search_content: bool,
    regex: bool,
    ignore_case: bool,
    depth: usize, // 修改参数名和类型
    match_filename: bool,
    size: &Option<String>,
    file_type: &Option<String>,
    modified: &Option<String>,
    accessed: &Option<String>,
    created: &Option<String>,
    permission: &Option<String>,
    owner: &Option<String>,
    group: &Option<String>,
) {
    let paths_to_search = match fs::read_dir(path) {
        Ok(entries) => entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .collect(),
        Err(_) => {
            // 处理文件搜索
            if path.is_file() {
                vec![path.to_path_buf()]
            } else {
                vec![]
            }
        }
    };

    // todo: 按照ls部分的着色逻辑来改写这部分
    for p in paths_to_search {
        // 添加文件筛选条件
        let mut should_skip = false;

        if let Some(size_str) = size {
            if let Ok(passed) = crate::utils::utils::check_size_condition(&p, size_str) {
                if !passed {
                    should_skip = true;
                }
            }
        }

        if let Some(file_type_str) = file_type {
            if let Ok(passed) = crate::utils::utils::check_file_type(&p, file_type_str) {
                if !passed {
                    should_skip = true;
                }
            }
        }

        if let Some(modified_str) = modified {
            if let Ok(passed) =
                crate::utils::utils::check_datetime_condition(&p, modified_str, "mtime")
            {
                if !passed {
                    should_skip = true;
                }
            }
        }

        if let Some(accessed_str) = accessed {
            if let Ok(passed) =
                crate::utils::utils::check_datetime_condition(&p, accessed_str, "atime")
            {
                if !passed {
                    should_skip = true;
                }
            }
        }

        if let Some(created_str) = created {
            if let Ok(passed) =
                crate::utils::utils::check_datetime_condition(&p, created_str, "ctime")
            {
                if !passed {
                    should_skip = true;
                }
            }
        }
        if let Some(permission_str) = permission {
            if let Ok(passed) = crate::utils::utils::check_permission(&p, permission_str) {
                if !passed {
                    should_skip = true;
                }
            }
        }

        if let Some(owner_str) = owner {
            if let Ok(passed) = crate::utils::utils::check_owner(&p, owner_str) {
                if !passed {
                    should_skip = true;
                }
            }
        }

        if let Some(group_str) = group {
            if let Ok(passed) = crate::utils::utils::check_group(&p, group_str) {
                if !passed {
                    should_skip = true;
                }
            }
        }

        if should_skip {
            continue;
        }

        // 1. 处理文件名匹配逻辑
        if match_filename {
            if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                // 调用高亮函数，同时获取匹配状态和高亮结果
                let (name_matched, highlighted_name) =
                    highlight_keyword(name, keyword, ignore_case, "", regex);
                if name_matched {
                    //显示前缀，区分文件和目录
                    let prefix = if p.is_dir() { "" } else { "" };
                    println!("{}{}", prefix, p.with_file_name(highlighted_name).display());
                }
                if depth > 0 && p.is_dir() {
                    // 修改递归条件
                    // 递归搜索子目录，深度减1
                    search_and_highlight(
                        &p,
                        keyword,
                        search_content,
                        regex,
                        ignore_case,
                        depth - 1, // 深度递减
                        true,
                        size,
                        file_type,
                        modified,
                        accessed,
                        created,
                        permission,
                        owner,
                        group,
                    );
                }
            }
        }

        // 2. 处理文件内容搜索（仅当是文件且需要搜索内容时）
        if p.is_file() && search_content {
            if let Ok(content) = fs::read_to_string(&p) {
                let ext = get_extension(&p);
                for (line_num, line) in content.lines().enumerate() {
                    let (line_matched, highlighted_line) =
                        highlight_keyword(line, keyword, ignore_case, &ext, regex);
                    if line_matched {
                        println!("{}:{} - {}", p.display(), line_num + 1, highlighted_line);
                    }
                }
            }
        }
    }
}

/// 合并搜索与高亮逻辑，返回（是否匹配，高亮后的字符串）
fn highlight_keyword(
    line: &str,
    keyword: &str,
    ignore_case: bool,
    ext: &str,
    regex: bool,
) -> (bool, String) {
    // 加载默认的语法集和主题集
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let mut is_highlight = true;

    // 根据文件扩展名查找对应的语法，找不到时标记不高亮
    let syntax = match ps.find_syntax_by_extension(ext) {
        Some(s) => s,
        None => {
            is_highlight = false;
            // 回退到纯文本语法（实际不会应用复杂高亮）
            ps.find_syntax_plain_text()
        }
    };

    // 创建 HighlightLines 实例（即使不高亮也需要实例，但实际不会应用复杂高亮）
    let mut highlighter = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

    let mut highlighted = String::new();
    let mut last_index = 0;
    let mut has_match = false;

    if regex {
        let pattern = if ignore_case {
            Regex::new(&format!("(?i){}", keyword)).unwrap_or_else(|_| Regex::new("").unwrap())
        } else {
            Regex::new(keyword).unwrap_or_else(|_| Regex::new("").unwrap())
        };

        for mat in pattern.find_iter(line) {
            let mat = mat.expect("Failed to get match");
            let start = mat.start();
            let end = mat.end();

            // 处理未匹配部分（根据 is_highlight 决定是否语法高亮）
            let unmatched_part = &line[last_index..start];
            if is_highlight {
                let unmatched_ranges: Vec<(Style, &str)> = highlighter
                    .highlight_line(unmatched_part, &ps)
                    .unwrap_or_default();
                highlighted.push_str(&as_24_bit_terminal_escaped(&unmatched_ranges[..], true));
            } else {
                highlighted.push_str(unmatched_part); // 无语法高亮时直接拼接原始文本
            }

            // 处理匹配部分（始终保留黄色背景高亮）
            let matched_part = &line[start..end];
            highlighted.push_str(&matched_part.on_yellow().to_string());
            last_index = end;
            has_match = true;
        }
    } else {
        let (search_line, search_keyword) = if ignore_case {
            (line.to_lowercase(), keyword.to_lowercase())
        } else {
            (line.to_string(), keyword.to_string())
        };

        while let Some(start) = search_line[last_index..].find(&search_keyword) {
            let start = last_index + start;
            let end = start + keyword.len();

            // 处理未匹配部分（根据 is_highlight 决定是否语法高亮）
            let unmatched_part = &line[last_index..start];
            if is_highlight {
                let unmatched_ranges: Vec<(Style, &str)> = highlighter
                    .highlight_line(unmatched_part, &ps)
                    .unwrap_or_default();
                highlighted.push_str(&as_24_bit_terminal_escaped(&unmatched_ranges[..], true));
            } else {
                highlighted.push_str(unmatched_part); // 无语法高亮时直接拼接原始文本
            }

            // 处理匹配部分（始终保留黄色背景高亮）
            let matched_part = &line[start..end];
            highlighted.push_str(&matched_part.on_yellow().to_string());
            last_index = end;
            has_match = true;
        }
    }

    // 处理剩余未匹配的文本（根据 is_highlight 决定是否语法高亮）
    let remaining_part = &line[last_index..];
    if is_highlight {
        let remaining_ranges: Vec<(Style, &str)> = highlighter
            .highlight_line(remaining_part, &ps)
            .unwrap_or_default();
        highlighted.push_str(&as_24_bit_terminal_escaped(&remaining_ranges[..], true));
    } else {
        highlighted.push_str(remaining_part); // 无语法高亮时直接拼接原始文本
    }

    (has_match, highlighted)
}
