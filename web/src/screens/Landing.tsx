interface Props {
  onEnter: () => void;
  repoUrl: string;
}

const FEATURES: [string, string][] = [
  ["Rust kernel", "compiled to WebAssembly"],
  ["Real shell", "nish with pipes & redirects"],
  ["Live /proc", "observability, not a mock"],
  ["AI-native", "knowledge at the core"],
];

export function Landing({ onEnter, repoUrl }: Props) {
  return (
    <main className="landing">
      <div className="landing-inner">
        <div className="eyebrow">An AI-native personal computing environment</div>
        <h1>
          NIKHIL<span className="slash">//</span>OS
        </h1>
        <p className="tagline">
          A Unix-inspired operating system running in your browser — a simulated
          kernel, a real shell, and a desktop built on top of both.
        </p>
        <div className="actions">
          <button className="btn btn-primary" onClick={onEnter} autoFocus>
            Enter NIKHIL//OS
          </button>
          <button className="btn" onClick={() => window.open(repoUrl, "_blank")}>
            Explore the architecture ↗
          </button>
        </div>
        <div className="features">
          {FEATURES.map(([a, b]) => (
            <span key={a}>
              <b>{a}</b> {b}
            </span>
          ))}
        </div>
      </div>
      <footer>
        MIT licensed · fork it for your own portfolio ·{" "}
        <a href={repoUrl} target="_blank" rel="noreferrer">
          github.com/nikhilkumarpanigrahi/nikhil-os
        </a>
      </footer>
    </main>
  );
}
