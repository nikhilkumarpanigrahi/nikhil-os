import { createContext, useContext } from "react";
import type { WindowManager } from "./windows";

export const WmContext = createContext<WindowManager | null>(null);

export function useWm(): WindowManager {
  const ctx = useContext(WmContext);
  if (!ctx) throw new Error("useWm must be used inside <WmContext.Provider>");
  return ctx;
}
