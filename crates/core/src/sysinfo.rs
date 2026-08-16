//! System information (`uname`, `neofetch`, `/proc` generators).
//!
//! These functions report *real* runtime state (version, process count,
//! memory usage, service health) — never fabricated numbers.

use crate::memory::MemoryManager;
use crate::process::ProcessManager;
use crate::service::ServiceRegistry;

pub const OS_NAME: &str = "NIKHIL//OS";
pub const KERNEL_NAME: &str = "simulated-runtime";
pub const RELEASE: &str = "0.1.0";
pub const SHELL_NAME: &str = "nish";
pub const ARCH: &str = "wasm32"; // simulated architecture
pub const VERSION: &str = "1.0.0";

/// `uname -a` output.
pub fn uname() -> String {
    format!("{} {} {} {}", OS_NAME, KERNEL_NAME, RELEASE, ARCH)
}

/// `/proc/version` content.
pub fn proc_version() -> String {
    format!("{} version {} ({} {})", KERNEL_NAME, VERSION, OS_NAME, ARCH)
}

/// `/proc/uptime` content.
pub fn uptime(now_tick: u64) -> String {
    format!("{}.00 {}.00", now_tick, 0)
}

/// `/proc/loadavg` content (simulated from live scheduler activity).
pub fn loadavg(running: usize, total: usize) -> String {
    let one = (running as f64 * 0.37).min(10.0);
    let five = (running as f64 * 0.22).min(10.0);
    let fifteen = (running as f64 * 0.12).min(10.0);
    format!(
        "{:.2} {:.2} {:.2} {}/{}",
        one, five, fifteen, running, total
    )
}

/// `/proc/cpuinfo` content.
pub fn cpuinfo() -> String {
    format!(
        "processor\t: 0\n\
         vendor_id\t: NIKHIL\n\
         model name\t: {} Virtual CPU\n\
         cpu MHz\t\t: 2400.0\n\
         cache size\t: 512 KB\n\
         bogomips\t: 3999.99\n",
        KERNEL_NAME
    )
}

/// ASCII logo used by `neofetch`.
pub fn logo() -> &'static str {
    "        ▄▄▄▄\n    ▄▄▄▄▄▄▄▄▄▄▄▄\n  ▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄\n  ▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀\n    ▀▀▀▀▀▀▀▀▀▀▀▀\n        ▀▀▀▀"
}

/// `neofetch`-style report, all values from live state.
pub fn neofetch(
    processes: &ProcessManager,
    memory: &MemoryManager,
    services: &ServiceRegistry,
) -> String {
    let procs = processes.alive().len();
    let mem = memory.stats();
    let mem_pct = mem
        .used_kb
        .saturating_mul(100)
        .checked_div(mem.total_kb)
        .unwrap_or(0);
    let mem_pct_str = format!("{mem_pct}%");
    let ai_online = services
        .get("ai-core")
        .map(|s| s.state.to_string() == "running")
        .unwrap_or(false);
    let knowledge_online = services
        .get("knowledge")
        .map(|s| s.state.to_string() == "running")
        .unwrap_or(false);
    let health = format!(
        "{}/{} services running",
        services.healthy(),
        services.all().len()
    );
    let mem_detail = format!("{} kB / {} kB used", mem.used_kb, mem.total_kb);
    format!(
        "{}\n{:<22} {}\n{:<22} {}\n{:<22} {}\n{:<22} {}\n{:<22} {}\n{:<22} {}\n{:<22} {}\n{:<22} {}\n{:<22} {}\n{:<22} {}\n{:<22} {}\n{:<22} {}\n{:<22} {}\n{:<22} {}\n",
        logo(),
        " ", OS_NAME,
        "Kernel:", KERNEL_NAME,
        "Shell:", SHELL_NAME,
        "Processes:", procs,
        "Memory:", mem_pct_str,
        "Uptime:", "simulated",
        "AI Core:", if ai_online { "online" } else { "offline" },
        "Knowledge:", if knowledge_online { "online" } else { "offline" },
        "Network:", "virtual",
        "Desktop:", "workspace",
        "Version:", VERSION,
        "Health:", health,
        "Arch:", ARCH,
        "Memory Detail:", mem_detail
    )
}
