import { appLocale } from "./ui/i18n.svelte";
import { t } from "./ui/i18n";
import type { ThemePreference } from "./ui/theme.svelte";

export const RECENT_STORAGE_KEY = "vibeshare.recent.v2";
export const SETTINGS_STORAGE_KEY = "vibeshare.settings.v1";
export const MAX_RECENT = 100;
export const RECENT_PAGE_SIZE = 20;

export const EXPIRY_PRESETS = [
  { value: "30 分钟后", hours: 0, minutes: 30 },
  { value: "1 小时后", hours: 1, minutes: 0 },
  { value: "2 小时后", hours: 2, minutes: 0 },
  { value: "3 小时后", hours: 3, minutes: 0 },
  { value: "4 小时后", hours: 4, minutes: 0 },
  { value: "custom", hours: 0, minutes: 0 },
] as const;

export type RecentItem = {
  name: string;
  folder: string;
  entry: string;
  assets: string;
  updated: string;
  htmlEntries?: string[];
};

export type AppSettings = {
  accessProtection: boolean;
  autoStop: boolean;
  defaultExpiry: string;
  defaultCustomHours: number;
  defaultCustomMinutes: number;
  locale: "zh" | "en";
  theme: ThemePreference;
};

export const DEFAULT_SETTINGS: AppSettings = {
  accessProtection: true,
  autoStop: true,
  defaultExpiry: "30 分钟后",
  defaultCustomHours: 0,
  defaultCustomMinutes: 30,
  locale: "zh",
  theme: "system",
};

export function serializeRecentItem(item: Partial<RecentItem> | null): RecentItem | null {
  if (!item) return null;
  return {
    name: item.name || "未命名网站",
    folder: item.folder || "",
    entry: item.entry || "index.html",
    assets: item.assets || "",
    updated: item.updated || "",
    htmlEntries: Array.isArray(item.htmlEntries) ? item.htmlEntries.slice(0, 20) : undefined,
  };
}

export function loadRecent(): RecentItem[] {
  try {
    const raw = localStorage.getItem(RECENT_STORAGE_KEY) || localStorage.getItem("vibeshare.recent.v1");
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.map((item: Partial<RecentItem>) => serializeRecentItem(item)).filter(Boolean).slice(0, MAX_RECENT) as RecentItem[];
  } catch {
    return [];
  }
}

export function saveRecent(list: RecentItem[]) {
  try {
    const payload = list.map((item) => serializeRecentItem(item)).filter(Boolean).slice(0, MAX_RECENT);
    localStorage.setItem(RECENT_STORAGE_KEY, JSON.stringify(payload));
  } catch {
    // ignore
  }
}

export function normalizeExpiryLabel(value: string, customHours = 0, customMinutes = 30) {
  const text = String(value || "").trim();
  if (EXPIRY_PRESETS.some((item) => item.value === text && item.value !== "custom")) return text;
  if (text === "应用退出时") return "应用退出时";
  const custom = text.match(/^(\d+)\s*小时\s*(\d+)\s*分钟后$/);
  if (custom) return `${Number(custom[1]) || 0} 小时 ${Number(custom[2]) || 0} 分钟后`;
  if (text === "custom") {
    const hours = Math.max(0, Math.min(48, Number(customHours) || 0));
    const minutes = Math.max(0, Math.min(59, Number(customMinutes) || 0));
    if (hours === 0 && minutes === 0) return "30 分钟后";
    if (hours === 0) return `${minutes} 分钟后`;
    if (minutes === 0) return `${hours} 小时后`;
    return `${hours} 小时 ${minutes} 分钟后`;
  }
  return DEFAULT_SETTINGS.defaultExpiry;
}

export function parseCustomExpiry(label: string) {
  const text = String(label || "");
  const both = text.match(/^(\d+)\s*小时\s*(\d+)\s*分钟后$/);
  if (both) return { hours: Number(both[1]) || 0, minutes: Number(both[2]) || 0 };
  const hoursOnly = text.match(/^(\d+)\s*小时后$/);
  if (hoursOnly) return { hours: Number(hoursOnly[1]) || 0, minutes: 0 };
  const minutesOnly = text.match(/^(\d+)\s*分钟后$/);
  if (minutesOnly) return { hours: 0, minutes: Number(minutesOnly[1]) || 0 };
  return { hours: 0, minutes: 30 };
}

