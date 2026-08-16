import { useEffect, useState } from "react";

const REVEAL_MS = 150;
const SETTLE_MS = 350;

// Real boot log from the kernel, revealed line-by-line. The content is the
// system's actual boot sequence — no cinematic filler.
export function BootScreen({ lines, onDone }: { lines: string[]; onDone: () => void }) {
  const [shown, setShown] = useState(0);

  // Step through the log one line at a time.
  useEffect(() => {
    if (lines.length === 0) return;
    const t = window.setInterval(() => {
      setShown((s) => Math.min(s + 1, lines.length));
    }, REVEAL_MS);
    return () => window.clearInterval(t);
  }, [lines]);

  // When every line is visible, let the last one settle before handing off.
  useEffect(() => {
    if (lines.length === 0 || shown < lines.length) return;
    const t = window.setTimeout(onDone, SETTLE_MS);
    return () => window.clearTimeout(t);
  }, [shown, lines, onDone]);

  return (
    <main className="boot">
      <div className="boot-console">
        <div className="headline">NIKHIL//OS 0.1.0 — boot sequence</div>
        {lines.slice(0, shown).map((line, i) => {
          const ok = line.startsWith("[ OK ]");
          const done = i === lines.length - 1;
          return (
            <div key={i} className={`boot-line${ok && !done ? "" : " pending"}`}>
              <span className="mark">{ok ? "[ OK ]" : "[ ... ]"}</span>
              <span className="text">
                {ok ? line.slice("[ OK ] ".length) : line}
              </span>
            </div>
          );
        })}
        <div className="boot-line pending" style={{ paddingTop: 6 }}>
          <span className="mark">_</span>
          <span className="text">
            {shown >= lines.length ? "ready. launching desktop…" : "booting…"}
          </span>
        </div>
      </div>
    </main>
  );
}
