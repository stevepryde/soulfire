import { invoke } from "@tauri-apps/api/core";
import {
  BookOpen,
  CheckCircle2,
  Flame,
  KeyRound,
  Library,
  Lock,
  MessageCircle,
  Settings,
  Sparkles,
  UserRound,
} from "lucide-react";
import { FormEvent, useEffect, useMemo, useState } from "react";

type StoreStatus = {
  initialized: boolean;
  unlocked: boolean;
  schemaVersion: number | null;
};

type NavKey = "worlds" | "characters" | "settings";

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
  return (
    <section className="workspace-band">
      <div className="panel-heading">
        <Library size={20} />
        <h2>Worlds</h2>
      </div>
      <div className="split-grid">
        <article className="feature-row">
          <BookOpen size={20} />
          <div>
            <h3>Adventures</h3>
            <p>No active adventures yet.</p>
          </div>
        </article>
        <article className="feature-row">
          <Sparkles size={20} />
          <div>
            <h3>Worlds</h3>
            <p>Your worlds will appear here.</p>
          </div>
        </article>
      </div>
    </section>
  );
}

function CharactersPanel() {
  return (
    <section className="workspace-band">
      <div className="panel-heading">
        <UserRound size={20} />
        <h2>Characters</h2>
      </div>
      <div className="split-grid">
        <article className="feature-row">
          <MessageCircle size={20} />
          <div>
            <h3>Character Chat</h3>
            <p>Saved characters will appear here.</p>
          </div>
        </article>
        <article className="feature-row">
          <Sparkles size={20} />
          <div>
            <h3>Builder</h3>
            <p>Character drafts and refinements will appear here.</p>
          </div>
        </article>
      </div>
    </section>
  );
}

function SettingsPanel({ status }: { status: StoreStatus }) {
  return (
    <section className="workspace-band">
      <div className="panel-heading">
        <Settings size={20} />
        <h2>Settings</h2>
      </div>
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
      </dl>
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
