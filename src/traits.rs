use crate::error::Result;
use crate::types::Mirror;
use async_trait::async_trait;
use std::path::PathBuf;

/// SourceManager: 所有镜像源管理模块必须实现的接口
#[async_trait]
pub trait SourceManager: Sync + Send {
    /// 工具名称 (如 "pip", "docker")
    fn name(&self) -> &'static str;

    /// 是否需要 Root 权限
    fn requires_sudo(&self) -> bool;

    /// 获取内置的推荐源列表
    fn list_candidates(&self) -> Vec<Mirror>;

    /// 获取当前正在使用的源 URL
    async fn current_url(&self) -> Result<Option<String>>;

    /// 应用新的镜像源
    async fn set_source(&self, mirror: &Mirror) -> Result<()>;

    /// 获取配置文件的路径
    fn config_path(&self) -> PathBuf;

    /// 恢复到上一次的配置或默认配置
    async fn restore(&self) -> Result<()>;
}
