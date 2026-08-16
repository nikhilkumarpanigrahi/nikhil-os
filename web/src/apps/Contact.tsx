import { useMemo, useState } from "react";
import * as wasm from "../core/wasm";
import { submitContact } from "../core/api";
import type { Profile } from "../core/types";

// The Contact app — the visitor-facing front door of the backend. Mirrors the
// server's validation rules so users see errors instantly, then hands off to
// the live API. If the service is unreachable it degrades to a mailto fallback
// (offline-first contract — the OS never depends on the network).

const TOPICS = [
  { id: "general", label: "General" },
  { id: "collaboration", label: "Collaborate" },
  { id: "opportunity", label: "Opportunity" },
  { id: "feedback", label: "Feedback" },
  { id: "recruiting", label: "Recruiting" },
];

type Status =
  | { kind: "idle" }
  | { kind: "sending" }
  | { kind: "sent"; id: string }
  | { kind: "error"; message: string; recoverable: boolean };

const MAX_BODY = 2000;

export function Contact() {
  const [profile] = useState<Profile | null>(() => wasm.profile());
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [topic, setTopic] = useState("collaboration");
  const [subject, setSubject] = useState("");
  const [body, setBody] = useState("");
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [status, setStatus] = useState<Status>({ kind: "idle" });

  const emailAddress = useMemo(
    () => profile?.person.contact.email?.trim() ?? "",
    [profile],
  );
  const githubUrl = useMemo(
    () =>
      profile?.person.contact.github
        ? `https://github.com/${profile.person.contact.github}`
        : "",
    [profile],
  );

  const validate = (): boolean => {
    const next: Record<string, string> = {};
    const trimmedName = name.trim();
    const trimmedEmail = email.trim();
    const trimmedBody = body.trim();
    if (trimmedName.length < 2 || trimmedName.length > 80) {
      next.name = "Name must be 2–80 characters.";
    }
    if (!isValidEmail(trimmedEmail)) {
      next.email = "Enter a valid email so you can be replied to.";
    }
    if (trimmedBody.length < 10) {
      next.body = "Say a little more — at least 10 characters.";
    } else if (trimmedBody.length > MAX_BODY) {
      next.body = `Keep it under ${MAX_BODY} characters.`;
    }
    if (subject.trim().length > 120) {
      next.subject = "Subject must be at most 120 characters.";
    }
    setErrors(next);
    return Object.keys(next).length === 0;
  };

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    if (status.kind === "sending") return;
    setErrors({});
    if (!validate()) return;
    setStatus({ kind: "sending" });
    submitContact({ name, email, topic, subject, body })
      .then((r) =>
        r.ok
          ? setStatus({ kind: "sent", id: r.id })
          : setStatus({ kind: "error", message: r.message, recoverable: r.recoverable }),
      )
      .catch(() =>
        setStatus({ kind: "error", message: "Something went wrong.", recoverable: true }),
      );
  };

  const reset = () => {
    setName("");
    setEmail("");
    setTopic("collaboration");
    setSubject("");
    setBody("");
    setErrors({});
    setStatus({ kind: "idle" });
  };

  const fallback = () => {
    // No service reachable — hand the visitor a prefilled email instead.
    const from = name.trim() || "a visitor";
    const subjectLine = subject.trim() || `Message from ${from}`;
    if (emailAddress) {
      const mailto =
        `mailto:${emailAddress}` +
        `?subject=${encodeURIComponent(subjectLine)}` +
        `&body=${encodeURIComponent(`${body.trim()}\n\n— ${from} (${email.trim()})`)}`;
      window.location.href = mailto;
    } else if (githubUrl) {
      window.open(githubUrl, "_blank", "noopener");
    }
  };

  return (
    <div className="app app-body contact-app">
      <div className="contact-layout">
        <section className="contact-form-pane">
          <header className="contact-head">
            <h1 className="contact-title">
              Message <span className="accent">Nikhil</span>
            </h1>
            <p className="muted contact-sub">
              Goes straight to my inbox — I reply to real messages.
            </p>
          </header>

          {status.kind === "sent" ? (
            <div className="contact-state">
              <div className="contact-state-icon" aria-hidden>
                <svg width="34" height="34" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M20 6 9 17l-5-5" />
                </svg>
              </div>
              <h2>Message sent</h2>
              <p className="muted">
                Thanks — it's in the inbox now. Expect a reply within a day or two.
              </p>
              <button className="btn btn-primary" onClick={reset}>
                Send another
              </button>
            </div>
          ) : (
            <form onSubmit={submit} noValidate>
              <div className="contact-grid">
                <label className="contact-field">
                  <span className="contact-label">Name</span>
                  <input
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    placeholder="Ada Lovelace"
                    autoComplete="name"
                    aria-invalid={!!errors.name}
                  />
                  {errors.name && <span className="contact-err">{errors.name}</span>}
                </label>

                <label className="contact-field">
                  <span className="contact-label">Email</span>
                  <input
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    placeholder="ada@example.com"
                    type="email"
                    autoComplete="email"
                    aria-invalid={!!errors.email}
                  />
                  {errors.email && <span className="contact-err">{errors.email}</span>}
                </label>
              </div>

              <div className="contact-field">
                <span className="contact-label">Topic</span>
                <div className="contact-chips" role="radiogroup" aria-label="Topic">
                  {TOPICS.map((t) => (
                    <button
                      key={t.id}
                      type="button"
                      role="radio"
                      aria-checked={topic === t.id}
                      className={`chip${topic === t.id ? " active" : ""}`}
                      onClick={() => setTopic(t.id)}
                    >
                      {t.label}
                    </button>
                  ))}
                </div>
              </div>

              <label className="contact-field">
                <span className="contact-label">Subject (optional)</span>
                <input
                  value={subject}
                  onChange={(e) => setSubject(e.target.value)}
                  placeholder="What's this about?"
                  aria-invalid={!!errors.subject}
                />
                {errors.subject && <span className="contact-err">{errors.subject}</span>}
              </label>

              <label className="contact-field">
                <span className="contact-label">Message</span>
                <textarea
                  value={body}
                  onChange={(e) => setBody(e.target.value)}
                  placeholder="Tell me what you're building, an idea, or just say hi…"
                  rows={6}
                  aria-invalid={!!errors.body}
                />
                <span className="contact-count muted">
                  {body.length.toLocaleString()} / {MAX_BODY.toLocaleString()}
                </span>
                {errors.body && <span className="contact-err">{errors.body}</span>}
              </label>

              {status.kind === "error" && (
                <div className="contact-errbox" role="alert">
                  <span>{status.message}</span>
                  {status.recoverable && (
                    <button
                      type="button"
                      className="contact-email-btn"
                      onClick={fallback}
                    >
                      {emailAddress ? "Use email instead →" : "Reach me on GitHub →"}
                    </button>
                  )}
                </div>
              )}

              <button
                className="btn btn-primary contact-submit"
                type="submit"
                disabled={status.kind === "sending"}
              >
                {status.kind === "sending" ? "Sending…" : "Send message"}
              </button>
            </form>
          )}
        </section>

        <aside className="contact-aside card">
          <div className="dim" style={{ marginBottom: 8 }}>WHY THIS WORKS</div>
          <ul className="contact-why">
            <li>
              <strong>No download, no sign-up.</strong> This page talks to a
              real service over HTTPS — your message lands in a database, not a
              contact form plugin.
            </li>
            <li>
              <strong>Instant.</strong> I get a push the moment you send.
            </li>
            <li>
              <strong>Prefer email?</strong>{" "}
              {emailAddress ? (
                <a href={`mailto:${emailAddress}`}>{emailAddress}</a>
              ) : (
                "Use the fallback if the service is down."
              )}
            </li>
          </ul>
          <div className="contact-hint muted">
            Built with Rust + PostgreSQL + Caddy on the AWS free tier — see{" "}
            <code className="accent">/proc</code> for what else is running.
          </div>
        </aside>
      </div>
    </div>
  );
}

function isValidEmail(email: string): boolean {
  if (!email || email.length > 254 || /\s/.test(email)) return false;
  const at = email.indexOf("@");
  if (at <= 0 || at !== email.lastIndexOf("@")) return false;
  const local = email.slice(0, at);
  const domain = email.slice(at + 1);
  if (local.length > 64 || local.startsWith(".") || local.endsWith(".") || local.includes("..")) {
    return false;
  }
  if (
    domain.length < 2 ||
    domain.length > 253 ||
    domain.startsWith(".") ||
    domain.endsWith(".") ||
    domain.includes("..")
  ) {
    return false;
  }
  return domain
    .split(".")
    .every((label) => label.length > 0 && !label.startsWith("-") && !label.endsWith("-"));
}
