import {
  AlertCircle,
  BarChart3,
  BookOpen,
  KeyRound,
  Library,
  MessageCircle,
  PlayCircle,
  RefreshCcw,
  Settings,
  ShieldCheck,
  Sparkles,
  Trash2,
  UserRound,
} from "lucide-react";
import { FormEvent, useEffect, useState } from "react";

import {
  AdventureDetail,
  AdventureSummary,
  AppSettings,
  COLOR_THEMES,
  CharacterDetail,
  CharacterSummary,
  CredentialStatus,
  DEFAULT_DATA_DIR,
  ListPage,
  PlayerProfile,
  PromptView,
  StoreStatus,
  TokenStatsReport,
  TokenTotals,
  WorldBlueprintSummary,
  WorldBlueprintDetail,
  canInvokeTauri,
  command,
  formatDate,
  labelFromSnake,
} from "./bridge";
import { AppLogo, ConfirmDialog, InlineNotice, ToolbarButton } from "./chrome";

export function UnlockSurface({
  status,
  onStatus,
}: {
  status: StoreStatus;
  onStatus: (status: StoreStatus) => void;
}) {
  const [dataDir, setDataDir] = useState(DEFAULT_DATA_DIR);
  const [masterPassword, setMasterPassword] = useState("");
  const [mode, setMode] = useState<"unlock" | "setup">("unlock");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!status.initialized) {
      setMode("setup");
    }
  }, [status.initialized]);

  async function refreshStatus(nextDataDir = dataDir) {
    const next = await command<StoreStatus>("store_status", {
      dataDir: nextDataDir || null,
    });
    onStatus(next);
  }

  async function submit(event: FormEvent) {
    event.preventDefault();
    setBusy(true);
    setError(null);
    try {
      const next = await command<StoreStatus>(
        mode === "setup" ? "setup_store" : "unlock_store",
        {
          dataDir,
          masterPassword,
        },
      );
      setMasterPassword("");
      onStatus(next);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  async function checkStore() {
    setBusy(true);
    setError(null);
    try {
      await refreshStatus();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <main className="auth-shell">
      <section className="auth-panel" aria-label="Store unlock">
        <div className="auth-brand">
          <AppLogo />
          <h1>Soulfire</h1>
        </div>
        <div className="mode-tabs" role="tablist" aria-label="Store mode">
          <button
            className={mode === "unlock" ? "segmented active" : "segmented"}
            type="button"
            onClick={() => setMode("unlock")}
            disabled={!status.initialized}
          >
            <KeyRound size={17} />
            Unlock
          </button>
          <button
            className={mode === "setup" ? "segmented active" : "segmented"}
            type="button"
            onClick={() => setMode("setup")}
          >
            <Sparkles size={17} />
            Setup
          </button>
        </div>
        <form className="auth-form" onSubmit={submit}>
          <label>
            <span>Data folder</span>
            <input
              value={dataDir}
              onChange={(event) => setDataDir(event.target.value)}
              spellCheck={false}
              autoCapitalize="none"
            />
          </label>
          <label>
            <span>Master password</span>
            <input
              value={masterPassword}
              onChange={(event) => setMasterPassword(event.target.value)}
              type="password"
            />
          </label>
          {error ? <p className="error-line">{error}</p> : null}
          <div className="auth-actions">
            <button className="secondary-button" type="button" onClick={checkStore} disabled={busy}>
              Check
            </button>
            <button className="primary-button" type="submit" disabled={busy}>
              {mode === "setup" ? "Create" : "Unlock"}
            </button>
          </div>
        </form>
      </section>
    </main>
  );
}

export function WorldsPanel() {
  const [adventures, setAdventures] = useState<AdventureSummary[]>([]);
  const [selectedAdventure, setSelectedAdventure] = useState<AdventureDetail | null>(null);
  const [adventurePromptView, setAdventurePromptView] = useState<PromptView | null>(null);
  const [blueprints, setBlueprints] = useState<WorldBlueprintSummary[]>([]);
  const [selectedWorld, setSelectedWorld] = useState<WorldBlueprintDetail | null>(null);
  const [blueprintCursor, setBlueprintCursor] = useState<string | null>(null);
  const [blueprintsHaveMore, setBlueprintsHaveMore] = useState(false);
  const [search, setSearch] = useState("");
  const [loading, setLoading] = useState(true);
  const [detailLoading, setDetailLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function refresh(nextSearch = search) {
    setLoading(true);
    setError(null);
    try {
      const [active, worlds] = await Promise.all([
        command<AdventureSummary[]>("list_in_progress_adventures", { limit: 6 }),
        command<ListPage<WorldBlueprintSummary>>("list_world_blueprints", {
          search: nextSearch || null,
          afterBlueprintId: null,
          limit: 6,
        }),
      ]);
      setAdventures(active);
      setBlueprints(worlds.items);
      setBlueprintCursor(worlds.nextCursor);
      setBlueprintsHaveMore(worlds.hasMore);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }

  async function loadMoreBlueprints() {
    if (!blueprintCursor) return;
    setLoading(true);
    setError(null);
    try {
      const page = await command<ListPage<WorldBlueprintSummary>>("list_world_blueprints", {
        search: search || null,
        afterBlueprintId: blueprintCursor,
        limit: 6,
      });
      setBlueprints((current) => [...current, ...page.items]);
      setBlueprintCursor(page.nextCursor);
      setBlueprintsHaveMore(page.hasMore);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }

  async function loadWorldDetail(blueprintId: string) {
    setDetailLoading(true);
    setError(null);
    try {
      setSelectedWorld(
        await command<WorldBlueprintDetail>("load_world_blueprint", { blueprintId }),
      );
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setDetailLoading(false);
    }
  }

  async function loadAdventureDetail(adventureId: string) {
    setDetailLoading(true);
    setError(null);
    try {
      setSelectedAdventure(await command<AdventureDetail>("load_adventure", { adventureId }));
      setAdventurePromptView(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setDetailLoading(false);
    }
  }

  async function loadAdventurePromptView(adventureId: string) {
    setDetailLoading(true);
    setError(null);
    try {
      setAdventurePromptView(
        await command<PromptView>("get_adventure_prompt_view", {
          adventureId,
          draftAction: null,
        }),
      );
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setDetailLoading(false);
    }
  }

  useEffect(() => {
    void refresh();
  }, []);

  return (
    <section className="workspace-band">
      <div className="panel-heading with-action">
        <div>
          <Library size={20} />
          <h2>Worlds</h2>
        </div>
        <ToolbarButton
          icon={<RefreshCcw size={16} />}
          label="Refresh"
          onClick={refresh}
          disabled={loading}
        />
      </div>
      {error ? (
        <InlineNotice icon={<AlertCircle size={20} />} title="Worlds unavailable" detail={error} />
      ) : null}
      <form
        className="search-row"
        onSubmit={(event) => {
          event.preventDefault();
          void refresh();
        }}
      >
        <input
          value={search}
          onChange={(event) => setSearch(event.target.value)}
          placeholder="Search worlds"
          spellCheck={false}
        />
        <button className="secondary-button" type="submit" disabled={loading}>
          Search
        </button>
      </form>
      <div className="split-grid">
        <section className="list-panel">
          <div className="list-title">
            <PlayCircle size={18} />
            <h3>Active Adventures</h3>
          </div>
          {loading ? <p className="muted">Loading adventures...</p> : null}
          {!loading && adventures.length === 0 ? (
            <p className="muted">No active adventures yet.</p>
          ) : null}
          <div className="item-list">
            {adventures.map((adventure) => (
              <button
                className="data-row data-row-button"
                key={adventure.adventure_id}
                type="button"
                onClick={() => loadAdventureDetail(adventure.adventure_id)}
              >
                <div>
                  <h4>{adventure.world_title ?? "Untitled adventure"}</h4>
                  <p>{adventure.world_description || labelFromSnake(adventure.story_status)}</p>
                </div>
                <span>{labelFromSnake(adventure.ready_status)}</span>
              </button>
            ))}
          </div>
        </section>
        <section className="list-panel">
          <div className="list-title">
            <BookOpen size={18} />
            <h3>World Blueprints</h3>
          </div>
          {loading ? <p className="muted">Loading worlds...</p> : null}
          {!loading && blueprints.length === 0 ? (
            <p className="muted">Your worlds will appear here.</p>
          ) : null}
          <div className="item-list">
            {blueprints.map((world) => (
              <button
                className="data-row data-row-button"
                key={world.blueprint_id}
                type="button"
                onClick={() => loadWorldDetail(world.blueprint_id)}
              >
                <div>
                  <h4>{world.title}</h4>
                  <p>{world.description || "No description"}</p>
                </div>
                <span>{formatDate(world.updated_at)}</span>
              </button>
            ))}
          </div>
          {blueprintsHaveMore ? (
            <button className="secondary-button list-footer-button" type="button" onClick={loadMoreBlueprints} disabled={loading}>
              Load More
            </button>
          ) : null}
        </section>
      </div>
      {selectedWorld ? (
        <section className="detail-panel">
          <div>
            <h3>{selectedWorld.title}</h3>
            <p>{selectedWorld.description || "No description"}</p>
          </div>
          <pre>{selectedWorld.world_prompt}</pre>
        </section>
      ) : detailLoading ? (
        <p className="muted">Loading world...</p>
      ) : null}
      {selectedAdventure ? (
        <section className="detail-panel">
          <div>
            <h3>{selectedAdventure.adventure.world_title ?? "Untitled adventure"}</h3>
            <p>
              {labelFromSnake(selectedAdventure.adventure.story_status)} ·{" "}
              {selectedAdventure.messages.length} messages ·{" "}
              {selectedAdventure.pendingProposals.length} pending proposals
            </p>
          </div>
          <pre>{selectedAdventure.adventure.adventure_state || "No adventure state"}</pre>
          <button
            className="secondary-button list-footer-button"
            type="button"
            onClick={() => loadAdventurePromptView(selectedAdventure.adventure.adventure_id)}
            disabled={detailLoading}
          >
            Prompt View
          </button>
          {adventurePromptView ? <PromptViewPanel promptView={adventurePromptView} /> : null}
        </section>
      ) : null}
    </section>
  );
}

export function CharactersPanel() {
  const [characters, setCharacters] = useState<CharacterSummary[]>([]);
  const [selectedCharacter, setSelectedCharacter] = useState<CharacterDetail | null>(null);
  const [promptView, setPromptView] = useState<PromptView | null>(null);
  const [characterCursor, setCharacterCursor] = useState<string | null>(null);
  const [charactersHaveMore, setCharactersHaveMore] = useState(false);
  const [search, setSearch] = useState("");
  const [loading, setLoading] = useState(true);
  const [detailLoading, setDetailLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function refresh(nextSearch = search) {
    setLoading(true);
    setError(null);
    try {
      const page = await command<ListPage<CharacterSummary>>("list_characters", {
        search: nextSearch || null,
        afterCharacterId: null,
        limit: 12,
      });
      setCharacters(page.items);
      setCharacterCursor(page.nextCursor);
      setCharactersHaveMore(page.hasMore);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }

  async function loadMoreCharacters() {
    if (!characterCursor) return;
    setLoading(true);
    setError(null);
    try {
      const page = await command<ListPage<CharacterSummary>>("list_characters", {
        search: search || null,
        afterCharacterId: characterCursor,
        limit: 12,
      });
      setCharacters((current) => [...current, ...page.items]);
      setCharacterCursor(page.nextCursor);
      setCharactersHaveMore(page.hasMore);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }

  async function loadCharacterDetail(characterId: string) {
    setDetailLoading(true);
    setError(null);
    try {
      setSelectedCharacter(await command<CharacterDetail>("load_character", { characterId }));
      setPromptView(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setDetailLoading(false);
    }
  }

  async function loadCharacterPromptView(characterId: string) {
    setDetailLoading(true);
    setError(null);
    try {
      setPromptView(await command<PromptView>("get_character_prompt_view", { characterId }));
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setDetailLoading(false);
    }
  }

  useEffect(() => {
    void refresh();
  }, []);

  return (
    <section className="workspace-band">
      <div className="panel-heading with-action">
        <div>
          <UserRound size={20} />
          <h2>Characters</h2>
        </div>
        <ToolbarButton
          icon={<RefreshCcw size={16} />}
          label="Refresh"
          onClick={refresh}
          disabled={loading}
        />
      </div>
      {error ? (
        <InlineNotice icon={<AlertCircle size={20} />} title="Characters unavailable" detail={error} />
      ) : null}
      <form
        className="search-row"
        onSubmit={(event) => {
          event.preventDefault();
          void refresh();
        }}
      >
        <input
          value={search}
          onChange={(event) => setSearch(event.target.value)}
          placeholder="Search characters"
          spellCheck={false}
        />
        <button className="secondary-button" type="submit" disabled={loading}>
          Search
        </button>
      </form>
      <div className="list-panel">
        <div className="list-title">
          <MessageCircle size={18} />
          <h3>Saved Characters</h3>
        </div>
        {loading ? <p className="muted">Loading characters...</p> : null}
        {!loading && characters.length === 0 ? (
          <InlineNotice
            icon={<Sparkles size={20} />}
            title="No characters yet"
            detail="Character drafts and refinements will appear here."
          />
        ) : null}
        <div className="character-grid">
          {characters.map((character) => (
            <button
              className="character-card character-card-button"
              key={character.character_id}
              type="button"
              onClick={() => loadCharacterDetail(character.character_id)}
            >
              <h4>{character.name}</h4>
              <p>{character.subtitle || character.description || "No subtitle"}</p>
              <span>{formatDate(character.updated_at)}</span>
            </button>
          ))}
        </div>
        {charactersHaveMore ? (
          <button className="secondary-button list-footer-button" type="button" onClick={loadMoreCharacters} disabled={loading}>
            Load More
          </button>
        ) : null}
      </div>
      {selectedCharacter ? (
        <section className="detail-panel">
          <div>
            <h3>{selectedCharacter.name}</h3>
            <p>{selectedCharacter.subtitle || selectedCharacter.description || "No subtitle"}</p>
          </div>
          <pre>{selectedCharacter.prompt || "No character prompt"}</pre>
          <button
            className="secondary-button list-footer-button"
            type="button"
            onClick={() => loadCharacterPromptView(selectedCharacter.character_id)}
            disabled={detailLoading}
          >
            Prompt View
          </button>
          {promptView ? <PromptViewPanel promptView={promptView} /> : null}
        </section>
      ) : detailLoading ? (
        <p className="muted">Loading character...</p>
      ) : null}
    </section>
  );
}

function formatNumber(value: number): string {
  return new Intl.NumberFormat().format(value);
}

function totalTokens(totals: TokenTotals): number {
  return totals.inputTokens + totals.outputTokens;
}

function PromptViewPanel({ promptView }: { promptView: PromptView }) {
  return (
    <div className="prompt-view">
      <div className="prompt-total">
        <strong>{formatNumber(promptView.tokenEstimate)}</strong>
        <span>estimated tokens</span>
      </div>
      {promptView.sections.map((section) => (
        <article className="prompt-section" key={`${section.source}-${section.header}`}>
          <header>
            <h4>{section.header}</h4>
            <span>{section.locked ? "Locked" : "Editable"}</span>
          </header>
          <p>{labelFromSnake(section.source)}</p>
          <pre>{section.body}</pre>
          <small>{formatNumber(section.tokenEstimate)} tokens</small>
        </article>
      ))}
    </div>
  );
}

function StatsTable({
  rows,
  labelKey,
}: {
  rows: Array<{ label?: string; model?: string; period?: string; totals: TokenTotals }>;
  labelKey: "label" | "model" | "period";
}) {
  if (rows.length === 0) {
    return <p className="muted">No breakdown yet.</p>;
  }
  return (
    <div className="stats-table">
      {rows.slice(0, 8).map((row) => (
        <div key={row[labelKey]}>
          <span>{labelFromSnake(String(row[labelKey]))}</span>
          <strong>{formatNumber(totalTokens(row.totals))}</strong>
        </div>
      ))}
    </div>
  );
}

export function StatsPanel() {
  const [report, setReport] = useState<TokenStatsReport | null>(null);
  const [loading, setLoading] = useState(true);
  const [clearing, setClearing] = useState(false);
  const [confirmClear, setConfirmClear] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function refresh() {
    setLoading(true);
    setError(null);
    try {
      setReport(await command<TokenStatsReport>("get_token_stats"));
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }

  async function clearStats() {
    setClearing(true);
    setError(null);
    try {
      setReport(await command<TokenStatsReport>("clear_token_stats"));
      setConfirmClear(false);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setClearing(false);
    }
  }

  useEffect(() => {
    void refresh();
  }, []);

  return (
    <section className="workspace-band">
      <div className="panel-heading with-action">
        <div>
          <BarChart3 size={20} />
          <h2>Stats</h2>
        </div>
        <ToolbarButton
          icon={<RefreshCcw size={16} />}
          label="Refresh"
          onClick={refresh}
          disabled={loading || clearing}
        />
      </div>
      {confirmClear ? (
        <ConfirmDialog
          title="Clear Token Stats"
          detail="This removes the local token-usage history stored in this encrypted Soulfire store."
          confirmLabel="Clear"
          busy={clearing}
          onCancel={() => setConfirmClear(false)}
          onConfirm={clearStats}
        />
      ) : null}
      {error ? (
        <InlineNotice icon={<AlertCircle size={20} />} title="Stats unavailable" detail={error} />
      ) : null}
      <div className="metric-grid">
        <article className="metric-card">
          <span>Requests</span>
          <strong>{formatNumber(report?.totals.requests ?? 0)}</strong>
        </article>
        <article className="metric-card">
          <span>Input Tokens</span>
          <strong>{formatNumber(report?.totals.inputTokens ?? 0)}</strong>
        </article>
        <article className="metric-card">
          <span>Cached Input</span>
          <strong>{formatNumber(report?.totals.cachedInputTokens ?? 0)}</strong>
        </article>
        <article className="metric-card">
          <span>Output Tokens</span>
          <strong>{formatNumber(report?.totals.outputTokens ?? 0)}</strong>
        </article>
      </div>
      <div className="split-grid">
        <section className="list-panel">
          <div className="list-title">
            <BarChart3 size={18} />
            <h3>By Operation</h3>
          </div>
          {loading ? <p className="muted">Loading stats...</p> : null}
          {report ? <StatsTable rows={report.byOperation} labelKey="label" /> : null}
        </section>
        <section className="list-panel">
          <div className="list-title">
            <BarChart3 size={18} />
            <h3>By Model</h3>
          </div>
          {loading ? <p className="muted">Loading stats...</p> : null}
          {report ? <StatsTable rows={report.byModel} labelKey="model" /> : null}
        </section>
      </div>
      <div className="danger-zone">
        <div>
          <h3>Token History</h3>
          <p>Clearing history only removes local usage metrics. It does not affect chats, adventures, or provider billing records.</p>
        </div>
        <button
          className="danger-text-button"
          type="button"
          onClick={() => setConfirmClear(true)}
          disabled={loading || clearing || (report?.metricCount ?? 0) === 0}
        >
          Clear History
        </button>
      </div>
    </section>
  );
}

export function SettingsPanel({ status }: { status: StoreStatus }) {
  const [credential, setCredential] = useState<CredentialStatus | null>(null);
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [profile, setProfile] = useState<PlayerProfile | null>(null);
  const [apiKey, setApiKey] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function refresh() {
    setBusy(true);
    setError(null);
    try {
      const [nextCredential, nextSettings, nextProfile] = await Promise.all([
        command<CredentialStatus>("get_openai_credential_status"),
        command<AppSettings>("get_app_settings"),
        command<PlayerProfile>("get_player_profile"),
      ]);
      setCredential(nextCredential);
      setSettings(nextSettings);
      setProfile(nextProfile);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  async function saveCredential(event: FormEvent) {
    event.preventDefault();
    if (!apiKey.trim()) return;
    setBusy(true);
    setError(null);
    try {
      const next = await command<CredentialStatus>("save_openai_credential", { apiKey });
      setCredential(next);
      setApiKey("");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  async function deleteCredential() {
    setBusy(true);
    setError(null);
    try {
      const next = await command<CredentialStatus>("delete_openai_credential");
      setCredential(next);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  async function saveSettings(nextSettings: AppSettings) {
    setBusy(true);
    setError(null);
    try {
      const next = await command<AppSettings>("save_app_settings", { settings: nextSettings });
      setSettings(next);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  function setColorTheme(colorTheme: string) {
    if (!settings || settings.color_theme === colorTheme) return;
    void saveSettings({ ...settings, color_theme: colorTheme });
  }

  function setAdultContent(adultContent: boolean) {
    if (!settings) return;
    void saveSettings({
      ...settings,
      content_toggles: {
        ...settings.content_toggles,
        adult_content: adultContent,
      },
    });
  }

  async function savePlayerProfile(event: FormEvent) {
    event.preventDefault();
    if (!profile) return;
    setBusy(true);
    setError(null);
    try {
      const next = await command<PlayerProfile>("save_player_profile", { profile });
      setProfile(next);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  useEffect(() => {
    void refresh();
  }, []);

  return (
    <section className="workspace-band">
      <div className="panel-heading with-action">
        <div>
          <Settings size={20} />
          <h2>Settings</h2>
        </div>
        <ToolbarButton
          icon={<RefreshCcw size={16} />}
          label="Refresh"
          onClick={refresh}
          disabled={busy}
        />
      </div>
      {error ? (
        <InlineNotice icon={<AlertCircle size={20} />} title="Settings unavailable" detail={error} />
      ) : null}
      <dl className="settings-list">
        <div>
          <dt>Store</dt>
          <dd>{status.unlocked ? "Unlocked" : "Locked"}</dd>
        </div>
        <div>
          <dt>Schema</dt>
          <dd>{status.schemaVersion ?? "Unknown"}</dd>
        </div>
        <div>
          <dt>Runtime</dt>
          <dd>{canInvokeTauri() ? "Tauri" : "Browser preview"}</dd>
        </div>
        <div>
          <dt>Accent</dt>
          <dd>{settings ? labelFromSnake(settings.color_theme) : "Loading"}</dd>
        </div>
        <div>
          <dt>Adult Content</dt>
          <dd>{settings?.content_toggles.adult_content ? "Enabled" : "Disabled"}</dd>
        </div>
      </dl>
      <section className="settings-card">
        <div className="list-title">
          <UserRound size={18} />
          <h3>Player Profile</h3>
        </div>
        <form className="profile-form" onSubmit={savePlayerProfile}>
          <label>
            <span>Name</span>
            <input
              value={profile?.player_name ?? ""}
              onChange={(event) =>
                setProfile((current) =>
                  current ? { ...current, player_name: event.target.value } : current,
                )
              }
              disabled={busy || !profile}
            />
          </label>
          <label>
            <span>Attributes</span>
            <textarea
              value={profile?.player_attributes ?? ""}
              onChange={(event) =>
                setProfile((current) =>
                  current ? { ...current, player_attributes: event.target.value } : current,
                )
              }
              disabled={busy || !profile}
              rows={4}
            />
          </label>
          <label>
            <span>Prompt Extension</span>
            <textarea
              value={profile?.prompt_extension ?? ""}
              onChange={(event) =>
                setProfile((current) =>
                  current ? { ...current, prompt_extension: event.target.value || null } : current,
                )
              }
              disabled={busy || !profile}
              rows={4}
            />
          </label>
          <button className="primary-button" type="submit" disabled={busy || !profile}>
            Save
          </button>
        </form>
      </section>
      <section className="settings-card">
        <div className="list-title">
          <Sparkles size={18} />
          <h3>Appearance</h3>
        </div>
        <div className="swatch-row" aria-label="Accent color">
          {COLOR_THEMES.map((theme) => (
            <button
              className={settings?.color_theme === theme.value ? "swatch-button active" : "swatch-button"}
              key={theme.value}
              type="button"
              onClick={() => setColorTheme(theme.value)}
              disabled={busy || !settings}
              title={theme.label}
              aria-label={theme.label}
            >
              <span style={{ background: theme.color }} />
            </button>
          ))}
        </div>
        <label className="toggle-row">
          <input
            checked={settings?.content_toggles.adult_content ?? false}
            onChange={(event) => setAdultContent(event.target.checked)}
            type="checkbox"
            disabled={busy || !settings}
          />
          <span>Adult Content</span>
        </label>
      </section>
      <section className="settings-card">
        <div className="list-title">
          <ShieldCheck size={18} />
          <h3>OpenAI Key</h3>
        </div>
        <p className="muted">
          {credential?.configured ? `Configured as ${credential.masked}` : "No key configured."}
        </p>
        <form className="key-form" onSubmit={saveCredential}>
          <input
            value={apiKey}
            onChange={(event) => setApiKey(event.target.value)}
            type="password"
            placeholder="sk-..."
            autoComplete="off"
            spellCheck={false}
          />
          <button className="primary-button" type="submit" disabled={busy || !apiKey.trim()}>
            Save
          </button>
          <button
            className="danger-button"
            type="button"
            onClick={deleteCredential}
            disabled={busy || !credential?.configured}
            title="Delete OpenAI key"
            aria-label="Delete OpenAI key"
          >
            <Trash2 size={16} />
          </button>
        </form>
      </section>
    </section>
  );
}
