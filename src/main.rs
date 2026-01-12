#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod core;
mod services;
mod tools;

use commands::*;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            detect_python,
            get_current_pip_mirror,
            list_pip_mirrors,
            apply_pip_mirror,
            restore_pip_default,
            test_mirrors_speed,
            test_mirror_speed,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
