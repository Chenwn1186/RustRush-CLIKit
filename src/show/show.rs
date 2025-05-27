
use crate::utils::utils::print_highlighted_lines;
pub fn show_command(file_path: String, lines: usize) {
    if let Err(e) = print_highlighted_lines(&file_path, lines) {
        eprintln!("Error: {}", e);
    }
}

