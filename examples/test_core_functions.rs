use devhub::*;
use devhub::core::{ToolDetector, MirrorConfigurator};

fn test_python_detection() {
    println!("=== 测试 Python 检测 ===");
    let detector = tools::python::PythonDetector::new();
    match detector.detect() {
        Ok(info) => {
            println!("✅ Python 检测成功:");
            println!("   - 已安装: {}", info.installed);
            if let Some(version) = info.version {
                println!("   - 版本: {}", version);
            }
            if let Some(path) = info.path {
                println!("   - 路径: {}", path.display());
            }
        }
        Err(e) => {
            println!("❌ Python 检测失败: {}", e);
        }
    }
    println!();
}

fn test_pip_mirrors() {
    println!("=== 测试 pip 镜像源 ===");
    let config = tools::python::PipMirror::new();

    println!("可用镜像源:");
    for mirror in config.list_mirrors() {
        println!("   - {}: {}", mirror.name, mirror.url);
    }

    match config.get_current() {
        Ok(Some(current)) => {
            println!("\n当前镜像源: {}", current);
        }
        Ok(None) => {
            println!("\n当前使用默认配置");
        }
        Err(e) => {
            println!("\n❌ 获取当前镜像源失败: {}", e);
        }
    }
    println!();
}

fn main() {
    println!("\n🚀 DevHub Pro - 功能验证测试\n");
    println!("========================================\n");

    test_python_detection();
    test_pip_mirrors();

    println!("========================================");
    println!("\n✅ 所有核心功能测试完成!\n");
}

