import { Library, Settings, UserRound } from "lucide-react";
import { useEffect, useMemo, useState } from "react";

import { DEFAULT_STATUS, NavKey, StoreStatus, command } from "./bridge";
import { AppLogo, NavButton, StatusPill } from "./chrome";
import { CharactersPanel, SettingsPanel, UnlockSurface, WorldsPanel } from "./panels";

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
