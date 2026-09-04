import { invoke } from "@tauri-apps/api/core";

export type NetworkInfo = {
  ip: string;
  interface: string;
  label: string;
  interfaces: { name: string; ip: string; label: string }[];
  reachable: boolean;
};

export type SiteInfo = {
  name: string;
  root: string;
  entry: string;
  html_entries: string[];
};

export type ConnectedDevice = {
  ip: string;
  name: string;
  /** phone / tablet / desktop / unknown —— 用于选图标 */
  kind: string;
  last_seen: number;
};

/** static = 直出磁盘文件；proxy = 转发到本机 dev server */
export type ShareMode = "static" | "proxy";

export type ShareStatus = {
  active: boolean;
  url: string | null;
  preview_url: string | null;
  access_code: string | null;
  expires_at: number | null;
  connections: number;
  devices?: ConnectedDevice[];
  entry: string | null;
  root: string | null;
  port: number | null;
  mode: ShareMode;
  proxy_port: number | null;
  network: NetworkInfo;
};

export type DevServer = {
  port: number;
  title: string;
};

export type AssetReport = {
  scanned: number;
  missing: string[];
  hardcoded_local: string[];
  truncated: boolean;
};

export type StartShareInput = {
  expiryMinutes?: number;
  accessProtection?: boolean;
  networkIp?: string;
};

export async function pickFolder() {
  return invoke<SiteInfo>("pick_folder");
}

export async function pickHtmlFile() {
  return invoke<SiteInfo>("pick_html_file");
}

export async function openSite(path: string, entry?: string) {
  return invoke<SiteInfo>("open_site", { path, entry: entry ?? null });
}

export async function setEntry(entry: string) {
  return invoke<SiteInfo>("set_site_entry", { entry });
}

export async function shareStatus() {
  return invoke<ShareStatus>("share_status");
}

export async function shareStart(input: StartShareInput = {}) {
  return invoke<ShareStatus>("share_start", { input });
}

export async function shareStop() {
  return invoke<ShareStatus>("share_stop");
}

export async function regenerateCode() {
  return invoke<ShareStatus>("share_regenerate_code");
}

/** 探测本机在跑的开发服务器（Vite / Next / …）。 */
export async function devServers() {
  return invoke<DevServer[]>("dev_servers");
}

/** 切换到代理模式，把请求转发给本机 dev server。 */
export async function useDevServer(port: number) {
  return invoke<ShareStatus>("use_dev_server_port", { port });
}

/** 回到静态直出模式。 */
export async function useStaticFiles() {
  return invoke<ShareStatus>("use_static_files");
}

/** 选择共享用的网卡。传 null 表示交回自动选择。 */
export async function selectNetwork(ip: string | null) {
  return invoke<ShareStatus>("select_network", { ip });
}

/** 扫描入口页面的资源引用。 */
export async function assetReport() {
  return invoke<AssetReport>("asset_report");
}

export async function qrPng(text: string) {
  return invoke<string>("qr_png", { text });
}

/** 仅收回该设备访问当前分享的权限。 */
export async function blockDevice(ip: string) {
  return invoke<ShareStatus>("share_block_device", { ip });
}

/** 加入黑名单后，该设备不能访问之后发起的任何分享。 */
export async function blacklistDevice(ip: string) {
  return invoke<ShareStatus>("share_blacklist_device", { ip });
}

export async function blacklistedDevices() {
  return invoke<string[]>("share_blacklisted_devices");
}

export async function removeBlacklistedDevice(ip: string) {
  return invoke<string[]>("share_remove_blacklisted_device", { ip });
}
