use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 软件检测结果
///
/// 包含软件是否安装、版本号和安装路径等信息。
///
/// # 示例
/// ```no_run
/// let info = DetectionInfo {
///     installed: true,
///     version: Some("3.14.2".to_string()),
///     path: Some(PathBuf::from("/usr/bin/python3")),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionInfo {
    /// 是否已安装
    pub installed: bool,
    /// 版本号（如"3.14.2"）
    pub version: Option<String>,
    /// 安装路径
    pub path: Option<PathBuf>,
}

/// 软件检测器接口
///
/// 用于检测系统中是否安装了特定软件，并获取其版本和路径信息。
/// 遵循单一职责原则：只负责检测，不负责安装或配置。
///
/// # 设计模式
/// 策略模式 - 不同的软件可以有不同的检测策略：
/// - Shell命令检测（which + --version）
/// - 注册表检测（Windows）
/// - 文件系统检测（固定路径）
///
/// # 示例
/// ```no_run
/// use devhub::core::ToolDetector;
///
/// fn check_installation(detector: &impl ToolDetector) -> anyhow::Result<()> {
///     let info = detector.detect()?;
///
///     if info.installed {
///         println!("已安装，版本: {:?}", info.version);
///     } else {
///         println!("未安装");
///     }
///
///     Ok(())
/// }
/// ```
pub trait ToolDetector: Send + Sync {
    /// 检测软件是否安装
    ///
    /// # 返回值
    /// - `Ok(DetectionInfo)` - 检测成功，包含安装信息
    /// - `Err(...)` - 检测失败（如权限不足）
    fn detect(&self) -> Result<DetectionInfo>;
}

