use crate::config;
use crate::error::{DevHubError, Result};
use crate::traits::SourceManager;
use crate::types::Mirror;
use crate::utils;
use async_trait::async_trait;
use directories::BaseDirs;
use regex::Regex;
use std::path::PathBuf;
use tokio::fs;

pub struct GitManager {
    custom_path: Option<PathBuf>,
}

impl GitManager {
    pub fn new() -> Self {
        Self { custom_path: None }
    }
}

impl Default for GitManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SourceManager for GitManager {
    fn name(&self) -> &'static str {
        "git"
    }

    fn requires_sudo(&self) -> bool {
        false
    }

    fn list_candidates(&self) -> Vec<Mirror> {
        config::get_candidates("git")
    }

    fn config_path(&self) -> PathBuf {
        if let Some(ref path) = self.custom_path {
            return path.clone();
        }

        if let Some(base_dirs) = BaseDirs::new() {
            base_dirs.home_dir().join(".gitconfig")
        } else {
            PathBuf::from(".gitconfig")
        }
    }

    async fn current_url(&self) -> Result<Option<String>> {
        let path = self.config_path();
        if !fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(None);
        }

        let content = fs::read_to_string(&path).await?;
        let re = Regex::new(r#"(?m)^\s*insteadOf\s*=\s*https://github\.com"#)?;

        if re.is_match(&content) {
            let url_re = Regex::new(r#"(?m)^\[url\s+"([^"]+)"\]"#)?;
            if let Some(caps) = url_re.captures(&content) {
                return Ok(Some(caps[1].trim_end_matches('/').to_string()));
            }
        }

        Ok(None)
    }

    async fn set_source(&self, mirror: &Mirror) -> Result<()> {
        if mirror.name.to_lowercase() == "official" {
            return self.restore().await;
        }

        let url = &mirror.url;

        utils::run_command(
            "git",
            &["config", "--global", &format!("url.{}/.", url), "insteadOf", "https://github.com/"],
        )
        .await
        .map_err(|e| DevHubError::Custom(format!("设置 Git 镜像失败: {}", e)))?;

        println!("Git 镜像已设置为: {}", url);
        println!("所有 https://github.com/ 的请求将被重定向到 {}/", url);

        Ok(())
    }

    async fn restore(&self) -> Result<()> {
        let current = self.current_url().await?;

        if let Some(url) = current {
            utils::run_command(
                "git",
                &["config", "--global", "--remove-section", &format!("url.{}/.", url)],
            )
            .await
            .ok();
        }

        println!("Git 配置已恢复默认");
        Ok(())
    }
}
