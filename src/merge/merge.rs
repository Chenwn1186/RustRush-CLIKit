use std::fs;
use crate::show::show::show_command;
pub fn merge_command(file_paths: Vec<String>, output: Option<String>, lines: usize) {
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
            // 这里调用 show 模块的函数来显示内容
            show_command(output_path, lines);
        }
    }
}