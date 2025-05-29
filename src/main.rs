use clap::Parser;
use show::show::show_command;
use std::path::PathBuf;
mod merge;
mod show;
use merge::merge::merge_command;
mod search;
mod utils;
use search::search::search_command;
mod rename;
use rename::rename::rename_command;
mod ls;
use crate::ls::ls_command;
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
    /// todo: - 是文件夹，*是可执行文件，
    #[arg(default_value = ".")]
    directory: PathBuf,

    /// 与-l一起使用时，显示作者信息
    #[arg(short = 'A', long)]
    author: bool,

    /// 显示详细信息\
    /// 显示的条目：permission, modified, file_name, size
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

    /// 为文件或文件夹添加超链接
    #[arg(long)]
    hyperlink: bool,

    /// 显示每个文件的inode信息
    #[arg(short, long)]
    inode: bool,

    /// 按扩展名字母顺序排序
    #[arg(short = 'X', long)]
    ext_sort: bool,

    /// 按文件大小降序排序
    #[arg(short = 'S', long)]
    size_sort: bool,

    /// 显示文件树
    #[arg(short = 'T', long, default_value_t = 0)]
    tree: usize,

    /// 文件树每层最大显示行数
    #[arg(short = 'm', long, default_value_t = 10)]
    max_tree_lines: usize,

    /// 仅显示目录
    #[arg(short = 'D', long)]
    directories_only: bool,

    /// 仅显示文件
    #[arg(short = 'f', long)]
    files_only: bool,

    /// 显示完整路径
    #[arg(short = 'F', long)]
    full_path: bool,

    /// 高亮显示不同类型文件
    #[arg(short, long, default_value_t = false)]
    color: bool,

    /// 区分文件和文件夹
    #[arg(short = 'd', long, default_value_t = false)]
    differentiated: bool,

    /// 打印表头
    #[arg(long, default_value_t = false)]
    header: bool,

    /// 自定义显示的条目\
    /// 可选条目：size,file_name,modified,is_dir,author,inode,link_count,
    /// block_size,blocks,device,atime,ctime,mtime,is_executable,permission,group
    #[arg(short = 'C', long, value_delimiter = ',')]
    custom_show: Vec<String>,

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
    ///
    /// 支持正则表达式、多种高级模板匹配
    ///
    /// 默认通配符:
    /// - {source}: 整个文件名，包含前缀和后缀
    /// - {prefix}: 文件名前缀，比如 "example" 中的 "example"
    /// - {suffix}: 文件名后缀，比如 "example.txt" 中的 "txt"，"abc.c.d"中的"c.d"
    /// - {date_time:%format%}: 文件修改日期, %format%为格式控制字符，默认为%Y-%m-%d %H:%M:%S
    /// - {n}: 序号
    ///     - {n:start=1}: 起始值为1, 如1, 2...**默认起始值为1**
    ///     - {n:width=2}: 宽度为2，不足2位用0填充, 如001, 002...**默认宽度为0** <!-- 十六进制需要在0x后面补0-->
    ///     - {n:step=2}: 步长为2, 如1, 3, 5...**默认步长为1；步长只能是正数**
    ///     - {n:radix=16}: 进制为16, 如0x01, 0x02...**默认进制为10**
    ///     - {n:reverse=10}: 反向, 并从10开始, 如10, 9, 8...**默认不反向**
    /// - {+p}: 将p指定的内容转换成大写，如{+source}->ABC.TXT
    /// - {-p}: 将p指定的内容转换成小写，如{-source}->abc.txt
    /// - {p:l}: 对p指定的内容进行截取，l为截取的长度，如{source:3}->abc；只对{source}、{prefix}、{suffix}有效
    /// - {p:s:l}: 对p指定的内容进行截取，l为截取的长度，s为起始位置，如{source:1:3}->bca；只对{source}、{prefix}、{suffix}有效
    /// - {p:s-e}: 对p指定的内容进行截取，s为起始位置，e为结束位置，如{source:1-3}->bca；只对{source}、{prefix}、{suffix}有效
    /// - {rand:n}: 生成随机数，n为生成的随机数的长度，如{rand:3}->123 <!-- 需要保证不重复-->
    /// - 元数据：
    ///     - {image:width, height, manufacturer, model, datetime, location, ISO, aperture, exposure_time, focal_length, metering_mode, orientation, flash, white_balance}: 获取图片的元数据，如{exif:width}->1920
    ///     - {music:artist, album, title, year, genre, duration, bitrate, sample_rate, channels}: 获取音乐的元数据，如{music:artist}->Artist
    ///     - {video:width, height, duration, bitrate, frame_rate, codec, resolution, aspect_ratio}: 获取视频的元数据，如{video:width}->1920
    ///     -
    Rename {
        /// 要重命名的文件路径列表
        source: String,
        /// 重命名后的文件名
        target: String,
        /// 指定文件夹下面的文件进行重命名
        #[arg(short, long, default_value = ".")]
        directory: String,
        /// 开启正则表达式
        #[arg(short, long, default_value_t = false)]
        regex: bool,
        /// 开启模式匹配模式，是正则表达式+自定义变量匹配功能
        ///
        ///
        #[arg(short, long, default_value_t = false)]
        pattern: bool,
    },
    //todo: 批量移动、压缩文件、整合文件
}

// ** D:\docs\rust\bhw\big_homework\target\debug\rt.exe **
// ** /home/chenwn/RustRush-CLIKit/target/debug/rt **

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
            search_command(
                paths,
                keyword,
                search_content,
                regex,
                ignore_case,
                recursive,
            );
        }
        Some(SubCommands::Show { file_path, lines }) => {
            show_command(file_path, lines);
        }
        Some(SubCommands::Merge {
            file_paths,
            output,
            lines,
        }) => {
            merge_command(file_paths, output, lines);
        }
        Some(SubCommands::Rename {
            source,
            target,
            directory,
            regex,
            pattern,
        }) => {
            rename_command(source, target, directory, regex, pattern);
        }
        None => {
            ls_command(
                args.directory,
                args.author,
                args.long,
                args.all,
                args.recursive,
                args.human_readable,
                args.time_sort,
                args.reverse,
                args.hyperlink,
                args.inode,
                args.ext_sort,
                args.size_sort,
                args.directories_only,
                args.files_only,
                args.color,
                args.differentiated,
                args.header,
                &args.custom_show,
                args.full_path,
                args.tree,
                args.max_tree_lines,
            );
        }
    }
}
