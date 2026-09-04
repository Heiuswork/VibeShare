export type AppLocale = "zh" | "en";

export const appLocale = $state({
  current: "zh" as AppLocale,
});

export function setAppLocale(next: AppLocale) {
  appLocale.current = next === "en" ? "en" : "zh";
}
