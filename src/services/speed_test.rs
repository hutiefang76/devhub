//! 网络测速服务
//!
//! 提供镜像源速度测试功能，用于帮助用户选择最快的镜像源。

use reqwest::Client;
use std::time::{Duration, Instant};

/// 网络测速服务
///
/// 通过HTTP HEAD请求测试镜像源的响应延迟。
///
/// # 实现细节
/// - 使用HTTP HEAD方法（不下载内容，只获取响应头）
/// - 超时时间：5秒
/// - 测量从发送请求到收到响应的总时间
///
/// # 示例
/// ```no_run
/// use devhub::services::SpeedTestService;
///
/// #[tokio::main]
/// async fn main() {
///     let service = SpeedTestService::new();
///     let latency = service.test_mirror("https://mirrors.aliyun.com").await;
///
///     if latency < 1000 {
///         println!("镜像源延迟: {}ms (良好)", latency);
///     } else {
///         println!("镜像源延迟: {}ms (较慢)", latency);
///     }
/// }
/// ```
pub struct SpeedTestService {
    /// HTTP客户端（包含5秒超时配置）
    client: Client,
}

impl SpeedTestService {
    /// 创建新的测速服务实例
    ///
    /// # 配置
    /// - 超时时间：5秒
    /// - 自动跟随重定向
    /// - 支持HTTP/2
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap(),
        }
    }

    /// 测试单个镜像源的响应速度
    ///
    /// # 参数
    /// - `url`: 镜像源URL（如"https://mirrors.aliyun.com"）
    ///
    /// # 返回值
    /// - 成功：返回延迟毫秒数（如150表示150ms）
    /// - 失败/超时：返回`u64::MAX`（表示不可用）
    ///
    /// # 注意
    /// 返回值为`u64::MAX`表示镜像源不可用，应该在UI中特殊处理。
    pub async fn test_mirror(&self, url: &str) -> u64 {
        let start = Instant::now();

        match self.client.head(url).send().await {
            Ok(resp) if resp.status().is_success() => {
                start.elapsed().as_millis() as u64
            }
            _ => u64::MAX,  // 超时或失败
        }
    }

    /// 批量测试多个镜像源（顺序执行）
    ///
    /// # 参数
    /// - `urls`: 镜像源URL列表
    ///
    /// # 返回值
    /// 延迟列表，顺序与输入URL对应
    ///
    /// # 性能
    /// 当前为顺序执行，5个镜像源最多需要25秒。
    /// 未来可改为并发执行以提升速度。
    pub async fn test_mirrors(&self, urls: Vec<String>) -> Vec<u64> {
        let mut results = Vec::new();
        for url in urls {
            results.push(self.test_mirror(&url).await);
        }
        results
    }
}

impl Default for SpeedTestService {
    fn default() -> Self {
        Self::new()
    }
}

