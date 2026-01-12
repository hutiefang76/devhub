//! 基础服务层 (Infrastructure Services)
//!
//! 提供底层的Shell命令执行和网络测速服务。
//! 这些服务被上层的工具实现所使用。

pub mod shell;
pub mod speed_test;

// 重新导出常用类型
pub use shell::ShellExecutor;
pub use speed_test::SpeedTestService;

