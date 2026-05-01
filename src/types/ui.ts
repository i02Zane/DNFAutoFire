export type Page = "autofire" | "combo" | "config-management" | "settings" | "about";

export type EditTarget = { type: "global" } | { type: "profile"; configId: string };
