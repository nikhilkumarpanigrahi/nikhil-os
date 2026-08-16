# NIKHIL//OS --- OS and Shell Specification

## 1. Goal

Build a Unix-inspired simulated operating environment with real internal
state.

The terminal is not a visual prop.

Commands must operate through the OS core.

## 2. Process Model

States:

``` text
NEW
READY
RUNNING
WAITING
TERMINATED
```

Process fields:

-   PID
-   parent PID
-   state
-   priority
-   memory
-   CPU time
-   capabilities
-   start time

## 3. Scheduler

Phase 1:

Round Robin

Phase 2:

Priority

Phase 3:

Multilevel Feedback Queue

Expose:

-   queue state
-   current process
-   time slice
-   context-switch count

## 4. Memory

Simulate:

-   allocation
-   deallocation
-   pages
-   frames
-   page tables
-   virtual addresses

Expose:

``` bash
free
```

and:

``` bash
cat /proc/meminfo
```

## 5. Filesystem

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

Commands:

``` bash
ls
cd
pwd
cat
mkdir
rm
mv
cp
find
grep
```

## 6. Permissions

Support:

-   users
-   groups
-   owner
-   read
-   write
-   execute

Later:

-   capabilities

## 7. IPC

Typed messages.

Examples:

``` text
Terminal → Shell
Shell → Service
Service → Knowledge
AI → Service
```

## 8. Syscalls

Internal:

``` text
open
read
write
close
stat
mkdir
spawn
exec
kill
send
receive
subscribe
```

## 9. Service Manager

Commands:

``` bash
service status
service start
service stop
service restart
```

Show dependencies and state.

## 10. Package Manager

Working name:

`pkgctl`

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

## 11. Shell Architecture

``` text
Input
 ↓
Lexer
 ↓
Parser
 ↓
AST
 ↓
Executor
 ↓
Service / syscall
```

Support:

-   pipes
-   redirection
-   environment variables
-   aliases
-   history
-   autocomplete
-   exit codes

Example:

``` bash
ps | grep ai
```

## 12. Special Commands

``` bash
neofetch
uname
ps
top
free
df
mount
service
pkgctl
ai
graph
career
```

## 13. Neofetch

Use actual NIKHIL//OS state:

``` text
NIKHIL//OS
Kernel: simulated-runtime
Shell: nish
Processes: 7
Memory: 42%
AI Core: online
Knowledge: online
Version: 1.0.0
```

## 14. Shell Philosophy

The shell should be powerful enough to be useful but small enough to
understand.

Prefer composability over dozens of built-in commands.
