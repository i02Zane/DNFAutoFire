export type Page = "autofire" | "combo" | "auto-run" | "config-management" | "settings" | "about";

export type EditTarget = { type: "global" } | { type: "profile"; configId: string };
