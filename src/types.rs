use serde::{Deserialize, Serialize};

/// 镜像源定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mirror {
    pub name: String,
    pub url: String,
}

impl Mirror {
    pub fn new(name: &str, url: &str) -> Self {
        Self {
            name: name.to_string(),
            url: url.to_string(),
        }
    }
}

/// 测速结果
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub mirror: Mirror,
    pub latency_ms: u64,
}

impl BenchmarkResult {
    pub fn is_timeout(&self) -> bool {
        self.latency_ms == u64::MAX
    }
}

/// 工具检测信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionInfo {
    pub name: String,
    pub installed: bool,
    pub version: Option<String>,
    pub path: Option<String>,
}

impl DetectionInfo {
    pub fn not_found(name: &str) -> Self {
        Self {
            name: name.to_string(),
            installed: false,
            version: None,
            path: None,
        }
    }

    pub fn found(name: &str, version: &str, path: &str) -> Self {
        Self {
            name: name.to_string(),
            installed: true,
            version: Some(version.to_string()),
            path: Some(path.to_string()),
        }
    }
}
