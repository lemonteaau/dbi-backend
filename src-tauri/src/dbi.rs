/// DBI USB protocol implementation for Nintendo Switch.
///
/// This module implements the DBI backend protocol, communicating with the
/// DBI homebrew app on Switch over USB bulk transfers.
///
/// Uses nusb 0.2.x with its `std::io::Read` / `std::io::Write` high-level API.
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use nusb::transfer::{Bulk, In, Out};
use nusb::MaybeFuture;
use tauri::{AppHandle, Emitter};

// ===== DBI Protocol Constants =====

const SWITCH_VENDOR_ID: u16 = 0x057E;
const SWITCH_PRODUCT_ID: u16 = 0x3000;

const CMD_ID_EXIT: u32 = 0;
const CMD_ID_FILE_RANGE: u32 = 2;
const CMD_ID_LIST: u32 = 3;

const CMD_TYPE_RESPONSE: u32 = 1;
const CMD_TYPE_ACK: u32 = 2;

const MAGIC: &[u8; 4] = b"DBI0";

/// 1 MB chunk size for file transfer, matching the Python original.
const BUFFER_SEGMENT_DATA_SIZE: usize = 0x100000;

/// USB endpoint addresses.
const EP_OUT: u8 = 0x01;
const EP_IN: u8 = 0x81;

/// Buffer size for the endpoint reader/writer (slightly larger than our chunk).
const EP_BUF_SIZE: usize = BUFFER_SEGMENT_DATA_SIZE + 4096;

// ===== Helper Functions =====

/// Build a 16-byte DBI command header.
fn build_header(cmd_type: u32, cmd_id: u32, data_size: u32) -> Vec<u8> {
    let mut buf = Vec::with_capacity(16);
    buf.extend_from_slice(MAGIC);
    buf.extend_from_slice(&cmd_type.to_le_bytes());
    buf.extend_from_slice(&cmd_id.to_le_bytes());
    buf.extend_from_slice(&data_size.to_le_bytes());
    buf
}

/// Emit a log message to the frontend.
fn emit_log(app: &AppHandle, message: &str, level: &str) {
    let _ = app.emit(
        "log",
        serde_json::json!({ "message": message, "level": level }),
    );
}

/// Emit connection status to the frontend.
fn emit_connection(app: &AppHandle, connected: bool) {
    let _ = app.emit(
        "connection-status",
        serde_json::json!({ "connected": connected }),
    );
}

/// Emit transfer progress to the frontend.
fn emit_progress(app: &AppHandle, file: &str, bytes_sent: u64, total_bytes: u64) {
    let _ = app.emit(
        "transfer-progress",
        serde_json::json!({ "file": file, "bytes_sent": bytes_sent, "total_bytes": total_bytes }),
    );
}

/// Format byte size to human-readable string.
fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

// ===== USB I/O wrappers =====

/// USB I/O handle that wraps nusb's EndpointRead and EndpointWrite.
struct UsbIo {
    reader: nusb::io::EndpointRead<Bulk>,
    writer: nusb::io::EndpointWrite<Bulk>,
}

impl UsbIo {
    /// Write data to the OUT endpoint.
    fn write_all(&mut self, data: &[u8]) -> Result<(), String> {
        self.writer
            .write_all(data)
            .map_err(|e| format!("USB write error: {e}"))?;
        self.writer
            .flush()
            .map_err(|e| format!("USB flush error: {e}"))?;
        Ok(())
    }

    /// Read exactly `len` bytes from the IN endpoint.
    fn read_exact(&mut self, len: usize) -> Result<Vec<u8>, String> {
        let mut buf = vec![0u8; len];
        self.reader
            .read_exact(&mut buf)
            .map_err(|e| format!("USB read error: {e}"))?;
        Ok(buf)
    }
}

// ===== Transfer Statistics =====

/// Tracks cumulative transfer statistics for a session.
struct TransferStats {
    files_transferred: u32,
    total_bytes_sent: u64,
}

impl TransferStats {
    fn new() -> Self {
        Self {
            files_transferred: 0,
            total_bytes_sent: 0,
        }
    }

    fn record(&mut self, bytes: u64) {
        self.files_transferred += 1;
        self.total_bytes_sent += bytes;
    }

    fn summary(&self) -> String {
        if self.files_transferred == 0 {
            "No files were transferred".to_string()
        } else {
            format!(
                "Transferred {} file{} ({})",
                self.files_transferred,
                if self.files_transferred > 1 { "s" } else { "" },
                format_size(self.total_bytes_sent)
            )
        }
    }
}

// ===== DBI Command Handlers =====

