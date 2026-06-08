import { invoke } from "@tauri-apps/api/core";

export type StoreStatus = {
  initialized: boolean;
  unlocked: boolean;
  schemaVersion: number | null;
};

export type NavKey = "worlds" | "characters" | "settings";

export type CharacterSummary = {
  character_id: string;
  name: string;
  subtitle?: string;
  description?: string;
  updated_at?: string;
};

export type WorldBlueprintSummary = {
  blueprint_id: string;
  title: string;
  description?: string;
  updated_at?: string;
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
  color_theme: string;
  content_toggles: {
    adult_content: boolean;
  };
};

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
