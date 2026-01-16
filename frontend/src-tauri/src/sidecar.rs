//! Sidecar 进程管理模块
//!
//! 负责启动、停止和监控 vibe-kanban-server 进程

use std::{
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};

use crate::error::{AppError, Result};

/// Sidecar 进程管理器
pub struct SidecarManager {
    /// 子进程句柄
    child: Option<std::process::Child>,
    /// 服务器端口
    port: Option<u16>,
}

impl SidecarManager {
    /// 创建新的 Sidecar 管理器
    pub fn new() -> Self {
        Self {
            child: None,
            port: None,
        }
    }

    /// 启动 sidecar server
    ///
    /// # 返回
    /// 返回服务器监听的端口号
    pub fn start(&mut self) -> Result<u16> {
        // 1. 获取 sidecar 二进制路径
        let sidecar_path = Self::get_sidecar_path()?;

        tracing::info!("Starting sidecar at: {}", sidecar_path.display());

        // 2. 启动进程（让 server 自动分配端口）
        let child = Command::new(&sidecar_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| AppError::SidecarError(format!("Failed to start: {}", e)))?;

        self.child = Some(child);

        // 3. 轮询读取端口文件
        let port = self.wait_for_port_file(Duration::from_secs(15))?;
        self.port = Some(port);

        tracing::info!("Sidecar started on port {}", port);
        Ok(port)
    }

    /// 等待并读取端口文件
    ///
    /// # 参数
    /// * `timeout` - 超时时间
    ///
    /// # 返回
    /// 返回读取到的端口号
    fn wait_for_port_file(&self, timeout: Duration) -> Result<u16> {
        let port_file = std::env::temp_dir()
            .join("vibe-kanban")
            .join("vibe-kanban.port");

        tracing::debug!("Waiting for port file at: {}", port_file.display());

        let start = std::time::Instant::now();
        let mut last_error = String::new();

        loop {
            if start.elapsed() > timeout {
                return Err(AppError::SidecarError(format!(
                    "Timeout waiting for port file. Last error: {}",
                    last_error
                )));
            }

            // 尝试读取端口文件
            match std::fs::read_to_string(&port_file) {
                Ok(content) => {
                    let content = content.trim();
                    if let Ok(port) = content.parse::<u16>() {
                        // 验证端口可连接
                        match Self::check_port(port) {
                            Ok(()) => {
                                tracing::info!("Port file read successfully: {}", port);
                                return Ok(port);
                            }
                            Err(e) => {
                                last_error = e.to_string();
                                tracing::debug!("Port {} not ready yet: {}", port, e);
                            }
                        }
                    } else {
                        last_error = format!("Invalid port number in file: {}", content);
                        tracing::debug!("{}", last_error);
                    }
                }
                Err(e) => {
                    // 文件尚未创建，继续等待
                    last_error = format!("Port file not readable: {}", e);
                    tracing::trace!("Port file not ready: {}", e);
                }
            }

            std::thread::sleep(Duration::from_millis(200));
        }
    }

    /// 检查端口是否可连接
    ///
    /// # 参数
    /// * `port` - 要检查的端口号
    fn check_port(port: u16) -> Result<()> {
        use std::net::TcpStream;

        // 给服务器一点时间完全启动
        std::thread::sleep(Duration::from_millis(100));

        let timeout = Duration::from_secs(2);
        let socket_addr = format!("127.0.0.1:{}", port);
        TcpStream::connect_timeout(&socket_addr.parse()?, timeout)
            .map_err(|_| AppError::ServerNotReady)?;
        Ok(())
    }

    /// 停止 sidecar
    pub fn stop(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            tracing::info!("Stopping sidecar process...");

            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                let _ = child.kill();
            }
            #[cfg(windows)]
            {
                let _ = child.kill();
            }
            let _ = child.wait();

            self.port = None;
            tracing::info!("Sidecar stopped");
        }
        Ok(())
    }

    /// 获取当前端口
    pub fn port(&self) -> Option<u16> {
        self.port
    }

    /// 获取 sidecar 二进制路径
    fn get_sidecar_path() -> Result<PathBuf> {
        #[cfg(debug_assertions)]
        {
            // 开发环境: 使用相对路径
            let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
                .unwrap_or_else(|_| ".".to_string());
            let path = PathBuf::from(manifest_dir).join("../../target/release/server");

            if !path.exists() {
                return Err(AppError::SidecarError(format!(
                    "Server binary not found at {}. Run: cargo build --release --bin server",
                    path.display()
                )));
            }

            Ok(path)
        }

        #[cfg(not(debug_assertions))]
        {
            // 生产环境: Tauri 会处理路径
            std::env::var("SIDECAR_BIN")
                .map(PathBuf::from)
                .or_else(|_| {
                    // Fallback: 尝试相对路径
                    std::env::var("CARGO_MANIFEST_DIR").map(|dir| {
                        PathBuf::from(dir).join("../../target/release/server")
                    })
                })
                .map_err(|_| AppError::SidecarError("SIDECAR_BIN not set".to_string()))
        }
    }
}

impl Default for SidecarManager {
    fn default() -> Self {
        Self::new()
    }
}

// 实现 Drop 以确保进程被清理
impl Drop for SidecarManager {
    fn drop(&mut self) {
        if self.child.is_some() {
            tracing::warn!("SidecarManager dropped without explicit stop, cleaning up...");
            let _ = self.stop();
        }
    }
}
