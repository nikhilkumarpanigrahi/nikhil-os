// Owns the single System instance for the whole session: init + boot once,
// then hands out the live state. The browser owns one machine, like a kernel.

import { useEffect, useRef, useState } from "react";
import * as wasm from "../core/wasm";

export function useCore() {
  const bootLogRef = useRef<string[]>([]);
  const [ready, setReady] = useState(false);
  const [version, setVersion] = useState("");

  useEffect(() => {
    wasm.init();
    bootLogRef.current = wasm.boot().split("\n").filter(Boolean);
    setVersion(wasm.version());
    setReady(true);
  }, []);

  return { ready, getBootLog: () => bootLogRef.current, version };
}
