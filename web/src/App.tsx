import { useState } from "react";
import { useCore } from "./hooks/useCore";
import { BootScreen } from "./screens/BootScreen";
import { Desktop } from "./screens/Desktop";
import { Landing } from "./screens/Landing";

const REPO_URL = "https://github.com/nikhilkumarpanigrahi/nikhil-os";

type Stage = "landing" | "boot" | "desktop";

export default function App() {
  const { ready, getBootLog, version } = useCore();
  const [stage, setStage] = useState<Stage>("landing");

  const enter = () => {
    if (ready) setStage("boot");
  };

  const powerOff = () => {
    setStage("landing");
  };

  if (stage === "boot") {
    return <BootScreen lines={getBootLog()} onDone={() => setStage("desktop")} />;
  }
  if (stage === "desktop") {
    return <Desktop version={version} onPowerOff={powerOff} />;
  }
  return <Landing onEnter={enter} repoUrl={REPO_URL} />;
}
