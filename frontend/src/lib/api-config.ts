/**
 * API 配置管理
 *
 * 处理不同环境的 API 基础 URL：
 * - 浏览器环境: 使用相对路径（由 Vite 代理处理）
 * - Tauri 开发模式: 连接到 Vite 开发服务器
 * - Tauri 生产模式: 连接到 Sidecar 动态端口
 */

// 全局端口变量类型声明
declare global {
  interface Window {
    __SIDECAR_PORT__?: number;
  }
}

/**
 * 检测是否在 Tauri 环境中运行
 */
export const isTauri = (): boolean => {
  return '__TAURI__' in window;
};

/**
 * 检测是否在开发环境中运行
 */
export const isDev = (): boolean => {
  return import.meta.env.DEV;
};

/**
 * 获取 API 基础 URL
 *
 * 根据当前运行环境返回正确的 API 基础 URL
 */
export const getApiBaseUrl = (): string => {
  if (!isTauri()) {
    // 浏览器环境: 使用相对路径（由 Vite 代理处理）
    return '';
  }

  if (isDev()) {
    // Tauri 开发模式: 连接到 Vite 开发服务器
    return 'http://localhost:3000';
  }

  // Tauri 生产模式: 从全局变量获取 Sidecar 动态端口
  if (window.__SIDECAR_PORT__) {
    return `http://127.0.0.1:${window.__SIDECAR_PORT__}`;
  }

  // 默认: 返回空字符串，使用相对路径
  return '';
};

/**
 * 构建完整的 API URL
 *
 * @param path - API 路径（如 '/api/projects'）
 * @returns 完整的 API URL
 */
export const getApiUrl = (path: string): string => {
  const baseUrl = getApiBaseUrl();
  return `${baseUrl}${path}`;
};

/**
 * 设置 Sidecar 端口
 *
 * @param port - Sidecar 服务器端口号
 */
export const setSidecarPort = (port: number): void => {
  window.__SIDECAR_PORT__ = port;
  console.log(`[API Config] Sidecar port set to: ${port}`);
};