export function isCustomExpiry(label: string) {
  return (
    !EXPIRY_PRESETS.some((item) => item.value === label && item.value !== "custom") &&
    label !== "应用退出时" &&
    !["30 分钟后", "1 小时后", "2 小时后", "3 小时后", "4 小时后"].includes(label)
  );
}

function formatDuration(hours: number, minutes: number) {
  void appLocale.current;
  if (hours === 0) return t("nMinutes", { n: minutes || 30 });
  if (minutes === 0) return t("nHours", { n: hours });
  return t("nHoursMinutes", { n: hours, m: minutes });
}

export function expiryPresetLabel(value: string) {
  void appLocale.current;
  if (value === "custom") return t("custom");
  return expiryDisplayLabel(value);
}

export function expiryDisplayLabel(label: string) {
  void appLocale.current;
  if (label === "应用退出时") return t("onQuit");
  const parsed = parseCustomExpiry(label);
  return formatDuration(parsed.hours, parsed.minutes);
}

export function expiryMinutes(label: string, autoStop: boolean) {
  if (!autoStop || label === "应用退出时") return 0;
  const parsed = parseCustomExpiry(label);
  return parsed.hours * 60 + parsed.minutes;
}

export function formatExpiry(expiresAt: number | null | undefined) {
  void appLocale.current;
  if (!expiresAt) return t("onQuit");
  const remaining = Math.max(0, expiresAt - Date.now());
  const totalMinutes = Math.max(1, Math.ceil(remaining / (60 * 1000)));
  if (totalMinutes < 60) return t("nMinutes", { n: totalMinutes });
  const hours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;
  if (!minutes) return t("nHours", { n: hours });
  return t("nHoursMinutes", { n: hours, m: minutes });
}

export function formatRemaining(expiresAt: number | null | undefined) {
  void appLocale.current;
  if (!expiresAt) return t("onQuit");
  return t("remainingTime", { time: formatExpiry(expiresAt) });
}

export function loadSettings(): AppSettings {
  try {
    const raw = localStorage.getItem(SETTINGS_STORAGE_KEY);
    if (!raw) return { ...DEFAULT_SETTINGS };
    const parsed = JSON.parse(raw);
    const defaultCustomHours = Number(parsed?.defaultCustomHours ?? parsed?.customHours) || 0;
    const defaultCustomMinutes = Number.isFinite(Number(parsed?.defaultCustomMinutes ?? parsed?.customMinutes))
      ? Number(parsed?.defaultCustomMinutes ?? parsed?.customMinutes)
      : 30;
    const defaultExpiry = normalizeExpiryLabel(
      parsed?.defaultExpiry || parsed?.expiry,
      defaultCustomHours,
      defaultCustomMinutes,
    );
    return {
      accessProtection: parsed?.accessProtection !== false,
      autoStop: parsed?.autoStop !== false,
      defaultExpiry,
      defaultCustomHours,
      defaultCustomMinutes,
      locale: parsed?.locale === "en" ? "en" : "zh",
      theme: parsed?.theme === "light" || parsed?.theme === "dark" ? parsed.theme : "system",
    };
  } catch {
    return { ...DEFAULT_SETTINGS };
  }
}

export function saveSettings(settings: AppSettings) {
  try {
    localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(settings));
  } catch {
    // ignore
  }
}

export function formatUpdated() {
  void appLocale.current;
  return new Date().toLocaleString(appLocale.current === "en" ? "en-US" : "zh-CN", {
    month: "numeric",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

/**
 * 把时间戳变成"刚刚 / 12 秒前"这类相对说法。
 *
 * 在线设备列表需要它：只显示名字和 IP 的话，用户无法判断某台设备是还在看
 * 还是早就走了。
 */
export function formatSince(timestamp: number | null | undefined) {
  void appLocale.current;
  if (!timestamp) return "";
  const seconds = Math.floor((Date.now() - timestamp) / 1000);
  if (seconds < 5) return t("justNow");
  if (seconds < 60) return t("secondsAgo", { n: seconds });
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return t("minutesAgo", { n: minutes });
  return t("hoursAgo", { n: Math.floor(minutes / 60) });
}
