import { useEffect, useState } from "react";
import * as wasm from "../core/wasm";
import type { Profile } from "../core/types";

export function Resume() {
  const [profile, setProfile] = useState<Profile | null>(null);

  useEffect(() => {
    setProfile(wasm.profile());
  }, []);

  if (!profile) return <div className="app app-empty">Loading knowledge…</div>;
  const { person } = profile;

  const skillGroups = profile.skills.reduce<Record<string, string[]>>((acc, s) => {
    (acc[s.category] ??= []).push(s.name);
    return acc;
  }, {});

  return (
    <div className="app app-body" style={{ maxWidth: 720, margin: "0 auto", width: "100%" }}>
      <header style={{ marginBottom: 18 }}>
        <h1 style={{ margin: 0, fontSize: 22 }}>{person.name}</h1>
        <div className="muted" style={{ fontSize: 14 }}>{person.role}</div>
        <div className="dim" style={{ fontSize: 12.5 }}>{person.location}</div>
      </header>

      <p className="muted" style={{ marginTop: 0 }}>{person.summary}</p>

      <Section title="Experience">
        {profile.experience.map((exp, i) => (
          <div key={i} style={{ marginBottom: 14, position: "relative", paddingLeft: 16, borderLeft: "1px solid var(--border)" }}>
            <div style={{ fontWeight: 600 }}>{exp.role}</div>
            <div className="accent" style={{ fontSize: 13 }}>
              {exp.organization} <span className="dim">· {exp.period}</span>
            </div>
            <p className="muted" style={{ margin: "4px 0", fontSize: 13 }}>{exp.summary}</p>
            {exp.highlights.length > 0 && (
              <ul style={{ margin: 0, paddingLeft: 18, fontSize: 12.5, color: "var(--text-secondary)" }}>
                {exp.highlights.map((h, j) => (
                  <li key={j}>{h}</li>
                ))}
              </ul>
            )}
          </div>
        ))}
      </Section>

      <Section title="Skills">
        {Object.entries(skillGroups).map(([group, skills]) => (
          <div key={group} style={{ marginBottom: 10 }}>
            <div className="dim" style={{ fontSize: 12, textTransform: "uppercase", letterSpacing: "0.04em" }}>{group}</div>
            <div style={{ display: "flex", gap: 6, flexWrap: "wrap", marginTop: 4 }}>
              {skills.map((s) => (
                <span key={s} className="chip">{s}</span>
              ))}
            </div>
          </div>
        ))}
      </Section>

      <Section title="Education">
        {profile.education.map((ed, i) => (
          <div key={i} style={{ marginBottom: 8 }}>
            <div style={{ fontWeight: 600 }}>{ed.degree}</div>
            <div className="muted" style={{ fontSize: 13 }}>
              {ed.institution} <span className="dim">· {ed.period}</span>
            </div>
          </div>
        ))}
      </Section>

      {profile.certifications.length > 0 && (
        <Section title="Certifications">
          <ul style={{ margin: 0, paddingLeft: 18, fontSize: 13, color: "var(--text-secondary)" }}>
            {profile.certifications.map((c, i) => (
              <li key={i}>
                {c.name} <span className="dim">— {c.issuer}, {c.year}</span>
              </li>
            ))}
          </ul>
        </Section>
      )}
    </div>
  );
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section style={{ marginBottom: 22 }}>
      <h2
        style={{
          fontSize: 13,
          textTransform: "uppercase",
          letterSpacing: "0.06em",
          color: "var(--text-muted)",
          margin: "0 0 10px",
          borderBottom: "1px solid var(--border)",
          paddingBottom: 6,
        }}
      >
        {title}
      </h2>
      {children}
    </section>
  );
}
