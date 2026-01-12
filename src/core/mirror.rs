use anyhow::Result;
use serde::{Deserialize, Serialize};

/// 镜像源信息
///
/// # 示例
/// ```no_run
/// let mirror = Mirror {
///     name: "阿里云".to_string(),
///     url: "https://mirrors.aliyun.com/pypi/simple/".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mirror {
    /// 镜像源名称（如"阿里云"）
    pub name: String,
    /// 镜像源URL
    pub url: String,
}

/// 镜像源配置器接口
///
/// 负责管理软件包管理器的镜像源配置。
/// 遵循单一职责原则：只关注镜像源的读取、列举、应用和恢复。
///
/// # 示例
/// ```no_run
/// use devhub::core::MirrorConfigurator;
///
/// fn configure_mirrors(config: &impl MirrorConfigurator) -> anyhow::Result<()> {
///     // 列出可用镜像源
///     let mirrors = config.list_mirrors();
///
///     // 应用第一个镜像源
///     if let Some(mirror) = mirrors.first() {
///         config.apply(mirror)?;
///     }
///
///     Ok(())
/// }
/// ```
pub trait MirrorConfigurator: Send + Sync {
    /// 获取当前使用的镜像源URL
    fn get_current(&self) -> Result<Option<String>>;

    /// 列出所有可用的镜像源
    fn list_mirrors(&self) -> Vec<Mirror>;

    /// 应用指定的镜像源配置
    fn apply(&self, mirror: &Mirror) -> Result<()>;

    /// 恢复默认配置（删除自定义配置）
    fn restore_default(&self) -> Result<()>;
}