/// Handle CMD_ID_LIST: send the file name list to Switch.
fn process_list_command(
    usb: &mut UsbIo,
    file_list: &HashMap<String, PathBuf>,
    app: &AppHandle,
) -> Result<(), String> {
    emit_log(app, "Switch requested file list", "info");

    let nsp_path_list: String = file_list.keys().map(|k| format!("{k}\n")).collect();
    let nsp_bytes = nsp_path_list.as_bytes();
    let nsp_len = nsp_bytes.len() as u32;

    // Send response header with total list length
    usb.write_all(&build_header(CMD_TYPE_RESPONSE, CMD_ID_LIST, nsp_len))?;

    if nsp_len > 0 {
        // Wait for ACK from Switch
        let _ack = usb.read_exact(16)?;
        // Send the actual file list
        usb.write_all(nsp_bytes)?;
    }

    emit_log(
        app,
        &format!("Sent file list ({} files)", file_list.len()),
        "success",
    );
    Ok(())
}

/// Handle CMD_ID_FILE_RANGE: send a chunk of file data to Switch.
/// Returns the total bytes sent for this range on success.
fn process_file_range_command(
    usb: &mut UsbIo,
    data_size: u32,
    file_list: &HashMap<String, PathBuf>,
    app: &AppHandle,
    stop: &AtomicBool,
) -> Result<u64, String> {
    // ACK the file range request
    usb.write_all(&build_header(CMD_TYPE_ACK, CMD_ID_FILE_RANGE, data_size))?;

    // Read the file range header from Switch
    let header_data = usb.read_exact(data_size as usize)?;

    if header_data.len() < 16 {
        return Err("File range header too short".into());
    }

    let range_size = u32::from_le_bytes(header_data[0..4].try_into().unwrap()) as u64;
    let range_offset = u64::from_le_bytes(header_data[4..12].try_into().unwrap());
    let _nsp_name_len = u32::from_le_bytes(header_data[12..16].try_into().unwrap());
    let nsp_name = String::from_utf8_lossy(&header_data[16..]).to_string();

    emit_log(
        app,
        &format!(
            "Sending: {nsp_name} (offset={}, size={})",
            format_size(range_offset),
            format_size(range_size)
        ),
        "info",
    );

    // Send response header with range_size
    usb.write_all(&build_header(
        CMD_TYPE_RESPONSE,
        CMD_ID_FILE_RANGE,
        range_size as u32,
    ))?;

    // Wait for ACK
    let _ack = usb.read_exact(16)?;

    // Find the file in our list
    let file_path = file_list
        .get(&nsp_name)
        .ok_or_else(|| format!("File not found: {nsp_name}"))?;

    // Open file, seek, and transfer in chunks
    let mut f = File::open(file_path).map_err(|e| format!("Failed to open file: {e}"))?;
    f.seek(SeekFrom::Start(range_offset))
        .map_err(|e| format!("Failed to seek: {e}"))?;

    let mut remaining = range_size;
    let mut sent: u64 = 0;

    while remaining > 0 {
        // Check stop signal between chunks
        if stop.load(Ordering::Relaxed) {
            return Err("Server stopped by user".into());
        }

        let chunk_size = std::cmp::min(remaining as usize, BUFFER_SEGMENT_DATA_SIZE);
        let mut buf = vec![0u8; chunk_size];
        f.read_exact(&mut buf)
            .map_err(|e| format!("Failed to read file: {e}"))?;

        usb.write_all(&buf)?;

        sent += chunk_size as u64;
        remaining -= chunk_size as u64;

        // Emit progress every 4 MB or on completion
        if sent % (4 * 1024 * 1024) < BUFFER_SEGMENT_DATA_SIZE as u64 || remaining == 0 {
            emit_progress(app, &nsp_name, sent, range_size);
        }
    }

    // Only log completion when the entire range is at offset 0 or when this is the last range
    if range_offset == 0 && remaining == 0 {
        emit_log(
            app,
            &format!("Completed: {nsp_name} ({})", format_size(range_size)),
            "success",
        );
    }

    Ok(sent)
}

/// Handle CMD_ID_EXIT: send response and signal the frontend.
fn process_exit_command(usb: &mut UsbIo, app: &AppHandle) -> Result<(), String> {
    emit_log(app, "Switch sent exit command", "info");
    usb.write_all(&build_header(CMD_TYPE_RESPONSE, CMD_ID_EXIT, 0))?;
    Ok(())
}

// ===== Main Server Loop =====

/// Reason the server stopped.
pub enum StopReason {
    /// Switch sent CMD_EXIT — normal completion.
    Completed(String),
    /// User clicked Stop.
    UserStopped,
    /// An unrecoverable error occurred.
    Error(String),
}

