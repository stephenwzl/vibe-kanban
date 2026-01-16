//! Tauri 命令定义
//!
//! 暴露给前端调用的 Tauri 命令

use tauri::{AppHandle, Emitter};

use crate::error::{AppError, Result as AppResult};
use crate::sidecar::SidecarManager;

/// 全局状态的 Mutex 类型别名
pub type MutexSidecar = std::sync::Mutex<SidecarManager>;

/// 应用全局状态
pub struct AppState {
    /// Sidecar 管理器
    pub sidecar: MutexSidecar,
}

/// 启动 Sidecar Server
///
/// 启动 vibe-kanban-server 进程并等待其就绪
#[tauri::command]
pub fn start_sidecar(
    state: tauri::State<'_, AppState>,
    app: AppHandle,
) -> std::result::Result<u16, String> {
    let mut manager = state.sidecar.lock()
        .map_err(|e| format!("Lock failed: {}", e))?;

    let port = manager.start()
        .map_err(|e| e.to_string())?;

    // 通知前端服务器就绪
    app.emit("sidecar-ready", port)
        .map_err(|e| format!("Emit failed: {}", e))?;

    Ok(port)
}

/// 停止 Sidecar Server
///
/// 停止 vibe-kanban-server 进程
#[tauri::command]
pub fn stop_sidecar(state: tauri::State<'_, AppState>) -> std::result::Result<(), String> {
    let mut manager = state.sidecar.lock()
        .map_err(|e| format!("Lock failed: {}", e))?;

    manager.stop()
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// 获取 Sidecar 端口
///
/// 获取当前运行的 Sidecar 服务器端口号
#[tauri::command]
pub fn get_sidecar_port(state: tauri::State<'_, AppState>) -> std::result::Result<Option<u16>, String> {
    let manager = state.sidecar.lock()
        .map_err(|e| format!("Lock failed: {}", e))?;

    Ok(manager.port())
}
