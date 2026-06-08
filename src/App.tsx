import { invoke } from "@tauri-apps/api/core";
import {
  AlertCircle,
  BookOpen,
  CheckCircle2,
  Flame,
  KeyRound,
  Library,
  Lock,
  MessageCircle,
  PlayCircle,
  RefreshCcw,
  Settings,
  ShieldCheck,
  Sparkles,
  Trash2,
  UserRound,
} from "lucide-react";
import { FormEvent, useEffect, useMemo, useState } from "react";

type StoreStatus = {
  initialized: boolean;
  unlocked: boolean;
  schemaVersion: number | null;
};

type NavKey = "worlds" | "characters" | "settings";

type CharacterSummary = {
  character_id: string;
  name: string;
  subtitle?: string;
  description?: string;
  updated_at?: string;
};

type WorldBlueprintSummary = {
  blueprint_id: string;
  title: string;
  description?: string;
  updated_at?: string;
};

type AdventureSummary = {
  adventure_id: string;
  world_title?: string | null;
  world_description?: string | null;
  story_status: string;
  ready_status: string;
  updated_at?: string;
};

type ListPage<T> = {
  items: T[];
  hasMore: boolean;
  nextCursor: string | null;
};

type CredentialStatus = {
  configured: boolean;
  masked: string | null;
};

type AppSettings = {
  color_theme: string;
  content_toggles: {
    adult_content: boolean;
  };
};

const DEFAULT_STATUS: StoreStatus = {
  initialized: false,
  unlocked: false,
  schemaVersion: null,
};

const DEFAULT_DATA_DIR = "soulfire-data";

function canInvokeTauri(): boolean {
  return "__TAURI_INTERNALS__" in window;
}

async function command<T>(name: string, args?: Record<string, unknown>): Promise<T> {
  if (!canInvokeTauri()) {
    throw new Error("Tauri runtime unavailable");
  }
  return invoke<T>(name, args);
}

