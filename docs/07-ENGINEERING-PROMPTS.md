# NIKHIL//OS --- Engineering Prompts

These prompts are intended for AI coding agents. Use them incrementally.
Never ask an agent to build the entire project in one request.

------------------------------------------------------------------------

# 1. Master Architect Prompt

You are the principal software architect and systems engineer for
NIKHIL//OS.

NIKHIL//OS is an AI-native personal computing environment that functions
as an interactive portfolio and serious systems-engineering project.

It must NOT become a conventional portfolio website.

Engineering philosophy:

-   Arch Linux-inspired
-   Unix-like
-   modular
-   transparent
-   CLI-first
-   keyboard-first
-   observable
-   minimal
-   composable
-   explainable

Use a Rust shared core.

Targets:

-   WebAssembly for browser
-   Tauri for desktop
-   future x86_64 bare-metal kernel

Do not implement superficial system behavior.

Every major feature must have a real underlying subsystem.

Do not let React components directly mutate OS state.

Use service boundaries and typed interfaces.

AI must never have unrestricted system access.

Before implementing a major subsystem:

1.  explain responsibilities
2.  define interface
3.  define dependencies
4.  define failure modes
5.  define tests
6.  define observability
7.  implement
8.  document

Use ADRs for significant decisions.

Prefer a modular monolith over premature microservices.

------------------------------------------------------------------------

# 2. OS Core Prompt

Implement the shared NIKHIL//OS core in Rust.

Modules:

process scheduler memory filesystem permissions ipc syscall service
package event logging

Implement:

-   process lifecycle
-   PID
-   scheduler
-   virtual filesystem
-   permissions
-   users/groups
-   typed IPC
-   syscall interface
-   event bus
-   service lifecycle
-   structured logs

Start with Round Robin scheduling.

Design interfaces for Priority and MLFQ.

The core must run without the UI.

Write unit and integration tests.

Expose read-only telemetry.

Do not fake state.

------------------------------------------------------------------------

# 3. Shell Prompt

Implement a Unix-like shell.

Architecture:

Input → Lexer → Parser → AST → Executor → Services/syscalls

Support:

-   arguments
-   pipes
-   redirection
-   environment variables
-   aliases
-   history
-   autocomplete
-   exit codes

Commands:

ls cd pwd cat mkdir rm mv cp grep find ps top kill free df mount uname
pkgctl service ai graph career

`ps | grep ai` must execute through a real pipeline.

Do not implement commands in React.

------------------------------------------------------------------------

# 4. AI Core Prompt

Implement AI as a controlled OS service.

Pipeline:

Intent → Planner → Retriever → Tool selector → Schema validation →
Permission validation → OS service → Result → Explanation

Tools must be typed and allowlisted.

Example:

OPEN_APPLICATION SEARCH_PROJECTS SEARCH_EVIDENCE READ_PROFILE
QUERY_GRAPH GET_SYSTEM_STATUS

Never allow arbitrary shell commands or filesystem operations.

Log all AI actions.

Add tests for prompt injection and malicious tool arguments.

------------------------------------------------------------------------

# 5. Knowledge/RAG Prompt

Build the knowledge engine.

Entities:

Person Skill Technology Project Experience Contribution Achievement
Certification Organization Claim Evidence Event

Implement:

-   canonical schema
-   graph relationships
-   document ingestion
-   chunking
-   embeddings
-   vector retrieval
-   hybrid retrieval
-   reranking
-   evidence linking

Every important generated claim must be traceable to evidence.

Evaluate:

-   Recall@K
-   Precision@K
-   MRR
-   latency
-   unsupported claim rate

------------------------------------------------------------------------

# 6. Visitor ML Prompt

Implement session-level visitor interest modeling.

Signals:

-   application opened
-   project explored
-   search query
-   graph node
-   explicit role

Interest vector:

backend AI ML systems mobile open-source frontend databases

Start with an interpretable deterministic model.

Later compare:

-   semantic baseline
-   rule-based ranking
-   learned ranking

Do not collect unnecessary identity information.

------------------------------------------------------------------------

# 7. Developer Mode Prompt

Build Developer Mode.

Shortcut:

Ctrl + Shift + D

Panels:

-   Processes
-   Memory
-   Filesystem
-   Services
-   IPC
-   Events
-   AI Trace
-   Retrieval Trace
-   Graph
-   Network
-   Logs

Values must come from real runtime state.

Do not fake CPU/memory values.

AI trace:

request → intent → retrieval → tools → permissions → actions → result →
latency

------------------------------------------------------------------------

# 8. UI Prompt

Build a premium modern developer interface.

Visual direction:

Linear + Raycast + modern Linux workstation + AI research tool.

Avoid:

-   generic portfolio templates
-   excessive glassmorphism
-   cyberpunk overload
-   neon everywhere
-   macOS cloning
-   fake terminal animations
-   excessive gradients

Use:

-   dark-first
-   restrained accent
-   Inter/Geist
-   JetBrains Mono
-   subtle borders
-   high information density
-   keyboard navigation
-   minimal motion
-   responsive layouts

The UI should make the underlying architecture feel credible.

------------------------------------------------------------------------

# 9. Architecture Viewer Prompt

Build an interactive architecture viewer.

Do not use static diagrams.

Each node must expose:

-   responsibility
-   inputs
-   outputs
-   technology
-   dependencies
-   rationale
-   failure modes

Support:

-   zoom
-   pan
-   hover
-   selection
-   search
-   relationship highlighting

The graph should be generated from structured architecture data where
practical.

------------------------------------------------------------------------

# 10. Recruiter Mode Prompt

Build a concise recruiter-facing application.

Show:

-   name
-   role
-   strongest areas
-   experience
-   selected projects
-   evidence
-   resume
-   contact

Target:

A recruiter should understand the candidate in under 60 seconds.

Use minimal visual density and strong hierarchy.

------------------------------------------------------------------------

# 11. Job Analyzer Prompt

Input a job description.

Pipeline:

JD → extraction → normalization → semantic matching → evidence retrieval
→ gap analysis → match score

Show:

-   strong matches
-   partial matches
-   gaps
-   evidence

Do not claim exact hiring probability.

------------------------------------------------------------------------

# 12. Career Simulator Prompt

Build a scenario simulator.

Inputs:

-   target role
-   specialization
-   desired technologies

Output:

-   strengths
-   gaps
-   evidence
-   possible projects
-   learning recommendations

Label output as simulation.

Do not present speculative career predictions as facts.

------------------------------------------------------------------------

# 13. Testing Prompt

For every subsystem:

-   unit tests
-   integration tests
-   property tests where useful
-   deterministic simulation tests

For AI:

-   retrieval evaluation
-   tool execution tests
-   prompt injection tests
-   malformed output tests
-   unsupported claim tests

For the desktop:

-   end-to-end boot
-   terminal
-   application launch
-   AI action

No feature is complete without tests.

------------------------------------------------------------------------

# 14. Code Review Prompt

Review the implementation as a senior systems engineer.

Look for:

-   circular dependencies
-   leaked abstractions
-   UI-to-core coupling
-   unsafe AI access
-   unnecessary microservices
-   global mutable state
-   poor error handling
-   weak types
-   missing tests
-   unobservable behavior
-   security issues
-   performance bottlenecks

Do not rewrite code merely for stylistic preference.

Return:

1.  critical issues
2.  architectural issues
3.  maintainability issues
4.  security issues
5.  performance issues
6.  recommended changes
