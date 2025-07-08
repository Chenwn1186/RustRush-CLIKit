# RustRush-CLIKit - 多功能文件系统工具集

一个用Rust实现的Linux现代化命令行工具，集成文件管理、内容查看、批量重命名等实用功能，支持彩色输出和语法高亮显示。

## 功能特性

| 模块       | 功能描述                                                                 |
|------------|--------------------------------------------------------------------------|
| **ls**     | 增强版目录列表，支持递归/排序/彩色输出/文件树视图                         |
| **show**   | 带语法高亮的文件查看器（支持200+语言）                                   |
| **merge**  | 多文件合并工具，支持输出到文件并预览                                    |
| **search** | 支持正则的文件名和内容搜索（含上下文高亮）                               |
| **rename** | 批量重命名工具（支持正则和智能模板）                                    |

## 安装方法

### 源码编译
```bash
# 目前只支持Linux平台
git clone https://github.com/Chenwn1186/RustRush-CLIKit.git
cd RustRush-CLIKit
cargo build --release
```
## 创新点
### 1. 简单而高效强大的文件筛选功能
#### 例子1：按文件大小筛选
```
筛选条件：文件大小
格式：xx-yyZ、xxZ、-yyZ（需要用双引号包含）
xx: 起始大小，数字
yy: 结束大小，数字
Z: 可以是k、m、g、t、p，表示KB、MB、GB、TB、PB
例如：100k-200m表示100KB到200MB之间的文件
范围可以叠加，用逗号分隔，例如：100k-200m,10g表示100KB到200MB之间的文件，或者10GB以上的文件
```
#### 例子2：按日期筛选（原创格式）
```
筛选条件：文件修改时间
格式：xx:yyZ、xxZ:、:yyZ、xxZ、special_datetime
xx: 起始时间，数字
yy: 结束时间，数字
Z: 可以是y、m、d、h、M、s，表示年、月、日、时、分、秒
xx:yyZ 表示在Z的单位内，从xx开始到yy结束的时间范围
xxZ: 表示从xx开始到当前时间的时间范围
:yyZ 表示yyZ及往前的所有时间范围
xxZ 表示xxZ表示的时间范围，时间跨度与Z的单位相同
special_datetime: 特殊时间，如today、yesterday、this_month、last_month、this_year、last_year
范围可以用逗号分隔以取并集，例如：2021:2022y,10m表示在2021年到2022年或者在10月份
单个时间范围内的不同时间单位用“-”分隔，例如：2021y-7:8m-10:20d-:10h表示在2021年7月或8月的10日到20日，并且在00:00到10:00之间的时间范围
可以用括号来约定时间点，例如: (2021y-7m-10d-0h):(2021y-8m-20d-10h)表示在2021年7月10日00:00到2021年8月20日10:00之间的时间范围
```
#### 例子3：按文件类型筛选
```
筛选条件：文件类型
支持的文件类型：
1.text: 纯文本文件，包括代码文件、配置文件、日志文件等
2.image: 图像文件
3.audio: 音频文件
4.video: 视频文件
5.document: 文档文件，包括PDF、Word、Excel等
6.archive: 压缩文件，包括zip、tar、rar等
7.executable: 可执行文件
8.font: 字体文件
9.library: 库文件
10.database: 数据库文件
11.3D_model: 三维模型文件
12.vitural_box: 虚拟机、容器等虚拟环境文件
13.dump: 内存转储文件
在文件类型前面加上!表示不匹配该类型的文件
例如：!text表示不匹配纯文本文件
多个类型可以用逗号分隔，例如：text,image表示匹配纯文本文件和图像文件
```

### 2. 将显示树形文件结构图功能嵌入到rt命令中
ls命令不支持显示树形文件结构图，本工具支持这个功能，并且支持彩色显示、更多信息显示和自定义递归深度等参数。

### 3. 可自定义所有的文件信息展示
如自定义只显示文件名、ctime、mtime、inode，并且支持显示表头。

### 4. 强大的批量重命名文件功能
- 自主设计的通配符功能
  - 例子1：一键为当前目录下所有txt文件按字典序添加序号：
  `rt rename ".*\.txt" "{source}{n}\.{suffix}" -r`
  ```
  result:
  a.txt -> a0.txt
  b.txt -> b1.txt
  c.txt -> c2.txt
  ......
  ```
  - 例子2：为当前目录下所有png图片文件名添加长度和宽度
  `rt rename ".*\.png" "{source}:{image:width}x{image:height}.{suffix}" -r`
  ```
  result:
  a.png -> a:100x200.png
  b.png -> b:1080x1920.png
  c.png -> c:400x400.png
  ......
  ```
  - 完整功能：
  ```
    默认通配符:
  - {source}: 整个文件名，包含前缀和后缀
  - {prefix}: 文件名前缀，比如 "example.txt" 中的 "example"
  - {suffix}: 文件名后缀，比如 "example.txt" 中的 "txt"，"abc.c.d"中的"c.d"
  - { n }: 序号，从0开始，如0, 1, 2...
      - {n:start=1}: 起始值为1, 如1, 2...**默认起始值为1**
      - {n:width=2}: 宽度为2，不足2位用0填充, 如001, 002...**默认宽度为0** <!-- 十六进制需要在0x后面补0-->
      - {n:step=2}: 步长为2, 如1, 3, 5...**默认步长为1；步长只能是正数**
      - {n:radix=16}: 进制为16, 如0x01, 0x02...**默认进制为10**
      - {n:reverse}: 将生成的列表反向, **默认不反向, 并且不是十进制时无效**
  - {rand:n}: 生成随机数，n为生成的随机数的长度，如{rand:3}->123 <!-- 尽量保证不重复-->
  - 元数据：
      - {image:width, height, make, model, create_date, location, ISO,
      aperture, exposure_time, focal_length,
      orientation, flash}: 获取图片的元数据，如{exif:width}->1920
      - {audio:artist, album, title, year, genre, duration, disc, date_recorded, date_released}: 获取音乐的元数据，如{music:artist}->Artist
      - {video:width, height, duration, bitrate, frame_rate, codec, resolution, aspect_ratio}: 获取视频的元数据，如{video:width}->1920
  ```
