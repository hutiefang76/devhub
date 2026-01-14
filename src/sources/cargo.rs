use crate::config;
use crate::error::Result;
use crate::traits::SourceManager;
use crate::types::Mirror;
use crate::utils;
use async_trait::async_trait;
use directories::BaseDirs;
use std::path::PathBuf;
use tokio::fs;

pub struct CargoManager {
    custom_path: Option<PathBuf>,
}

impl CargoManager {
    pub fn new() -> Self {
        Self { custom_path: None }
    }
}

impl Default for CargoManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SourceManager for CargoManager {
    fn name(&self) -> &'static str {
        "cargo"
    }

    fn requires_sudo(&self) -> bool {
        false
    }

    fn list_candidates(&self) -> Vec<Mirror> {
        config::get_candidates("cargo")
    }

    fn config_path(&self) -> PathBuf {
        if let Some(ref path) = self.custom_path {
            return path.clone();
        }

        if let Some(base_dirs) = BaseDirs::new() {
            base_dirs.home_dir().join(".cargo").join("config.toml")
        } else {
            PathBuf::from(".cargo/config.toml")
        }
    }

    async fn current_url(&self) -> Result<Option<String>> {
        let path = self.config_path();
        if !fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(None);
        }

        let content = fs::read_to_string(&path).await?;
        let value: toml::Value = toml::from_str(&content)?;

        if let Some(source) = value.get("source") {
            if let Some(mirror) = source.get("mirror") {
                if let Some(registry) = mirror.get("registry") {
                    return Ok(registry.as_str().map(String::from));
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
            r#"[source.crates-io]
replace-with = "mirror"

[source.mirror]
registry = "{}"
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
