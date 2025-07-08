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
/// 2. 搜索文件、文件夹和文件内容：支持正则表达式（含Perl扩展）
/// 3. 合并文本文件
/// 4. 打开文本文件并高亮显示前 n 行
/// 5. 批量重命名：支持正则表达式、多种高级模板匹配
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 要列出内容的目录路径，默认为当前目录
    #[arg(default_value = ".")]
    directory: PathBuf,

    /// 与-l一起使用时，显示作者信息
    #[arg(short = 'A', long)]
    author: bool,

    /// 显示详细信息
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
    //todo:文件树支持显示隐藏文件，支持条件筛选
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
        /// 要搜索的文件路径列表，用逗号分隔，默认为当前目录
        #[arg(short, long, default_value = ".", value_delimiter = ',')]
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
        /// 递归搜索的深度，默认为1，即不递归搜索
        #[arg(short = 'R', long, default_value_t = 1)]
        recursive_depth: usize,

        /// 筛选条件：文件大小
        /// 格式：xx-yyZ、xxZ、-yyZ（需要用双引号包含）
        /// xx: 起始大小，数字
        /// yy: 结束大小，数字
        /// Z: 可以是k、m、g、t、p，表示KB、MB、GB、TB、PB
        /// 例如：100k-200m表示100KB到200MB之间的文件
        /// 范围可以叠加，用逗号分隔，例如：100k-200m,10g表示100KB到200MB之间的文件，或者10GB以上的文件
        #[arg(short = 'S', long)]
        size: Option<String>,

        //不是后缀的原因：文件类型范围更大，会更方便；指定的后缀可以只用正则表达式匹配
        /// 筛选条件：文件类型
        ///
        /// 支持的文件类型：
        /// 1.text: 纯文本文件，包括代码文件、配置文件、日志文件等
        /// 2.image: 图像文件
        /// 3.audio: 音频文件
        /// 4.video: 视频文件
        /// 5.document: 文档文件，包括PDF、Word、Excel等
        /// 6.archive: 压缩文件，包括zip、tar、rar等
        /// 7.executable: 可执行文件
        /// 8.font: 字体文件
        /// 9.library: 库文件
        /// 10.database: 数据库文件
        /// 11.3D_model: 三维模型文件
        /// 12.vitural_box: 虚拟机、容器等虚拟环境文件
        /// 13.dump: 内存转储文件
        ///
        /// 在文件类型前面加上!表示不匹配该类型的文件
        /// 例如：!text表示不匹配纯文本文件
        /// 多个类型可以用逗号分隔，例如：text,image表示匹配纯文本文件和图像文件
        #[arg(short = 't', long)]
        file_type: Option<String>,

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
        #[arg(short = 'm', long)]
        modified: Option<String>,

        /// 筛选条件：文件访问时间
        /// 格式与modified相同
        #[arg(short = 'a', long)]
        accessed: Option<String>,

        /// 筛选条件：文件创建时间
        /// 格式与modified相同
        #[arg(short = 'c', long)]
        created: Option<String>,

        /// 筛选条件：文件权限
        /// 格式：rwxrwxrwx
        /// r: 可读
        /// w: 可写
        /// x: 可执行
        /// -: 无权限
        /// 例如：r-xr-xr-x表示可读、可执行，不可写
        #[arg(short = 'p', long)]
        permission: Option<String>,

        /// 筛选条件：文件所有者
        /// uid或者用户名
        /// 优先搜索用户名，找不到再搜索uid
        #[arg(short = 'o', long)]
        owner: Option<String>,

        /// 筛选条件：文件所属组
        /// gid或者组名
        /// 优先搜索组名，找不到再搜索gid
        #[arg(short = 'g', long)]
        group: Option<String>,
    },
    /// 批量重命名
    /// 支持正则表达式、多种高级模板匹配
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
    /// - {rand:n}: 生成随机数，n为生成的随机数的长度，如{rand:3}->123 <!-- 尽量保证不重复-->
    /// - 元数据：
    ///     - {image:width, height, make, model, create_date, location, ISO, 
    ///     aperture, exposure_time, focal_length, 
    ///     orientation, flash}: 获取图片的元数据，如{exif:width}->1920
    ///     - {audio:artist, album, title, year, genre, duration, disc, date_recorded, date_released}: 获取音乐的元数据，如{music:artist}->Artist
    ///     - {video:width, height, duration, bitrate, frame_rate, codec, resolution, aspect_ratio}: 获取视频的元数据，如{video:width}->1920
    /// 特殊功能(只对模板或通配符有效)： 
    /// - {+p}: 将p指定的内容转换成大写，如{+source}->ABC.TXT
    /// - {-p}: 将p指定的内容转换成小写，如{-source}->abc.txt
    /// - {p:l}: 对p指定的内容进行截取，l为截取的长度，如{source:3}->abc；只对{source}、{prefix}、{suffix}有效
    /// - {p:s:l}: 对p指定的内容进行截取，l为截取的长度，s为起始位置，如{source:1:3}->bca；只对{source}、{prefix}、{suffix}有效
    /// - {p:s-e}: 对p指定的内容进行截取，s为起始位置，e为结束位置，如{source:1-3}->bca；只对{source}、{prefix}、{suffix}有效
    /// 如果不开启模板匹配或通配符功能，就无法批量重命名
    Rename {
        /// 要重命名的文件路径列表
        source: String,
        /// 重命名后的文件名
        target: String,
        /// 指定文件夹下面的文件进行重命名，默认为当前目录
        #[arg(short, long, default_value = ".")]
        directory: String,
        /// 开启正则表达式
        #[arg(short, long, default_value_t = false)]
        regex: bool,
        /// 开启模式匹配模式，是正则表达式+自定义变量匹配功能
        /// 基本语法：在正则表达式里添加用大括号包围的变量名，变量名可以是任意字母的组合，如{var}；但不能与通配符冲突！
        /// target部分不是正则表达式；在target的变量前加上+表示转换为大写，加-表示转换为小写，如{+var1}、{-var1}
        /// 例子1： source: "{pre_name}-{other}\.{ext}", target: "{pre_name}.{ext}"，将会在匹配到的文件名中删除"-{other}"
        /// 例子2： source: "abc{test}\.{ext}", target: "{test}{n:start=1}.{ext}", 将会删除所有abc前缀，并且添加从1开始的序号
        #[arg(short, long, default_value_t = false)]
        pattern: bool,

        /// 开启通配符功能
        #[arg(short, long, default_value_t = false)]
        wildcard: bool,

        /// 开启替换功能，和普通的搜索替换功能一致
        #[arg(short = 'R', long, default_value_t = false)]
        replace: bool,

        // /// 递归深度，默认为1
        // #[arg(short = 'd', long, default_value_t = 1)]
        // recursive_depth: usize,

        /// 重命名后移动到新文件夹
        #[arg(short = 'm', long)]
        move_to: Option<String>,

        // /// 重命名后复制到新文件夹
        // #[arg(short = 'c', long)]
        // copy_to: Option<String>,

        /// 显示详细信息，如变量对应的实际值等
        #[arg(short = 'i', long, default_value_t = false)]
        info: bool,

        
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
            recursive_depth,
            size,
            file_type,
            modified,
            accessed,
            created,
            permission,
            owner,
            group,
        }) => {
            search_command(
                paths,
                keyword,
                search_content,
                regex,
                ignore_case,
                recursive_depth,
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
            wildcard,
            move_to,
            info,
            replace,
        }) => {
            let _ = rename_command(
                source,
                target,
                directory,
                regex,
                pattern,
                wildcard,
                move_to
            );
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
