use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Shell命令执行结果
///
/// 包含命令的退出状态、标准输出和标准错误。
///
/// # 示例
/// ```no_run
/// let output = CommandOutput {
///     status: true,
///     stdout: "Python 3.14.2\n".to_string(),
///     stderr: String::new(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandOutput {
    /// 命令是否成功执行（退出码为0）
    pub status: bool,
    /// 标准输出内容
    pub stdout: String,
    /// 标准错误内容
    pub stderr: String,
}

/// Shell命令执行器接口
///
/// 提供统一的Shell命令执行接口，支持跨平台命令执行。
/// 遵循单一职责原则：只负责执行命令，不关心命令的具体含义。
///
/// # 示例
/// ```no_run
/// use devhub::core::CommandExecutor;
///
/// fn get_python_version(executor: &impl CommandExecutor) -> anyhow::Result<String> {
///     let output = executor.exec("python3", &["--version"])?;
///
///     if output.status {
///         Ok(output.stdout.trim().to_string())
///     } else {
///         Err(anyhow::anyhow!("Command failed: {}", output.stderr))
///     }
/// }
/// ```
pub trait CommandExecutor: Send + Sync {
    /// 执行Shell命令
    ///
    /// # 参数
    /// - `cmd`: 命令名称（如"python3", "node"）
    /// - `args`: 命令参数数组（如&["--version"]）
    ///
    /// # 返回值
    /// - `Ok(CommandOutput)` - 命令执行完成，包含输出和状态
    /// - `Err(...)` - 命令执行失败（如命令不存在）
    fn exec(&self, cmd: &str, args: &[&str]) -> Result<CommandOutput>;

    /// 获取软件版本号
    ///
    /// 便捷方法，自动处理常见的版本查询flag。
    ///
    /// # 参数
    /// - `cmd`: 命令名称
    /// - `flag`: 版本查询参数（如"--version", "-v"）
    ///
    /// # 返回值
    /// - `Ok(Some(version))` - 成功获取版本号
    /// - `Ok(None)` - 命令执行失败或无版本输出
    /// - `Err(...)` - 其他错误
    fn get_version(&self, cmd: &str, flag: &str) -> Result<Option<String>>;
}

