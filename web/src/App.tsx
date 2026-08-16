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

  // Aurora backdrop: a fixed layer of blurred gradient blobs behind every
  // screen. Rendered once here so landing, boot, and desktop all share it.
  const aurora = (
    <div className="aurora" aria-hidden>
      <i className="b1" />
      <i className="b2" />
      <i className="b3" />
      <i className="b4" />
    </div>
  );

  const enter = () => {
    if (ready) setStage("boot");
  };

  const powerOff = () => {
    setStage("landing");
  };

  if (stage === "boot") {
    return (
      <>
        {aurora}
        <BootScreen lines={getBootLog()} onDone={() => setStage("desktop")} />
      </>
    );
  }
  if (stage === "desktop") {
    return (
      <>
        {aurora}
        <Desktop version={version} onPowerOff={powerOff} />
      </>
    );
  }
  return (
    <>
      {aurora}
      <Landing onEnter={enter} repoUrl={REPO_URL} />
    </>
  );
}
