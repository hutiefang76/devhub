//! Integration tests for DevHub Pro

use devhub::sources::{get_manager, SUPPORTED_TOOLS};
use devhub::types::Mirror;
use devhub::utils::benchmark_mirrors;

#[test]
fn test_supported_tools_list() {
    // 验证支持的工具列表不为空
    assert!(!SUPPORTED_TOOLS.is_empty());

    // 验证包含核心工具
    assert!(SUPPORTED_TOOLS.contains(&"pip"));
    assert!(SUPPORTED_TOOLS.contains(&"npm"));
    assert!(SUPPORTED_TOOLS.contains(&"cargo"));
    assert!(SUPPORTED_TOOLS.contains(&"go"));
    assert!(SUPPORTED_TOOLS.contains(&"maven"));
    assert!(SUPPORTED_TOOLS.contains(&"docker"));
}

#[test]
fn test_get_manager_valid_tools() {
    // 测试所有支持的工具都能获取到管理器
    for tool in SUPPORTED_TOOLS {
        let result = get_manager(tool);
        assert!(result.is_ok(), "Failed to get manager for tool: {}", tool);
    }
}

#[test]
fn test_get_manager_invalid_tool() {
    // 测试无效工具返回错误
    let result = get_manager("nonexistent_tool");
    assert!(result.is_err());
}

#[test]
fn test_mirror_list_not_empty() {
    // 验证每个工具都有镜像源列表
    for tool in SUPPORTED_TOOLS {
        if let Ok(manager) = get_manager(tool) {
            let mirrors = manager.list_candidates();
            assert!(!mirrors.is_empty(), "Mirror list is empty for tool: {}", tool);
        }
    }
}

#[test]
fn test_mirror_struct() {
    // 测试 Mirror 结构体
    let mirror = Mirror::new("Test", "https://test.com/");
    assert_eq!(mirror.name, "Test");
    assert_eq!(mirror.url, "https://test.com/");
}

#[test]
fn test_pip_mirrors_contain_china_sources() {
    // 验证 pip 镜像源包含中国镜像
    let manager = get_manager("pip").unwrap();
    let mirrors = manager.list_candidates();

    let mirror_names: Vec<&str> = mirrors.iter().map(|m| m.name.as_str()).collect();

    // 验证包含常见的中国镜像源
    assert!(mirror_names.iter().any(|n| n.contains("阿里") || n.contains("Aliyun") || n.contains("aliyun")),
        "pip mirrors should contain Aliyun source");
    assert!(mirror_names.iter().any(|n| n.contains("清华") || n.contains("Tuna") || n.contains("tuna")),
        "pip mirrors should contain Tsinghua source");
}

#[test]
fn test_npm_mirrors_contain_china_sources() {
    // 验证 npm 镜像源包含中国镜像
    let manager = get_manager("npm").unwrap();
    let mirrors = manager.list_candidates();

    let mirror_names: Vec<&str> = mirrors.iter().map(|m| m.name.as_str()).collect();

    // npm 应该包含 taobao/npmmirror 等中国镜像
    assert!(mirror_names.iter().any(|n|
        n.to_lowercase().contains("taobao") ||
        n.to_lowercase().contains("npmmirror") ||
        n.to_lowercase().contains("淘宝")),
        "npm mirrors should contain Chinese source");
}

#[test]
fn test_mirror_url_format() {
    // 验证镜像 URL 格式正确
    for tool in SUPPORTED_TOOLS {
        if let Ok(manager) = get_manager(tool) {
            let mirrors = manager.list_candidates();
            for mirror in mirrors {
                // cargo 使用 sparse+ 前缀
                let url = mirror.url.trim_start_matches("sparse+");
                assert!(url.starts_with("http://") || url.starts_with("https://"),
                    "Mirror URL should start with http:// or https://: {} - {}", tool, mirror.url);
            }
        }
    }
}

#[tokio::test]
async fn test_benchmark_mirrors_returns_results() {
    // 测试测速函数返回结果
    let mirrors = vec![
        Mirror::new("Test1", "https://httpbin.org/get"),
        Mirror::new("Test2", "https://httpbin.org/delay/10"), // 会超时
    ];

    let results = benchmark_mirrors(mirrors).await;

    assert_eq!(results.len(), 2);

    // 第一个应该成功（延迟不是 MAX）
    assert!(results.iter().any(|r| r.mirror.name == "Test1"));
    assert!(results.iter().any(|r| r.mirror.name == "Test2"));
}

#[test]
fn test_cargo_mirrors() {
    // 验证 cargo 镜像源配置
    let manager = get_manager("cargo").unwrap();
    let mirrors = manager.list_candidates();

    // cargo 镜像通常是 crates-io 替代源
    assert!(!mirrors.is_empty(), "Cargo should have mirror sources");

    for mirror in mirrors {
        // cargo 镜像 URL 应该包含 crates 相关路径
        assert!(mirror.url.contains("crates") || mirror.url.contains("cargo") || mirror.url.contains("rust"),
            "Cargo mirror URL should be related to crates: {}", mirror.url);
    }
}

#[test]
fn test_docker_mirrors() {
    // 验证 docker 镜像源配置
    let manager = get_manager("docker").unwrap();
    let mirrors = manager.list_candidates();

    assert!(!mirrors.is_empty(), "Docker should have mirror sources");

    // docker 镜像是 registry 地址，验证 URL 格式正确
    for mirror in mirrors {
        assert!(mirror.url.starts_with("http://") || mirror.url.starts_with("https://"),
            "Docker mirror should have valid URL: {}", mirror.url);
    }
}

#[test]
fn test_go_mirrors() {
    // 验证 go 镜像源配置
    let manager = get_manager("go").unwrap();
    let mirrors = manager.list_candidates();

    assert!(!mirrors.is_empty(), "Go should have mirror sources");

    // go 镜像是 proxy 地址
    for mirror in mirrors {
        assert!(mirror.url.contains("proxy") || mirror.url.contains("goproxy") ||
                mirror.url.contains("go") || mirror.url.contains("aliyun"),
            "Go mirror should be a proxy address: {}", mirror.url);
    }
}

#[test]
fn test_maven_mirrors() {
    // 验证 maven 镜像源配置
    let manager = get_manager("maven").unwrap();
    let mirrors = manager.list_candidates();

    assert!(!mirrors.is_empty(), "Maven should have mirror sources");
}

#[test]
fn test_all_tools_have_name() {
    // 验证所有管理器都有名称
    for tool in SUPPORTED_TOOLS {
        if let Ok(manager) = get_manager(tool) {
            let name = manager.name();
            assert!(!name.is_empty(), "Manager should have a name");
            assert_eq!(name, *tool, "Manager name should match tool name");
        }
    }
}
