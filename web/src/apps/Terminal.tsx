import { useEffect, useRef, useState } from "react";
import * as wasm from "../core/wasm";
import { registerTerminal, takePending } from "../state/terminalBus";

interface Line {
  input?: string;
  output: string;
}

// The real `nish` shell, driven by the WASM core. No fake commands.
export function Terminal() {
  const [lines, setLines] = useState<Line[]>([
    { output: "NIKHIL//OS 0.1.0 — type 'help' for a command list." },
  ]);
  const [promptStr, setPromptStr] = useState("nikhil@nikhil-os:~$");
  const [input, setInput] = useState("");
  const [history, setHistory] = useState<string[]>([]);
  const [histIdx, setHistIdx] = useState(-1);
  const boxRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    setPromptStr(wasm.prompt());
    const queued = takePending();
    if (queued) run(queued);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    registerTerminal((cmd) => run(cmd));
    return () => registerTerminal(null);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    const el = boxRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [lines]);

  const run = (raw: string) => {
    const cmd = raw.trim();
    if (!cmd) return;
    const output = wasm.runCommand(cmd);
    setLines((l) => [...l, { input: cmd, output }]);
    setHistory((h) => [...h, cmd]);
    setHistIdx(-1);
    setPromptStr(wasm.prompt());
    setInput("");
  };

  const complete = () => {
    const word = input.split(/\s+/).pop() ?? "";
    if (!word) return;
    const names = wasm.autocomplete(word);
    if (names.length === 1) {
      const prefix = input.slice(0, input.length - word.length);
      setInput(prefix + names[0]);
    }
  };

  const onKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      e.preventDefault();
      run(input);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      if (history.length === 0) return;
      const idx = histIdx < 0 ? history.length - 1 : Math.max(0, histIdx - 1);
      setHistIdx(idx);
      setInput(history[idx]);
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      const idx = histIdx + 1;
      if (idx >= history.length) {
        setHistIdx(-1);
        setInput("");
      } else {
        setHistIdx(idx);
        setInput(history[idx]);
      }
    } else if (e.key === "Tab") {
      e.preventDefault();
      complete();
    } else if (e.ctrlKey && e.key.toLowerCase() === "l") {
      e.preventDefault();
      setLines([]);
    }
  };

  return (
    <div
      className="app terminal"
      style={{ background: "var(--bg)", fontFamily: "var(--font-mono)" }}
    >
      <div
        ref={boxRef}
        className="terminal-scroll"
        style={{ flex: 1, overflow: "auto", padding: "12px 14px", fontSize: 13 }}
        onClick={() => inputRef.current?.focus()}
      >
        {lines.map((line, i) => (
          <div key={i}>
            {line.input !== undefined && (
              <div>
                <span style={{ color: "var(--accent)" }}>{promptStr}</span>{" "}
                <span>{line.input}</span>
              </div>
            )}
            {line.output.split("\n").map((o, j) => (
              <pre key={j} style={{ margin: 0, whiteSpace: "pre-wrap", color: "var(--text)" }}>
                {o}
              </pre>
            ))}
          </div>
        ))}
        <div style={{ display: "flex", alignItems: "baseline", gap: 8 }}>
          <span style={{ color: "var(--accent)", whiteSpace: "pre" }}>{promptStr}</span>
          <input
            ref={inputRef}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={onKeyDown}
            autoFocus
            aria-label="shell input"
            spellCheck={false}
            autoComplete="off"
            style={{
              flex: 1,
              background: "transparent",
              border: "none",
              outline: "none",
              color: "var(--text)",
              fontFamily: "inherit",
              fontSize: 13,
              caretColor: "var(--accent)",
            }}
          />
        </div>
      </div>
    </div>
  );
}
