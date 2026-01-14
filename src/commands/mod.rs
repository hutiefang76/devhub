use crate::sources::{get_manager, SUPPORTED_TOOLS};
use crate::types::Mirror;
use crate::utils::benchmark_mirrors;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStatus {
    pub name: String,
    pub current_url: Option<String>,
    pub current_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedTestResult {
    pub name: String,
    pub url: String,
    pub latency_ms: u64,
    pub is_timeout: bool,
}

#[tauri::command]
pub fn list_supported_tools() -> Vec<String> {
    SUPPORTED_TOOLS.iter().map(|s| s.to_string()).collect()
}

#[tauri::command]
pub async fn get_tool_status(name: String) -> Result<ToolStatus, String> {
    let manager = get_manager(&name).map_err(|e| e.to_string())?;
    let current_url = manager.current_url().await.map_err(|e| e.to_string())?;
    let candidates = manager.list_candidates();

    let current_name = current_url.as_ref().and_then(|url| {
        candidates.iter()
            .find(|m| m.url.trim_end_matches('/') == url.trim_end_matches('/'))
            .map(|m| m.name.clone())
    });

    Ok(ToolStatus {
        name: manager.name().to_string(),
        current_url,
        current_name,
    })
}

#[tauri::command]
pub async fn get_all_status() -> Vec<ToolStatus> {
    let mut results = Vec::new();
    for tool in SUPPORTED_TOOLS {
        if let Ok(status) = get_tool_status(tool.to_string()).await {
            results.push(status);
        }
    }
    results
}

#[tauri::command]
pub fn list_mirrors(name: String) -> Result<Vec<Mirror>, String> {
    let manager = get_manager(&name).map_err(|e| e.to_string())?;
    Ok(manager.list_candidates())
}

#[tauri::command]
pub async fn test_mirrors(name: String) -> Result<Vec<SpeedTestResult>, String> {
    let manager = get_manager(&name).map_err(|e| e.to_string())?;
    let mirrors = manager.list_candidates();
    let results = benchmark_mirrors(mirrors).await;

    Ok(results.into_iter().map(|r| SpeedTestResult {
        name: r.mirror.name,
        url: r.mirror.url,
        latency_ms: if r.latency_ms == u64::MAX { 0 } else { r.latency_ms },
        is_timeout: r.latency_ms == u64::MAX,
    }).collect())
}

#[tauri::command]
pub async fn test_single_mirror(url: String) -> Result<u64, String> {
    let mirror = Mirror::new("test", &url);
    let results = benchmark_mirrors(vec![mirror]).await;
    if let Some(r) = results.first() {
        if r.latency_ms == u64::MAX {
            Ok(9999)
        } else {
            Ok(r.latency_ms)
        }
    } else {
        Ok(9999)
    }
}

#[tauri::command]
pub async fn apply_mirror(name: String, mirror: Mirror) -> Result<(), String> {
    let manager = get_manager(&name).map_err(|e| e.to_string())?;
    manager.set_source(&mirror).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn restore_default(name: String) -> Result<(), String> {
    let manager = get_manager(&name).map_err(|e| e.to_string())?;
    manager.restore().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn apply_fastest_mirror(name: String) -> Result<Mirror, String> {
    let manager = get_manager(&name).map_err(|e| e.to_string())?;
    let mirrors = manager.list_candidates();
    let results = benchmark_mirrors(mirrors).await;

    let fastest = results.into_iter()
        .filter(|r| r.latency_ms < u64::MAX)
        .min_by_key(|r| r.latency_ms)
        .ok_or_else(|| "所有镜像源均超时".to_string())?;

    manager.set_source(&fastest.mirror).await.map_err(|e| e.to_string())?;
    Ok(fastest.mirror)
}

#[tauri::command]
pub async fn sync_java_mirrors(mirror_name: String) -> Result<(), String> {
    let maven_manager = get_manager("maven").map_err(|e| e.to_string())?;
    let gradle_manager = get_manager("gradle").map_err(|e| e.to_string())?;

    let maven_mirrors = maven_manager.list_candidates();
    let gradle_mirrors = gradle_manager.list_candidates();

    let maven_mirror = maven_mirrors.iter()
        .find(|m| m.name.eq_ignore_ascii_case(&mirror_name))
        .ok_or_else(|| format!("Maven 镜像源 {} 不存在", mirror_name))?;

    let gradle_mirror = gradle_mirrors.iter()
        .find(|m| m.name.eq_ignore_ascii_case(&mirror_name))
        .ok_or_else(|| format!("Gradle 镜像源 {} 不存在", mirror_name))?;

    maven_manager.set_source(maven_mirror).await.map_err(|e| e.to_string())?;
    gradle_manager.set_source(gradle_mirror).await.map_err(|e| e.to_string())?;

    Ok(())
}
