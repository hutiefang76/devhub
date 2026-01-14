use crate::config;
use crate::error::Result;
use crate::traits::SourceManager;
use crate::types::Mirror;
use crate::utils;
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs;

pub struct DockerManager {
    custom_path: Option<PathBuf>,
}

impl DockerManager {
    pub fn new() -> Self {
        Self { custom_path: None }
    }
}

impl Default for DockerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SourceManager for DockerManager {
    fn name(&self) -> &'static str {
        "docker"
    }

    fn requires_sudo(&self) -> bool {
        true
    }

    fn list_candidates(&self) -> Vec<Mirror> {
        config::get_candidates("docker")
    }

    fn config_path(&self) -> PathBuf {
        if let Some(ref path) = self.custom_path {
            return path.clone();
        }

        if cfg!(target_os = "macos") {
            PathBuf::from(std::env::var("HOME").unwrap_or_default())
                .join(".docker")
                .join("daemon.json")
        } else if cfg!(target_os = "windows") {
            PathBuf::from(std::env::var("PROGRAMDATA").unwrap_or_default())
                .join("docker")
                .join("config")
                .join("daemon.json")
        } else {
            PathBuf::from("/etc/docker/daemon.json")
        }
    }

    async fn current_url(&self) -> Result<Option<String>> {
        let path = self.config_path();
        if !fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(None);
        }

        let content = fs::read_to_string(&path).await?;
        let value: serde_json::Value = serde_json::from_str(&content)?;

        if let Some(mirrors) = value.get("registry-mirrors") {
            if let Some(arr) = mirrors.as_array() {
                if let Some(first) = arr.first() {
                    return Ok(first.as_str().map(String::from));
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

        let mut config: serde_json::Value = if fs::try_exists(&path).await.unwrap_or(false) {
            utils::backup_file(&path).await?;
            let content = fs::read_to_string(&path).await?;
            serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        config["registry-mirrors"] = serde_json::json!([mirror.url]);

        let content = serde_json::to_string_pretty(&config)?;
        fs::write(&path, content).await?;

        println!("注意: 需要重启 Docker 服务使配置生效");
        println!("  macOS: 重启 Docker Desktop");
        println!("  Linux: sudo systemctl restart docker");

        Ok(())
    }

    async fn restore(&self) -> Result<()> {
        utils::restore_latest_backup(&self.config_path()).await
    }
}
