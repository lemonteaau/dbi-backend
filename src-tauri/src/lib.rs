mod commands;
mod dbi;

use commands::AppState;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState {
            files: Mutex::new(HashMap::new()),
            server_running: Mutex::new(false),
            stop_flag: Arc::new(AtomicBool::new(false)),
        })
        .invoke_handler(tauri::generate_handler![
            commands::add_files,
            commands::add_folder,
            commands::add_paths,
            commands::clear_files,
            commands::get_file_list,
            commands::remove_file,
            commands::start_server,
            commands::stop_server,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
