use crate::config;
use crate::error::{DevHubError, Result};
use crate::traits::SourceManager;
use crate::types::Mirror;
use crate::utils;
use async_trait::async_trait;
use std::path::PathBuf;

pub struct GoManager;

impl GoManager {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GoManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SourceManager for GoManager {
    fn name(&self) -> &'static str {
        "go"
    }

    fn requires_sudo(&self) -> bool {
        false
    }

    fn list_candidates(&self) -> Vec<Mirror> {
        config::get_candidates("go")
    }

    fn config_path(&self) -> PathBuf {
        PathBuf::from("(env var GOPROXY)")
    }

    async fn current_url(&self) -> Result<Option<String>> {
        match utils::run_command("go", &["env", "GOPROXY"]).await {
            Ok(output) => {
                if output.is_empty() || output == "off" {
                    Ok(None)
                } else {
                    Ok(Some(output))
                }
            }
            Err(_) => Ok(None),
        }
    }

    async fn set_source(&self, mirror: &Mirror) -> Result<()> {
        utils::run_command("go", &["env", "-w", &format!("GOPROXY={}", mirror.url)])
            .await
            .map_err(|e| DevHubError::Custom(format!("设置 GOPROXY 失败: {}", e)))?;
        Ok(())
    }

    async fn restore(&self) -> Result<()> {
        utils::run_command("go", &["env", "-w", "GOPROXY=https://proxy.golang.org,direct"])
            .await
            .map_err(|e| DevHubError::Custom(format!("恢复 GOPROXY 失败: {}", e)))?;
        Ok(())
    }
}
