use std::path::Path;
use std::fs;
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

pub fn print_highlighted_lines(
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