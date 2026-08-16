import { useEffect, useState } from "react";
import * as wasm from "../core/wasm";
import type { Profile } from "../core/types";

const GITHUB = "https://github.com/nikhilkumarpanigrahi";

function ContactIcon({ kind }: { kind: "github" | "linkedin" | "email" | "web" }) {
  return (
    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
      {kind === "github" && (
        <path d="M9 19c-4.3 1.4-4.3-2.5-6-3m12 5v-3.5c0-1 .1-1.4-.5-2 2.8-.3 5.5-1.4 5.5-6a4.6 4.6 0 0 0-1.3-3.2 4.2 4.2 0 0 0-.1-3.2s-1.1-.3-3.5 1.3a12.3 12.3 0 0 0-6.2 0C6.5 2.8 5.4 3.1 5.4 3.1a4.2 4.2 0 0 0-.1 3.2A4.6 4.6 0 0 0 4 9.5c0 4.6 2.7 5.7 5.5 6-.6.6-.6 1.2-.5 2V21" />
      )}
      {kind === "linkedin" && (
        <>
          <path d="M16 8a6 6 0 0 1 6 6v7h-4v-7a2 2 0 0 0-4 0v7h-4v-7a6 6 0 0 1 6-6z" />
          <rect x="2" y="9" width="4" height="12" />
          <circle cx="4" cy="4" r="2" />
        </>
      )}
      {kind === "email" && (
        <>
          <rect x="3" y="5" width="18" height="14" rx="2" />
          <path d="m3 7 9 6 9-6" />
        </>
      )}
      {kind === "web" && (
        <>
          <circle cx="12" cy="12" r="9" />
          <path d="M3 12h18M12 3a15 15 0 0 1 0 18a15 15 0 0 1 0-18z" />
        </>
      )}
    </svg>
  );
}

