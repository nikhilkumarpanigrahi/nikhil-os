# NIKHIL//OS --- Technical Architecture

## 1. Architectural Principle

NIKHIL//OS should behave like a real software system rather than a
collection of UI screens.

The key boundary is:

``` text
Presentation
    ↓
Application Runtime
    ↓
OS Services
    ↓
OS Core
```

AI is an operating-system service, not a privileged replacement for the
OS.

## 2. High-Level Architecture

``` text
                         NIKHIL//OS
                              |
          +-------------------+-------------------+
          |                   |                   |
      WEB EDITION        DESKTOP EDITION      KERNEL EDITION
       Browser              Tauri               Bare Metal
          |                   |                   |
          +----------+--------+-------------------+
                     |
                SHARED CORE
                     |
       +-------------+-------------+
       |             |             |
   OS Runtime     AI Runtime   Knowledge Core
       |             |             |
   Processes       Intent       Career Graph
   Scheduler       RAG          Evidence
   Memory          Planner      Embeddings
   VFS             Actions      Vector Search
   IPC             Tools        Recommendations
   Permissions     Validation   Timeline
       |             |             |
       +-------------+-------------+
                     |
              SYSTEM SERVICES
                     |
       +-------------+-------------+
       |             |             |
     Shell        Package       Service
                  Manager       Manager
       |             |             |
       +-------------+-------------+
                     |
                Applications
```

## 3. Shared Rust Core

Rust is the preferred language for the shared runtime because it
provides:

-   strong type safety
-   predictable performance
-   explicit ownership
-   WASM support
-   native integration
-   good fit for systems programming

The core should be usable without React.

## 4. Web Architecture

``` text
React / TypeScript
        |
   UI Adapter
        |
      WASM
        |
   Rust Core
        |
   OS Services
        |
  Backend APIs
```

Use Web Workers where long-running simulation tasks should not block the
UI.

## 5. Desktop Architecture

``` text
React / TypeScript
        |
      Tauri
        |
   Rust Core
        |
Native OS Integration
```

The desktop application should reuse the same domain and OS logic as the
Web Edition.

## 6. Backend Architecture

Recommended:

-   Python
-   FastAPI
-   PostgreSQL
-   vector search
-   optional Redis for transient state

Backend responsibilities:

-   AI orchestration
-   embeddings
-   retrieval
-   knowledge graph queries
-   job analysis
-   model gateway
-   analytics
-   secure tool execution

Avoid unnecessary microservices.

Use a modular monolith first.

## 7. OS Core Modules

``` text
core/
├── process/
├── scheduler/
├── memory/
├── filesystem/
├── permissions/
├── ipc/
├── syscall/
├── service/
├── package/
├── event/
└── logging/
```

## 8. Process Model

States:

``` text
NEW
 ↓
READY
 ↓
RUNNING
 ↓
WAITING
 ↓
READY
 ↓
TERMINATED
```

Each process has:

-   PID
-   parent PID
-   state
-   priority
-   simulated CPU usage
-   memory allocation
-   capabilities
-   timestamps

## 9. Scheduler

Start with Round Robin.

Scheduler interface should allow later implementations:

-   Priority
-   Multilevel Feedback Queue

Expose scheduler telemetry.

## 10. Virtual Filesystem

``` text
/
├── bin
├── dev
├── etc
├── home
├── opt
├── proc
├── sys
├── tmp
├── usr
└── var
```

`/proc` and `/sys` are virtual, dynamic filesystems exposing actual
runtime state.

## 11. System Calls

Internal syscall layer:

``` text
open()
read()
write()
close()
stat()
mkdir()
spawn()
exec()
kill()
send()
receive()
subscribe()
```

Applications must use service/syscall interfaces rather than mutating
kernel state directly.

## 12. IPC

Use typed messages.

Example:

``` text
Terminal
  ↓ IPC
Shell Service
  ↓ IPC
Project Service
  ↓ IPC
Knowledge Service
```

AI uses the same controlled service boundaries.

## 13. Service Manager

Boot sequence:

