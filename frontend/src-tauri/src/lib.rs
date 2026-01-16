// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod sidecar;
mod error;

use tauri::{Manager, Emitter};
use crate::commands::AppState;
use std::sync::Arc;

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

            // 获取 state 的 Arc，然后提取内部的 Arc<Mutex<SidecarManager>>
            let state = app.state::<AppState>();
            let app_state_arc = state.inner().clone();
            let sidecar_arc = Arc::clone(&app_state_arc.sidecar);

            // 自动启动 Sidecar（开发模式和生产模式都启动）
            if let Some(window) = app.get_webview_window("main") {
                let window_clone = window.clone();

                // 在后台启动 sidecar
                std::thread::spawn(move || {
                    let mut manager = sidecar_arc.lock().unwrap();
                    match manager.start() {
                        Ok(port) => {
                            tracing::info!("Sidecar auto-started on port {}", port);
                            if let Some(port) = manager.port() {
                                let _ = window_clone.emit("sidecar-ready", port);
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to auto-start sidecar: {}", e);
                            let _ = window_clone.emit("sidecar-error", e.to_string());
                        }
                    }
                });
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
