/**
 * React Hook: 管理 Sidecar Server 状态
 *
 * 监听 Sidecar 启动事件，管理服务器连接状态
 */

import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { isTauri, setSidecarPort } from '../lib/api-config';

/**
 * 服务器状态类型
 */
export type ServerStatus = 'unknown' | 'starting' | 'ready' | 'error';

/**
 * 服务器状态接口
 */
export interface ServerStatusState {
  status: ServerStatus;
  port: number | null;
  error?: string;
}

/**
 * 管理 Sidecar Server 状态
 *
 * @returns 服务器状态对象
 */
export function useServerStatus(): ServerStatusState {
  const [state, setState] = useState<ServerStatusState>({
    status: 'unknown',
    port: null,
  });

  useEffect(() => {
    // 如果不在 Tauri 环境中，假设服务器就绪（浏览器环境）
    if (!isTauri()) {
      setState({ status: 'ready', port: null });
      return;
    }

    // Tauri 环境（开发模式和生产模式）都使用 Sidecar
    // 初始状态为 starting，等待 Sidecar 启动
    setState({ status: 'starting', port: null });

    // 监听 sidecar 就绪事件
    const unlistenPromise = listen<number>('sidecar-ready', (event) => {
      console.log('[Sidecar] Ready on port:', event.payload);
      setSidecarPort(event.payload);
      setState({
        status: 'ready',
        port: event.payload,
      });
    });

    // 监听 sidecar 错误事件
    const unlistenErrorPromise = listen<string>('sidecar-error', (event) => {
      console.error('[Sidecar] Error:', event.payload);
      setState({
        status: 'error',
        port: null,
        error: event.payload,
      });
    });

    // 检查当前状态（Sidecar 可能已经启动）
    invoke<number>('get_sidecar_port')
      .then((port) => {
        if (port) {
          setSidecarPort(port);
          setState({ status: 'ready', port });
        }
        // 如果没有端口，保持 starting 状态，等待 sidecar-ready 事件
      })
      .catch((err) => {
        console.error('[Sidecar] Failed to get port:', err);
        setState({
          status: 'error',
          port: null,
          error: err.toString(),
        });
      });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
      unlistenErrorPromise.then((unlisten) => unlisten());
    };
  }, []);

  return state;
}
