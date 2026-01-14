use crate::config;
use crate::error::{DevHubError, Result};
use crate::traits::SourceManager;
use crate::types::Mirror;
use async_trait::async_trait;
use std::path::PathBuf;

pub struct BrewManager;

impl BrewManager {
    pub fn new() -> Self {
        Self
    }
}

impl Default for BrewManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SourceManager for BrewManager {
    fn name(&self) -> &'static str {
        "brew"
    }

    fn requires_sudo(&self) -> bool {
        false
    }

    fn list_candidates(&self) -> Vec<Mirror> {
        config::get_candidates("brew")
    }

    fn config_path(&self) -> PathBuf {
        PathBuf::from("(shell profile)")
    }

    async fn current_url(&self) -> Result<Option<String>> {
        Ok(std::env::var("HOMEBREW_BOTTLE_DOMAIN").ok())
    }

    async fn set_source(&self, mirror: &Mirror) -> Result<()> {
        println!("请手动添加以下内容到您的 shell 配置文件 (~/.zshrc 或 ~/.bashrc):\n");
        println!("export HOMEBREW_API_DOMAIN=\"{}/api\"", mirror.url);
        println!("export HOMEBREW_BOTTLE_DOMAIN=\"{}\"", mirror.url);
        println!("export HOMEBREW_BREW_GIT_REMOTE=\"{}/git/homebrew/brew.git\"", mirror.url);
        println!("export HOMEBREW_CORE_GIT_REMOTE=\"{}/git/homebrew/homebrew-core.git\"", mirror.url);
        println!("\n然后执行: source ~/.zshrc (或 source ~/.bashrc)");

        Ok(())
    }

    async fn restore(&self) -> Result<()> {
        println!("请手动从您的 shell 配置文件中删除 HOMEBREW_* 环境变量");
        Err(DevHubError::Custom(
            "Homebrew 配置需要手动恢复".to_string(),
        ))
    }
}
