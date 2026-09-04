export type ThemePreference = "light" | "dark" | "system";
export type ResolvedTheme = "light" | "dark";

export const appTheme = $state({
  preference: "system" as ThemePreference,
  resolved: "light" as ResolvedTheme,
});

export function resolveTheme(preference: ThemePreference): ResolvedTheme {
  if (preference === "light" || preference === "dark") return preference;
  if (typeof window === "undefined") return "light";
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function applyResolved() {
  if (typeof document === "undefined") return;
  const resolved = resolveTheme(appTheme.preference);
  appTheme.resolved = resolved;
  const root = document.documentElement;
  root.dataset.theme = resolved;
  root.style.colorScheme = resolved;
  void import("@tauri-apps/api/window")
    .then(({ getCurrentWindow }) =>
      getCurrentWindow().setTheme(appTheme.preference === "system" ? null : resolved),
    )
    .catch(() => {});
}

export function setAppTheme(next: ThemePreference) {
  appTheme.preference = next === "light" || next === "dark" ? next : "system";
  applyResolved();
}

export function bindTheme() {
  applyResolved();
  const mq = window.matchMedia("(prefers-color-scheme: dark)");
  const onChange = () => {
    if (appTheme.preference === "system") applyResolved();
  };
  mq.addEventListener("change", onChange);
  return () => mq.removeEventListener("change", onChange);
}
