//! Shell命令执行服务
//!
//! 提供跨平台的Shell命令执行功能，是所有工具检测的基础。

use crate::core::command::*;
use anyhow::Result;
use std::process::Command;

/// Shell命令执行器
///
/// 跨平台的Shell命令执行实现，封装了Rust标准库的`std::process::Command`。
///
/// # 线程安全
/// 该结构体是零大小类型(ZST)，可以安全地在多线程间共享。
///
/// # 示例
/// ```no_run
/// use devhub::services::ShellExecutor;
/// use devhub::core::CommandExecutor;
///
/// let executor = ShellExecutor;
/// let output = executor.exec("echo", &["hello"]).unwrap();
/// assert_eq!(output.stdout.trim(), "hello");
/// ```
pub struct ShellExecutor;

impl CommandExecutor for ShellExecutor {
    /// 执行Shell命令并返回结果
    ///
    /// # 实现细节
    /// - 使用`std::process::Command`执行命令
    /// - 自动捕获stdout和stderr
    /// - 通过退出码判断命令是否成功（0表示成功）
    ///
    /// # 错误处理
    /// - 命令不存在：返回Err
    /// - 命令执行失败（退出码非0）：返回Ok，但status=false
    fn exec(&self, cmd: &str, args: &[&str]) -> Result<CommandOutput> {
        let output = Command::new(cmd)
            .args(args)
            .output()?;

        Ok(CommandOutput {
            status: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }

    /// 获取软件版本号
    ///
    /// # 实现逻辑
    /// 1. 执行 `cmd flag`（如`python3 --version`）
    /// 2. 如果成功，提取stdout的第一行作为版本号
    /// 3. 如果失败，返回None（表示无法获取版本）
    fn get_version(&self, cmd: &str, flag: &str) -> Result<Option<String>> {
        match self.exec(cmd, &[flag]) {
            Ok(output) if output.status => {
                let version = output
                    .stdout
                    .lines()
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();

                Ok(Some(version))
            }
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_executor() {
        let executor = ShellExecutor;
        let result = executor.exec("echo", &["hello"]);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.status);
        assert!(output.stdout.contains("hello"));
    }
}

