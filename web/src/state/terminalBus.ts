// Bridges the command palette to the Terminal app: if a Terminal window is
// open, push the command straight in; otherwise stash it for the next mount.

let pending: string | null = null;
let handler: ((cmd: string) => void) | null = null;

export function requestCommand(cmd: string) {
  if (handler) {
    handler(cmd);
  } else {
    pending = cmd;
  }
}

export function takePending(): string | null {
  const c = pending;
  pending = null;
  return c;
}

export function registerTerminal(fn: ((cmd: string) => void) | null) {
  handler = fn;
}
