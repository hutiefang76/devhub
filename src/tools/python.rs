//! Python工具支持
//!
//! 提供Python环境检测和pip镜像源配置功能。
//!
//! # 示例
//! ```no_run
//! use devhub::tools::python::{PythonDetector, PipMirror};
//! use devhub::core::{ToolDetector, MirrorConfigurator};
//!
//! // 检测Python
//! let detector = PythonDetector::new();
//! let info = detector.detect().unwrap();
//! println!("Python已安装: {}", info.installed);
//!
//! // 配置pip镜像源
//! let pip = PipMirror::new();
//! let mirrors = pip.list_mirrors();
//! pip.apply(&mirrors[0]).unwrap();
//! ```

use crate::core::command::*;
use crate::core::detector::*;
use crate::core::mirror::*;
use crate::services::shell::ShellExecutor;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

/// Python环境检测器
///
/// 通过执行`which python3`和`python3 --version`检测Python是否安装。
///
/// # 实现细节
/// - 使用`which`命令定位python3路径
/// - 使用`python3 --version`获取版本号
/// - 跨平台支持（macOS/Linux/Windows）
///
/// # 示例
/// ```no_run
/// use devhub::tools::PythonDetector;
/// use devhub::core::ToolDetector;
///
/// let detector = PythonDetector::new();
/// match detector.detect() {
///     Ok(info) => {
///         if info.installed {
///             println!("Python版本: {:?}", info.version);
///         }
///     }
///     Err(e) => eprintln!("检测失败: {}", e),
/// }
/// ```
pub struct PythonDetector {
    /// Shell命令执行器
    executor: ShellExecutor,
}

impl PythonDetector {
    /// 创建新的Python检测器
    pub fn new() -> Self {
        Self {
            executor: ShellExecutor,
        }
    }
}

impl Default for PythonDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolDetector for PythonDetector {
    /// 检测Python安装状态
    ///
    /// # 检测逻辑
    /// 1. 执行`which python3`获取路径
    /// 2. 如果找到路径，执行`python3 --version`获取版本
    /// 3. 返回检测结果
    ///
    /// # 错误处理
    /// 如果命令执行失败，返回"未安装"状态而非错误
    fn detect(&self) -> Result<DetectionInfo> {
        // 检查路径
        let path = self
            .executor
            .exec("which", &["python3"])
            .ok()
            .filter(|o| o.status)
            .map(|o| PathBuf::from(o.stdout.trim()));

        // 检查版本
        let version = if path.is_some() {
            self.executor
                .get_version("python3", "--version")
                .ok()
                .flatten()
        } else {
            None
        };

        Ok(DetectionInfo {
            installed: path.is_some(),
            version,
            path,
        })
    }
}

/// pip镜像源配置管理器
///
/// 管理pip的镜像源配置文件`~/.pip/pip.conf`。
///
/// # 配置文件位置
/// - macOS/Linux: `~/.pip/pip.conf`
/// - Windows: `%APPDATA%\pip\pip.ini`（未实现）
///
/// # 配置文件格式
/// ```ini
/// [global]
/// index-url = https://mirrors.aliyun.com/pypi/simple/
/// trusted-host = mirrors.aliyun.com
/// ```
///
/// # 示例
/// ```no_run
/// use devhub::tools::PipMirror;
/// use devhub::core::MirrorConfigurator;
///
/// let pip = PipMirror::new();
///
/// // 列出可用镜像源
/// for mirror in pip.list_mirrors() {
///     println!("{}: {}", mirror.name, mirror.url);
/// }
///
/// // 应用第一个镜像源
/// let mirrors = pip.list_mirrors();
/// pip.apply(&mirrors[0]).unwrap();
/// ```
pub struct PipMirror {
    /// pip配置文件路径
    config_path: PathBuf,
}

impl PipMirror {
    /// 创建新的pip镜像源管理器
    ///
    /// # 配置文件路径
    /// 使用`~/.pip/pip.conf`作为配置文件路径
    pub fn new() -> Self {
        let config_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".pip");

