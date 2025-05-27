#import "@preview/codelst:2.0.2": sourcecode
#import "@preview/mitex:0.2.5": *
#import "@preview/muchpdf:0.1.0": muchpdf
#import "@preview/fletcher:0.5.7": *
#set page(width: 8.5in, height: 11in)
#set text(font: ("Times New Roman", "SimSun"), size: 12pt)

#let title(content) = {
  align(center)[
    #text(font: "SimHei", size: 20pt)[
      #content
    ]
  ]
}

#let ctitle(content, radius: 5pt, fill: blue, stroke: 0pt + black, textColor: white, textAlign: left) = {
  align(textAlign)[
    #rect(radius: radius, fill: fill, stroke: stroke)[
      #text(fill: textColor)[
        #content
      ]
    ]
  ]
}

// #muchpdf(read("1.pdf", encoding: none))
#title[Rust课程：大作业需求收集]


#ctitle()[
  == 项目需求
]
{source}
- 项目名称：基于rust的命令行工具（rt：rust commandline tools）
- 项目简介：支持各种命令行工具的功能，如ls、cat、grep、find等
- 项目目标：支持彩色终端文本显示，操作简单，功能完善
- 项目功能：
  - 支持ls、cat、grep、find等命令
  - 支持彩色终端文本显示
  - 支持文件搜索、文件比较、批量重命名
  - 支持预览文本文件，并且支持高亮显示 done
  - 支持文件大小显示 done
    