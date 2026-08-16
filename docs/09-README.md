# NIKHIL//OS

> **An AI-native personal computing environment.**

NIKHIL//OS turns a personal portfolio into an interactive Unix-inspired
computing environment.

It combines systems programming, AI/ML, knowledge graphs, semantic
retrieval, developer tooling, and modern product design.

## Why?

A traditional portfolio asks visitors to scroll.

NIKHIL//OS lets visitors use the system.

## Core Ideas

-   Rust shared OS runtime
-   WebAssembly Web Edition
-   Tauri Desktop Edition
-   future x86_64 Kernel Edition
-   Unix-inspired shell
-   virtual filesystem
-   process manager
-   scheduler
-   IPC
-   service manager
-   package manager
-   AI runtime
-   RAG
-   vector search
-   knowledge graph
-   recommendation engine
-   evidence engine
-   recruiter mode
-   developer mode

## Editions

### Web

No installation required.

### Desktop

Native application for macOS, Windows, and Linux.

### Kernel

Future bootable x86_64 systems project.

## Architecture

``` text
Presentation
    ↓
Application Runtime
    ↓
OS Services
    ↓
Rust Core

AI Runtime ↔ OS Services

Knowledge Core ↔ AI Runtime
```

## Repository

``` text
core/
ai/
knowledge/
apps/
web/
desktop/
backend/
kernel/
docs/
tests/
```

## Design Philosophy

Inspired by Arch Linux principles:

-   minimal
-   modular
-   transparent
-   user-controlled
-   CLI-first
-   documented
-   observable

The visual design is intentionally modern rather than a direct Linux
desktop clone.

## Security

The AI cannot directly execute arbitrary commands.

All AI actions use:

``` text
Intent
→ Tool
→ Schema validation
→ Permission validation
→ OS service
→ Audit
```

## Status

Pre-implementation / architecture phase.

## Roadmap

1.  OS core
2.  shell
3.  desktop
4.  knowledge core
5.  AI core
6.  ML
7.  applications
8.  security and hardening
9.  desktop packaging
10. kernel edition

## Philosophy

The project should not optimize for the number of technologies used.

It should optimize for:

-   clear architecture
-   real implementation
-   explainability
-   observability
-   security
-   excellent UX
-   technical depth

> **The UI is the surface. The system underneath is the project.**
