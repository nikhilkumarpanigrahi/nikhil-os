//! System call layer.
//!
//! Applications never mutate kernel state directly; they request operations
//! through the syscall interface, which enforces permission checks and
//! emits observability events. This mirrors the documented syscall surface:
//!
//!   open read write close stat mkdir spawn exec kill send receive subscribe

use crate::event::{EventBus, Severity};
use crate::filesystem::{FileSystem, FsError, FsResult};
use crate::memory::MemoryManager;
use crate::process::{ProcessManager, ProcessState};
use std::rc::Rc;

/// Result of a syscall, formatted like a real kernel.
pub type SyscallResult = Result<(), String>;

/// Who is performing a syscall.
#[derive(Debug, Clone)]
pub struct Caller {
    pub pid: u32,
    pub uid: u32,
    pub user: String,
    pub groups: Vec<String>,
}

/// Coarse-grained capability names used in permission checks.
pub mod caps {
    pub const PROCESS: &str = "sys_process";
    pub const FILESYSTEM: &str = "sys_filesystem";
    pub const SERVICE: &str = "sys_service";
    pub const PACKAGE: &str = "sys_package";
    pub const IPC: &str = "sys_ipc";
    pub const MONITOR: &str = "sys_monitor";
}

/// Central syscall entrypoint. Owns shared references to kernel subsystems.
pub struct Syscall {
    pub fs: Rc<std::cell::RefCell<FileSystem>>,
    pub processes: Rc<std::cell::RefCell<ProcessManager>>,
    pub memory: Rc<std::cell::RefCell<MemoryManager>>,
    pub events: Rc<EventBus>,
    /// Kernel tick (read at call time).
    pub tick: Rc<dyn Fn() -> u64>,
}

impl Syscall {
    pub fn new(
        fs: Rc<std::cell::RefCell<FileSystem>>,
        processes: Rc<std::cell::RefCell<ProcessManager>>,
        memory: Rc<std::cell::RefCell<MemoryManager>>,
        events: Rc<EventBus>,
        tick: Rc<dyn Fn() -> u64>,
    ) -> Self {
        Self {
            fs,
            processes,
            memory,
            events,
            tick,
        }
    }

    fn has_cap(&self, caller: &Caller, cap: &str) -> bool {
        // The default user has all capabilities except service management
        // without args; root has everything.
        if caller.uid == 0 {
            return true;
        }
        caller.groups.iter().any(|g| g == "wheel") || {
            let pm = self.processes.borrow();
            pm.get(caller.pid)
                .map(|p| p.capabilities.iter().any(|c| c == cap))
                .unwrap_or(false)
        }
    }

    fn audit(&self, caller: &Caller, op: &str, ok: bool) {
        let tick = (self.tick)();
        self.events.emit(
            tick,
            if ok {
                Severity::Debug
            } else {
                Severity::Warning
            },
            "syscall",
            format!(
                "pid={} {} {}",
                caller.pid,
                op,
                if ok { "ok" } else { "denied" }
            ),
        );
    }

    // -- filesystem --------------------------------------------------------

    pub fn read(&self, caller: &Caller, path: &str) -> FsResult<String> {
        if !self.has_cap(caller, caps::FILESYSTEM) {
            self.audit(caller, &format!("read {path}"), false);
            return Err(FsError::PermissionDenied);
        }
        self.audit(caller, &format!("read {path}"), true);
        self.fs.borrow().read_text(path)
    }

    pub fn write(&self, caller: &Caller, path: &str, data: &str) -> SyscallResult {
        if !self.has_cap(caller, caps::FILESYSTEM) {
            self.audit(caller, &format!("write {path}"), false);
            return Err("permission denied".into());
        }
        self.audit(caller, &format!("write {path}"), true);
        let tick = (self.tick)();
        self.fs
            .borrow_mut()
            .write(path, data.as_bytes(), &caller.user, "users", tick)
            .map_err(|e| e.to_string())
    }

    pub fn stat(&self, caller: &Caller, path: &str) -> FsResult<()> {
        let _ = caller;
        self.fs.borrow().stat(path).map(|_| ())
    }

    // -- process management ------------------------------------------------

    pub fn spawn(
        &self,
        caller: &Caller,
        name: &str,
        priority: u8,
        memory_kb: usize,
        capabilities: &[&str],
    ) -> Result<u32, String> {
        if !self.has_cap(caller, caps::PROCESS) {
            self.audit(caller, &format!("spawn {name}"), false);
            return Err("permission denied".into());
        }
        let tick = (self.tick)();
        let pid = self.processes.borrow_mut().spawn(
            name,
            &caller.user,
            priority,
            memory_kb,
            capabilities,
            tick,
            caller.pid,
        );
        self.memory
            .borrow_mut()
            .alloc(pid, name, memory_kb * 1024)
            .ok();
        self.events.emit(
            (self.tick)(),
            Severity::Info,
            "process-manager",
            format!("spawned pid={pid} name={name}"),
        );
        self.audit(caller, &format!("spawn {name}"), true);
        Ok(pid)
    }

    pub fn kill(&self, caller: &Caller, pid: u32) -> SyscallResult {
        if !self.has_cap(caller, caps::PROCESS) {
            self.audit(caller, &format!("kill {pid}"), false);
            return Err("permission denied".into());
        }
        let ok = self.processes.borrow_mut().kill(pid);
        if ok {
            self.memory.borrow_mut().free(pid);
        }
        self.audit(caller, &format!("kill {pid}"), ok);
        if ok {
            Ok(())
        } else {
            Err(format!("no such process: {pid}"))
        }
    }

    /// Mark a process WAITING (e.g. blocked on I/O).
    pub fn block(&self, _caller: &Caller, pid: u32) -> SyscallResult {
        if self
            .processes
            .borrow_mut()
            .set_state(pid, ProcessState::Waiting)
        {
            Ok(())
        } else {
            Err(format!("no such process: {pid}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::EventBus;

    fn setup() -> (Syscall, Caller) {
        let events = Rc::new(EventBus::new());
        let fs = Rc::new(std::cell::RefCell::new(FileSystem::new()));
        let processes = Rc::new(std::cell::RefCell::new(ProcessManager::new()));
        let memory = Rc::new(std::cell::RefCell::new(MemoryManager::new(16 * 1024)));
        let tick: Rc<dyn Fn() -> u64> = Rc::new(|| 0);
        let sc = Syscall::new(fs, processes, memory, Rc::clone(&events), tick);
        let caller = Caller {
            pid: 1,
            uid: 1000,
            user: "nikhil".into(),
            groups: vec!["wheel".into()],
        };
        (sc, caller)
    }

    #[test]
    fn write_and_read_roundtrip() {
        let (sc, caller) = setup();
        sc.fs
            .borrow_mut()
            .mkdir("/home", "root", "root", 0)
            .unwrap();
        sc.write(&caller, "/home/test.txt", "hello").unwrap();
        assert_eq!(sc.read(&caller, "/home/test.txt").unwrap(), "hello");
    }

    #[test]
    fn spawn_creates_process() {
        let (sc, caller) = setup();
        let pid = sc
            .spawn(&caller, "ls", 1, 256, &[caps::FILESYSTEM])
            .unwrap();
        assert!(pid > 0);
        assert!(sc.processes.borrow().get(pid).is_some());
    }

    #[test]
    fn kill_reports_missing_pid() {
        let (sc, caller) = setup();
        assert!(sc.kill(&caller, 9999).is_err());
    }
}
