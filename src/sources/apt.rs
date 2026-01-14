use crate::config;
use crate::error::{DevHubError, Result};
use crate::traits::SourceManager;
use crate::types::Mirror;
use crate::utils;
use async_trait::async_trait;
use regex::Regex;
use std::path::PathBuf;
use tokio::fs;

pub struct AptManager {
    custom_path: Option<PathBuf>,
}

impl AptManager {
    pub fn new() -> Self {
        Self { custom_path: None }
    }

    fn detect_distro() -> Option<String> {
        if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
            if content.contains("Ubuntu") || content.contains("ubuntu") {
                return Some("ubuntu".to_string());
            }
            if content.contains("Debian") || content.contains("debian") {
                return Some("debian".to_string());
            }
        }
        None
    }
}

impl Default for AptManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SourceManager for AptManager {
    fn name(&self) -> &'static str {
        "apt"
    }

    fn requires_sudo(&self) -> bool {
        true
    }

    fn list_candidates(&self) -> Vec<Mirror> {
        let distro = Self::detect_distro().unwrap_or_else(|| "ubuntu".to_string());
        config::get_candidates(&format!("apt-{}", distro))
    }

    fn config_path(&self) -> PathBuf {
        if let Some(ref path) = self.custom_path {
            return path.clone();
        }
        PathBuf::from("/etc/apt/sources.list")
    }

    async fn current_url(&self) -> Result<Option<String>> {
        let path = self.config_path();
        if !fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(None);
        }

        let content = fs::read_to_string(&path).await?;
        let re = Regex::new(r"(?m)^deb\s+(?:\[.*?\]\s+)?(?P<url>https?://\S+)\s+")?;

        if let Some(caps) = re.captures(&content) {
            Ok(Some(caps["url"].to_string()))
        } else {
            Ok(None)
        }
    }

    async fn set_source(&self, mirror: &Mirror) -> Result<()> {
        let distro = Self::detect_distro().ok_or_else(|| {
            DevHubError::Custom("无法检测 Linux 发行版 (仅支持 Ubuntu/Debian)".to_string())
        })?;

        let path = self.config_path();

        if fs::try_exists(&path).await.unwrap_or(false) {
            utils::backup_file(&path).await?;
        }

        let codename = get_codename().await.unwrap_or_else(|| "jammy".to_string());

        let content = if distro == "ubuntu" {
            format!(
                r#"deb {url} {codename} main restricted universe multiverse
deb {url} {codename}-updates main restricted universe multiverse
deb {url} {codename}-backports main restricted universe multiverse
deb {url} {codename}-security main restricted universe multiverse
"#,
                url = mirror.url,
                codename = codename
            )
        } else {
            format!(
                r#"deb {url} {codename} main contrib non-free
deb {url} {codename}-updates main contrib non-free
deb {url}-security {codename}-security main contrib non-free
"#,
                url = mirror.url,
                codename = codename
            )
        };

        fs::write(&path, content).await?;
        println!("注意: 请执行 sudo apt update 更新软件源");

        Ok(())
    }

    async fn restore(&self) -> Result<()> {
        utils::restore_latest_backup(&self.config_path()).await
    }
}

async fn get_codename() -> Option<String> {
    if let Ok(content) = fs::read_to_string("/etc/os-release").await {
        for line in content.lines() {
            if line.starts_with("VERSION_CODENAME=") {
                return Some(line.trim_start_matches("VERSION_CODENAME=").trim_matches('"').to_string());
            }
        }
    }
    None
}
