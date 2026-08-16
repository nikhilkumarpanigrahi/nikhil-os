import { useEffect, useState } from "react";
import * as wasm from "../core/wasm";
import type { Profile } from "../core/types";

// A concise, evidence-backed candidate view for recruiters and hiring
// managers. Every claim links to its proof.
export function Recruiter() {
  const [profile, setProfile] = useState<Profile | null>(null);

  useEffect(() => {
    setProfile(wasm.profile());
  }, []);

  if (!profile) return <div className="app app-empty">Loading knowledge…</div>;
  const { person } = profile;

  const topSkills = [...profile.skills]
    .sort((a, b) => b.level - a.level)
    .slice(0, 8);

  const contactItems = [
    person.contact.email && ["Email", person.contact.email, `mailto:${person.contact.email}`],
    person.contact.github && ["GitHub", person.contact.github, person.contact.github.startsWith("http") ? person.contact.github : `https://github.com/${person.contact.github}`],
    person.contact.linkedin && ["LinkedIn", person.contact.linkedin, person.contact.linkedin.startsWith("http") ? person.contact.linkedin : `https://linkedin.com/in/${person.contact.linkedin}`],
    person.contact.website && ["Website", person.contact.website, person.contact.website],
  ].filter(Boolean) as [string, string, string][];

  return (
    <div className="app app-body" style={{ maxWidth: 620, margin: "0 auto", width: "100%" }}>
      <header className="card" style={{ marginBottom: 16 }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", gap: 12 }}>
          <div>
            <h1 style={{ margin: 0, fontSize: 20 }}>{person.name}</h1>
            <div className="muted">{person.role}</div>
          </div>
          <span className="chip" style={{ color: "var(--ok)", borderColor: "var(--ok)" }}>
            ● Open to opportunities
          </span>
        </div>
        <p className="muted" style={{ margin: "12px 0 0", fontSize: 13 }}>{person.summary}</p>
      </header>

      {profile.highlights.length > 0 && (
        <Section title="Highlights">
          <ul style={{ margin: 0, paddingLeft: 18, fontSize: 13, color: "var(--text)" }}>
            {profile.highlights.map((h, i) => (
              <li key={i} style={{ marginBottom: 4 }}>{h}</li>
            ))}
          </ul>
        </Section>
      )}

      <Section title="Core strengths">
        <div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
          {topSkills.map((s) => (
            <span key={s.name} className="chip">
              {s.name}
              <span className="dim">·{s.level}</span>
            </span>
          ))}
        </div>
      </Section>

      {profile.claims.length > 0 && (
        <Section title="Verifiable claims">
          {profile.claims.map((c, i) => (
            <div key={i} className="card" style={{ padding: "12px 14px", marginBottom: 8 }}>
              <div style={{ fontSize: 13 }}>{c.claim}</div>
              {c.evidence.length > 0 && (
                <div className="dim" style={{ fontSize: 12, marginTop: 4 }}>
                  Evidence: {c.evidence.join(" · ")}
                </div>
              )}
            </div>
          ))}
        </Section>
      )}

      <Section title="Contact">
        <div style={{ display: "grid", gap: 6 }}>
          {contactItems.map(([label, value, href]) => (
            <div key={label} style={{ display: "flex", gap: 8, fontSize: 13 }}>
              <span className="dim" style={{ width: 72 }}>{label}</span>
              <a href={href} target="_blank" rel="noreferrer">{value}</a>
            </div>
          ))}
        </div>
      </Section>
    </div>
  );
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section style={{ marginBottom: 20 }}>
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
