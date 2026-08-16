import { useEffect, useState } from "react";
import * as wasm from "../core/wasm";
import type { Profile } from "../core/types";
import { APPS, APP_ORDER } from "./registry";
import { useWm } from "../state/WmContext";

// A small orientation card shown when the desktop starts. It also proves the
// knowledge core is loaded by greeting the profile owner by name.
export function Welcome() {
  const [profile, setProfile] = useState<Profile | null>(null);
  const wm = useWm();

  useEffect(() => {
    setProfile(wasm.profile());
  }, []);

  const firstName = profile?.person.name.split(" ")[0] ?? "user";

  return (
    <div className="app app-body" style={{ gap: 16 }}>
      <div>
        <h1 style={{ margin: "0 0 2px", fontSize: 20 }}>
          Welcome to NIKHIL//OS
        </h1>
        <p className="muted" style={{ margin: 0 }}>
          {firstName}, this is a personal computing environment. Every number
          on screen comes from the simulated kernel running in your browser.
        </p>
      </div>

      <div style={{ display: "grid", gap: 8, gridTemplateColumns: "1fr 1fr" }}>
        {APP_ORDER.filter((id) => id !== "welcome").map((id) => {
          const def = APPS[id];
          return (
            <button
              key={id}
              className="btn"
              style={{
                display: "flex",
                alignItems: "center",
                gap: 10,
                justifyContent: "flex-start",
                textAlign: "left",
              }}
              onClick={() => wm.open(id, def.title)}
            >
              {def.icon}
              <span>{def.title}</span>
            </button>
          );
        })}
      </div>

      <div className="card" style={{ fontSize: 12.5 }}>
        <div className="dim" style={{ marginBottom: 6 }}>PRO TIPS</div>
        <ul style={{ margin: 0, paddingLeft: 18, color: "var(--text-secondary)" }}>
          <li>
            Press <kbd>⌘</kbd> or <kbd>Ctrl</kbd>+<kbd>K</kbd> anywhere to open
            the command palette.
          </li>
          <li>
            Open Terminal and try <code className="accent">neofetch</code>,{" "}
            <code className="accent">ps | grep ai</code>,{" "}
            <code className="accent">pkgctl list</code>, or{" "}
            <code className="accent">help</code>.
          </li>
          <li>
            Browse the live kernel state in <code className="accent">/proc</code>{" "}
            and <code className="accent">/sys</code> with the Files app.
          </li>
        </ul>
      </div>
    </div>
  );
}
