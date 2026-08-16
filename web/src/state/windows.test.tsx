import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { openWindow, useWindows } from "./windows";

// `registry.tsx` imports every app, each of which reaches the WASM core.
// Stub the core here; the wrapper itself is exercised in wasm.test.ts.
vi.mock("../core/wasm", () => ({
  init: () => {},
  boot: () => "",
  tick: () => {},
  runCommand: () => "",
  prompt: () => "$ ",
  autocomplete: () => [],
  snapshot: () => ({}),
  listDir: () => [],
  statPath: () => null,
  readFile: () => "",
  profile: () => ({}),
  events: () => [],
  version: () => "0.1.0-test",
}));

function setup() {
  const hook = renderHook(() => useWindows());
  const read = () => hook.result.current;
  return { hook, read };
}

describe("useWindows", () => {
  it("opens a window, cascades it, and marks it active", () => {
    const { read } = setup();
    act(() => read().open("terminal", "Terminal"));
    const wm = read();
    expect(wm.windows).toHaveLength(1);
    expect(wm.windows[0].app).toBe("terminal");
    expect(wm.windows[0].title).toBe("Terminal");
    expect(wm.activeId).toBe(wm.windows[0].id);
  });

  it("focuses a background window and raises its z-order", () => {
    const { read } = setup();
    act(() => {
      read().open("files", "Files");
      read().open("projects", "Projects");
    });
    const first = read().windows.find((w) => w.app === "files")!;
    const zBefore = first.z;
    act(() => read().focus(first.id));
    const raised = read().windows.find((w) => w.app === "files")!;
    expect(read().activeId).toBe(first.id);
    expect(raised.z).toBeGreaterThan(zBefore);
  });

  it("closes a window and re-activates the top remaining one", () => {
    const { read } = setup();
    act(() => {
      read().open("terminal", "Terminal");
      read().open("files", "Files");
    });
    const terminal = read().windows.find((w) => w.app === "terminal")!;
    act(() => read().close(terminal.id));
    const after = read();
    expect(after.windows.map((w) => w.app)).toEqual(["files"]);
    expect(after.activeId).toBe(after.windows[0].id);
  });

  it("minimizes and restores without losing position", () => {
    const { read } = setup();
    act(() => read().open("terminal", "Terminal"));
    const id = read().windows[0].id;
    act(() => read().minimize(id));
    expect(read().windows[0].minimized).toBe(true);
    expect(read().activeId).toBeNull();
    act(() => read().restore(id));
    expect(read().windows[0].minimized).toBe(false);
    expect(read().activeId).toBe(id);
  });

  it("toggles maximize", () => {
    const { read } = setup();
    act(() => read().open("terminal", "Terminal"));
    const id = read().windows[0].id;
    act(() => read().toggleMaximize(id));
    expect(read().windows[0].maximized).toBe(true);
  });

  it("moves and resizes windows", () => {
    const { read } = setup();
    act(() => read().open("terminal", "Terminal"));
    const id = read().windows[0].id;
    act(() => {
      read().move(id, 120, 80);
      read().resize(id, 640, 480);
    });
    expect(read().windows[0]).toMatchObject({
      x: 120,
      y: 80,
      w: 640,
      h: 480,
    });
  });

  it("closeAll empties the workspace", () => {
    const { read } = setup();
    act(() => {
      read().open("terminal", "Terminal");
      read().open("files", "Files");
    });
    act(() => read().closeAll());
    const after = read();
    expect(after.windows).toHaveLength(0);
    expect(after.activeId).toBeNull();
  });
});

describe("openWindow", () => {
  it("reuses an existing window instead of stacking a duplicate", () => {
    const { read } = setup();
    act(() => read().open("terminal", "Terminal"));
    act(() => openWindow(read(), "terminal", "Terminal"));
    const after = read();
    expect(after.windows).toHaveLength(1);
    expect(after.activeId).toBe(after.windows[0].id);
  });

  it("opens a new window when the app is not running", () => {
    const { read } = setup();
    act(() => openWindow(read(), "projects", "Projects"));
    expect(read().windows).toHaveLength(1);
    expect(read().windows[0].app).toBe("projects");
  });
});
