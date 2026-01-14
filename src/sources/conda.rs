use crate::config;
use crate::error::Result;
use crate::traits::SourceManager;
use crate::types::Mirror;
use crate::utils;
use async_trait::async_trait;
use directories::BaseDirs;
use regex::Regex;
use std::path::PathBuf;
use tokio::fs;

pub struct CondaManager {
    custom_path: Option<PathBuf>,
}

impl CondaManager {
    pub fn new() -> Self {
        Self { custom_path: None }
    }
}

impl Default for CondaManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SourceManager for CondaManager {
    fn name(&self) -> &'static str {
        "conda"
    }

    fn requires_sudo(&self) -> bool {
        false
    }

    fn list_candidates(&self) -> Vec<Mirror> {
        config::get_candidates("conda")
    }

    fn config_path(&self) -> PathBuf {
        if let Some(ref path) = self.custom_path {
            return path.clone();
        }

        if let Some(base_dirs) = BaseDirs::new() {
            base_dirs.home_dir().join(".condarc")
        } else {
            PathBuf::from(".condarc")
        }
    }

    async fn current_url(&self) -> Result<Option<String>> {
        let path = self.config_path();
        if !fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(None);
        }

        let content = fs::read_to_string(&path).await?;
        let re = Regex::new(r"(?m)^default_channels:\s*\n\s*-\s*(.+?)(?:/|$)")?;

        if let Some(caps) = re.captures(&content) {
            Ok(Some(caps[1].trim().to_string()))
        } else {
            Ok(None)
        }
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
            r#"channels:
  - defaults
show_channel_urls: true
default_channels:
  - {url}/pkgs/main
  - {url}/pkgs/r
  - {url}/pkgs/msys2
custom_channels:
  conda-forge: {url}/cloud
  pytorch: {url}/cloud
"#,
            url = mirror.url
        );

        fs::write(&path, content).await?;
        Ok(())
    }

    async fn restore(&self) -> Result<()> {
        utils::restore_latest_backup(&self.config_path()).await
    }
}
