// Window manager store: position, size, z-order, focus, minimize, maximize.
// Pure React state — no DOM math, so it is trivially testable.

import { useCallback, useRef, useState } from "react";
import type { AppId } from "../apps/registry";
import { APPS } from "../apps/registry";

export interface WindowState {
  id: number;
  app: AppId;
  title: string;
  x: number;
  y: number;
  w: number;
  h: number;
  z: number;
  minimized: boolean;
  maximized: boolean;
}

const CASCADE_STEP = 28;
const CASCADE_WRAP = 6;

export function useWindows() {
  const [windows, setWindows] = useState<WindowState[]>([]);
  const [activeId, setActiveId] = useState<number | null>(null);
  const nextId = useRef(1);
  const zRef = useRef(10);

  const topZ = useCallback(() => {
    zRef.current += 1;
    return zRef.current;
  }, []);

  const open = useCallback(
    (app: AppId, title: string) => {
      const id = nextId.current++;
      const size = APPS[app].defaultSize;
      const x = 80 + (windows.length % CASCADE_WRAP) * CASCADE_STEP;
      const y = 60 + (windows.length % CASCADE_WRAP) * CASCADE_STEP;
      const win: WindowState = {
        id,
        app,
        title,
        x,
        y,
        w: size.w,
        h: size.h,
        z: topZ(),
        minimized: false,
        maximized: false,
      };
      setWindows((ws) => [...ws, win]);
      setActiveId(id);
    },
    [windows.length, topZ],
  );

  const focus = useCallback(
    (id: number) => {
      setWindows((ws) => ws.map((w) => (w.id === id ? { ...w, z: topZ() } : w)));
      setActiveId(id);
    },
    [topZ],
  );

  const close = useCallback((id: number) => {
    setWindows((ws) => {
      const remaining = ws.filter((w) => w.id !== id);
      return remaining;
    });
    setActiveId((cur) => {
      if (cur !== id) return cur;
      const top = [...windows]
        .filter((w) => w.id !== id && !w.minimized)
        .sort((a, b) => b.z - a.z)[0];
      return top ? top.id : null;
    });
  }, [windows]);

  const minimize = useCallback(
    (id: number) => {
      setWindows((ws) =>
        ws.map((w) => (w.id === id ? { ...w, minimized: !w.minimized } : w)),
      );
      setActiveId((cur) => {
        if (cur !== id) return cur;
        const top = windows
          .filter((w) => w.id !== id && !w.minimized)
          .sort((a, b) => b.z - a.z)[0];
        return top ? top.id : null;
      });
    },
    [windows],
  );

  const toggleMaximize = useCallback(
    (id: number) => {
      setWindows((ws) =>
        ws.map((w) => (w.id === id ? { ...w, maximized: !w.maximized } : w)),
      );
      focus(id);
    },
    [focus],
  );

  const move = useCallback((id: number, x: number, y: number) => {
    setWindows((ws) => ws.map((w) => (w.id === id ? { ...w, x, y } : w)));
  }, []);

  const resize = useCallback((id: number, w: number, h: number) => {
    setWindows((ws) => ws.map((win) => (win.id === id ? { ...win, w, h } : win)));
  }, []);

  const closeAll = useCallback(() => {
    setWindows([]);
    setActiveId(null);
  }, []);

  const restore = useCallback((id: number) => {
    setWindows((ws) => ws.map((w) => (w.id === id ? { ...w, minimized: false } : w)));
    focus(id);
  }, [focus]);

  return {
    windows,
    activeId,
    open,
    close,
    focus,
    minimize,
    toggleMaximize,
    move,
    resize,
    closeAll,
    restore,
  };
}

export type WindowManager = ReturnType<typeof useWindows>;

export const openWindow = (
  wm: WindowManager,
  app: AppId,
  title: string,
) => {
  if (wm.windows.some((w) => w.app === app && !w.minimized)) {
    const existing = wm.windows.find((w) => w.app === app);
    if (existing) wm.restore(existing.id);
  } else {
    wm.open(app, title);
  }
};
