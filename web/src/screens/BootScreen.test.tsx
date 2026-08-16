import { act, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { BootScreen } from "./BootScreen";

const LINES = [
  "[ OK ] nikhil-os-core 0.1.0 (wasm32-unknown-unknown)",
  "[ OK ] scheduler: round-robin online",
  "[ OK ] desktop environment ready",
];

const REVEAL_MS = 150;
const SETTLE_MS = 350;

describe("BootScreen", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it("reveals lines progressively and fires onDone when finished", () => {
    const onDone = vi.fn();
    render(<BootScreen lines={LINES} onDone={onDone} />);

    expect(screen.getByText("booting…")).toBeInTheDocument();
    expect(onDone).not.toHaveBeenCalled();

    // Phase 1: all lines revealed. React flushes at the act boundary, so the
    // settle timer is only scheduled once the reveal batch is committed.
    act(() => vi.advanceTimersByTime(REVEAL_MS * LINES.length));
    expect(screen.getByText(/desktop environment ready/)).toBeInTheDocument();
    expect(onDone).not.toHaveBeenCalled();

    // Phase 2: the settle timeout fires and hands off to the desktop.
    act(() => vi.advanceTimersByTime(SETTLE_MS));
    expect(onDone).toHaveBeenCalledTimes(1);
  });

  it("never calls onDone with no boot lines", () => {
    const onDone = vi.fn();
    render(<BootScreen lines={[]} onDone={onDone} />);
    act(() => vi.advanceTimersByTime(5_000));
    expect(onDone).not.toHaveBeenCalled();
  });
});
