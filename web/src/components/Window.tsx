import { useCallback, useRef } from "react";
import { APPS } from "../apps/registry";
import type { WindowState, WindowManager } from "../state/windows";

const MIN_W = 360;
const MIN_H = 240;

interface Props {
  win: WindowState;
  wm: WindowManager;
  bounds: { width: number; height: number };
}

export function Window({ win, wm, bounds }: Props) {
  const dragRef = useRef<{ dx: number; dy: number } | null>(null);
  const resizeRef = useRef<{ x: number; y: number; w: number; h: number } | null>(null);
  const def = APPS[win.app];

  const onTitleDown = useCallback(
    (e: React.PointerEvent) => {
      if (win.maximized) return;
      e.preventDefault();
      wm.focus(win.id);
      dragRef.current = { dx: e.clientX - win.x, dy: e.clientY - win.y };
      const onMove = (ev: PointerEvent) => {
        if (!dragRef.current) return;
        const x = Math.max(-50, Math.min(bounds.width - 60, ev.clientX - dragRef.current.dx));
        const y = Math.max(0, Math.min(bounds.height - 40, ev.clientY - dragRef.current.dy));
        wm.move(win.id, x, y);
      };
      const onUp = (ev: PointerEvent) => {
        dragRef.current = null;
        window.removeEventListener("pointermove", onMove);
        window.removeEventListener("pointerup", onUp);
        // Snap: releasing within 24px of an edge docks the window to that half.
        const half = bounds.width / 2;
        if (ev.clientX < 24) {
          wm.resize(win.id, half, bounds.height - 2);
          wm.move(win.id, 0, 0);
        } else if (ev.clientX > bounds.width - 24) {
          wm.resize(win.id, half, bounds.height - 2);
          wm.move(win.id, half, 0);
        }
      };
      window.addEventListener("pointermove", onMove);
      window.addEventListener("pointerup", onUp);
    },
    [win, wm, bounds],
  );

  const onResizeDown = useCallback(
    (e: React.PointerEvent) => {
      e.preventDefault();
      e.stopPropagation();
      wm.focus(win.id);
      resizeRef.current = { x: e.clientX, y: e.clientY, w: win.w, h: win.h };
      const onMove = (ev: PointerEvent) => {
        if (!resizeRef.current) return;
        const r = resizeRef.current;
        const w = Math.max(MIN_W, Math.min(bounds.width - win.x, r.w + ev.clientX - r.x));
        const h = Math.max(MIN_H, Math.min(bounds.height - win.y, r.h + ev.clientY - r.y));
        wm.resize(win.id, w, h);
      };
      const onUp = () => {
        resizeRef.current = null;
        window.removeEventListener("pointermove", onMove);
        window.removeEventListener("pointerup", onUp);
      };
      window.addEventListener("pointermove", onMove);
      window.addEventListener("pointerup", onUp);
    },
    [win, wm, bounds],
  );

  const style: React.CSSProperties = win.maximized
    ? { left: 0, top: 0, width: bounds.width, height: bounds.height, zIndex: win.z }
    : { left: win.x, top: win.y, width: win.w, height: win.h, zIndex: win.z };

  const App = def.component;

  return (
    <section
      className="os-window"
      role="dialog"
      aria-label={win.title}
      aria-hidden={win.minimized}
      onPointerDown={() => wm.focus(win.id)}
      style={{
        ...style,
        display: win.minimized ? "none" : "flex",
      }}
    >
      <div
        className="window-titlebar"
        onPointerDown={onTitleDown}
        onDoubleClick={() => wm.toggleMaximize(win.id)}
      >
        <span className="window-title">
          {def.icon}
          <span className="title-text">{win.title}</span>
        </span>
        <div className="window-controls" onPointerDown={(e) => e.stopPropagation()}>
          <button className="wc" aria-label={`Minimize ${win.title}`} onClick={() => wm.minimize(win.id)}>−</button>
          <button className="wc" aria-label={`Maximize ${win.title}`} onClick={() => wm.toggleMaximize(win.id)}>
            {win.maximized ? "❐" : "□"}
          </button>
          <button className="wc wc-close" aria-label={`Close ${win.title}`} onClick={() => wm.close(win.id)}>×</button>
        </div>
      </div>
      <div className="window-body">
        <App />
      </div>
      {!win.maximized && (
        <div
          className="window-resize"
          role="separator"
          aria-label="resize"
          onPointerDown={onResizeDown}
        />
      )}
    </section>
  );
}
