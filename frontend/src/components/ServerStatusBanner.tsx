/**
 * 服务器状态横幅组件
 *
 * 显示 Sidecar 服务器连接状态
 */

import React from 'react';
import { useServerStatus } from '../hooks/useServerStatus';

/**
 * 服务器状态横幅组件
 *
 * 在服务器未就绪时显示状态信息
 */
export function ServerStatusBanner() {
  const { status, error } = useServerStatus();

  // 如果服务器就绪，不显示横幅
  if (status === 'ready') {
    return null;
  }

  // 根据状态显示不同内容
  const getMessage = () => {
    switch (status) {
      case 'starting':
        return '正在启动后端服务器...';
      case 'error':
        return `后端错误: ${error || '未知错误'}`;
      case 'unknown':
      default:
        return '正在连接后端服务器...';
    }
  };

  const getColorClass = () => {
    switch (status) {
      case 'starting':
      case 'unknown':
        return 'bg-yellow-500';
      case 'error':
        return 'bg-red-500';
      default:
        return 'bg-yellow-500';
    }
  };

  return (
    <div
      className={`fixed top-0 left-0 right-0 z-50 ${getColorClass()} text-white p-2 text-center text-sm`}
      role="status"
      aria-live="polite"
    >
      {getMessage()}
    </div>
  );
}
