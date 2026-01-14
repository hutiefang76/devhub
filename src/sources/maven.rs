use crate::config;
use crate::error::Result;
use crate::traits::SourceManager;
use crate::types::Mirror;
use crate::utils;
use async_trait::async_trait;
use directories::BaseDirs;
use std::path::PathBuf;
use tokio::fs;

pub struct MavenManager {
    custom_path: Option<PathBuf>,
}

impl MavenManager {
    pub fn new() -> Self {
        Self { custom_path: None }
    }
}

impl Default for MavenManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SourceManager for MavenManager {
    fn name(&self) -> &'static str {
        "maven"
    }

    fn requires_sudo(&self) -> bool {
        false
    }

    fn list_candidates(&self) -> Vec<Mirror> {
        config::get_candidates("maven")
    }

    fn config_path(&self) -> PathBuf {
        if let Some(ref path) = self.custom_path {
            return path.clone();
        }

        if let Some(base_dirs) = BaseDirs::new() {
            base_dirs.home_dir().join(".m2").join("settings.xml")
        } else {
            PathBuf::from(".m2/settings.xml")
        }
    }

    async fn current_url(&self) -> Result<Option<String>> {
        let path = self.config_path();
        if !fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(None);
        }

        let content = fs::read_to_string(&path).await?;

        if let Some(start) = content.find("<url>") {
            if let Some(end) = content[start..].find("</url>") {
                let url = &content[start + 5..start + end];
                return Ok(Some(url.trim().to_string()));
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
            r#"<?xml version="1.0" encoding="UTF-8"?>
<settings xmlns="http://maven.apache.org/SETTINGS/1.0.0"
          xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
          xsi:schemaLocation="http://maven.apache.org/SETTINGS/1.0.0
                              http://maven.apache.org/xsd/settings-1.0.0.xsd">
  <mirrors>
    <mirror>
      <id>{id}</id>
      <name>{name} Mirror</name>
      <url>{url}</url>
      <mirrorOf>central</mirrorOf>
    </mirror>
  </mirrors>
</settings>
"#,
            id = mirror.name.to_lowercase(),
            name = mirror.name,
            url = mirror.url
        );

        fs::write(&path, content).await?;
        Ok(())
    }

    async fn restore(&self) -> Result<()> {
        utils::restore_latest_backup(&self.config_path()).await
    }
}
