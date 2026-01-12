//! 核心抽象层 (Core Abstractions)
//!
//! 定义了工具检测和镜像源配置的核心trait接口，遵循依赖倒置原则。
//! 所有具体工具实现都应该实现这些trait。

pub mod detector;
pub mod mirror;
pub mod command;

// 重新导出核心类型
pub use detector::{DetectionInfo, ToolDetector};
pub use mirror::{Mirror, MirrorConfigurator};
pub use command::{CommandOutput, CommandExecutor};

