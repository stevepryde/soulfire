import { invoke } from "@tauri-apps/api/core";

export type StoreStatus = {
  initialized: boolean;
  unlocked: boolean;
  schemaVersion: number | null;
};

export type NavKey = "worlds" | "characters" | "stats" | "settings";

export type CharacterSummary = {
  character_id: string;
  name: string;
  subtitle?: string;
  description?: string;
  updated_at?: string;
};

export type CharacterDetail = CharacterSummary & {
  prompt?: string;
  extracted_context?: string | null;
  character_state?: string | null;
};

export type WorldBlueprintSummary = {
  blueprint_id: string;
  title: string;
  description?: string;
  updated_at?: string;
};

export type WorldBlueprintDetail = WorldBlueprintSummary & {
  world_prompt: string;
};

export type AdventureSummary = {
  adventure_id: string;
  world_title?: string | null;
  world_description?: string | null;
  story_status: string;
  ready_status: string;
  updated_at?: string;
};

export type ListPage<T> = {
  items: T[];
  hasMore: boolean;
  nextCursor: string | null;
};

export type CredentialStatus = {
  configured: boolean;
  masked: string | null;
};

export type AppSettings = {
  version?: number;
  color_theme: string;
  content_toggles: {
    adult_content: boolean;
  };
};

export type PlayerProfile = {
  version?: number;
  player_name: string;
  player_attributes: string;
  prompt_extension?: string | null;
};

export type TokenTotals = {
  requests: number;
  inputTokens: number;
  cachedInputTokens: number;
  outputTokens: number;
};

export type TokenStatsReport = {
  metricCount: number;
  totals: TokenTotals;
  byModel: Array<{ model: string; totals: TokenTotals }>;
  byOperation: Array<{ label: string; totals: TokenTotals }>;
  byDay: Array<{ period: string; totals: TokenTotals }>;
  byMonth: Array<{ period: string; totals: TokenTotals }>;
};

export const COLOR_THEMES = [
  { value: "purple", label: "Purple", color: "#8b5cf6" },
  { value: "blue", label: "Blue", color: "#3b82f6" },
  { value: "green", label: "Green", color: "#22c55e" },
  { value: "red", label: "Red", color: "#ef4444" },
  { value: "orange", label: "Orange", color: "#f97316" },
  { value: "teal", label: "Teal", color: "#14b8a6" },
  { value: "grey", label: "Grey", color: "#6b7280" },
] as const;

export const DEFAULT_STATUS: StoreStatus = {
  initialized: false,
  unlocked: false,
  schemaVersion: null,
};

export const DEFAULT_DATA_DIR = "soulfire-data";

export function canInvokeTauri(): boolean {
  return "__TAURI_INTERNALS__" in window;
}

export async function command<T>(name: string, args?: Record<string, unknown>): Promise<T> {
  if (!canInvokeTauri()) {
    throw new Error("Tauri runtime unavailable");
  }
  return invoke<T>(name, args);
}

export function formatDate(value?: string): string {
  if (!value) return "No date";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
  }).format(date);
}

export function labelFromSnake(value: string): string {
  return value
    .split("_")
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}
