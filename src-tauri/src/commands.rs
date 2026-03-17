/// Tauri IPC command handlers for the DBI Backend app.
///
/// These commands are invoked from the frontend via `window.__TAURI__.core.invoke()`.
/// They manage the file list (shared state) and control the USB server lifecycle.
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::dbi;

/// Shared application state holding the file list.
pub struct AppState {
    pub files: Mutex<HashMap<String, PathBuf>>,
    pub server_running: Mutex<bool>,
    pub stop_flag: Arc<AtomicBool>,
}

/// File info returned to the frontend.
#[derive(Serialize, Clone)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
}

/// Valid file extensions for Switch installable files.
const VALID_EXTENSIONS: &[&str] = &["nsp", "nsz", "xci", "xcz"];

fn is_valid_file(path: &PathBuf) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| VALID_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Add a single file to the file list.
fn add_file_to_map(files: &mut HashMap<String, PathBuf>, path: PathBuf) {
    if is_valid_file(&path) && path.is_file() {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            files.insert(name.to_string(), path);
        }
    }
}

// ===== Tauri Commands =====

/// Add individual files by their full paths.
#[tauri::command]
pub fn add_files(paths: Vec<String>, state: State<'_, AppState>) -> Result<(), String> {
    let mut files = state.files.lock().map_err(|e| e.to_string())?;
    for p in paths {
        add_file_to_map(&mut files, PathBuf::from(p));
    }
    Ok(())
}

/// Add a folder: recursively scan for NSP/NSZ/XCI/XCZ files.
#[tauri::command]
pub fn add_folder(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let dir = PathBuf::from(&path);
    if !dir.is_dir() {
        return Err(format!("Not a directory: {path}"));
    }

    let mut files = state.files.lock().map_err(|e| e.to_string())?;
    scan_directory(&dir, &mut files);
    Ok(())
}

/// Recursively scan a directory for valid game files.
fn scan_directory(dir: &PathBuf, files: &mut HashMap<String, PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                scan_directory(&path, files);
            } else {
                add_file_to_map(files, path);
            }
        }
    }
}

/// Add paths that can be either files or folders (used by drag-and-drop).
#[tauri::command]
pub fn add_paths(paths: Vec<String>, state: State<'_, AppState>) -> Result<(), String> {
    let mut files = state.files.lock().map_err(|e| e.to_string())?;
    for p in paths {
        let path = PathBuf::from(&p);
        if path.is_dir() {
            scan_directory(&path, &mut files);
        } else {
            add_file_to_map(&mut files, path);
        }
    }
    Ok(())
}

/// Clear all files from the list.
#[tauri::command]
pub fn clear_files(state: State<'_, AppState>) -> Result<(), String> {
    let mut files = state.files.lock().map_err(|e| e.to_string())?;
    files.clear();
    Ok(())
}

/// Remove a single file by name.
#[tauri::command]
pub fn remove_file(name: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut files = state.files.lock().map_err(|e| e.to_string())?;
    files.remove(&name);
    Ok(())
}

/// Get the current file list with metadata.
#[tauri::command]
pub fn get_file_list(state: State<'_, AppState>) -> Result<Vec<FileInfo>, String> {
    let files = state.files.lock().map_err(|e| e.to_string())?;
    let mut list: Vec<FileInfo> = files
        .iter()
        .map(|(name, path)| {
            let size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
            FileInfo {
                name: name.clone(),
                path: path.to_string_lossy().to_string(),
                size,
            }
        })
        .collect();
    list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(list)
}

/// Start the DBI USB server on a background thread.
#[tauri::command]
pub fn start_server(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    // Check if server is already running
    {
        let running = state.server_running.lock().map_err(|e| e.to_string())?;
        if *running {
            return Err("Server is already running".into());
        }
    }

    // Check if there are files to serve
    {
        let files = state.files.lock().map_err(|e| e.to_string())?;
        if files.is_empty() {
            return Err("No files to serve".into());
        }
    }

    // Reset stop flag and mark server as running
    state.stop_flag.store(false, Ordering::SeqCst);
    {
        let mut running = state.server_running.lock().map_err(|e| e.to_string())?;
        *running = true;
    }

    // Clone what we need for the background thread
    let file_list = {
        let files = state.files.lock().map_err(|e| e.to_string())?;
        Arc::new(std::sync::Mutex::new(files.clone()))
    };
    let stop_flag = state.stop_flag.clone();
    let app_handle = app.clone();

    std::thread::spawn(move || {
        let reason = dbi::run_server(file_list, app_handle.clone(), stop_flag);

        // Emit summary based on stop reason
        let summary = match &reason {
            dbi::StopReason::Completed(s) => s.clone(),
            dbi::StopReason::UserStopped => "Server stopped by user".to_string(),
            dbi::StopReason::Error(e) => format!("Server error: {e}"),
        };

        let _ = app_handle.emit(
            "server-stopped",
            serde_json::json!({ "summary": summary }),
        );

        // Reset server_running
        {
            let state = app_handle.state::<AppState>();
            if let Ok(mut running) = state.server_running.lock() {
                *running = false;
            };
        }
    });

    Ok(())
}

/// Stop the DBI USB server.
#[tauri::command]
pub fn stop_server(state: State<'_, AppState>) -> Result<(), String> {
    let running = state.server_running.lock().map_err(|e| e.to_string())?;
    if !*running {
        return Err("Server is not running".into());
    }

    state.stop_flag.store(true, Ordering::SeqCst);
    Ok(())
}
