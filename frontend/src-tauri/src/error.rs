//! Tauri 应用错误类型定义

use std::sync::PoisonError;
use thiserror::Error;

/// 应用错误类型
#[derive(Debug, Error)]
pub enum AppError {
    /// Sidecar 进程相关错误
    #[error("Sidecar process error: {0}")]
    SidecarError(String),

    /// 端口文件读取错误
    #[error("Failed to read port file: {0}")]
    PortFileError(String),

    /// 服务器未就绪
    #[error("Server not ready")]
    ServerNotReady,

    /// IO 错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// 解析错误
    #[error("Parse error: {0}")]
    Parse(#[from] std::net::AddrParseError),

    /// Mutex 锁错误
    #[error("Mutex lock error: {0}")]
    MutexLock(String),
}

/// 结果类型别名
pub type Result<T> = std::result::Result<T, AppError>;

impl<T> From<PoisonError<T>> for AppError {
    fn from(err: PoisonError<T>) -> Self {
        AppError::MutexLock(format!("Mutex poisoned: {}", err))
    }
}

impl AppError {
    /// 转换为字符串（用于 Tauri command 返回）
    pub fn to_string_error(&self) -> String {
        self.to_string()
    }
}
