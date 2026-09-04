export type View = "overview" | "visitors" | "recent" | "diagnostics" | "settings";

export type CheckItem = {
  id: string;
  label: string;
  detail: string;
  status: "ok" | "warn";
  action?: string;
};

export type ChecksSummary = {
  label: string;
  tone: "checking" | "ok" | "warn";
  okCount: number;
  warnCount: number;
  total: number;
};

export type ShareButtonState = {
  tone: "idle" | "ready" | "starting" | "live";
  label: string;
  detail: string;
};
