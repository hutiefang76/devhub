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

pub struct PipManager {
    custom_path: Option<PathBuf>,
}

impl PipManager {
    pub fn new() -> Self {
        Self { custom_path: None }
    }

    #[cfg(test)]
    pub fn with_path(path: PathBuf) -> Self {
        Self {
            custom_path: Some(path),
        }
    }
}

impl Default for PipManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SourceManager for PipManager {
    fn name(&self) -> &'static str {
        "pip"
    }

    fn requires_sudo(&self) -> bool {
        false
    }

    fn list_candidates(&self) -> Vec<Mirror> {
        config::get_candidates("pip")
    }

    fn config_path(&self) -> PathBuf {
        if let Some(ref path) = self.custom_path {
            return path.clone();
        }

        if let Some(base_dirs) = BaseDirs::new() {
            let config_dir = base_dirs.config_dir();
            if cfg!(target_os = "windows") {
                config_dir.join("pip").join("pip.ini")
            } else {
                config_dir.join("pip").join("pip.conf")
            }
        } else {
            PathBuf::from(".").join("pip.conf")
        }
    }

    async fn current_url(&self) -> Result<Option<String>> {
        let path = self.config_path();
        if !fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(None);
        }

        let content = fs::read_to_string(&path).await?;
        let re = Regex::new(r"(?m)^index-url\s*=\s*(.+)$")?;

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

        let content = if fs::try_exists(&path).await.unwrap_or(false) {
            fs::read_to_string(&path).await?
        } else {
            String::new()
        };

        if !content.is_empty() {
            utils::backup_file(&path).await?;
        }

        let new_url_line = format!("index-url = {}", mirror.url);
        let trusted_host = extract_host(&mirror.url);
        let trusted_line = format!("trusted-host = {}", trusted_host);

        let re = Regex::new(r"(?m)^index-url\s*=\s*.*$")?;
        let re_trusted = Regex::new(r"(?m)^trusted-host\s*=\s*.*$")?;

        let new_content = if re.is_match(&content) {
            let temp = re.replace(&content, new_url_line.as_str()).to_string();
            if re_trusted.is_match(&temp) {
                re_trusted.replace(&temp, trusted_line.as_str()).to_string()
            } else {
                temp.replace("[global]", &format!("[global]\n{}", trusted_line))
            }
        } else if content.contains("[global]") {
            content.replace("[global]", &format!("[global]\n{}\n{}", new_url_line, trusted_line))
        } else {
            let prefix = if content.is_empty() { "" } else { "\n" };
            format!("{}{}[global]\n{}\n{}\n", content, prefix, new_url_line, trusted_line)
        };

        fs::write(&path, new_content).await?;
        Ok(())
    }

    async fn restore(&self) -> Result<()> {
        utils::restore_latest_backup(&self.config_path()).await
    }
}

fn extract_host(url: &str) -> String {
    url.trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_pip_flow() -> Result<()> {
        let dir = tempdir()?;
        let config_path = dir.path().join("pip.conf");
        let manager = PipManager::with_path(config_path.clone());

        assert!(manager.current_url().await?.is_none());

        let mirror = Mirror::new("Test", "https://test.pypi.org/simple");
        manager.set_source(&mirror).await?;

        let current = manager.current_url().await?;
        assert_eq!(current, Some(mirror.url.clone()));

        Ok(())
    }
}
