import { afterEach, describe, expect, it, vi } from "vitest";
import { submitContact } from "./api";

// The api client is pure fetch — no WASM involved — so we exercise it directly
// with a mocked global fetch: success, validation/rate-limit failures, and the
// offline path that drives the Contact app's mailto fallback.

function mockFetchOnce(status: number, body: unknown): void {
  vi.stubGlobal(
    "fetch",
    vi.fn().mockResolvedValue(
      new Response(JSON.stringify(body), {
        status,
        headers: { "Content-Type": "application/json" },
      }),
    ),
  );
}

afterEach(() => {
  vi.unstubAllGlobals();
});

const input = {
  name: "Ada Lovelace",
  email: "ada@example.com",
  topic: "collaboration",
  body: "A real, sufficiently long message from a visitor.",
};

describe("submitContact", () => {
  it("POSTs the message and returns the stored id on 201", async () => {
    mockFetchOnce(201, { id: "abc-123", status: "new" });
    const result = await submitContact(input);
    expect(result).toEqual({ ok: true, id: "abc-123", status: "new" });

    const [url, init] = vi.mocked(fetch).mock.calls[0]!;
    expect(String(url)).toMatch(/\/api\/v1\/contact$/);
    expect(init!.method).toBe("POST");
    expect(JSON.parse(String(init!.body))).toMatchObject({
      name: "Ada Lovelace",
      email: "ada@example.com",
      topic: "collaboration",
    });
  });

  it("normalizes the topic to 'general' and subject to '' when omitted", async () => {
    mockFetchOnce(201, { id: "x", status: "new" });
    await submitContact({ name: "N", email: "n@example.com", body: "Hello hello hello hello" });
    const [, init] = vi.mocked(fetch).mock.calls[0]!;
    expect(JSON.parse(String(init!.body))).toMatchObject({
      topic: "general",
      subject: "",
      website: "",
    });
  });

  it("surfaces the server's validation error on 422", async () => {
    mockFetchOnce(422, {
      error: { code: "validation_error", message: "a valid email address is required" },
    });
    const result = await submitContact(input);
    expect(result).toEqual({
      ok: false,
      code: "validation_error",
      message: "a valid email address is required",
      recoverable: false,
    });
  });

  it("marks rate-limit responses as recoverable", async () => {
    mockFetchOnce(429, {
      error: { code: "rate_limited", message: "too many requests, try again shortly" },
    });
    const result = await submitContact(input);
    expect(result.ok).toBe(false);
    if (!result.ok) expect(result.recoverable).toBe(true);
  });

  it("degrades to a network error (offline / backend down)", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockRejectedValue(new TypeError("Failed to fetch")),
    );
    const result = await submitContact(input);
    expect(result).toEqual({
      ok: false,
      code: "network",
      message: "The message service is unreachable right now.",
      recoverable: true,
    });
  });

  it("tolerates a non-JSON error body", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(new Response("gateway timeout", { status: 502 })),
    );
    const result = await submitContact(input);
    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.code).toBe("http_502");
      expect(result.recoverable).toBe(true);
    }
  });
});
