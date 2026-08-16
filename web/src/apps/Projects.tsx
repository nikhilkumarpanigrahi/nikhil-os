import { useMemo, useState, useEffect } from "react";
import * as wasm from "../core/wasm";
import type { Profile, Project } from "../core/types";

const CATEGORY_LABEL: Record<string, string> = {
  "ai-ml": "AI/ML",
  backend: "Backend",
  systems: "Systems",
  "open-source": "Open Source",
};

export function Projects() {
  const [profile, setProfile] = useState<Profile | null>(null);
  const [filter, setFilter] = useState<string>("all");
  const [openId, setOpenId] = useState<string | null>(null);

  useEffect(() => {
    setProfile(wasm.profile());
  }, []);

  const categories = useMemo(() => {
    const set = new Set(profile?.projects.map((p) => p.category) ?? []);
    return ["all", ...set];
  }, [profile]);

  const visible = useMemo(() => {
    if (!profile) return [];
    if (filter === "all") return profile.projects;
    return profile.projects.filter((p) => p.category === filter);
  }, [profile, filter]);

  if (!profile) return <div className="app app-empty">Loading knowledge…</div>;

  return (
    <div className="app app-body" style={{ padding: 0 }}>
      <div className="app-toolbar">
        <div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
          {categories.map((c) => (
            <button
              key={c}
              className={`chip ${filter === c ? "active" : ""}`}
              style={{ background: "none", border: "1px solid var(--border)", cursor: "pointer" }}
              onClick={() => setFilter(c)}
            >
              {CATEGORY_LABEL[c] ?? c}
            </button>
          ))}
        </div>
      </div>

      <div className="app-body">
        <div className="project-grid">
          {visible.map((p) => (
            <ProjectCard
              key={p.id}
              project={p}
              expanded={openId === p.id}
              onToggle={() => setOpenId(openId === p.id ? null : p.id)}
            />
          ))}
          {visible.length === 0 && <div className="app-empty">No projects in this category.</div>}
        </div>
      </div>
    </div>
  );
}

function ProjectCard({
  project,
  expanded,
  onToggle,
}: {
  project: Project;
  expanded: boolean;
  onToggle: () => void;
}) {
  return (
    <article className="card" style={{ display: "flex", flexDirection: "column", gap: 10 }}>
      <div style={{ display: "flex", justifyContent: "space-between", gap: 8, alignItems: "flex-start" }}>
        <h3 style={{ margin: 0, fontSize: 15 }}>{project.title}</h3>
        <span className="chip">{CATEGORY_LABEL[project.category] ?? project.category}</span>
      </div>
      <p className="muted" style={{ margin: 0, fontSize: 13 }}>{project.summary}</p>

      <div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
        {project.technologies.map((t) => (
          <span key={t} className="chip" style={{ color: "var(--text-secondary)" }}>{t}</span>
        ))}
      </div>

      {project.highlights.length > 0 && (
        <ul style={{ margin: 0, paddingLeft: 18, fontSize: 12.5, color: "var(--text-secondary)" }}>
          {project.highlights.slice(0, 3).map((h, i) => (
            <li key={i}>{h}</li>
          ))}
        </ul>
      )}

      <div style={{ display: "flex", gap: 12, alignItems: "center", marginTop: "auto" }}>
        {project.repo && (
          <a href={project.repo} target="_blank" rel="noreferrer" onClick={(e) => e.stopPropagation()}>
            Repository ↗
          </a>
        )}
        {project.demo && (
          <a href={project.demo} target="_blank" rel="noreferrer" onClick={(e) => e.stopPropagation()}>
            Live demo ↗
          </a>
        )}
        {project.evidence.map((ev, i) => (
          <a key={i} href={ev.url} target="_blank" rel="noreferrer" onClick={(e) => e.stopPropagation()}>
            {ev.title} ↗
          </a>
        ))}
        <button className="btn" style={{ marginLeft: "auto", padding: "4px 10px" }} onClick={onToggle}>
          {expanded ? "Hide" : "Architecture"}
        </button>
      </div>

      {expanded && (
        <pre className="mono" style={{ margin: 0, padding: 12, background: "var(--bg)", border: "1px solid var(--border)", borderRadius: 8, fontSize: 12, overflow: "auto", whiteSpace: "pre-wrap" }}>
          {project.architecture}
        </pre>
      )}
    </article>
  );
}
