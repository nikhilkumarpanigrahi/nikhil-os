// Typed, defensive HTTP client for the NIKHIL//OS backend.
//
// The OS is offline-first: every call here can fail (no network, backend down,
// CORS) and must degrade gracefully. Callers receive structured results, never
// unhandled exceptions. The WASM core stays the source of truth for knowledge;
// this client is only for live service surfaces (contact).
//
// Base URL resolution:
//   - VITE_API_URL set (prod)  → https://api.nikhil.is-a.dev
//   - VITE_API_URL unset (dev) → "/api", which the Vite dev server proxies to
//     the local backend (see vite.config.ts). On the static host this 404s and
//     callers fall back to mailto — by design.

export interface ContactInput {
  name: string;
  email: string;
  subject?: string;
  topic?: string;
  body: string;
  /** Honeypot. Must stay empty — bots that fill it get a fake 201. */
  website?: string;
}

export type ContactResult =
  | { ok: true; id: string; status: string }
  | {
      ok: false;
      code: string;
      message: string;
      /** true → retrying later may succeed (rate-limit, server error, offline). */
      recoverable: boolean;
    };

const API_ORIGIN = (import.meta.env.VITE_API_URL ?? "").trim().replace(/\/+$/, "");
const BASE = API_ORIGIN ? `${API_ORIGIN}/api` : "/api";

export async function submitContact(input: ContactInput): Promise<ContactResult> {
  try {
    const resp = await fetch(`${BASE}/v1/contact`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        name: input.name,
        email: input.email,
        subject: input.subject ?? "",
        topic: input.topic ?? "general",
        body: input.body,
        website: input.website ?? "",
      }),
    });

    const data = await parseBody(resp);

    if (resp.ok) {
      return {
        ok: true,
        id: typeof data.id === "string" ? data.id : "",
        status: typeof data.status === "string" ? data.status : "new",
      };
    }

    const err = (data as { error?: { code?: string; message?: string } }).error ?? {};
    return {
      ok: false,
      code: err.code ?? `http_${resp.status}`,
      message: err.message ?? `Request failed (${resp.status})`,
      recoverable: resp.status === 429 || resp.status >= 500,
    };
  } catch {
    // fetch threw: offline, DNS, or TLS failure. The caller should offer a
    // mailto fallback rather than block the visitor.
    return {
      ok: false,
      code: "network",
      message: "The message service is unreachable right now.",
      recoverable: true,
    };
  }
}

async function parseBody(resp: Response): Promise<Record<string, unknown>> {
  try {
    return (await resp.json()) as Record<string, unknown>;
  } catch {
    return {};
  }
}
