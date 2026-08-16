# Kernel Edition (placeholder)

> **Status: planned.** The ultimate extension of the architecture: boot `nikhil-os-core`
> as a **bare-metal kernel** on real or emulated hardware.

## What it will do

- Cross-compile the core's process/scheduler/memory/IPC layers to a freestanding
  target (`x86_64-unknown-none`), replacing the simulated clock and allocator with
  real ones.
- Boot over a serial console / framebuffer using the `multiboot2` protocol (or run on
  QEMU for development).
- Eventually drive a minimal shell on the UART — `nish` running directly on hardware.

## Why it is in the repo now

The core is designed so the OS subsystems are not UI props: the same `ProcessManager`,
`Scheduler`, and `MemoryManager` that power the browser desktop are the kernel's engine.
This directory documents the path to hardware and keeps the ambition visible.

## Status / roadmap

- [ ] Freestanding target + panic handler
- [ ] Boot protocol (QEMU + multiboot2)
- [ ] Serial console + `nish` on UART

See [`docs/06-ROADMAP.md`](../docs/06-ROADMAP.md) for phase details.