``` text
Boot
 ↓
Kernel initialization
 ↓
Filesystem mount
 ↓
Init
 ↓
Service manager
 ↓
Network
 ↓
AI Core
 ↓
Knowledge Core
 ↓
Window Manager
 ↓
Desktop
```

Services expose:

-   status
-   dependencies
-   uptime
-   errors
-   restart behavior

## 14. Package Manager

Working name: `pkgctl`

Commands:

``` bash
pkgctl search
pkgctl install
pkgctl remove
pkgctl update
pkgctl upgrade
pkgctl info
pkgctl list
```

Package manifest:

``` yaml
name:
version:
description:
dependencies:
permissions:
entrypoint:
```

## 15. Shell

Architecture:

``` text
Input
 ↓
Lexer
 ↓
Parser
 ↓
AST
 ↓
Command Executor
 ↓
Syscalls / Services
```

Support:

-   arguments
-   pipes
-   redirection
-   environment variables
-   aliases
-   history
-   autocomplete
-   exit codes

## 16. AI Runtime

``` text
User Intent
 ↓
Intent Parser
 ↓
Planner
 ↓
Retriever
 ↓
Tool Selector
 ↓
Schema Validation
 ↓
Permission Validation
 ↓
OS Action
 ↓
Result
 ↓
Explanation
```

The model must never receive arbitrary filesystem, shell, database, or
JavaScript access.

## 17. AI Tool Contract

Every tool defines:

-   name
-   input schema
-   output schema
-   permissions
-   risk level
-   timeout
-   audit metadata

Example:

``` json
{
  "action": "open_app",
  "app": "projects",
  "query": "backend"
}
```

The OS validates this before execution.

## 18. Knowledge Architecture

Use PostgreSQL as the canonical source of structured truth.

Represent graph relationships relationally initially.

Use vector storage for semantic retrieval.

Use object storage for documents/assets.

Only add a dedicated graph database if real graph workloads justify it.

## 19. RAG Pipeline

``` text
Documents
 ↓
Parsing
 ↓
Chunking
 ↓
Metadata
 ↓
Embeddings
 ↓
Vector Storage
 ↓
Hybrid Retrieval
 ↓
Reranking
 ↓
Evidence Validation
 ↓
Response
```

## 20. Visitor Intelligence

Maintain a session-level interest vector.

Example:

``` text
backend       0.91
AI            0.84
systems       0.76
open-source   0.63
frontend      0.18
```

Do not collect unnecessary identity data.

## 21. Recommendation

Rank using:

``` text
semantic similarity
+ visitor interest
+ evidence strength
+ relevance
+ recency
```

Every recommendation should have an explanation.

## 22. Security Architecture

AI actions require:

1.  intent
2.  schema validation
3.  permission validation
4.  allowlisted tool
5.  audit logging
6.  execution
7.  result validation

Threats to test:

-   prompt injection
-   malicious tool arguments
-   privilege escalation
-   arbitrary command execution
-   data exfiltration
-   tool loops
-   malformed model output

## 23. Observability

Every subsystem emits structured events.

Example:

``` text
[INFO] process-manager: spawned pid=17
[INFO] ai-core: intent=project_search
[INFO] retriever: candidates=18
[INFO] reranker: selected=3
[INFO] action: open_app(projects)
```

Developer Mode consumes these events.

## 24. Repository Structure

``` text
nikhil-os/
├── core/
│   ├── process/
│   ├── scheduler/
│   ├── memory/
│   ├── filesystem/
│   ├── permissions/
│   ├── ipc/
│   ├── syscall/
│   ├── service/
│   ├── package/
│   ├── event/
│   └── logging/
├── ai/
│   ├── intent/
│   ├── retrieval/
│   ├── embeddings/
│   ├── planner/
│   ├── tools/
│   └── agents/
├── knowledge/
│   ├── schema/
│   ├── graph/
│   ├── evidence/
│   └── ingestion/
├── apps/
│   ├── terminal/
│   ├── files/
│   ├── projects/
│   ├── resume/
│   ├── recruiter/
│   ├── lab/
│   └── career/
├── web/
├── desktop/
├── backend/
├── kernel/
├── docs/
└── tests/
```
