// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod sidecar;
mod error;

use tauri::Manager;
use crate::commands::AppState;

fn main() {
    run()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // 初始化全局状态
            let sidecar_manager = sidecar::SidecarManager::new();
            app.manage(AppState {
                sidecar: std::sync::Mutex::new(sidecar_manager),
            });

            // 只在生产环境自动启动 sidecar
            #[cfg(not(debug_assertions))]
            {
                if let Some(window) = app.get_webview_window("main") {
                    let state = window.state::<AppState>();

                    // 在后台启动 sidecar
                    std::thread::spawn(move || {
                        let mut manager = state.sidecar.lock().unwrap();
                        match manager.start() {
                            Ok(port) => {
                                tracing::info!("Sidecar auto-started on port {}", port);
                                if let Some(port) = manager.port() {
                                    let _ = window.emit("sidecar-ready", port);
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to auto-start sidecar: {}", e);
                            }
                        }
                    });
                }
            }

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::start_sidecar,
            commands::stop_sidecar,
            commands::get_sidecar_port,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
