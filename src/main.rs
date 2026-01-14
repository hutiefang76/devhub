#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use devhub::commands::*;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            list_supported_tools,
            get_tool_status,
            get_all_status,
            list_mirrors,
            test_mirrors,
            test_single_mirror,
            apply_mirror,
            restore_default,
            apply_fastest_mirror,
            sync_java_mirrors,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