export function Resume() {
  const [profile, setProfile] = useState<Profile | null>(null);

  useEffect(() => {
    setProfile(wasm.profile());
  }, []);

  if (!profile) return <div className="app app-empty">Loading knowledge…</div>;
  const { person } = profile;

  // Group skills by category, sorted strongest first for the bars.
  const groups = new Map<string, typeof profile.skills>();
  for (const s of profile.skills) {
    const g = groups.get(s.category) ?? [];
    g.push(s);
    groups.set(s.category, g);
  }
  for (const g of groups.values()) g.sort((a, b) => b.level - a.level);

  const contacts: { kind: "github" | "linkedin" | "email" | "web"; href: string; label: string }[] = [];
  if (person.contact.github) contacts.push({ kind: "github", href: person.contact.github.startsWith("http") ? person.contact.github : `${GITHUB}`, label: person.contact.github.replace(/^https?:\/\/(www\.)?/, "") });
  if (person.contact.linkedin) contacts.push({ kind: "linkedin", href: person.contact.linkedin, label: person.contact.linkedin.replace(/^https?:\/\/(www\.)?/, "") });
  if (person.contact.email) contacts.push({ kind: "email", href: `mailto:${person.contact.email}`, label: person.contact.email });
  if (person.contact.website) contacts.push({ kind: "web", href: person.contact.website, label: person.contact.website.replace(/^https?:\/\/(www\.)?/, "") });

  const stats: [number, string][] = [
    [profile.skills.length, "Skills"],
    [profile.technologies.length, "Technologies"],
    [profile.projects.length, "Projects"],
    [profile.certifications.length, "Certifications"],
    [profile.contributions.length, "Contributions"],
  ];

  const showStats = stats.filter(([n]) => n > 0).slice(0, 4);

  return (
    <div className="app app-body resume">
      {/* Hero */}
      <header className="resume-hero">
        <h1 className="resume-name">{person.name}</h1>
        <p className="resume-role">{person.role}</p>
        {person.location && (
          <p className="resume-loc">
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
              <path d="M20 10c0 6-8 12-8 12S4 16 4 10a8 8 0 0 1 16 0z" />
              <circle cx="12" cy="10" r="3" />
            </svg>
            {person.location}
          </p>
        )}

        {contacts.length > 0 && (
          <div className="resume-contact">
            {contacts.map((c) => (
              <a key={c.kind} className="chip" href={c.href} target={c.href.startsWith("mailto:") ? undefined : "_blank"} rel="noreferrer">
                <ContactIcon kind={c.kind} /> {c.label}
              </a>
            ))}
          </div>
        )}

        {showStats.length > 0 && (
          <div className="resume-stats">
            {showStats.map(([n, label]) => (
              <div className="resume-stat" key={label}>
                <b>{n}</b>
                <span>{label}</span>
              </div>
            ))}
          </div>
        )}

        {person.summary && <p className="resume-summary">{person.summary}</p>}
      </header>

      {/* Two-column: skills | experience */}
      <div className="resume-grid">
        <section className="resume-section">
          <h2>Skill profile</h2>
          {[...groups.entries()].map(([group, skills]) => (
            <div className="skill-group" key={group}>
              <div className="skill-group-label">{group}</div>
              {skills.map((s) => (
                <div className="skill-row" key={s.name}>
                  <span className="skill-name" title={s.name}>{s.name}</span>
                  <span className="bar">
                    <span style={{ width: `${Math.max(4, s.level)}%` }} />
                  </span>
                  <span className="skill-pct">{s.level}</span>
                </div>
              ))}
            </div>
          ))}
          {profile.technologies.length > 0 && (
            <>
              <div className="skill-group-label" style={{ marginTop: 18 }}>Toolchain</div>
              <div className="skill-tags">
                {profile.technologies.map((t) => (
                  <span className="chip" key={t}>{t}</span>
                ))}
              </div>
            </>
          )}
        </section>

        <section className="resume-section">
          <h2>Experience</h2>
          {profile.experience.length > 0 ? (
            <div className="resume-timeline">
              {profile.experience.map((exp, i) => (
                <div className="timeline-item" key={i}>
                  <h3>{exp.role}</h3>
                  {exp.organization && <div className="org">{exp.organization}</div>}
                  {exp.period && <span className="period">{exp.period}</span>}
                  {exp.summary && <p>{exp.summary}</p>}
                  {exp.highlights.length > 0 && (
                    <ul>
                      {exp.highlights.map((h, j) => (
                        <li key={j}>{h}</li>
                      ))}
                    </ul>
                  )}
                </div>
              ))}
            </div>
          ) : (
            <div className="app-empty">No experience on record yet.</div>
          )}
        </section>
      </div>

      {/* Education + certifications */}
      {(profile.education.length > 0 || profile.certifications.length > 0) && (
        <section className="resume-section">
          <h2>Education & credentials</h2>
          {profile.education.map((ed, i) => (
            <div className="resume-list-item" key={`ed-${i}`}>
              <div>
                <div className="what">{ed.degree}</div>
                <div className="where">{ed.institution}</div>
              </div>
              {ed.period && <span className="when">{ed.period}</span>}
            </div>
          ))}
          {profile.certifications.map((c, i) => (
            <div className="resume-list-item" key={`c-${i}`}>
              <div>
                <div className="what">{c.name}</div>
                <div className="where">{c.issuer}</div>
              </div>
              {c.year && <span className="when">{c.year}</span>}
            </div>
          ))}
        </section>
      )}

      {/* Achievements + contributions */}
      {(profile.achievements.length > 0 || profile.contributions.length > 0) && (
        <section className="resume-section">
          <h2>Notable work</h2>
          {profile.achievements.map((a, i) => (
            <div className="resume-list-item" key={`a-${i}`}>
              <div>
                <div className="what">{a.title}</div>
                {a.description && <div className="where">{a.description}</div>}
              </div>
            </div>
          ))}
          {profile.contributions.map((c, i) => (
            <div className="resume-list-item" key={`c-${i}`}>
              <div>
                <div className="what">{c.repo}</div>
                <div className="where">{c.description}</div>
              </div>
            </div>
          ))}
        </section>
      )}
    </div>
  );
}
