import { useEffect, useMemo, useState } from "react";
import * as wasm from "../core/wasm";
import { useSystemSnapshot } from "../hooks/useSystemSnapshot";
import type { ProcessState } from "../core/types";

type SortKey = "pid" | "cpu" | "mem" | "name" | "state";

// Live telemetry straight from the kernel — processes, memory, services,
// scheduler, and the structured event bus. No fabricated values.
export function SystemMonitor() {
  const snap = useSystemSnapshot(700);
  const [sortKey, setSortKey] = useState<SortKey>("cpu");
  const [events, setEvents] = useState<unknown[]>([]);

  useEffect(() => {
    setEvents(wasm.events(40));
  }, [snap.tick]);

  const procs = useMemo(() => {
    return [...snap.processes].sort((a, b) => {
      switch (sortKey) {
        case "pid": return a.pid - b.pid;
        case "cpu": return b.cpu_usage - a.cpu_usage;
        case "mem": return b.memory_kb - a.memory_kb;
        case "name": return a.name.localeCompare(b.name);
        case "state": return a.state.localeCompare(b.state);
      }
    });
  }, [snap.processes, sortKey]);

  const memPct = snap.memory.used_percent;
  const memColor = memPct > 80 ? "var(--err)" : memPct > 55 ? "var(--warn)" : "var(--accent)";

  return (
    <div className="app">
      <div className="app-toolbar">
        <span className="title">System Monitor</span>
        <span className="chip mono dim">tick {snap.tick}</span>
        <span className="chip mono dim">cpu {snap.cpu.toFixed(1)}%</span>
      </div>

      <div className="app-body" style={{ padding: 0, display: "flex", flexDirection: "column", gap: 0 }}>
        <div style={{ display: "flex", gap: 14, padding: 14, flexWrap: "wrap" }}>
          <Gauge label="CPU" value={snap.cpu} color="var(--accent)" />
          <Gauge label="Memory" value={memPct} color={memColor} detail={`${snap.memory.used_kb} / ${snap.memory.total_kb} kB`} />
          <Gauge label="Processes" value={snap.processes.length} max={64} color="var(--text)" />
        </div>

        <div style={{ padding: "0 14px 14px", overflow: "auto" }}>
          <table className="data-table">
            <thead>
              <tr>
                <th onClick={() => setSortKey("pid")} style={{ cursor: "pointer" }}>PID</th>
                <th onClick={() => setSortKey("name")} style={{ cursor: "pointer" }}>Process</th>
                <th onClick={() => setSortKey("state")} style={{ cursor: "pointer" }}>State</th>
                <th onClick={() => setSortKey("cpu")} style={{ cursor: "pointer" }}>CPU%</th>
                <th onClick={() => setSortKey("mem")} style={{ cursor: "pointer" }}>MEM (kB)</th>
                <th>User</th>
              </tr>
            </thead>
            <tbody>
              {procs.map((p) => (
                <tr key={p.pid}>
                  <td className="mono dim">{p.pid}</td>
                  <td>{p.name}</td>
                  <td><StatePill state={p.state} /></td>
                  <td className="mono">{p.cpu_usage.toFixed(1)}</td>
                  <td className="mono">{p.memory_kb}</td>
                  <td className="dim">{p.user}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 14, padding: "0 14px 14px" }}>
          <div className="card" style={{ padding: 12 }}>
            <div className="dim" style={{ fontSize: 11, textTransform: "uppercase", letterSpacing: "0.04em", marginBottom: 8 }}>
              Services
            </div>
            {snap.services.map((s) => (
              <div key={s.name} style={{ display: "flex", justifyContent: "space-between", padding: "3px 0", fontSize: 12.5 }}>
                <span>{s.name}</span>
                <span className="mono">
                  <span className={s.state === "running" ? "ok" : "warn"}>{s.state}</span>
                  {s.restarts > 0 && <span className="dim"> ×{s.restarts}</span>}
                </span>
              </div>
            ))}
          </div>

          <div className="card" style={{ padding: 12 }}>
            <div className="dim" style={{ fontSize: 11, textTransform: "uppercase", letterSpacing: "0.04em", marginBottom: 8 }}>
              Scheduler
            </div>
            <div style={{ fontSize: 12.5, display: "grid", gap: 3 }}>
              <div><span className="dim">Algorithm</span> <span className="mono">{snap.scheduler.algorithm}</span></div>
              <div><span className="dim">Context switches</span> <span className="mono">{snap.scheduler.context_switches}</span></div>
              <div><span className="dim">Current pid</span> <span className="mono">{snap.scheduler.current_pid}</span></div>
              <div><span className="dim">Time slice</span> <span className="mono">{snap.scheduler.time_slice_ticks} ticks</span></div>
            </div>
            <div className="dim" style={{ fontSize: 11, textTransform: "uppercase", letterSpacing: "0.04em", margin: "12px 0 6px" }}>
              Event bus
            </div>
            <div style={{ fontSize: 12, maxHeight: 180, overflow: "auto", fontFamily: "var(--font-mono)" }}>
              {events.slice().reverse().map((ev, i) => (
                <div key={i} className="dim" style={{ padding: "1px 0", whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>
                  {JSON.stringify(ev)}
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function Gauge({ label, value, color, max = 100, detail }: { label: string; value: number; color: string; max?: number; detail?: string }) {
  const pct = Math.min(100, Math.round((value / max) * 100));
  return (
    <div style={{ minWidth: 170, flex: 1 }}>
      <div style={{ display: "flex", justifyContent: "space-between", fontSize: 12, marginBottom: 4 }}>
        <span className="dim">{label}</span>
        <span className="mono" style={{ color }}>{detail ?? `${value}%`}</span>
      </div>
      <div className="bar"><span style={{ width: `${pct}%`, background: color }} /></div>
    </div>
  );
}

function StatePill({ state }: { state: ProcessState }) {
  const color = {
    NEW: "var(--proc-new)",
    READY: "var(--proc-ready)",
    RUNNING: "var(--proc-running)",
    WAITING: "var(--proc-waiting)",
    TERMINATED: "var(--proc-terminated)",
  }[state];
  return (
    <span className="mono" style={{ color, fontSize: 11.5 }}>{state}</span>
  );
}
