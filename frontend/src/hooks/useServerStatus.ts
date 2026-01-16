/**
 * React Hook: 管理 Sidecar Server 状态
 *
 * 监听 Sidecar 启动事件，管理服务器连接状态
 */

import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { isTauri, isDev, setSidecarPort } from '../lib/api-config';

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

    // 开发模式: 使用 Vite 代理，服务器已就绪
    if (isDev()) {
      setState({ status: 'ready', port: null });
      return;
    }

    // 生产模式: 监听 sidecar 启动事件
    const unlistenPromise = listen<number>('sidecar-ready', (event) => {
      console.log('[Sidecar] Ready on port:', event.payload);
      // 设置全局端口
      setSidecarPort(event.payload);
      setState({
        status: 'ready',
        port: event.payload,
      });
    });

    // 检查当前状态
    invoke<number>('get_sidecar_port')
      .then((port) => {
        if (port) {
          setSidecarPort(port);
          setState({ status: 'ready', port });
        } else {
          setState({ status: 'starting', port: null });
        }
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
    };
  }, []);

  return state;
}
