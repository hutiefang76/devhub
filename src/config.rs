use crate::types::Mirror;
use directories::ProjectDirs;
use std::collections::HashMap;
use std::fs;
use std::sync::OnceLock;

const MIRRORS_JSON: &str = include_str!("../assets/mirrors.json");

static MIRRORS_CACHE: OnceLock<HashMap<String, Vec<Mirror>>> = OnceLock::new();

/// 获取指定工具的镜像候选列表
pub fn get_candidates(tool_name: &str) -> Vec<Mirror> {
    let mirrors = MIRRORS_CACHE.get_or_init(|| {
        // 1. 尝试加载用户配置
        if let Some(proj_dirs) = ProjectDirs::from("", "", "devhub") {
            let config_path = proj_dirs.config_dir().join("mirrors.json");
            if config_path.exists() {
                if let Ok(content) = fs::read_to_string(&config_path) {
                    if let Ok(parsed) = serde_json::from_str(&content) {
                        return parsed;
                    }
                }
            }
        }

        // 2. 使用内置配置
        serde_json::from_str(MIRRORS_JSON).expect("内置 mirrors.json 解析失败")
    });

    mirrors.get(tool_name).cloned().unwrap_or_default()
}
