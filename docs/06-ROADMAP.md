# NIKHIL//OS --- Development Roadmap

## Overall Strategy

Do not wait months for a perfect release.

Build a useful Web Alpha early and progressively deepen the system.

## Phase 0 --- Architecture

**Duration:** 1 week

Deliver:

-   repository
-   architecture document
-   ADR process
-   domain boundaries
-   interfaces
-   data model
-   design tokens
-   CI
-   testing strategy

## Phase 1 --- OS Core

**Duration:** 2--3 weeks

Deliver:

-   process manager
-   scheduler
-   virtual filesystem
-   permissions
-   IPC
-   syscalls
-   events
-   services

Milestone:

**OS Core Alpha**

## Phase 2 --- Shell

**Duration:** 2 weeks

Deliver:

-   lexer
-   parser
-   AST
-   executor
-   pipes
-   redirection
-   history
-   autocomplete
-   `/proc`
-   `/sys`

Milestone:

**Terminal Alpha**

## Phase 3 --- Desktop

**Duration:** 2 weeks

Deliver:

-   window manager
-   workspace
-   navigation
-   application runtime
-   system bar
-   keyboard shortcuts

Milestone:

**NIKHIL//OS Web Alpha**

This should be the first public LinkedIn demo.

## Phase 4 --- Knowledge Core

**Duration:** 2 weeks

Deliver:

-   canonical profile data
-   projects
-   skills
-   experience
-   evidence
-   graph relationships
-   embeddings

Milestone:

**Knowledge Core Alpha**

## Phase 5 --- AI Core

**Duration:** 3 weeks

Deliver:

-   intent
-   RAG
-   tools
-   planner
-   permission validation
-   action executor
-   audit logs

Milestone:

**AI Core Alpha**

## Phase 6 --- ML

**Duration:** 2--3 weeks

Deliver:

-   visitor interest model
-   project ranking
-   recommendation
-   job analyzer
-   retrieval evaluation

Milestone:

**Adaptive OS Alpha**

## Phase 7 --- Applications

**Duration:** 2--3 weeks

Build:

-   recruiter mode
-   architecture explorer
-   career debugger
-   career simulator
-   AI lab
-   knowledge graph
-   system monitor
-   developer console

## Phase 8 --- Hardening

**Duration:** 2 weeks

Focus:

-   security
-   tests
-   accessibility
-   performance
-   observability
-   error handling
-   UX polish

## Phase 9 --- Desktop

**Duration:** 1--2 weeks

Package:

-   macOS
-   Windows
-   Linux

Use Tauri.

## Phase 10 --- Public Launch

Prepare:

-   landing page
-   Web Edition
-   GitHub README
-   architecture docs
-   demo video
-   screenshots
-   LinkedIn launch post

## Phase 11 --- Kernel Edition

**Duration:** 2--4+ months

Separate systems project.

Start:

-   bootloader
-   x86_64
-   framebuffer
-   GDT/IDT
-   interrupts
-   memory allocator
-   paging
-   keyboard
-   shell
-   filesystem

## Estimated Timeline

  Scope                             Estimate
  --------------------------- --------------
  OS core prototype               3--4 weeks
  Browser OS                      6--8 weeks
  AI OS                         10--14 weeks
  Full polished Web Edition      4--6 months
  Desktop Edition                +2--4 weeks
  Bare-metal kernel             +2--4 months

If built alongside college/internship, plan around 12--20 hours/week.

## Public Milestones

### Week 2

Architecture repository.

### Week 4

OS core demo.

### Week 6

Terminal + filesystem.

### Week 8

Public Web Alpha.

### Week 10

Knowledge graph.

### Week 12

AI Core.

### Week 14--16

Adaptive features.

### Month 5

Polished public release.

### Month 6+

Desktop and kernel work.
