//! Tauri IPC Command层
//!
//! 定义前端可以调用的所有命令接口。
//! 这些命令通过Tauri的IPC机制与前端通信。

use crate::core::detector::*;
use crate::core::mirror::*;
use crate::services::speed_test::SpeedTestService;
use crate::tools::python::*;

/// 检测Python环境
///
/// # 前端调用
/// ```typescript
/// import { invoke } from '@tauri-apps/api/tauri'
///
/// const info = await invoke<DetectionInfo>('detect_python')
/// console.log('Python已安装:', info.installed)
/// ```
#[tauri::command]
pub fn detect_python() -> Result<DetectionInfo, String> {
    let detector = PythonDetector::new();
    detector.detect().map_err(|e| e.to_string())
}

/// 获取当前pip镜像源
#[tauri::command]
pub fn get_current_pip_mirror() -> Result<Option<String>, String> {
    let config = PipMirror::new();
    config.get_current().map_err(|e| e.to_string())
}

/// 列出pip镜像源
#[tauri::command]
pub fn list_pip_mirrors() -> Vec<Mirror> {
    let config = PipMirror::new();
    config.list_mirrors()
}

/// 应用pip镜像源
#[tauri::command]
pub fn apply_pip_mirror(mirror: Mirror) -> Result<(), String> {
    let config = PipMirror::new();
    config.apply(&mirror).map_err(|e| e.to_string())
}

/// 恢复pip默认配置
#[tauri::command]
pub fn restore_pip_default() -> Result<(), String> {
    let config = PipMirror::new();
    config.restore_default().map_err(|e| e.to_string())
}

/// 批量测试镜像源速度
#[tauri::command]
pub async fn test_mirrors_speed(urls: Vec<String>) -> Vec<u64> {
    let service = SpeedTestService::new();
    service.test_mirrors(urls).await
}

/// 测试单个镜像源速度
#[tauri::command]
pub async fn test_mirror_speed(url: String) -> u64 {
    let service = SpeedTestService::new();
    service.test_mirror(&url).await
}