function formatDate(value?: string): string {
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

function labelFromSnake(value: string): string {
  return value
    .split("_")
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function AppLogo() {
  return (
    <div className="brand-mark" aria-hidden="true">
      <Flame size={20} strokeWidth={2.4} />
    </div>
  );
}

function NavButton({
  active,
  icon,
  label,
  onClick,
}: {
  active: boolean;
  icon: React.ReactNode;
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      className={active ? "nav-button active" : "nav-button"}
      type="button"
      onClick={onClick}
      title={label}
      aria-label={label}
    >
      {icon}
      <span>{label}</span>
    </button>
  );
}

function StatusPill({ status }: { status: StoreStatus }) {
  return (
    <div className={status.unlocked ? "status-pill ready" : "status-pill locked"}>
      {status.unlocked ? <CheckCircle2 size={16} /> : <Lock size={16} />}
      <span>{status.unlocked ? "Unlocked" : "Locked"}</span>
    </div>
  );
}

function InlineNotice({
  icon,
  title,
  detail,
}: {
  icon: React.ReactNode;
  title: string;
  detail: string;
}) {
  return (
    <div className="inline-notice">
      {icon}
      <div>
        <h3>{title}</h3>
        <p>{detail}</p>
      </div>
    </div>
  );
}

function ToolbarButton({
  icon,
  label,
  onClick,
  disabled,
}: {
  icon: React.ReactNode;
  label: string;
  onClick: () => void;
  disabled?: boolean;
}) {
  return (
    <button className="icon-button" type="button" onClick={onClick} disabled={disabled} title={label}>
      {icon}
      <span>{label}</span>
    </button>
  );
}

function UnlockSurface({
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

function WorldsPanel() {
  const [adventures, setAdventures] = useState<AdventureSummary[]>([]);
  const [blueprints, setBlueprints] = useState<WorldBlueprintSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  async function refresh() {
    setLoading(true);
    setError(null);
    try {
      const [active, worlds] = await Promise.all([
        command<AdventureSummary[]>("list_in_progress_adventures", { limit: 6 }),
        command<ListPage<WorldBlueprintSummary>>("list_world_blueprints", {
          search: null,
          afterBlueprintId: null,
          limit: 6,
        }),
      ]);
      setAdventures(active);
      setBlueprints(worlds.items);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
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
      <div className="split-grid">
        <section className="list-panel">
          <div className="list-title">
            <PlayCircle size={18} />
            <h3>Active Adventures</h3>
          </div>
          {loading ? <p className="muted">Loading adventures...</p> : null}
          {!loading && adventures.length === 0 ? <p className="muted">No active adventures yet.</p> : null}
          <div className="item-list">
            {adventures.map((adventure) => (
              <article className="data-row" key={adventure.adventure_id}>
                <div>
                  <h4>{adventure.world_title ?? "Untitled adventure"}</h4>
                  <p>{adventure.world_description || labelFromSnake(adventure.story_status)}</p>
                </div>
                <span>{labelFromSnake(adventure.ready_status)}</span>
              </article>
            ))}
          </div>
        </section>
        <section className="list-panel">
          <div className="list-title">
            <BookOpen size={18} />
            <h3>World Blueprints</h3>
          </div>
          {loading ? <p className="muted">Loading worlds...</p> : null}
          {!loading && blueprints.length === 0 ? <p className="muted">Your worlds will appear here.</p> : null}
          <div className="item-list">
            {blueprints.map((world) => (
              <article className="data-row" key={world.blueprint_id}>
                <div>
                  <h4>{world.title}</h4>
                  <p>{world.description || "No description"}</p>
                </div>
                <span>{formatDate(world.updated_at)}</span>
              </article>
            ))}
          </div>
        </section>
      </div>
    </section>
  );
}

function CharactersPanel() {
  const [characters, setCharacters] = useState<CharacterSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  async function refresh() {
    setLoading(true);
    setError(null);
    try {
      const page = await command<ListPage<CharacterSummary>>("list_characters", {
        search: null,
        afterCharacterId: null,
        limit: 12,
      });
      setCharacters(page.items);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
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
            <article className="character-card" key={character.character_id}>
              <h4>{character.name}</h4>
              <p>{character.subtitle || character.description || "No subtitle"}</p>
              <span>{formatDate(character.updated_at)}</span>
            </article>
          ))}
        </div>
      </div>
    </section>
  );
}

function SettingsPanel({ status }: { status: StoreStatus }) {
  const [credential, setCredential] = useState<CredentialStatus | null>(null);
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [apiKey, setApiKey] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function refresh() {
    setBusy(true);
    setError(null);
    try {
      const [nextCredential, nextSettings] = await Promise.all([
        command<CredentialStatus>("get_openai_credential_status"),
        command<AppSettings>("get_app_settings"),
      ]);
      setCredential(nextCredential);
      setSettings(nextSettings);
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

export function App() {
  const [status, setStatus] = useState<StoreStatus>(DEFAULT_STATUS);
  const [nav, setNav] = useState<NavKey>("worlds");

  useEffect(() => {
    command<StoreStatus>("store_status", { dataDir: null })
      .then(setStatus)
      .catch(() => setStatus(DEFAULT_STATUS));
  }, []);

  const panel = useMemo(() => {
    if (nav === "characters") return <CharactersPanel />;
    if (nav === "settings") return <SettingsPanel status={status} />;
    return <WorldsPanel />;
  }, [nav, status]);

  if (!status.unlocked) {
    return <UnlockSurface status={status} onStatus={setStatus} />;
  }

  return (
    <div className="app-shell">
      <aside className="side-nav" aria-label="Primary navigation">
        <div className="side-brand">
          <AppLogo />
          <span>Soulfire</span>
        </div>
        <nav>
          <NavButton
            active={nav === "worlds"}
            icon={<Library size={18} />}
            label="Worlds"
            onClick={() => setNav("worlds")}
          />
          <NavButton
            active={nav === "characters"}
            icon={<UserRound size={18} />}
            label="Characters"
            onClick={() => setNav("characters")}
          />
          <NavButton
            active={nav === "settings"}
            icon={<Settings size={18} />}
            label="Settings"
            onClick={() => setNav("settings")}
          />
        </nav>
      </aside>
      <div className="main-column">
        <header className="titlebar">
          <AppLogo />
          <strong>Soulfire</strong>
          <StatusPill status={status} />
        </header>
        <main className="content">{panel}</main>
      </div>
    </div>
  );
}
