pub mod pip;
pub mod uv;
pub mod conda;
pub mod npm;
pub mod yarn;
pub mod pnpm;
pub mod cargo;
pub mod go;
pub mod maven;
pub mod gradle;
pub mod docker;
pub mod brew;
pub mod apt;
pub mod git;

use crate::error::{DevHubError, Result};
use crate::traits::SourceManager;

pub const SUPPORTED_TOOLS: &[&str] = &[
    "pip", "uv", "conda",           // Python
    "npm", "yarn", "pnpm",          // JavaScript
    "cargo",                         // Rust
    "go",                            // Go
    "maven", "gradle",              // Java
    "docker",                        // Container
    "brew", "apt",                  // System
    "git",                           // VCS
];

pub fn get_manager(name: &str) -> Result<Box<dyn SourceManager>> {
    match name.to_lowercase().as_str() {
        "pip" => Ok(Box::new(pip::PipManager::new())),
        "uv" => Ok(Box::new(uv::UvManager::new())),
        "conda" => Ok(Box::new(conda::CondaManager::new())),
        "npm" => Ok(Box::new(npm::NpmManager::new())),
        "yarn" => Ok(Box::new(yarn::YarnManager::new())),
        "pnpm" => Ok(Box::new(pnpm::PnpmManager::new())),
        "cargo" => Ok(Box::new(cargo::CargoManager::new())),
        "go" => Ok(Box::new(go::GoManager::new())),
        "maven" => Ok(Box::new(maven::MavenManager::new())),
        "gradle" => Ok(Box::new(gradle::GradleManager::new())),
        "docker" => Ok(Box::new(docker::DockerManager::new())),
        "brew" => Ok(Box::new(brew::BrewManager::new())),
        "apt" => Ok(Box::new(apt::AptManager::new())),
        "git" => Ok(Box::new(git::GitManager::new())),
        _ => Err(DevHubError::UnknownTool(format!(
            "不支持的工具: '{}'. 可用: {}",
            name,
            SUPPORTED_TOOLS.join(", ")
        ))),
    }
}