- 新颖强大的模板匹配功能
  - 将文件名的特定部分提取出来作为变量使用
  - 例子：对于文件`abc123.txt`，表达式`{var1}123.*`将abc匹配到变量var1中，目的文件名`{var1}{+var1}.txt`将会构造出新文件名`abcABC.txt`
  - 配合特殊功能会有更强大的截取、大小写转换等功能

- 特殊功能(只对模板变量或通配符有效)：
  - {+p}: 将p指定的内容转换成大写，如{+source}->ABC.TXT
  - {-p}: 将p指定的内容转换成小写，如{-source}->abc.txt
  - {p:l}: 对p指定的内容进行截取，l为截取的长度，如{source:3}->abc；
  - {p:s:l}: 对p指定的内容进行截取，l为截取的长度，s为起始位置，如{source:1:3}->bca；
  - {p:s-e}: 对p指定的内容进行截取，s为起始位置，e为结束位置，如{source:1-3}->bca；
  - 注：截取和大小写转换符号可同时使用
## 使用说明

### 基础命令 (类似ls)
```bash
# 列出当前目录（类似ls）
rt

# 长格式列表（类似ls -l）
rt -l

# 递归列出子目录
rt -R

# 按修改时间排序并反转
rt --time-sort --reverse

# 显示文件树（深度2，每层最多10项）
rt -T 2 -m 10

# 仅显示目录并按大小排序
rt -D --size-sort

# 自定义显示字段（权限、大小、文件名）
rt -C permission,size,file_name
```

**亮点**：
- **智能彩色系统**：基于文件类型和扩展名的自动着色（可通过`color_config.json`自定义）
- **多维度排序**：支持按大小、修改时间、扩展名等多维度排序
- **交互式文件树**：可指定深度和显示数量的**树形视图**
- **自定义列显示**：可按需选择显示字段（如inode、权限、作者、文件修改时间等18种属性）
- **高级筛选**：支持按文件类型、大小范围、修改时间等条件快速筛选；时间范围支持并集

### 文本查看器 (show)
```bash
# 高亮显示前20行代码
rt show Cargo.toml -n 20

# 查看Rust代码文件（自动语法高亮）
rt show src/main.rs

# 查看JSON配置文件
rt show color_config.json
```

**亮点**：
- **自动语法检测**：支持200+编程语言的语法高亮
- **高效行限制**：可指定只显示前N行，快速预览大文件
- **低内存占用**：流式读取文件，避免加载整个文件到内存

### 文本合并 (merge)
```bash
# 合并多个文本文件并输出前10行预览
rt merge file1.txt file2.txt -o merged.txt -n 10

# 合并代码文件并直接查看结果
rt merge src/ls.rs src/utils.rs -o combined.rs -n 50
```

**亮点**：
- **提前预览**：可提前预览合并后前N行结果
- **智能换行**：自动在文件间添加分隔换行符
- **错误容忍**：跳过无法读取的文件，继续处理其他文件

### 智能搜索 (search)
```bash
# 文件名搜索（支持正则）
rt search -R 2 --regex "\.rs$"

# 文件内容搜索（忽略大小写）
rt search "TODO" --search-content --ignore-case

# 按文件大小和类型筛选（100KB-200MB的文本文件）
rt search "error" --search-content -S "100k-200m" -t text

# 按修改时间筛选（过去7天内修改的Rust文件）
rt search --regex "\.rs$" --modified "7d"
```

**亮点**：
- **多维度筛选**：结合文件大小、类型、修改时间等条件精确搜索
- **递归深度控制**：可指定递归搜索深度，避免过度搜索
- **内容高亮**：搜索结果中高亮显示匹配关键字
- **管道支持**：可接收标准输入进行内容搜索

### 批量重命名 (rename)
```bash
# 正则模式（将IMG_前缀改为PHOTO_）
rt rename --regex 's/IMG_/PHOTO_/' *.jpg

# 模板模式（日期+序号）
rt rename --pattern "{date_time}_{n:width=4}.{suffix}" *.png

# 移动并重命名（检查目标文件夹）
rt rename --pattern "{n}.txt" *.log --move-to ./logs
```

**亮点**：
- **安全检查**：自动检测目标文件是否存在，避免意外覆盖
- **原子操作**：批量操作要么全部成功，要么全部取消
- **元数据支持**：可提取图片EXIF、音频ID3等元数据用于命名
- **智能模板**：支持日期、序号、哈希等多种动态模板变量
- **移动整合**：重命名的同时支持移动文件到指定目录

## 彩色输出配置
```bash
# 自定义颜色（编辑color_config.json）
{
  "file_ext_colors": [
    ["rs", "#FF00FF"],   // Rust文件品红色
    ["txt", "#FFFF00"]   // 文本文件黄色
  ],
  "special_dir_colors": [
    ["src", "#00FFFF"],  // src目录青色
    ["docs", "#FF4500"]  // docs目录橙红
  ]
}
```

## 依赖项

| 库名称         | 用途                     |
|----------------|--------------------------|
| clap           | 命令行参数解析           |
| colored        | 终端彩色输出             |
| syntect        | 语法高亮引擎             |
| chrono         | 时间日期处理             |
| serde          | 配置文件序列化           |
| fancy-regex    | 增强正则表达式支持       |


## 许可证

Apache-2.0 License © 2025 Chenwn1186
