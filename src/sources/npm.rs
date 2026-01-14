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

pub struct NpmManager {
    custom_path: Option<PathBuf>,
}

impl NpmManager {
    pub fn new() -> Self {
        Self { custom_path: None }
    }
}

impl Default for NpmManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SourceManager for NpmManager {
    fn name(&self) -> &'static str {
        "npm"
    }

    fn requires_sudo(&self) -> bool {
        false
    }

    fn list_candidates(&self) -> Vec<Mirror> {
        config::get_candidates("npm")
    }

    fn config_path(&self) -> PathBuf {
        if let Some(ref path) = self.custom_path {
            return path.clone();
        }

        if let Some(base_dirs) = BaseDirs::new() {
            base_dirs.home_dir().join(".npmrc")
        } else {
            PathBuf::from(".npmrc")
        }
    }

    async fn current_url(&self) -> Result<Option<String>> {
        let path = self.config_path();
        if !fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(None);
        }

        let content = fs::read_to_string(&path).await?;
        let re = Regex::new(r"(?m)^registry\s*=\s*(.+)$")?;

        if let Some(caps) = re.captures(&content) {
            Ok(Some(caps[1].trim().to_string()))
        } else {
            Ok(None)
        }
    }

    async fn set_source(&self, mirror: &Mirror) -> Result<()> {
        let path = self.config_path();

        let content = if fs::try_exists(&path).await.unwrap_or(false) {
            fs::read_to_string(&path).await?
        } else {
            String::new()
        };

        if !content.is_empty() {
            utils::backup_file(&path).await?;
        }

        let new_line = format!("registry={}", mirror.url);
        let re = Regex::new(r"(?m)^registry\s*=\s*.*$")?;

        let new_content = if re.is_match(&content) {
            re.replace(&content, new_line.as_str()).to_string()
        } else {
            let prefix = if content.is_empty() { "" } else { "\n" };
            format!("{}{}{}\n", content, prefix, new_line)
        };

        fs::write(&path, new_content).await?;
        Ok(())
    }

    async fn restore(&self) -> Result<()> {
        utils::restore_latest_backup(&self.config_path()).await
    }
}
