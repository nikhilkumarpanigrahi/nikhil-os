//! Process scheduler.
//!
//! Round Robin for Phase 1, behind a trait so Priority and Multilevel
//! Feedback Queue can be implemented later without touching callers.
//! Telemetry (queue state, current process, time slice, context-switch count)
//! is exposed so Developer Mode can show real scheduler behavior.

use serde::Serialize;
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize)]
pub struct SchedulerStats {
    pub algorithm: String,
    pub time_slice_ticks: u64,
    pub context_switches: u64,
    pub current_pid: Option<u32>,
    pub ready_queue: Vec<u32>,
    /// Ticks remaining for the current process before a switch.
    pub slice_remaining: u64,
}

/// A scheduling policy. Implementations are stateful.
pub trait Scheduler {
    fn name(&self) -> &str;
    /// Called each kernel tick with the current run queue.
    /// Returns the PID that should be running this tick.
    fn tick(&mut self, run_queue: &mut VecDeque<u32>) -> Option<u32>;
    fn stats(&self) -> SchedulerStats;
}

/// Phase 1 policy: simple round robin with a fixed time slice.
#[derive(Debug)]
pub struct RoundRobin {
    time_slice: u64,
    slice_remaining: u64,
    context_switches: u64,
    current: Option<u32>,
}

impl RoundRobin {
    pub fn new(time_slice: u64) -> Self {
        Self {
            time_slice,
            slice_remaining: time_slice,
            context_switches: 0,
            current: None,
        }
    }
}

impl Scheduler for RoundRobin {
    fn name(&self) -> &str {
        "round-robin"
    }

    fn tick(&mut self, run_queue: &mut VecDeque<u32>) -> Option<u32> {
        // If the current process is still eligible and its slice is not
        // exhausted, keep it running.
        if let Some(pid) = self.current {
            if run_queue.contains(&pid) && self.slice_remaining > 0 {
                self.slice_remaining -= 1;
                return Some(pid);
            }
        }

        // Slice exhausted (or current no longer ready): schedule the next.
        if let Some(pid) = run_queue.pop_front() {
            if let Some(prev) = self.current.take() {
                // Requeue the previous process if it is still ready and
                // different from the next one.
                if prev != pid && run_queue.contains(&prev) {
                    run_queue.push_back(prev);
                }
            }
            self.current = Some(pid);
            self.slice_remaining = self.time_slice.saturating_sub(1);
            self.context_switches += 1;
            Some(pid)
        } else {
            self.current = None;
            None
        }
    }

    fn stats(&self) -> SchedulerStats {
        SchedulerStats {
            algorithm: self.name().to_string(),
            time_slice_ticks: self.time_slice,
            context_switches: self.context_switches,
            current_pid: self.current,
            ready_queue: Vec::new(), // caller fills with live queue
            slice_remaining: self.slice_remaining,
        }
    }
}

/// Convenience: rotate the current running process back into the queue when
/// it blocks (enters WAITING). Used by the shell when a command awaits I/O.
pub fn requeue(run_queue: &mut VecDeque<u32>, pid: u32) {
    if !run_queue.contains(&pid) {
        run_queue.push_back(pid);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_robin_cycles_processes() {
        let mut sched = RoundRobin::new(3);
        let mut queue: VecDeque<u32> = [1, 2, 3].into_iter().collect();
        let mut order = Vec::new();
        for _ in 0..12 {
            if let Some(pid) = sched.tick(&mut queue) {
                order.push(pid);
                // Simulate: process runs for its slice then returns to queue.
                if pid != 3 {
                    queue.push_back(pid);
                }
            }
        }
        assert!(order.len() == 12);
        assert!(sched.stats().context_switches >= 3);
        assert_eq!(sched.name(), "round-robin");
    }

    #[test]
    fn empty_queue_yields_nothing() {
        let mut sched = RoundRobin::new(3);
        let mut queue: VecDeque<u32> = VecDeque::new();
        assert_eq!(sched.tick(&mut queue), None);
    }
}
