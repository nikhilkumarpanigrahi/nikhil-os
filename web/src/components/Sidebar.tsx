import { APP_ORDER, APPS } from "../apps/registry";
import { openWindow } from "../state/windows";
import type { WindowManager } from "../state/windows";

interface Props {
  wm: WindowManager;
}

export function Sidebar({ wm }: Props) {
  return (
    <nav className="sidebar" aria-label="Applications">
      <div className="sidebar-group">Applications</div>
      {APP_ORDER.map((id) => {
        const def = APPS[id];
        const open = wm.windows.some((w) => w.app === id);
        const focused = open && wm.activeId !== null && wm.windows.find((w) => w.id === wm.activeId)?.app === id;
        return (
          <button
            key={id}
            className={`sidebar-item${focused ? " active" : ""}`}
            onClick={() => openWindow(wm, id, def.title)}
            aria-current={focused ? "page" : undefined}
          >
            {def.icon}
            <span>{def.title}</span>
          </button>
        );
      })}
    </nav>
  );
}
