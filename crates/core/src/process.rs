//! Process model and process manager.
//!
//! Each simulated process has a PID, parent PID, lifecycle state, priority,
//! simulated CPU/memory usage, capabilities, and timestamps. The process
//! manager is the source of truth for `ps`, `top`, `/proc`, and Developer
//! Mode.

use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ProcessState {
    New,
    Ready,
    Running,
    Waiting,
    Terminated,
}

impl std::fmt::Display for ProcessState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ProcessState::New => "NEW",
            ProcessState::Ready => "READY",
            ProcessState::Running => "RUNNING",
            ProcessState::Waiting => "WAITING",
            ProcessState::Terminated => "TERMINATED",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Process {
    pub pid: u32,
    pub parent_pid: u32,
    pub name: String,
    pub user: String,
    pub state: ProcessState,
    pub priority: u8,
    /// Fractional CPU usage in the current window (0.0 - 100.0).
    pub cpu_usage: f32,
    /// Simulated resident memory in KiB.
    pub memory_kb: usize,
    pub capabilities: Vec<String>,
    /// Kernel tick at which the process was spawned.
    pub start_tick: u64,
    /// Total ticks this process has been scheduled.
    pub cpu_ticks: u64,
}

impl Process {
    pub fn uptime_ticks(&self, now: u64) -> u64 {
        now.saturating_sub(self.start_tick)
    }
}

/// Owns the process table and the next PID counter.
#[derive(Default)]
pub struct ProcessManager {
    processes: BTreeMap<u32, Process>,
    next_pid: u32,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            processes: BTreeMap::new(),
            next_pid: 1,
        }
    }

    /// Spawn a process in `NEW` state and return its PID.
    #[allow(clippy::too_many_arguments)]
    pub fn spawn(
        &mut self,
        name: &str,
        user: &str,
        priority: u8,
        memory_kb: usize,
        capabilities: &[&str],
        start_tick: u64,
        parent_pid: u32,
    ) -> u32 {
        let pid = self.next_pid;
        self.next_pid += 1;
        self.processes.insert(
            pid,
            Process {
                pid,
                parent_pid,
                name: name.to_string(),
                user: user.to_string(),
                state: ProcessState::New,
                priority,
                cpu_usage: 0.0,
                memory_kb,
                capabilities: capabilities.iter().map(|s| s.to_string()).collect(),
                start_tick,
                cpu_ticks: 0,
            },
        );
        pid
    }

    pub fn get(&self, pid: u32) -> Option<&Process> {
        self.processes.get(&pid)
    }

    pub fn get_mut(&mut self, pid: u32) -> Option<&mut Process> {
        self.processes.get_mut(&pid)
    }

    /// Transition a process to a new state, returning whether it changed.
    pub fn set_state(&mut self, pid: u32, state: ProcessState) -> bool {
        if let Some(p) = self.processes.get_mut(&pid) {
            p.state = state;
            true
        } else {
            false
        }
    }

    /// Charge CPU time to a running process and update its usage metric.
    pub fn charge_cpu(&mut self, pid: u32) {
        if let Some(p) = self.processes.get_mut(&pid) {
            p.cpu_ticks += 1;
            // Exponential moving average toward 100% per unit charge is too
            // aggressive; a small increment keeps usage smooth and honest.
            p.cpu_usage = (p.cpu_usage * 0.95 + 100.0 * 0.05).min(100.0);
        }
    }

    /// Decay usage so idle processes trend down.
    pub fn decay_cpu(&mut self) {
        for p in self.processes.values_mut() {
            if p.state != ProcessState::Running {
                p.cpu_usage *= 0.92;
                if p.cpu_usage < 0.5 {
                    p.cpu_usage = 0.0;
                }
            }
        }
    }

    pub fn kill(&mut self, pid: u32) -> bool {
        if let Some(p) = self.processes.get_mut(&pid) {
            p.state = ProcessState::Terminated;
            p.cpu_usage = 0.0;
            true
        } else {
            false
        }
    }

    /// All processes that have not terminated.
    pub fn alive(&self) -> Vec<&Process> {
        self.processes
            .values()
            .filter(|p| p.state != ProcessState::Terminated)
            .collect()
    }

    /// The full process table, ordered by PID.
    pub fn all(&self) -> Vec<&Process> {
        self.processes.values().collect()
    }

    pub fn len(&self) -> usize {
        self.processes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.processes.is_empty()
    }

    /// Total simulated resident memory across alive processes (KiB).
    pub fn total_memory_kb(&self) -> usize {
        self.alive().iter().map(|p| p.memory_kb).sum()
    }

    /// JSON serialization of the process table for `/proc` and Developer Mode.
    pub fn to_json(&self) -> String {
        let alive: Vec<&Process> = self.alive();
        serde_json::to_string(&alive).unwrap_or_else(|_| "[]".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_assigns_incrementing_pids() {
        let mut pm = ProcessManager::new();
        let p1 = pm.spawn("init", "root", 0, 1024, &["sys_admin"], 0, 0);
        let p2 = pm.spawn("shell", "nikhil", 1, 512, &[], 1, p1);
        assert_eq!(p1, 1);
        assert_eq!(p2, 2);
        assert_eq!(pm.len(), 2);
        assert_eq!(pm.get(p2).unwrap().parent_pid, 1);
    }

    #[test]
    fn kill_terminates_and_removes_from_alive() {
        let mut pm = ProcessManager::new();
        let pid = pm.spawn("sleep", "nikhil", 1, 64, &[], 0, 1);
        assert_eq!(pm.alive().len(), 1);
        assert!(pm.kill(pid));
        assert_eq!(pm.get(pid).unwrap().state, ProcessState::Terminated);
        assert_eq!(pm.alive().len(), 0);
    }

    #[test]
    fn cpu_charge_and_decay() {
        let mut pm = ProcessManager::new();
        let pid = pm.spawn("worker", "nikhil", 1, 64, &[], 0, 1);
        for _ in 0..40 {
            pm.charge_cpu(pid);
        }
        assert!(pm.get(pid).unwrap().cpu_usage > 0.0);
        pm.decay_cpu();
        assert!(pm.get(pid).unwrap().cpu_usage <= 100.0);
    }
}
