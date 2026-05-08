export type Page =
  | "autofire"
  | "combo"
  | "auto-run"
  | "runtime-diagnostics"
  | "config-management"
  | "settings"
  | "about";

export type EditTarget = { type: "global" } | { type: "profile"; configId: string };
