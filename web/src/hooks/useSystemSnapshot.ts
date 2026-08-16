// Polls the simulated clock: each interval advances a tick and reads a fresh
// snapshot. All telemetry in the UI is real — nothing is fabricated.

import { useEffect, useRef, useState } from "react";
import * as wasm from "../core/wasm";
import type { Snapshot } from "../core/types";

const EMPTY: Snapshot = {
  tick: 0,
  cpu: 0,
  processes: [],
  memory: { total_kb: 0, used_kb: 0, free_kb: 0, used_percent: 0 },
  services: [],
  scheduler: { algorithm: "round-robin", context_switches: 0, current_pid: 0, time_slice_ticks: 0 },
};

export function useSystemSnapshot(intervalMs = 1200) {
  const [snap, setSnap] = useState<Snapshot>(EMPTY);
  const timer = useRef<number | null>(null);

  useEffect(() => {
    const refresh = () => {
      wasm.tick();
      setSnap(wasm.snapshot());
    };
    refresh();
    timer.current = window.setInterval(refresh, intervalMs);
    return () => {
      if (timer.current) window.clearInterval(timer.current);
    };
  }, [intervalMs]);

  return snap;
}
