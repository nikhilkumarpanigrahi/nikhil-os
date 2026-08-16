import { useEffect, useMemo, useRef, useState } from "react";
import { APPS, APP_ORDER } from "../apps/registry";
import { openWindow } from "../state/windows";
import type { WindowManager } from "../state/windows";
import { requestCommand } from "../state/terminalBus";

interface Action {
  id: string;
  label: string;
  hint?: string;
  group: string;
  run: () => void;
}

interface Props {
  wm: WindowManager;
  onClose: () => void;
  onPowerOff: () => void;
}

export function CommandPalette({ wm, onClose, onPowerOff }: Props) {
  const [query, setQuery] = useState("");
  const [index, setIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const actions = useMemo<Action[]>(() => {
    const list: Action[] = [];
    for (const id of APP_ORDER) {
      const def = APPS[id];
      list.push({
        id: `open:${id}`,
        label: `Open ${def.title}`,
        hint: "app",
        group: "Applications",
        run: () => openWindow(wm, id, def.title),
      });
    }
    const commands: [string, string][] = [
      ["neofetch", "System report"],
      ["ps | grep ai", "Find AI processes"],
      ["pkgctl list", "Installed packages"],
      ["service status", "Service health"],
      ["ls /proc", "Live kernel state"],
      ["ls /sys", "Runtime sysfs"],
      ["top", "Process table"],
    ];
    for (const [cmd, hint] of commands) {
      list.push({
        id: `run:${cmd}`,
        label: cmd,
        hint,
        group: "Run in Terminal",
        run: () => {
          openWindow(wm, "terminal", "Terminal");
          requestCommand(cmd);
        },
      });
    }
    list.push({
      id: "act:clear",
      label: "Close all windows",
      hint: "workspace",
      group: "Actions",
      run: () => wm.closeAll(),
    });
    list.push({
      id: "act:power",
      label: "Power off",
      hint: "exit",
      group: "Actions",
      run: onPowerOff,
    });
    return list;
  }, [wm, onPowerOff]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return actions;
    return actions.filter(
      (a) => a.label.toLowerCase().includes(q) || (a.hint ?? "").toLowerCase().includes(q),
    );
  }, [actions, query]);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  useEffect(() => {
    setIndex(0);
  }, [query]);

  const exec = (a: Action) => {
    onClose();
    a.run();
  };

  const onKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      e.preventDefault();
      onClose();
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      setIndex((i) => Math.min(filtered.length - 1, i + 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setIndex((i) => Math.max(0, i - 1));
    } else if (e.key === "Enter") {
      e.preventDefault();
      const a = filtered[index];
      if (a) exec(a);
    }
  };

  let lastGroup = "";

  return (
    <div className="palette-overlay" onClick={onClose}>
      <div className="palette" onClick={(e) => e.stopPropagation()} role="dialog" aria-label="Command palette">
        <div className="palette-input-row">
          <span className="dim">⌘</span>
          <input
            ref={inputRef}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={onKeyDown}
            placeholder="Type a command or application…"
            aria-label="Search commands"
          />
          <span className="chip dim" style={{ fontSize: 11 }}>ESC to close</span>
        </div>
        <div className="palette-list" role="listbox">
          {filtered.length === 0 && (
            <div className="app-empty" style={{ padding: 20 }}>No results for “{query}”</div>
          )}
          {filtered.map((a, i) => {
            const showGroup = a.group !== lastGroup;
            lastGroup = a.group;
            return (
              <div key={a.id}>
                {showGroup && <div className="palette-group">{a.group}</div>}
                <button
                  role="option"
                  aria-selected={i === index}
                  className={`palette-item${i === index ? " selected" : ""}`}
                  onMouseEnter={() => setIndex(i)}
                  onClick={() => exec(a)}
                >
                  <span className="palette-label">{a.label}</span>
                  {a.hint && <span className="dim" style={{ fontSize: 11 }}>{a.hint}</span>}
                </button>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
