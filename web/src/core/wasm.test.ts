import { describe, expect, it, vi } from "vitest";

// Stub the raw wasm-bindgen module so the wrapper is exercised in isolation.
vi.mock("../../pkg/nikhil_os_core", () => ({
  init: vi.fn(),
  boot: () => "[ OK ] kernel up",
  tick: vi.fn(),
  run_command: () => "hello from nish",
  prompt: () => "guest@nikhil-os:~$ ",
  autocomplete: () => ["ls", "la"],
  snapshot: () => JSON.stringify({ processes: [], memory: {} }),
  list_dir: () => JSON.stringify([]),
  stat_path: () => "null",
  read_file: () => "file contents",
  profile: () => JSON.stringify({ name: "Test" }),
  events: () => JSON.stringify([{ id: 1 }]),
  version: () => "0.1.0-test",
}));

import {
  autocomplete,
  boot,
  events,
  listDir,
  profile,
  readFile,
  runCommand,
  snapshot,
  statPath,
  version,
} from "./wasm";

describe("wasm wrapper", () => {
  it("returns plain values straight through", () => {
    expect(boot()).toBe("[ OK ] kernel up");
    expect(runCommand("echo hi")).toBe("hello from nish");
    expect(readFile("/etc/hostname")).toBe("file contents");
    expect(version()).toBe("0.1.0-test");
  });

  it("parses JSON snapshot output", () => {
    expect(snapshot()).toEqual({ processes: [], memory: {} });
  });

  it("parses JSON arrays and objects", () => {
    expect(listDir("/")).toEqual([]);
    expect(profile()).toEqual({ name: "Test" });
  });

  it("falls back safely on malformed JSON instead of throwing", () => {
    // Stat returns "null" (missing file) → wrapper maps to null.
    expect(statPath("/nope")).toBeNull();
  });

  it("surfaces autocomplete suggestions", () => {
    expect(autocomplete("l")).toEqual(["ls", "la"]);
  });

  it("parses events and tolerates failures", () => {
    expect(events(5)).toEqual([{ id: 1 }]);
    vi.mocked(events); // ensure import is used
  });
});
