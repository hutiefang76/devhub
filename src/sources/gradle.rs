use crate::config;
use crate::error::Result;
use crate::traits::SourceManager;
use crate::types::Mirror;
use crate::utils;
use async_trait::async_trait;
use directories::BaseDirs;
use std::path::PathBuf;
use tokio::fs;

pub struct GradleManager {
    custom_path: Option<PathBuf>,
}

impl GradleManager {
    pub fn new() -> Self {
        Self { custom_path: None }
    }
}

impl Default for GradleManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SourceManager for GradleManager {
    fn name(&self) -> &'static str {
        "gradle"
    }

    fn requires_sudo(&self) -> bool {
        false
    }

    fn list_candidates(&self) -> Vec<Mirror> {
        config::get_candidates("gradle")
    }

    fn config_path(&self) -> PathBuf {
        if let Some(ref path) = self.custom_path {
            return path.clone();
        }

        if let Some(base_dirs) = BaseDirs::new() {
            base_dirs.home_dir().join(".gradle").join("init.gradle")
        } else {
            PathBuf::from(".gradle/init.gradle")
        }
    }

    async fn current_url(&self) -> Result<Option<String>> {
        let path = self.config_path();
        if !fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(None);
        }

        let content = fs::read_to_string(&path).await?;

        if let Some(start) = content.find("url '") {
            let rest = &content[start + 5..];
            if let Some(end) = rest.find('\'') {
                return Ok(Some(rest[..end].to_string()));
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
            r#"allprojects {{
    repositories {{
        maven {{ url '{url}' }}
        mavenLocal()
        mavenCentral()
    }}
}}

settingsEvaluated {{ settings ->
    settings.pluginManagement {{
        repositories {{
            maven {{ url '{url}' }}
            gradlePluginPortal()
            mavenCentral()
        }}
    }}
}}
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
