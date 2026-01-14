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
            get_system_info,
            get_tool_info,
            get_all_tools_info,
            get_version_manager_info,
            switch_version,
            install_tool,
            sync_java_home,
            check_version_update,
            check_all_updates,
            check_tool_conflict,
            check_all_conflicts,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
