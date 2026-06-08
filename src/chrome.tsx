import { CheckCircle2, Flame, Lock } from "lucide-react";
import { ReactNode } from "react";

import { StoreStatus } from "./bridge";

export function AppLogo() {
  return (
    <div className="brand-mark" aria-hidden="true">
      <Flame size={20} strokeWidth={2.4} />
    </div>
  );
}

export function NavButton({
  active,
  icon,
  label,
  onClick,
}: {
  active: boolean;
  icon: ReactNode;
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

export function StatusPill({ status }: { status: StoreStatus }) {
  return (
    <div className={status.unlocked ? "status-pill ready" : "status-pill locked"}>
      {status.unlocked ? <CheckCircle2 size={16} /> : <Lock size={16} />}
      <span>{status.unlocked ? "Unlocked" : "Locked"}</span>
    </div>
  );
}

export function InlineNotice({
  icon,
  title,
  detail,
}: {
  icon: ReactNode;
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

export function ToolbarButton({
  icon,
  label,
  onClick,
  disabled,
}: {
  icon: ReactNode;
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
