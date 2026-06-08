import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { App } from "./App";

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

function tokenTotals(requests: number) {
  return {
    requests,
    inputTokens: requests * 10,
    cachedInputTokens: requests,
    outputTokens: requests * 4,
  };
}

beforeEach(() => {
  window.__TAURI_INTERNALS__ = {};
  invokeMock.mockImplementation(async (name: string) => {
    switch (name) {
      case "store_status":
        return { initialized: true, unlocked: true, schemaVersion: 2 };
      case "list_in_progress_adventures":
        return [
          {
            adventure_id: "adv_1",
            world_title: "Crystal Vale",
            world_description: "A valley under glass stars.",
            story_status: "in_progress",
            ready_status: "ready",
            updated_at: "2026-06-08T00:00:00Z",
          },
        ];
      case "list_world_blueprints":
        return {
          items: [
            {
              blueprint_id: "world_1",
              title: "Lantern City",
              description: "A city built around a living lighthouse.",
              updated_at: "2026-06-08T00:00:00Z",
            },
          ],
          hasMore: false,
          nextCursor: null,
        };
      case "count_world_blueprints":
        return 1;
      case "list_characters":
        return {
          items: [
            {
              character_id: "char_1",
              name: "Mira Vale",
              subtitle: "Cartographer of impossible roads",
              updated_at: "2026-06-08T00:00:00Z",
            },
          ],
          hasMore: false,
          nextCursor: null,
        };
      case "count_characters":
        return 1;
      case "get_token_stats":
        return {
          metricCount: 1,
          totals: tokenTotals(3),
          byModel: [{ model: "gpt_5_1", totals: tokenTotals(2) }],
          byOperation: [{ label: "chat_reply", totals: tokenTotals(1) }],
          byDay: [],
          byMonth: [],
        };
      case "get_openai_credential_status":
        return { configured: true, masked: "sk-...1234" };
      case "get_app_settings":
        return {
          version: 1,
          color_theme: "teal",
          content_toggles: { adult_content: true },
        };
      case "get_player_profile":
        return {
          version: 1,
          player_name: "Steve",
          player_attributes: "Curious and stubborn.",
          prompt_extension: null,
        };
      default:
        throw new Error(`Unhandled command: ${name}`);
    }
  });
});

afterEach(() => {
  invokeMock.mockReset();
  delete window.__TAURI_INTERNALS__;
});

describe("App shell smoke", () => {
  it("TEST-6/UI-6 smoke: renders the unlocked data-backed shell and primary panels", async () => {
    render(<App />);

    expect(await screen.findByRole("heading", { name: "Worlds" })).toBeTruthy();
    expect(await screen.findByText("Lantern City")).toBeTruthy();
    expect(screen.getByText("Crystal Vale")).toBeTruthy();
    expect(screen.getByText("1 total")).toBeTruthy();

    fireEvent.click(screen.getByRole("button", { name: "Characters" }));
    expect(await screen.findByRole("heading", { name: "Characters" })).toBeTruthy();
    expect(await screen.findByText("Mira Vale")).toBeTruthy();
    expect(screen.getByText("1 total")).toBeTruthy();

    fireEvent.click(screen.getByRole("button", { name: "Stats" }));
    expect(await screen.findByRole("heading", { name: "Stats" })).toBeTruthy();
    await waitFor(() => expect(invokeMock).toHaveBeenCalledWith("get_token_stats", undefined));
    expect(screen.getByText("Chat Reply")).toBeTruthy();

    fireEvent.click(screen.getByRole("button", { name: "Settings" }));
    expect(await screen.findByRole("heading", { name: "Settings" })).toBeTruthy();
    expect(await screen.findByText(/sk-\.\.\.1234/)).toBeTruthy();
    expect(screen.getByRole("button", { name: "Lock Store" })).toBeTruthy();
  });
});
