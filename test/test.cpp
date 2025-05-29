#include <iostream>
#include <fstream>
#include <filesystem>
#include <vector>
#include <string>

namespace fs = std::filesystem;

// 定义文件扩展名和颜色对应关系
const std::vector<std::pair<std::string, std::string>> file_ext_colors = {
    {"rs", "#FF00FF"},     // 品红色
    {"txt", "#FFFF00"},    // 黄色
    {"exe", "#00FF00"},    // 绿色
    {"md", "#00FFFF"},     // 青色
    {"py", "#FF4500"},     // 橙红色
    {"json", "#0000FF"},   // 蓝色
    {"toml", "#4B0082"},   // 靛蓝色
    {"yml", "#8A2BE2"},    // 蓝色紫罗兰色
    {"yaml", "#8A2BE2"},   // 蓝色紫罗兰色
    {"lock", "#008080"},   // 水鸭色
    {"c", "#FF69B4"},      // 深粉色
    {"cpp", "#FF69B4"},    // 深粉色
    {"h", "#FF69B4"},      // 深粉色
    {"hpp", "#FF69B4"},    // 深粉色
    {"cs", "#DA70D6"},     // 淡紫色
    {"java", "#FFA500"},   // 橙色
    {"js", "#FFD700"},     // 金色
    {"ts", "#4169E1"},     // 皇家蓝色
    {"html", "#E34C26"},   // HTML 官方橙色
    {"css", "#264DE4"},    // CSS 官方蓝色
    {"php", "#777BB4"},    // PHP 官方紫色
    {"rb", "#CC342D"},     // Ruby 官方红色
    {"go", "#00ADD8"},     // Go 语言官方蓝色
    {"swift", "#F05138"},  // Swift 官方橙色
    {"kt", "#F88900"},     // Kotlin 官方橙色
    {"kts", "#F88900"},    // Kotlin 官方橙色
    {"sh", "#4EAA25"},     // 绿色
    {"bat", "#4EAA25"},    // 绿色
    {"ps1", "#012456"},    // PowerShell 官方蓝色
    {"psm1", "#012456"},   // PowerShell 官方蓝色
    {"psd1", "#012456"},   // PowerShell 官方蓝色
    {"ps1xml", "#012456"}  // PowerShell 官方蓝色
};

// 定义特殊目录和颜色对应关系
const std::vector<std::pair<std::string, std::string>> special_dir_colors = {
    {"target", "#0000FF"},   // 蓝色
    {"src", "#00FFFF"},      // 青色
    {"bin", "#00FF00"},      // 绿色
    {"lib", "#FF00FF"},      // 品红色
    {"include", "#FFFF00"},  // 黄色
    {"docs", "#FF4500"},     // 橙红色
    {"examples", "#FF4500"}, // 橙红色
    {"test", "#FF6347"},     // 番茄红色
    {"vendor", "#FF6347"},   // 番茄红色
    {"build", "#FF6347"},    // 番茄红色
    {"out", "#FF6347"},      // 番茄红色
    {"dist", "#FF6347"},     // 番茄红色
    {"node_modules", "#3C873A"}, // Node.js 官方绿色
    {"public", "#FFA500"},   // 橙色
    {"assets", "#FFA500"},   // 橙色
    {"styles", "#264DE4"},   // CSS 官方蓝色
    {"scripts", "#FFD700"},  // 金色
    {"images", "#87CEEB"},   // 天蓝色
    {"fonts", "#808080"},    // 灰色
    {"data", "#9370DB"},     // 暗紫色
    {"config", "#4B0082"},   // 靛蓝色
    {"logs", "#A52A2A"},     // 棕色
    {"tmp", "#A52A2A"},      // 棕色
    {"cache", "#A52A2A"},    // 棕色
    {"backup", "#A52A2A"},   // 棕色
    {"old", "#A52A2A"},      // 棕色
    {"temp", "#A52A2A"},     // 棕色
    {"draft", "#A52A2A"},    // 棕色
    {"unfinished", "#A52A2A"} // 棕色
};

void create_test_files(const fs::path& base_dir) {
    // 创建特殊目录
    for (const auto& [dir_name, _] : special_dir_colors) {
        fs::create_directories(base_dir / dir_name);
    }

    // // 在每个目录中创建对应类型的文件（不再创建所有类型的文件）
    // for (const auto& [dir_name, _] : special_dir_colors) {
    //     // 为每个目录创建与其名称相关的文件类型
    //     std::string ext = dir_name; // 使用目录名作为扩展名
    //     std::ofstream file(base_dir / dir_name / ("test." + ext));
    //     file << "This is a test " << ext << " file in " << dir_name << " directory.\n";
    // }

    // 在根目录创建各种类型的文件
    for (const auto& [ext, _] : file_ext_colors) {
        std::ofstream file(base_dir / ("root_test." + ext));
        file << "This is a test " << ext << " file in root directory.\n";
    }
}

int main() {
    const fs::path test_dir = "test_files";
    
    try {
        fs::remove_all(test_dir); // 先删除旧的测试目录
        fs::create_directories(test_dir);
        create_test_files(test_dir);
        std::cout << "测试文件和目录已成功生成在: " << fs::absolute(test_dir) << std::endl;
    } catch (const std::exception& e) {
        std::cerr << "错误: " << e.what() << std::endl;
        return 1;
    }

    return 0;
}