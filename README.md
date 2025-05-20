# RustRush-CLIKit - 多功能文件系统工具集


一个用Rust实现的现代化命令行工具，集成文件管理、内容查看、批量操作等实用功能，支持彩色输出和语法高亮。

## 功能特性

| 模块       | 功能描述                                                                 |
|------------|--------------------------------------------------------------------------|
| **ls**     | 增强版目录列表，支持递归/排序/彩色输出                                   |
| **show**   | 带语法高亮的文件查看器（支持200+语言）                                   |
| **merge**  | 多文件合并工具，支持输出到文件并预览                                    |
| **search** | 支持正则的文件名和内容搜索（含上下文高亮）                               |
| **rename** | 批量重命名工具（支持正则和智能模板）                                    |
| **where**  | 快速定位当前工作目录                                                     |

## 安装方法

<!-- ### 通过Cargo安装
```bash
cargo install rustycli
``` -->

### 源码编译
```bash
git clone https://github.com/Chenwn1186/RustRush-CLIKit.git
cd RustRush-CLIKit
cargo build --release
```

## 使用说明

### 基础命令
```bash
# 列出当前目录（类似ls）
rt

# 长格式列表（类似ls -l）
rt -l

# 递归列出子目录
rt -R
```

### 彩色输出配置
```bash
# 启用颜色支持
rt --color

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

### 文件查看器
```bash
# 高亮显示前20行代码
rt show Cargo.toml -n 20

# 支持的扩展名示例：
# .rs .py .js .md .json .toml .yaml .html .css
```

### 文件合并
```bash
# 合并多个文件并输出前10行预览
rt merge file1.txt file2.txt -o merged.txt -n 10
```

### 智能搜索
```bash
# 文件名搜索（支持正则）
rt search -R "\.rs$" --regex

# 文件内容搜索（忽略大小写）
rt search "TODO" --search-content --ignore-case
```

## 批量重命名（开发中）

```bash
# 正则模式（将IMG_前缀改为PHOTO_）
rt rename --regex 's/IMG_/PHOTO_/' *.jpg

# 模板模式（日期+序号）
rt rename --pattern "{date_time}_{n:width=4}.{suffix}"
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

## 贡献指南

欢迎提交Issue和PR！请遵循以下规范：
- 新增功能需包含单元测试
- 修改颜色配置需同步更新文档
- 涉及文件操作的命令必须包含dry-run模式

## 许可证

Apache-2.0 License © 2025 Chenwn1186
