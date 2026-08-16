import { useEffect, useState } from "react";
import { useSystemSnapshot } from "../hooks/useSystemSnapshot";
import type { Snapshot } from "../core/types";

interface Props {
  onPower: () => void;
  onTogglePalette: () => void;
  version: string;
}

export function SystemBar({ onPower, onTogglePalette, version }: Props) {
  const snap = useSystemSnapshot(2000);
  const [now, setNow] = useState(() => new Date());

  useEffect(() => {
    const t = window.setInterval(() => setNow(new Date()), 1000);
    return () => window.clearInterval(t);
  }, []);

  const time = now.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" });
  const date = now.toLocaleDateString([], { weekday: "short", month: "short", day: "numeric" });

  return (
    <header className="sysbar" role="banner">
      <div className="sysbar-left">
        <span className="brand" onClick={onTogglePalette} role="button" tabIndex={0} aria-label="Open command palette">
          <span className="brand-mark" aria-hidden>◉</span>
          NIKHIL//OS
        </span>
        <span className="dim mono" style={{ fontSize: 11 }}>v{version}</span>
      </div>

      <div className="sysbar-center">
        <button className="sysbar-item" onClick={onTogglePalette}>
          <span className="dim">⌘K</span> Commands
        </button>
      </div>

      <div className="sysbar-right">
        <Telemetry snap={snap} />
        <div className="clock" title={now.toLocaleString()}>
          <div>{time}</div>
          <div className="dim" style={{ fontSize: 10 }}>{date}</div>
        </div>
        <button className="sysbar-item power" onClick={onPower} aria-label="Power off" title="Power off">
          ⏻
        </button>
      </div>
    </header>
  );
}

function Telemetry({ snap }: { snap: Snapshot }) {
  return (
    <div className="telemetry" title="Live from the simulated kernel">
      <span className="tele">
        <span className="dim">cpu</span> <span className="mono">{snap.cpu.toFixed(1)}%</span>
      </span>
      <span className="tele">
        <span className="dim">mem</span>{" "}
        <span className="mono">
          {Math.round((snap.memory.used_kb / 1024) * 10) / 10}/{Math.round((snap.memory.total_kb / 1024) * 10) / 10} MB
        </span>
      </span>
    </div>
  );
}
