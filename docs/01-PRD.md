# NIKHIL//OS --- Product Requirements Document

**Version:** 1.0\
**Status:** Architecture / Pre-Implementation\
**Product:** NIKHIL//OS\
**Tagline:** *An AI-native personal computing environment.*

## 1. Vision

NIKHIL//OS is not a conventional portfolio website. It is a personal
computing environment that lets visitors explore an engineer's work
through a Unix-inspired operating-system experience.

The system combines:

-   OS and systems concepts
-   Arch Linux-inspired engineering philosophy
-   Rust
-   WebAssembly
-   native desktop packaging
-   AI agents
-   NLP
-   RAG
-   embeddings
-   vector search
-   knowledge graphs
-   recommendation systems
-   evidence-backed retrieval
-   developer observability
-   modern product design

The principle is:

> **The visitor should use the portfolio rather than browse the
> portfolio.**

## 2. Product Editions

### Web Edition

The primary public experience.

-   Browser-based
-   Zero installation
-   Modern desktop/workspace UI
-   Shared OS core compiled to WebAssembly
-   Deep-linkable applications
-   Optional session state
-   No mandatory account

### Desktop Edition

Native application using the same core.

Targets:

-   macOS
-   Windows
-   Linux

Recommended shell:

-   Tauri

### Kernel Edition

Long-term bare-metal systems project.

Initial target:

-   x86_64
-   QEMU
-   bootable ISO

The kernel edition demonstrates genuine OS development and does not need
to reproduce the entire web desktop.

## 3. Target Users

### Recruiter

Needs to understand:

-   who Nikhil is
-   strongest technical areas
-   experience
-   evidence
-   resume
-   contact

within 30--60 seconds.

### Software Engineer

Needs:

-   architecture
-   code
-   system behavior
-   technical decisions
-   developer mode

### AI/ML Engineer

Needs:

-   retrieval pipeline
-   embeddings
-   knowledge graph
-   ranking
-   evaluation
-   AI tool execution

### Systems Engineer / Student

Needs:

-   process model
-   scheduler
-   virtual filesystem
-   IPC
-   shell
-   package manager
-   observability

## 4. Design Philosophy

Inspired by Arch Linux philosophy, not by copying its interface.

Principles:

1.  Minimal base
2.  Modular components
3.  User control
4.  Transparent internals
5.  CLI-first capabilities
6.  Keyboard-first interaction
7.  Configuration over hidden magic
8.  Small composable services
9.  Observable state
10. Documentation-first development
11. Security by default
12. Explainable AI

## 5. Core Capabilities

### OS Core

-   process abstraction
-   PID system
-   scheduler
-   process states
-   memory simulation
-   virtual filesystem
-   permissions
-   users/groups
-   IPC
-   system calls
-   event bus
-   service manager
-   package manager
-   shell
-   telemetry

### AI Core

-   intent detection
-   planner
-   RAG
-   embeddings
-   tool calling
-   permission validation
-   action execution
-   explanation
-   session memory

### ML

-   visitor intent classification
-   interest modeling
-   semantic project ranking
-   job matching
-   recommendation
-   retrieval evaluation

### Knowledge Core

Entities:

-   Person
-   Skill
-   Technology
-   Project
-   Experience
-   Contribution
-   Achievement
-   Certification
-   Organization
-   Claim
-   Evidence
-   Event

Relationships:

-   HAS_SKILL
-   USES
-   BUILT
-   WORKED_AT
-   CONTRIBUTED_TO
-   DEMONSTRATES
-   SUPPORTED_BY
-   RELATED_TO

## 6. Applications

Initial applications:

1.  Terminal
2.  Files
3.  Projects
4.  Resume
5.  Experience
6.  Knowledge Graph
7.  AI Core
8.  Recruiter Mode
9.  Job Analyzer
10. Career Debugger
11. Career Simulator
12. AI Lab
13. System Monitor
14. Developer Console
15. Settings
16. Package Manager
17. Network Monitor

## 7. Signature Experiences

### Recruiter Mode

A concise candidate view with:

-   profile
-   core strengths
-   selected evidence
-   resume
-   contact

### AI Command Center

A Raycast-like command interface using:

`Cmd/Ctrl + K`

### Career Graph

Interactive graph connecting:

skills → projects → technologies → experience → evidence

### Evidence Engine

Every important claim should be traceable to supporting evidence.

### Developer Mode

A technical inspection environment showing:

-   processes
-   memory
-   filesystem
-   services
-   IPC
-   AI traces
-   retrieval traces
-   graph queries
-   logs
-   performance

### AI Lab

Interactive demonstrations of:

-   semantic search
-   RAG
-   intent classification
-   project ranking
-   visitor modeling
-   job matching
-   knowledge graph
-   career simulation
-   AI agents

## 8. Non-Goals

Do not:

-   clone Arch Linux
-   clone macOS
-   fake system telemetry
-   make an LLM the entire backend
-   give AI unrestricted OS access
-   add technologies only for resume keywords
-   collect unnecessary visitor PII
-   use excessive cyberpunk decoration
-   pretend a simulation is a real kernel

## 9. Success Criteria

The system is successful when:

-   a recruiter understands the candidate within 30 seconds
-   a recruiter can explore the portfolio for 5--10 minutes
-   engineers can inspect the architecture
-   AI/ML engineers can inspect the ML pipeline
-   systems engineers can inspect OS behavior
-   the Web Edition requires no installation
-   the Desktop Edition can be installed
-   the core is covered by tests
-   important AI claims have evidence
-   internal behavior is observable
-   the repository is understandable