/// Connect to the Switch and run the DBI command loop.
///
/// This function blocks the calling thread. It should be spawned on a
/// background thread via `std::thread::spawn`.
///
/// `stop_flag` is set to `true` by the UI when the user clicks "Stop Server".
pub fn run_server(
    file_list: Arc<std::sync::Mutex<HashMap<String, PathBuf>>>,
    app: AppHandle,
    stop_flag: Arc<AtomicBool>,
) -> StopReason {
    let mut stats = TransferStats::new();

    loop {
        // ---- Check stop before each phase ----
        if stop_flag.load(Ordering::Relaxed) {
            emit_log(&app, "Server stopped by user", "warn");
            emit_connection(&app, false);
            return StopReason::UserStopped;
        }

        // ---- Connection Phase ----
        emit_log(&app, "Waiting for Switch...", "info");
        emit_connection(&app, false);

        let device = loop {
            if stop_flag.load(Ordering::Relaxed) {
                emit_log(&app, "Server stopped by user", "warn");
                return StopReason::UserStopped;
            }

            let devices = match nusb::list_devices().wait() {
                Ok(d) => d,
                Err(e) => {
                    emit_log(&app, &format!("USB enumeration error: {e}"), "error");
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    continue;
                }
            };

            let found = devices
                .filter(|d| {
                    d.vendor_id() == SWITCH_VENDOR_ID && d.product_id() == SWITCH_PRODUCT_ID
                })
                .next();

            match found {
                Some(info) => match info.open().wait() {
                    Ok(dev) => break dev,
                    Err(e) => {
                        emit_log(&app, &format!("Failed to open device: {e}"), "error");
                        std::thread::sleep(std::time::Duration::from_secs(2));
                    }
                },
                None => {
                    // Short sleep but check stop more frequently
                    for _ in 0..10 {
                        if stop_flag.load(Ordering::Relaxed) {
                            emit_log(&app, "Server stopped by user", "warn");
                            return StopReason::UserStopped;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                }
            }
        };

        // Claim interface 0
        let iface = match device.claim_interface(0).wait() {
            Ok(i) => i,
            Err(e) => {
                emit_log(&app, &format!("Failed to claim interface: {e}"), "error");
                std::thread::sleep(std::time::Duration::from_secs(2));
                continue;
            }
        };

        // Open bulk endpoints and create reader/writer
        let ep_in = match iface.endpoint::<Bulk, In>(EP_IN) {
            Ok(ep) => ep,
            Err(e) => {
                emit_log(&app, &format!("Failed to open IN endpoint: {e}"), "error");
                std::thread::sleep(std::time::Duration::from_secs(2));
                continue;
            }
        };

        let ep_out = match iface.endpoint::<Bulk, Out>(EP_OUT) {
            Ok(ep) => ep,
            Err(e) => {
                emit_log(&app, &format!("Failed to open OUT endpoint: {e}"), "error");
                std::thread::sleep(std::time::Duration::from_secs(2));
                continue;
            }
        };

        let mut usb = UsbIo {
            reader: ep_in.reader(EP_BUF_SIZE),
            writer: ep_out.writer(EP_BUF_SIZE),
        };

        emit_log(&app, "Switch connected!", "success");
        emit_connection(&app, true);

        // ---- Command Loop ----
        loop {
            // Check stop between commands
            if stop_flag.load(Ordering::Relaxed) {
                emit_log(&app, "Server stopped by user", "warn");
                emit_connection(&app, false);
                return StopReason::UserStopped;
            }

            let header = match usb.read_exact(16) {
                Ok(h) => h,
                Err(e) => {
                    emit_log(&app, &format!("Connection lost: {e}"), "warn");
                    emit_connection(&app, false);
                    break; // Back to connection phase
                }
            };

            if header.len() < 16 || &header[0..4] != MAGIC {
                continue;
            }

            let _cmd_type = u32::from_le_bytes(header[4..8].try_into().unwrap());
            let cmd_id = u32::from_le_bytes(header[8..12].try_into().unwrap());
            let data_size = u32::from_le_bytes(header[12..16].try_into().unwrap());

            let files = file_list.lock().unwrap().clone();

            match cmd_id {
                CMD_ID_EXIT => {
                    let _ = process_exit_command(&mut usb, &app);
                    let summary = stats.summary();
                    emit_log(&app, &summary, "success");
                    emit_connection(&app, false);
                    return StopReason::Completed(summary);
                }
                CMD_ID_LIST => {
                    if let Err(e) = process_list_command(&mut usb, &files, &app) {
                        emit_log(&app, &format!("Error: {e}"), "error");
                        emit_connection(&app, false);
                        break;
                    }
                }
                CMD_ID_FILE_RANGE => {
                    match process_file_range_command(
                        &mut usb,
                        data_size,
                        &files,
                        &app,
                        &stop_flag,
                    ) {
                        Ok(bytes_sent) => {
                            stats.record(bytes_sent);
                        }
                        Err(e) => {
                            emit_log(&app, &format!("Error: {e}"), "error");
                            emit_connection(&app, false);
                            if stop_flag.load(Ordering::Relaxed) {
                                return StopReason::UserStopped;
                            }
                            break; // Reconnect
                        }
                    }
                }
                other => {
                    emit_log(&app, &format!("Unknown command: {other}"), "warn");
                }
            }
        }
    }
}