        Self {
            config_path: config_dir.join("pip.conf"),
        }
    }

    /// 从URL中提取域名
    ///
    /// 用于生成`trusted-host`配置。
    ///
    /// # 示例
    /// ```
    /// use devhub::tools::python::PipMirror;
    ///
    /// let domain = PipMirror::extract_domain("https://mirrors.aliyun.com/pypi/simple/");
    /// assert_eq!(domain, "mirrors.aliyun.com");
    /// ```
    pub fn extract_domain(url: &str) -> String {
        url.replace("https://", "")
            .replace("http://", "")
            .split('/')
            .next()
            .unwrap_or("mirrors.aliyun.com")
            .to_string()
    }
}

impl Default for PipMirror {
    fn default() -> Self {
        Self::new()
    }
}

impl MirrorConfigurator for PipMirror {
    /// 获取当前使用的镜像源
    ///
    /// # 返回值
    /// - `Ok(Some(url))` - 已配置自定义镜像源
    /// - `Ok(None)` - 使用默认配置（PyPI官方源）
    /// - `Err(...)` - 读取配置文件失败
    fn get_current(&self) -> Result<Option<String>> {
        if !self.config_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&self.config_path)?;
        let re = regex::Regex::new(r"index-url\s*=\s*(.+)")?;

        Ok(re
            .captures(&content)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().trim().to_string()))
    }

    /// 列出所有可用的中国镜像源
    ///
    /// # 镜像源列表
    /// 1. 阿里云 - 企业级，稳定性高
    /// 2. 清华大学 - 教育网首选
    /// 3. 腾讯云 - 速度快
    /// 4. 豆瓣 - 老牌镜像
    /// 5. 华为云 - 企业级
    fn list_mirrors(&self) -> Vec<Mirror> {
        vec![
            Mirror {
                name: "阿里云".to_string(),
                url: "https://mirrors.aliyun.com/pypi/simple/".to_string(),
            },
            Mirror {
                name: "清华大学".to_string(),
                url: "https://pypi.tuna.tsinghua.edu.cn/simple".to_string(),
            },
            Mirror {
                name: "腾讯云".to_string(),
                url: "https://mirrors.cloud.tencent.com/pypi/simple".to_string(),
            },
            Mirror {
                name: "豆瓣".to_string(),
                url: "https://pypi.doubanio.com/simple".to_string(),
            },
            Mirror {
                name: "华为云".to_string(),
                url: "https://repo.huaweicloud.com/repository/pypi/simple".to_string(),
            },
        ]
    }

    /// 应用指定的镜像源配置
    ///
    /// # 实现细节
    /// 1. 创建配置目录（如果不存在）
    /// 2. 生成配置文件内容
    /// 3. 写入`~/.pip/pip.conf`
    ///
    /// # 错误处理
    /// - 目录创建失败
    /// - 文件写入失败
    fn apply(&self, mirror: &Mirror) -> Result<()> {
        // 创建配置目录
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // 写入配置（KISS原则：简单直接）
        let content = format!(
            "[global]\nindex-url = {}\ntrusted-host = {}\n",
            mirror.url,
            Self::extract_domain(&mirror.url)
        );

        fs::write(&self.config_path, content)?;
        Ok(())
    }

    /// 恢复pip默认配置
    ///
    /// 删除自定义配置文件，恢复使用PyPI官方源。
    fn restore_default(&self) -> Result<()> {
        if self.config_path.exists() {
            fs::remove_file(&self.config_path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_detector() {
        let detector = PythonDetector::new();
        let result = detector.detect();
        assert!(result.is_ok());
    }

    #[test]
    fn test_pip_mirror_list() {
        let config = PipMirror::new();
        let mirrors = config.list_mirrors();
        assert!(!mirrors.is_empty());
        assert_eq!(mirrors[0].name, "阿里云");
    }

    #[test]
    fn test_domain_extraction() {
        let domain = PipMirror::extract_domain("https://mirrors.aliyun.com/pypi/simple/");
        assert_eq!(domain, "mirrors.aliyun.com");
    }
}
