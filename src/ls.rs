use crate::utils::utils::get_extension;
use chrono::DateTime;
use chrono::Local;
use colored::{Color, Colorize};
// use fancy_regex::Regex;
// use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub fn ls_command(
    directory: PathBuf,
    author: bool,
    long: bool,
    all: bool,
    recursive: bool,
    human_readable: bool,
    time_sort: bool,
    reverse: bool,
    hyperlink: bool,
    inode: bool,
    ext_sort: bool,
    size_sort: bool,
    directories_only: bool,
    files_only: bool,
    color: bool,
    differentiated: bool,
    header: bool,
    custom_show: &Vec<String>,
) {
    list_directory(
        directory,
        author,
        long,
        all,
        recursive,
        human_readable,
        time_sort,
        reverse,
        hyperlink,
        inode,
        ext_sort,
        size_sort,
        directories_only,
        files_only,
        color,
        differentiated,
        header,
        custom_show,
    );
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
/// 为文件/文件夹路径生成终端超链接
/// 
/// # 参数
/// - path: 文件/文件夹路径
/// 
/// # 返回值
/// 返回格式化后的超链接字符串
fn format_file_hyperlink(path: &Path) -> String {

    let abs_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().unwrap().join(path)
    };

    let url = format!("file://{}", abs_path.display());
    let text = path.to_string_lossy();
    
    format!("\x1b]8;;{}\x07{}\x1b]8;;\x07", url, text)
}
/// 应用颜色高亮
fn apply_color(
    name: &str,
    is_dir: bool,
    ext: &str,
    color_config: &ColorConfig,
    color_enabled: bool,
) -> String {
    if !color_enabled {
        return name.to_string();
    }

    if is_dir {
        if let Some(color) = color_config.get_dir_color(name) {
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
// /// 过滤 ANSI 转义序列的函数
// fn strip_ansi_escapes(s: &str) -> String {
//     lazy_static! {
//         static ref ANSI_ESCAPE: Regex =
//             Regex::new(r"\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])").unwrap();
//     }
//     ANSI_ESCAPE.replace_all(s, "").to_string()
// }

/// 列出目录内容
fn list_directory(
    directory: PathBuf,
    author: bool,
    long: bool,
    all: bool,
    recursive: bool,
    human_readable: bool,
    time_sort: bool,
    reverse: bool,
    hyperlink: bool,
    inode: bool,
    ext_sort: bool,
    size_sort: bool,
    directories_only: bool,
    files_only: bool,
    color: bool,
    differentiated: bool,
    header: bool,
    custom_show: &Vec<String>,
) {
    let color_config = ColorConfig::load_from_file();
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    if directories_only && !directory.is_dir() {
        return;
    }

    if let Ok(entries) = fs::read_dir(&directory) {
        let mut entries: Vec<fs::DirEntry> = entries.filter_map(Result::ok).collect();

        // 过滤隐藏文件
        if !all {
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

            if time_sort {
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
            } else if ext_sort {
                let a_ext = get_extension(&a.path());
                let b_ext = get_extension(&b.path());
                a_ext.cmp(&b_ext)
            } else if size_sort {
                let a_size = a.metadata().ok().map(|m| m.len()).unwrap_or(0);
                let b_size = b.metadata().ok().map(|m| m.len()).unwrap_or(0);
                a_size.cmp(&b_size)
            } else {
                a.file_name().cmp(&b.file_name())
            }
        });

        if reverse {
            entries.reverse();
        }
        let mut file_info_manager = FileInfoManager::new();
        for entry in entries.iter() {
            let full_path = entry.path();
            if directories_only && !full_path.is_dir() {
                continue;
            }
            if files_only && full_path.is_dir() {
                continue;
            }
            file_info_manager.add_file_info(entry, human_readable);
        }
        if long {
            file_info_manager.set_long_show(author, inode);
        } else {
            file_info_manager.set_classic_show();
        }

        if !custom_show.is_empty() {
            for name in custom_show.iter() {
                file_info_manager.show_name(name.to_string());
            }
        }

        file_info_manager.print(color, differentiated, header, hyperlink);

        if recursive {
            for entry in entries.iter() {
                let full_path = entry.path();
                if full_path.is_dir() {
                    let relative_path = get_relative_path(&current_dir, &full_path);
                    println!("\n{}:", relative_path.display());
                    list_directory(
                        full_path,
                        author,
                        long,
                        all,
                        recursive,
                        human_readable,
                        time_sort,
                        reverse,
                        hyperlink,
                        inode,
                        ext_sort,
                        size_sort,
                        directories_only,
                        files_only,
                        color,
                        differentiated,
                        header,
                        custom_show,
                    );
                }
            }
        }
    }

    // 保存配置（如果需要更新配置，可以在这里修改 ColorConfig 后保存）
    if let Err(e) = color_config.save_to_file() {
        eprintln!("Error saving color config: {}", e);
    }
}

/// 文件信息结构体
#[derive(Debug)]
struct FileInfo {
    size: String,
    file_name: String,
    modified: String,
    is_dir: bool,
    author: u32,
    inode: u64,
    link_count: u64,
    block_size: u64,
    blocks: u64,
    device: u64,
    atime: i64,
    ctime: i64,
    mtime: i64,
    is_executable: bool,
    permission: std::fs::Permissions,
    group: u32,
}

impl FileInfo {
    fn from_metadata(entry: &fs::DirEntry, human_readable: bool) -> Option<Self> {
        let metadata = entry.metadata().ok()?;
        let full_path = entry.path();
        let relative_path = get_relative_path(&std::env::current_dir().ok()?, &full_path);
        let file_name = relative_path.file_name()?.to_string_lossy().to_string();

        Some(Self {
            size: if human_readable {
                format_size(metadata.len())
            } else {
                format!("{} B", metadata.len())
            },
            file_name,
            modified: metadata.modified().ok().map_or_else(
                || "unknown".to_string(),
                |t| {
                    let dt: DateTime<Local> = t.into();
                    dt.format("%b %d %Y %H:%M").to_string()
                },
            ),
            is_dir: metadata.is_dir(),
            author: metadata.uid(),
            inode: metadata.ino(),
            link_count: metadata.nlink(),
            block_size: metadata.blksize(),
            blocks: metadata.blocks(),
            device: metadata.dev(),
            atime: metadata.atime(),
            ctime: metadata.ctime(),
            mtime: metadata.mtime(),
            is_executable: metadata.permissions().mode() & 0o111 != 0,
            permission: metadata.permissions(),
            group: metadata.gid(),
        })
    }

    fn get_info_vec(&self) -> Vec<String> {
        let mut info_vec = Vec::new();
        info_vec.push(self.size.clone());
        info_vec.push(self.file_name.clone());
        info_vec.push(self.modified.clone());
        info_vec.push(self.is_dir.to_string());
        info_vec.push(
            users::get_user_by_uid(self.author)
                .map(|u| u.name().to_string_lossy().into_owned())
                .unwrap_or_else(|| self.author.to_string()),
        ); // 显示用户名或回退到UID
        info_vec.push(self.inode.to_string());
        info_vec.push(self.link_count.to_string());
        info_vec.push(self.block_size.to_string());
        info_vec.push(self.blocks.to_string());
        info_vec.push(self.device.to_string());
        info_vec.push(
            DateTime::from_timestamp(self.atime, 0)
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
        ); // 转换atime
        info_vec.push(
            DateTime::from_timestamp(self.ctime, 0)
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
        ); // 转换ctime
        info_vec.push(
            DateTime::from_timestamp(self.mtime, 0)
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
        ); // 转换mtime
        info_vec.push(self.is_executable.to_string());
        info_vec.push(FileInfo::permission_to_string(
            self.permission.mode(),
            self.is_dir,
        ));
        info_vec.push(
            users::get_group_by_gid(self.group)
                .map(|u| u.name().to_string_lossy().into_owned())
                .unwrap_or_else(|| self.group.to_string()),
        ); // 显示组名或回退到GID
        info_vec
    }

    fn permission_to_string(mode: u32, is_dir: bool) -> String {
        let mut s = String::with_capacity(9);
        s.push(if is_dir { 'd' } else { '-' });
        s.push(if mode & 0o400 != 0 { 'r' } else { '-' });
        s.push(if mode & 0o200 != 0 { 'w' } else { '-' });
        s.push(if mode & 0o100 != 0 { 'x' } else { '-' });
        s.push(if mode & 0o040 != 0 { 'r' } else { '-' });
        s.push(if mode & 0o020 != 0 { 'w' } else { '-' });
        s.push(if mode & 0o010 != 0 { 'x' } else { '-' });
        s.push(if mode & 0o004 != 0 { 'r' } else { '-' });
        s.push(if mode & 0o002 != 0 { 'w' } else { '-' });
        s.push(if mode & 0o001 != 0 { 'x' } else { '-' });
        s
    }

    fn name_vec(&self) -> Vec<String> {
        vec![
            "size",
            "file_name",
            "modified",
            "is_dir",
            "author",
            "inode",
            "link_count",
            "block_size",
            "blocks",
            "device",
            "atime",
            "ctime",
            "mtime",
            "is_executable",
            "permission",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }
}

struct FileInfoManager {
    file_infos: Vec<FileInfo>,
    show_vec: Vec<bool>,
    print_order: Vec<usize>,
}

impl FileInfoManager {
    fn new() -> Self {
        Self {
            file_infos: Vec::new(),
            show_vec: Vec::new(),
            print_order: Vec::new(),
        }
    }

    fn add_file_info(&mut self, entry: &fs::DirEntry, human_readable: bool) {
        if let Some(file_info) = FileInfo::from_metadata(entry, human_readable) {
            self.file_infos.push(file_info);
            if self.show_vec.is_empty() {
                self.show_vec = vec![false; self.file_infos[0].name_vec().len()];
            }
            if self.print_order.is_empty() {
                self.print_order = (0..self.file_infos[0].name_vec().len()).rev().collect();
            }
        }
    }

    fn hide(&mut self, index: usize) {
        if index < self.show_vec.len() {
            self.show_vec[index] = false;
        }
    }
    // fn hide_vec(&mut self, indexs: Vec<usize>) {
    //     for index in indexs {
    //         self.hide(index);
    //     }
    // }
    fn hide_except_vec(&mut self, indexs: Vec<usize>) {
        for index in 0..self.show_vec.len() {
            if !indexs.contains(&index) {
                self.hide(index);
            }
        }
    }
    fn show(&mut self, index: usize) {
        if index < self.show_vec.len() {
            self.show_vec[index] = true;
        }
    }
    fn show_vec(&mut self, indexs: Vec<usize>) {
        for index in indexs {
            self.show(index);
        }
    }

    // fn show_except_vec(&mut self, indexs: Vec<usize>) {
    //     for index in 0..self.show_vec.len() {
    //         if !indexs.contains(&index) {
    //             self.show(index);
    //         }
    //     }
    // }

    fn show_name(&mut self, name: String) {
        let name_vec = self.file_infos[0].name_vec();
        for index in 0..name_vec.len() {
            if name_vec[index] == name {
                self.show(index);
                return;
            }
        }
        println!("Warning: {} not found", name);
    }

    fn set_classic_show(&mut self) {
        self.show(1);
        self.hide_except_vec(vec![1]);
    }
    fn set_long_show(&mut self, author: bool, inode: bool) {
        self.show_vec(vec![0, 1, 2, 14]);
        self.hide_except_vec(vec![0, 1, 2, 14]);
        if author {
            self.show_vec(vec![4]);
        }
        if inode {
            self.show_vec(vec![5]);
        }
    }
    fn get_max_widths(infos: &Vec<Vec<String>>) -> Vec<usize> {
        let mut max_widths = vec![0; infos[0].len()];
        for info in infos {
            for (i, info) in info.iter().enumerate() {
                max_widths[i] = max_widths[i].max(info.len());
            }
        }
        max_widths
    }
    fn print(&self, color: bool, differentiated: bool, header: bool, hyperlink: bool) {
        let name_vec_ori = self.file_infos[0].name_vec();
        // let print_list = self
        //     .print_order
        //     .iter()
        //     .filter(|&index| self.show_vec[*index])
        //     .cloned()
        //     .collect::<Vec<usize>>();
        let name_vec = self
            .print_order
            .iter()
            .filter_map(|&index| {
                if self.show_vec[index] {
                    Some(name_vec_ori[index].clone())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>();
        // 按照print_info排序, 并过滤掉show_vec为false的项
        let infos = self
            .file_infos
            .iter()
            .map(|file_info| file_info.get_info_vec())
            .collect::<Vec<Vec<String>>>()
            .iter()
            .map(|info_vec| {
                self.print_order
                    .iter()
                    // .map(|&index| info_vec[index].clone())
                    .filter_map(|&index| {
                        if self.show_vec[index] {
                            Some(info_vec[index].clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<String>>()
            })
            .collect::<Vec<Vec<String>>>();
        // println!(
        //     "{:?}, {:?}, {:?}, {:?}",
        //     infos.len(),
        //     print_list,
        //     self.show_vec,
        //     name_vec
        // );
        let mut max_widths = FileInfoManager::get_max_widths(&infos);
        for index in 0..infos.len() {
            let info_vec = &infos[index];
            if index == 0 {
                // 打印表头
                if header {
                    let header_width = FileInfoManager::get_max_widths(&vec![name_vec.clone()]);
                    for i in 0..max_widths.len() {
                        max_widths[i] = max_widths[i].max(header_width[i]);
                    }
                    for (i, name) in name_vec.iter().enumerate() {
                        if self.show_vec[i] {
                            if name == "file_name" && differentiated {
                                print!("{:>width$} ", name, width = max_widths[i]);
                            } else {
                                print!("{:>width$} ", name, width = max_widths[i]);
                            }
                            continue;
                        }
                        print!("{:<width$} ", name, width = max_widths[i]);
                    }
                    println!();
                }
            }
            for (i, info) in info_vec.iter().enumerate() {
                if color && name_vec[i] == "file_name" {
                    let is_dir = self.file_infos[index].is_dir;
                    let is_executable = self.file_infos[index].is_executable;
                    let ext = get_extension(&Path::new(&self.file_infos[index].file_name));
                    let hyperlink_path = format_file_hyperlink(Path::new(info));
                    let colored_info =
                        apply_color(info, is_dir, &ext, &ColorConfig::load_from_file(), color);
                    let color_split = colored_info.split(info).collect::<Vec<&str>>();
                    let color_prefix = color_split[0];
                    let color_suffix = if color_split.len() > 2 {
                        color_split[3]
                    } else {
                        ""
                    };
                    let prefix = if is_dir {
                        " - "
                    } else if is_executable {
                        " * "
                    } else {
                        "   "
                    };
                    print!(
                        "{}{}{:<width$}{}\x1b[0m ",
                        prefix,
                        color_prefix,
                        if hyperlink {hyperlink_path} else {info.to_string()},
                        color_suffix,
                        width = max_widths[i]
                    );
                    continue;
                }
                print!("{:>width$} ", info, width = max_widths[i]);
            }
            println!();
        }
    }
}
