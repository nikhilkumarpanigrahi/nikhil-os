import { useCallback, useEffect, useState } from "react";
import * as wasm from "../core/wasm";
import type { DirEntry, FileStat } from "../core/types";

const QUICK_LINKS = ["/home", "/etc", "/bin", "/usr", "/var", "/proc", "/sys", "/dev"];

// Browser for the real virtual filesystem, including live /proc and /sys.
export function Files() {
  const [path, setPath] = useState("/home");
  const [entries, setEntries] = useState<DirEntry[]>([]);
  const [stat, setStat] = useState<FileStat | null>(null);
  const [content, setContent] = useState<string | null>(null);
  const [selected, setSelected] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [history, setHistory] = useState<string[]>([]);

  const load = useCallback((p: string) => {
    const dirs = wasm.listDir(p);
    setPath(p);
    setEntries(dirs);
    setSelected(null);
    setContent(null);
    setStat(wasm.statPath(p));
    setError(null);
    setHistory((h) => (h[h.length - 1] === p ? h : [...h, p]));
  }, []);

  useEffect(() => {
    load("/home");
  }, [load]);

  const enter = (entry: DirEntry) => {
    const next = path === "/" ? `/${entry.name}` : `${path}/${entry.name}`;
    if (entry.file_type === "directory") {
      load(next);
    } else {
      setSelected(entry.name);
      setContent(wasm.readFile(next));
      setStat(wasm.statPath(next));
    }
  };

  const goBack = () => {
    if (history.length > 1) {
      const prev = history[history.length - 2];
      setHistory((h) => h.slice(0, -1));
      load(prev);
    }
  };

  const segments = path.split("/").filter(Boolean);
  const crumb = (i: number) => "/" + segments.slice(0, i + 1).join("/");

  return (
    <div className="app">
      <div className="app-toolbar">
        <button className="btn" onClick={goBack} disabled={history.length <= 1} aria-label="back">
          ←
        </button>
        <div className="crumbs" style={{ display: "flex", gap: 4, alignItems: "center", flex: 1, overflow: "hidden" }}>
          <button className="crumb-link" style={{ background: "none", border: "none", color: "var(--accent)", cursor: "pointer", font: "inherit" }} onClick={() => load("/")}>
            /
          </button>
          {segments.map((seg, i) => (
            <span key={i} style={{ display: "flex", gap: 4, whiteSpace: "nowrap" }}>
              <span className="dim">/</span>
              <button
                className="crumb-link"
                style={{ background: "none", border: "none", color: i === segments.length - 1 ? "var(--text)" : "var(--accent)", cursor: "pointer", font: "inherit" }}
                onClick={() => load(crumb(i))}
              >
                {seg}
              </button>
            </span>
          ))}
        </div>
      </div>

      <div style={{ display: "flex", flex: 1, minHeight: 0 }}>
        <div style={{ width: 160, borderRight: "1px solid var(--border)", padding: 8, overflow: "auto", flex: "none" }}>
          <div className="dim" style={{ fontSize: 11, textTransform: "uppercase", letterSpacing: "0.04em", marginBottom: 6 }}>
            Quick access
          </div>
          {QUICK_LINKS.map((q) => (
            <button
              key={q}
              className="btn"
              style={{
                display: "block",
                width: "100%",
                textAlign: "left",
                marginBottom: 4,
                padding: "4px 8px",
                background: path === q ? "var(--accent-dim)" : undefined,
                borderColor: path === q ? "var(--accent)" : undefined,
              }}
              onClick={() => load(q)}
            >
              {q}
            </button>
          ))}
        </div>

        <div style={{ flex: 1, display: "flex", flexDirection: "column", minWidth: 0 }}>
          {error && <div className="err" style={{ padding: 12 }}>{error}</div>}
          <div className="listing" style={{ flex: 1, overflow: "auto", padding: "8px 0" }}>
            {entries.map((e) => (
              <button
                key={e.name}
                onClick={() => enter(e)}
                style={{
                  display: "flex",
                  width: "100%",
                  alignItems: "center",
                  gap: 10,
                  padding: "5px 14px",
                  background: "none",
                  border: "none",
                  color: "var(--text)",
                  cursor: "pointer",
                  textAlign: "left",
                  font: "inherit",
                }}
                onMouseEnter={(ev) => (ev.currentTarget.style.background = "rgba(255,255,255,0.03)")}
                onMouseLeave={(ev) => (ev.currentTarget.style.background = "none")}
              >
                <span style={{ color: e.file_type === "directory" ? "var(--accent)" : "var(--text-muted)" }}>
                  {e.file_type === "directory" ? "▸" : "•"}
                </span>
                <span style={{ flex: 1 }}>{e.name}</span>
                <span className="dim mono" style={{ fontSize: 12 }}>{e.perms}</span>
                <span className="dim" style={{ fontSize: 12, width: 90, textAlign: "right" }}>
                  {e.file_type === "file" ? `${e.size} B` : ""}
                </span>
              </button>
            ))}
            {entries.length === 0 && <div className="app-empty">Empty directory</div>}
          </div>

          {content !== null && selected && (
            <div style={{ borderTop: "1px solid var(--border)", padding: 10, maxHeight: "38%", overflow: "auto" }}>
              <div className="dim" style={{ fontSize: 11, textTransform: "uppercase", letterSpacing: "0.04em", marginBottom: 4 }}>
                {selected}
                {stat && <span style={{ marginLeft: 12, textTransform: "none" }}>{stat.size} B · {stat.perms} · {stat.owner}:{stat.group}</span>}
              </div>
              <pre className="mono" style={{ margin: 0, fontSize: 12, whiteSpace: "pre-wrap", color: "var(--text-secondary)" }}>
                {content}
              </pre>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
