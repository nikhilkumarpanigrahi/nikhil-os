//! Simulated memory manager.
//!
//! Models a paged address space: pages are allocated to processes, tracked in
//! a page table, and released on free. Everything reported here (`free`, `/proc/meminfo`)
//! is derived from real allocation state — nothing is fabricated.

use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize)]
pub struct Allocation {
    pub pid: u32,
    pub process: String,
    pub pages: usize,
    pub bytes: usize,
    /// Simulated first virtual page of the allocation.
    pub base_page: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum MemoryError {
    OutOfMemory,
    InvalidProcess,
}

impl std::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryError::OutOfMemory => f.write_str("out of memory"),
            MemoryError::InvalidProcess => f.write_str("no such process"),
        }
    }
}

pub const PAGE_SIZE: usize = 4096;

#[derive(Debug, Clone, Serialize)]
pub struct MemoryStats {
    pub total_kb: usize,
    pub used_kb: usize,
    pub free_kb: usize,
    pub page_size: usize,
    pub total_pages: usize,
    pub used_pages: usize,
    pub allocations: Vec<Allocation>,
}

#[derive(Default)]
pub struct MemoryManager {
    /// Total simulated physical memory in KiB.
    total_kb: usize,
    /// Next free physical frame.
    next_frame: usize,
    /// Per-process allocations keyed by PID.
    allocations: BTreeMap<u32, Allocation>,
}

impl MemoryManager {
    pub fn new(total_kb: usize) -> Self {
        Self {
            total_kb,
            next_frame: 0,
            allocations: BTreeMap::new(),
        }
    }

    /// Allocate `bytes` for the given process, returning the base virtual page.
    pub fn alloc(&mut self, pid: u32, process: &str, bytes: usize) -> Result<usize, MemoryError> {
        if self.allocations.contains_key(&pid) {
            return Err(MemoryError::InvalidProcess);
        }
        let pages = bytes.div_ceil(PAGE_SIZE);
        let needed_kb = pages * PAGE_SIZE / 1024;
        if self.used_kb() + needed_kb > self.total_kb {
            return Err(MemoryError::OutOfMemory);
        }
        let base_page = self.next_frame;
        self.next_frame += pages;
        self.allocations.insert(
            pid,
            Allocation {
                pid,
                process: process.to_string(),
                pages,
                bytes,
                base_page,
            },
        );
        Ok(base_page)
    }

    /// Free all memory belonging to a process.
    pub fn free(&mut self, pid: u32) {
        self.allocations.remove(&pid);
    }

    pub fn used_kb(&self) -> usize {
        self.allocations
            .values()
            .map(|a| a.pages * PAGE_SIZE / 1024)
            .sum()
    }

    pub fn free_kb(&self) -> usize {
        self.total_kb.saturating_sub(self.used_kb())
    }

    pub fn total_kb(&self) -> usize {
        self.total_kb
    }

    pub fn stats(&self) -> MemoryStats {
        MemoryStats {
            total_kb: self.total_kb,
            used_kb: self.used_kb(),
            free_kb: self.free_kb(),
            page_size: PAGE_SIZE,
            total_pages: self.total_kb * 1024 / PAGE_SIZE,
            used_pages: self.used_kb() * 1024 / PAGE_SIZE,
            allocations: self.allocations.values().cloned().collect(),
        }
    }

    /// `free`-style text report.
    pub fn free_report(&self) -> String {
        let s = self.stats();
        format!(
            "               total        used        free\n\
             Mem:        {:>10} {:>10} {:>10}\n\
             Page size:  {}\n\
             Pages:      {:>10} used of {:>10} total",
            s.total_kb, s.used_kb, s.free_kb, s.page_size, s.used_pages, s.total_pages
        )
    }

    /// `/proc/meminfo`-style text report.
    pub fn meminfo(&self) -> String {
        let s = self.stats();
        format!(
            "MemTotal:        {:>10} kB\n\
             MemFree:        {:>10} kB\n\
             MemUsed:        {:>10} kB\n\
             PageSize:       {:>10} bytes\n\
             TotalPages:     {:>10}\n\
             UsedPages:      {:>10}\n",
            s.total_kb, s.free_kb, s.used_kb, s.page_size, s.total_pages, s.used_pages
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocation_tracks_usage() {
        let mut mem = MemoryManager::new(16 * 1024); // 16 MiB
        mem.alloc(1, "init", 1024 * 1024).unwrap();
        assert_eq!(mem.used_kb(), 1024);
        assert_eq!(mem.free_kb(), 16 * 1024 - 1024);
        mem.free(1);
        assert_eq!(mem.used_kb(), 0);
    }

    #[test]
    fn oom_is_reported() {
        let mut mem = MemoryManager::new(1024); // 1 MiB
        assert!(mem.alloc(1, "a", 512 * 1024).is_ok());
        assert!(mem.alloc(2, "b", 512 * 1024).is_ok());
        assert!(matches!(
            mem.alloc(3, "c", 1024),
            Err(MemoryError::OutOfMemory)
        ));
    }

    #[test]
    fn meminfo_is_real_state() {
        let mut mem = MemoryManager::new(16 * 1024);
        mem.alloc(1, "init", 2 * 1024 * 1024).unwrap();
        let report = mem.meminfo();
        assert!(report.contains("MemUsed:"));
        assert!(report.contains("2048"));
    }
}
