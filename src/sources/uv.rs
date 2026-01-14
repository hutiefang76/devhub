use crate::config;
use crate::error::Result;
use crate::traits::SourceManager;
use crate::types::Mirror;
use crate::utils;
use async_trait::async_trait;
use directories::BaseDirs;
use std::path::PathBuf;
use tokio::fs;

pub struct UvManager {
    custom_path: Option<PathBuf>,
}

impl UvManager {
    pub fn new() -> Self {
        Self { custom_path: None }
    }
}

impl Default for UvManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SourceManager for UvManager {
    fn name(&self) -> &'static str {
        "uv"
    }

    fn requires_sudo(&self) -> bool {
        false
    }

    fn list_candidates(&self) -> Vec<Mirror> {
        config::get_candidates("uv")
    }

    fn config_path(&self) -> PathBuf {
        if let Some(ref path) = self.custom_path {
            return path.clone();
        }

        if let Some(base_dirs) = BaseDirs::new() {
            base_dirs.config_dir().join("uv").join("uv.toml")
        } else {
            PathBuf::from("uv.toml")
        }
    }

    async fn current_url(&self) -> Result<Option<String>> {
        let path = self.config_path();
        if !fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(None);
        }

        let content = fs::read_to_string(&path).await?;
        let value: toml::Value = toml::from_str(&content)?;

        if let Some(index) = value.get("index") {
            if let Some(arr) = index.as_array() {
                for item in arr {
                    if let Some(is_default) = item.get("default") {
                        if is_default.as_bool() == Some(true) {
                            if let Some(url) = item.get("url") {
                                return Ok(url.as_str().map(String::from));
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    async fn set_source(&self, mirror: &Mirror) -> Result<()> {
        let path = self.config_path();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        if fs::try_exists(&path).await.unwrap_or(false) {
            utils::backup_file(&path).await?;
        }

        let content = format!(
            r#"[[index]]
url = "{}"
default = true
"#,
            mirror.url
        );

        fs::write(&path, content).await?;
        Ok(())
    }

    async fn restore(&self) -> Result<()> {
        utils::restore_latest_backup(&self.config_path()).await
    }
}
