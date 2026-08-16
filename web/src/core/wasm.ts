// Typed, defensive wrapper around the wasm-bindgen core.
//
// Every call returns plain data: JSON strings from the core are parsed here
// and coerced with fallbacks, so a transient core hiccup never crashes the UI.
// The web app never touches the pkg directly.

import * as core from "../../pkg/nikhil_os_core";
import type {
  DirEntry,
  FileStat,
  Profile,
  Snapshot,
} from "./types";

export function init(): void {
  core.init();
}

export function boot(): string {
  return core.boot();
}

export function tick(): void {
  core.tick();
}

export function runCommand(input: string): string {
  return core.run_command(input);
}

export function prompt(): string {
  return core.prompt();
}

export function autocomplete(prefix: string): string[] {
  return core.autocomplete(prefix);
}

export function snapshot(): Snapshot {
  return parse<Snapshot>(core.snapshot());
}

export function listDir(path: string): DirEntry[] {
  return parse<DirEntry[]>(core.list_dir(path));
}

export function statPath(path: string): FileStat | null {
  try {
    return JSON.parse(core.stat_path(path)) as FileStat;
  } catch {
    return null;
  }
}

export function readFile(path: string): string {
  return core.read_file(path);
}

export function profile(): Profile {
  return parse<Profile>(core.profile());
}

export function events(n: number): unknown[] {
  try {
    return JSON.parse(core.events(n)) as unknown[];
  } catch {
    return [];
  }
}

export function version(): string {
  return core.version();
}

function parse<T>(json: string): T {
  try {
    return JSON.parse(json) as T;
  } catch {
    // Never let a parse failure take down the desktop.
    return {} as T;
  }
}
