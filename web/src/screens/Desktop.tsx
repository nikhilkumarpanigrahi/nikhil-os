import { useEffect, useRef, useState } from "react";
import { CommandPalette } from "../components/CommandPalette";
import { Sidebar } from "../components/Sidebar";
import { SystemBar } from "../components/SystemBar";
import { Window } from "../components/Window";
import { WmContext } from "../state/WmContext";
import { openWindow, useWindows } from "../state/windows";

interface Props {
  version: string;
  onPowerOff: () => void;
}

export function Desktop({ version, onPowerOff }: Props) {
  const wm = useWindows();
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [bounds, setBounds] = useState({
    width: window.innerWidth,
    height: window.innerHeight - 40,
  });
  const welcomed = useRef(false);

  useEffect(() => {
    const onResize = () =>
      setBounds({ width: window.innerWidth, height: window.innerHeight - 40 });
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  }, []);

  useEffect(() => {
    if (!welcomed.current) {
      welcomed.current = true;
      openWindow(wm, "welcome", "Welcome");
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setPaletteOpen((p) => !p);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  return (
    <div className="desktop-shell">
      <SystemBar
        version={version}
        onPower={onPowerOff}
        onTogglePalette={() => setPaletteOpen(true)}
      />
      <div className="desktop-row">
        <Sidebar wm={wm} />
        <WmContext.Provider value={wm}>
          <div className="desktop">
            <div className="workspace">
              {wm.windows.map((w) => (
                <Window key={w.id} win={w} wm={wm} bounds={bounds} />
              ))}
            </div>
          </div>
        </WmContext.Provider>
      </div>
      {paletteOpen && (
        <CommandPalette
          wm={wm}
          onClose={() => setPaletteOpen(false)}
          onPowerOff={onPowerOff}
        />
      )}
    </div>
  );
}
