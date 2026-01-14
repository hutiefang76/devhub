use thiserror::Error;

#[derive(Error, Debug)]
pub enum DevHubError {
    #[error("IO 操作失败: {0}")]
    Io(#[from] std::io::Error),

    #[error("正则表达式错误: {0}")]
    Regex(#[from] regex::Error),

    #[error("JSON 解析错误: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML 序列化错误: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("TOML 反序列化错误: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("系统时间错误: {0}")]
    SystemTime(#[from] std::time::SystemTimeError),

    #[error("不支持的工具: {0}")]
    UnknownTool(String),

    #[error("命令执行失败: {0}")]
    CommandFailed(String),

    #[error("{0}")]
    Custom(String),
}

pub type Result<T> = std::result::Result<T, DevHubError>;
