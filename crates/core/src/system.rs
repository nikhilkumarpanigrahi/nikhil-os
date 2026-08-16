//! The NIKHIL//OS system: owns every kernel subsystem, runs the boot
//! sequence, advances the scheduler, and exposes live state to the UI.
//!
//! Boot order mirrors the architecture document:
//! Boot → Kernel init → Filesystem mount → Init → Service manager → Network
//! → AI Core → Knowledge Core → Window Manager → Desktop.

use crate::event::{EventBus, Severity};
use crate::filesystem::{
    DirEntry, FileStat, FileSystem, FileType, FsError, FsResult, VirtualFilesystem,
};
use crate::ipc::IpcBus;
use crate::knowledge::KnowledgeService;
use crate::logging::Logger;
use crate::memory::MemoryManager;
use crate::package::{PackageManifest, PackageRegistry};
use crate::process::{ProcessManager, ProcessState};
use crate::scheduler::{RoundRobin, Scheduler};
use crate::service::{RestartPolicy, ServiceRegistry};
use crate::shell::{Shell, ShellContext};
use crate::syscall::Syscall;
use crate::sysinfo;
use serde_json::json;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::Rc;

/// Total simulated physical memory: 16 MiB.
const TOTAL_MEMORY_KB: usize = 16 * 1024;

pub struct System {
    pub tick: Rc<Cell<u64>>,
    pub fs: Rc<RefCell<FileSystem>>,
    pub processes: Rc<RefCell<ProcessManager>>,
    pub memory: Rc<RefCell<MemoryManager>>,
    pub services: Rc<RefCell<ServiceRegistry>>,
    pub packages: Rc<RefCell<PackageRegistry>>,
    pub scheduler: Rc<RefCell<RoundRobin>>,
    pub events: Rc<EventBus>,
    pub ipc: Rc<IpcBus>,
    pub logger: Logger,
    pub shell: RefCell<Shell>,
    pub knowledge: Rc<KnowledgeService>,
    pub syscalls: Syscall,
    run_queue: RefCell<VecDeque<u32>>,
    boot_log: RefCell<Vec<String>>,
}

impl Default for System {
    fn default() -> Self {
        Self::new()
    }
}

impl System {
    pub fn new() -> Self {
        let events = Rc::new(EventBus::new());
        let logger = Logger::new(Rc::clone(&events));
        let knowledge = Rc::new(KnowledgeService::load());
        let user_name = knowledge.user_name();

        let fs = Rc::new(RefCell::new(FileSystem::new()));
        let processes = Rc::new(RefCell::new(ProcessManager::new()));
        let memory = Rc::new(RefCell::new(MemoryManager::new(TOTAL_MEMORY_KB)));
        let services = Rc::new(RefCell::new(ServiceRegistry::new()));
        let packages = Rc::new(RefCell::new(PackageRegistry::new()));
        let scheduler = Rc::new(RefCell::new(RoundRobin::new(5)));
        let ipc = Rc::new(IpcBus::new());

        let tick_cell = Rc::new(Cell::new(0u64));
        let tick_holder = Rc::clone(&tick_cell);
        let tick_fn: Rc<dyn Fn() -> u64> = Rc::new(move || tick_holder.get());

        let ctx = Rc::new(ShellContext {
            fs: Rc::clone(&fs),
            processes: Rc::clone(&processes),
            memory: Rc::clone(&memory),
            services: Rc::clone(&services),
            packages: Rc::clone(&packages),
            scheduler: Rc::clone(&scheduler),
            events: Rc::clone(&events),
            ipc: Rc::clone(&ipc),
            tick: Rc::clone(&tick_fn),
        });

        let syscalls = Syscall::new(
            Rc::clone(&fs),
            Rc::clone(&processes),
            Rc::clone(&memory),
            Rc::clone(&events),
            Rc::clone(&tick_fn),
        );

        let shell = Shell::new(ctx, &user_name);

        // Mount the live virtual filesystems.
        {
            let mut fs_borrow = fs.borrow_mut();
            let proc_fs = ProcFileSystem::new(
                FsKind::Proc,
                Rc::clone(&processes),
                Rc::clone(&memory),
                Rc::clone(&services),
                Rc::clone(&scheduler),
                Rc::clone(&tick_fn),
                Rc::clone(&knowledge),
            );
            let sys_fs = ProcFileSystem::new(
                FsKind::Sys,
                Rc::clone(&processes),
                Rc::clone(&memory),
                Rc::clone(&services),
                Rc::clone(&scheduler),
                Rc::clone(&tick_fn),
                Rc::clone(&knowledge),
            );
            fs_borrow.mount("/proc", Rc::new(proc_fs));
            fs_borrow.mount("/sys", Rc::new(sys_fs));
        }

        let mut system = System {
            tick: tick_cell,
            fs,
            processes,
            memory,
            services,
            packages,
            scheduler,
            events,
            ipc,
            logger,
            shell: RefCell::new(shell),
            knowledge,
            syscalls,
            run_queue: RefCell::new(VecDeque::new()),
            boot_log: RefCell::new(Vec::new()),
        };
        system.register_packages();
        system
    }

    fn register_packages(&mut self) {
        let apps: &[(&str, &str, &[&str], &[&str])] = &[
            ("terminal", "nish terminal emulator", &[], &["sys_ipc"]),
            (
                "files",
                "virtual filesystem browser",
                &[],
                &["sys_filesystem"],
            ),
            (
                "projects",
                "projects and evidence browser",
                &["knowledge"],
                &["sys_ipc"],
            ),
            (
                "resume",
                "experience and skills",
                &["knowledge"],
                &["sys_ipc"],
            ),
            (
                "recruiter",
                "recruiter-facing candidate view",
                &["knowledge"],
                &["sys_ipc"],
            ),
            (
                "system-monitor",
                "live process and memory telemetry",
                &[],
                &["sys_monitor"],
            ),
            (
                "developer-console",
                "kernel observability",
                &[],
                &["sys_monitor"],
            ),
            ("settings", "system settings", &[], &["sys_service"]),
            ("knowledge", "knowledge graph", &[], &["sys_ipc"]),
            ("lab", "AI/ML lab", &[], &["sys_ipc"]),
        ];
        let mut packages = self.packages.borrow_mut();
        for (name, description, deps, perms) in apps {
            packages.register(PackageManifest {
                name: name.to_string(),
                version: "1.0.0".to_string(),
                description: description.to_string(),
                dependencies: deps.iter().map(|s| s.to_string()).collect(),
                permissions: perms.iter().map(|s| s.to_string()).collect(),
                entrypoint: format!("apps.{name}"),
            });
        }
        // Installed by default.
        for name in [
            "terminal",
            "files",
            "projects",
            "resume",
            "recruiter",
            "system-monitor",
            "settings",
        ] {
            let _ = packages.install(name);
        }
    }

    // -- boot --------------------------------------------------------------

    pub fn boot(&mut self) -> Vec<String> {
        let mut log: Vec<String> = Vec::new();
        let tick = 0u64;

        // 1. Kernel initialization
        self.logger.info("kernel", "initializing simulated runtime");
        log.push("[ OK ] kernel: NIKHIL//OS simulated-runtime 0.1.0 (wasm32)".to_string());

        // 2. Filesystem mount
        {
            let mut fs = self.fs.borrow_mut();
            for dir in [
                "/bin", "/dev", "/etc", "/home", "/opt", "/tmp", "/usr", "/var",
            ] {
                let _ = fs.mkdir(dir, "root", "root", tick);
            }
            // /proc and /sys are virtual mounts (already mounted in new()).
            let _ = fs.write("/etc/hostname", b"nikhil-os\n", "root", "root", tick);
            let _ = fs.write(
                "/etc/os-release",
                format!("NAME={}\nVERSION={}\n", sysinfo::OS_NAME, sysinfo::VERSION).as_bytes(),
                "root",
                "root",
                tick,
            );
            let _ = fs.write(
                "/etc/motd",
                b"Welcome to NIKHIL//OS.\nType 'help' for a command list.\n",
                "root",
                "root",
                tick,
            );
            let _ = fs.write("/dev/null", b"", "root", "root", tick);
            let _ = fs.write("/dev/zero", b"", "root", "root", tick);
            let _ = fs.write("/dev/tty", b"", "root", "root", tick);
        }
        self.logger.info("filesystem", "mounted /proc /sys /dev");
        log.push("[ OK ] filesystem: mounted /, /proc, /sys, /dev, /etc, /home".to_string());

        // 3. Home directory for the profile user.
        let user = self.knowledge.user_name();
        {
            let mut fs = self.fs.borrow_mut();
            let _ = fs.mkdir(&format!("/home/{user}"), &user, "users", tick);
            let _ = fs.mkdir(&format!("/home/{user}/projects"), &user, "users", tick);
            let _ = fs.mkdir(&format!("/home/{user}/notes"), &user, "users", tick);
            let _ = fs.write(
                &format!("/home/{user}/.profile"),
                format!("# {}\n", self.knowledge.person().name).as_bytes(),
                &user,
                "users",
                tick,
            );
            let _ = fs.write(
                &format!("/home/{user}/README.txt"),
                b"Type 'help' for commands. Try: neofetch, ps | grep ai, pkgctl list, service status\n",
                &user,
                "users",
                tick,
            );
        }

        // 4. Init (pid 1)
        let init_pid = self.spawn_kernel("init", "root", 0, 1024, &["sys_admin"], 0);
        log.push(format!("[ OK ] init: spawned pid={init_pid}"));

        // 5. Service manager boot (dependency order)
        self.define_services();
        let mut services = self.services.borrow_mut();
        let order = [
            "kernel",
            "filesystem",
            "service-manager",
            "knowledge",
            "shell",
            "network",
            "ai-core",
            "window-manager",
        ];
        for name in order {
            let pid = self.spawn_kernel(name, "root", 1, 128, &["sys_service"], init_pid);
            services.start(name, tick, pid);
            log.push(format!("[ OK ] {name}: running (pid={pid})"));
        }
        drop(services);

        // 6. Desktop process
        let desktop_pid = self.spawn_kernel("desktop", &user, 1, 256, &[], init_pid);
        log.push(format!(
            "[ OK ] desktop: workspace ready (pid={desktop_pid})"
        ));
        log.push(format!(
            "[ OK ] login: welcome, {} ({})",
            self.knowledge.person().name,
            user
        ));

        self.logger.info("kernel", "boot complete");
        self.boot_log.replace(log.clone());
        log
    }

    fn define_services(&mut self) {
        let mut services = self.services.borrow_mut();
        services.define("kernel", "kernel initialization", &[], RestartPolicy::Never);
        services.define(
            "filesystem",
            "virtual filesystem",
            &["kernel"],
            RestartPolicy::OnFailure,
        );
        services.define(
            "service-manager",
            "service lifecycle",
            &["filesystem"],
            RestartPolicy::OnFailure,
        );
        services.define(
            "knowledge",
            "knowledge core",
            &["filesystem"],
            RestartPolicy::OnFailure,
        );
        services.define(
            "shell",
            "nish shell service",
            &["service-manager"],
            RestartPolicy::Always,
        );
        services.define(
            "network",
            "virtual network stack",
            &["service-manager"],
            RestartPolicy::OnFailure,
        );
        services.define(
            "ai-core",
            "AI runtime (planned)",
            &["knowledge"],
            RestartPolicy::Always,
        );
        services.define(
            "window-manager",
            "workspace window manager",
            &["shell", "network"],
            RestartPolicy::Always,
        );
    }

    fn spawn_kernel(
        &self,
        name: &str,
        user: &str,
        priority: u8,
        memory_kb: usize,
        capabilities: &[&str],
        parent_pid: u32,
    ) -> u32 {
        let tick = self.tick.get();
        let pid = self.processes.borrow_mut().spawn(
            name,
            user,
            priority,
            memory_kb,
            capabilities,
            tick,
            parent_pid,
        );
        self.memory
            .borrow_mut()
            .alloc(pid, name, memory_kb * 1024)
            .ok();
        self.events.emit(
            tick,
            Severity::Info,
            "process-manager",
            format!("spawned pid={pid} name={name}"),
        );
        pid
    }

    // -- kernel tick -------------------------------------------------------

    /// Advance the simulated clock one tick: promote NEW processes to READY,
    /// run the scheduler, charge CPU, and decay idle usage.
    pub fn tick_kernel(&self) {
        let now = self.tick.get() + 1;
        self.tick.set(now);
        self.logger.set_tick(now);

        // Promote NEW → READY and enqueue.
        {
            let mut pm = self.processes.borrow_mut();
            let mut run_queue = self.run_queue.borrow_mut();
            let pids: Vec<u32> = pm.all().iter().map(|p| p.pid).collect();
            for pid in pids {
                if let Some(p) = pm.get_mut(pid) {
                    if p.state == ProcessState::New {
                        p.state = ProcessState::Ready;
                        run_queue.push_back(pid);
                    }
                }
            }
        }
        // Re-enqueue any READY process that fell out of the queue.
        {
            let pm = self.processes.borrow();
            let mut run_queue = self.run_queue.borrow_mut();
            let ready: Vec<u32> = pm
                .alive()
                .iter()
                .filter(|p| p.state == ProcessState::Ready)
                .map(|p| p.pid)
                .collect();
            for pid in ready {
                if !run_queue.contains(&pid) {
                    run_queue.push_back(pid);
                }
            }
        }

        let prev_running = self
            .processes
            .borrow()
            .all()
            .iter()
            .find(|p| p.state == ProcessState::Running)
            .map(|p| p.pid);

        let next = self
            .scheduler
            .borrow_mut()
            .tick(&mut self.run_queue.borrow_mut());

        let mut pm = self.processes.borrow_mut();
        if let Some(pid) = next {
            if let Some(prev) = prev_running {
                if prev != pid {
                    if let Some(p) = pm.get_mut(prev) {
                        p.state = ProcessState::Ready;
                    }
                }
            }
            if let Some(p) = pm.get_mut(pid) {
                p.state = ProcessState::Running;
            }
            pm.charge_cpu(pid);
        } else if let Some(prev) = prev_running {
            if let Some(p) = pm.get_mut(prev) {
                p.state = ProcessState::Ready;
            }
        }
        pm.decay_cpu();
        drop(pm);

        self.services.borrow_mut().tick(now);
    }

    /// Aggregate CPU utilization (0.0 - 100.0), derived from real scheduling.
    pub fn cpu_utilization(&self) -> f32 {
        let pm = self.processes.borrow();
        let sum: f32 = pm.alive().iter().map(|p| p.cpu_usage).sum();
        sum.min(100.0)
    }

    // -- shell -------------------------------------------------------------

    pub fn run_command(&self, input: &str) -> String {
        let mut shell = self.shell.borrow_mut();
        let result = shell.run_line(input);
        result.output
    }

    pub fn prompt(&self) -> String {
        let shell = self.shell.borrow();
        let user = self.knowledge.user_name();
        let cwd = shell.cwd();
        format!("{user}@nikhil-os:{cwd}$ ")
    }

    pub fn cwd(&self) -> String {
        self.shell.borrow().cwd().to_string()
    }

    pub fn autocomplete(&self, prefix: &str) -> Vec<String> {
        self.shell.borrow().autocomplete(prefix)
    }

    // -- snapshots for the UI ---------------------------------------------

    /// Full runtime snapshot as JSON, for System Monitor / status bar / Files.
    pub fn snapshot_json(&self) -> String {
        let pm = self.processes.borrow();
        let processes_json = pm.to_json();
        drop(pm);
        let mem = self.memory.borrow().stats();
        let services_json = self.services.borrow().to_json();
        let scheduler_stats = self.scheduler.borrow().stats();
        let mounts = self.fs.borrow().mount_points();
        let snapshot = json!({
            "tick": self.tick.get(),
            "cpu": (self.cpu_utilization() * 10.0).round() / 10.0,
            "processes": serde_json::from_str::<serde_json::Value>(&processes_json).unwrap_or(json!([])),
            "memory": {
                "total_kb": mem.total_kb,
                "used_kb": mem.used_kb,
                "free_kb": mem.free_kb,
                "used_percent": mem
                    .used_kb
                    .saturating_mul(100)
                    .checked_div(mem.total_kb)
                    .unwrap_or(0),
            },
            "services": serde_json::from_str::<serde_json::Value>(&services_json).unwrap_or(json!([])),
            "scheduler": {
                "algorithm": scheduler_stats.algorithm,
                "context_switches": scheduler_stats.context_switches,
                "current_pid": scheduler_stats.current_pid,
                "time_slice_ticks": scheduler_stats.time_slice_ticks,
            },
            "filesystem": { "mounts": mounts },
            "boot": { "stage": "ready", "lines": self.boot_log.borrow().len() }
        });
        serde_json::to_string(&snapshot).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn fs_list_json(&self, path: &str) -> String {
        self.fs.borrow().list_json(path)
    }

    pub fn fs_stat_json(&self, path: &str) -> String {
        self.fs.borrow().stat_json(path)
    }

    pub fn fs_read(&self, path: &str) -> String {
        self.fs
            .borrow()
            .read_text(path)
            .unwrap_or_else(|e| format!("cat: {path}: {e}"))
    }

    pub fn knowledge_json(&self) -> String {
        self.knowledge.profile_json()
    }

    pub fn events_json(&self, n: usize) -> String {
        self.events.recent_json(n)
    }
}

/// Which virtual filesystem we are serving.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FsKind {
    Proc,
    Sys,
}

/// Live `/proc` and `/sys` — generated from current runtime state on every read.
struct ProcFileSystem {
    kind: FsKind,
    processes: Rc<RefCell<ProcessManager>>,
    memory: Rc<RefCell<MemoryManager>>,
    services: Rc<RefCell<ServiceRegistry>>,
    scheduler: Rc<RefCell<RoundRobin>>,
    tick: Rc<dyn Fn() -> u64>,
    knowledge: Rc<KnowledgeService>,
}

impl ProcFileSystem {
    #[allow(clippy::too_many_arguments)]
    fn new(
        kind: FsKind,
        processes: Rc<RefCell<ProcessManager>>,
        memory: Rc<RefCell<MemoryManager>>,
        services: Rc<RefCell<ServiceRegistry>>,
        scheduler: Rc<RefCell<RoundRobin>>,
        tick: Rc<dyn Fn() -> u64>,
        knowledge: Rc<KnowledgeService>,
    ) -> Self {
        Self {
            kind,
            processes,
            memory,
            services,
            scheduler,
            tick,
            knowledge,
        }
    }

    fn entry(name: &str, file_type: FileType) -> DirEntry {
        DirEntry {
            name: name.to_string(),
            file_type,
            size: 0,
            perms: "r--r--r--".to_string(),
            owner: "root".to_string(),
            group: "root".to_string(),
        }
    }

    fn pid_dirs(&self) -> Vec<String> {
        self.processes
            .borrow()
            .alive()
            .iter()
            .map(|p| p.pid.to_string())
            .collect()
    }

    fn process_status(&self, pid: u32) -> Option<String> {
        let pm = self.processes.borrow();
        let p = pm.get(pid)?;
        let caps = if p.capabilities.is_empty() {
            "-".to_string()
        } else {
            p.capabilities.join(",")
        };
        Some(format!(
            "Name:\t{}\nState:\t{:?}\nPid:\t{}\nPPid:\t{}\nUser:\t{}\nPriority:\t{}\nVmRSS:\t{} kB\nCpuUsage:\t{:.1}%\nStartTick:\t{}\nCapabilities:\t{}\n",
            p.name, p.state, p.pid, p.parent_pid, p.user, p.priority, p.memory_kb, p.cpu_usage,
            p.start_tick, caps
        ))
    }

    fn process_stat_line(&self, pid: u32) -> Option<String> {
        let pm = self.processes.borrow();
        let p = pm.get(pid)?;
        Some(format!(
            "{} ({}) {} {} {} 0 0 0 {} {}",
            p.pid,
            p.name,
            format!("{:?}", p.state).to_lowercase(),
            p.priority,
            p.parent_pid,
            p.cpu_ticks,
            p.memory_kb
        ))
    }

    /// What the OS knows about its owner, drawn from the embedded knowledge
    /// core. Read `/proc/knowledge` to see the live index.
    fn knowledge_summary(&self) -> String {
        let k = &self.knowledge;
        let person = k.person();
        let projects = k.projects().len();
        let skills = k.skills().len();
        let claims = k.claims().len();
        format!(
            "owner:\t{}\nrole:\t{}\nprojects:\t{}\nskills:\t{}\nclaims:\t{}\n",
            person.name, person.role, projects, skills, claims
        )
    }
}

impl VirtualFilesystem for ProcFileSystem {
    fn list(&self, path: &str) -> FsResult<Vec<DirEntry>> {
        match self.kind {
            FsKind::Proc => match path {
                "/" => {
                    let mut entries = vec![
                        Self::entry("cpuinfo", FileType::File),
                        Self::entry("knowledge", FileType::File),
                        Self::entry("loadavg", FileType::File),
                        Self::entry("meminfo", FileType::File),
                        Self::entry("uptime", FileType::File),
                        Self::entry("version", FileType::File),
                    ];
                    for pid in self.pid_dirs() {
                        entries.push(Self::entry(&pid, FileType::Directory));
                    }
                    Ok(entries)
                }
                p if is_pid_path(p) => Ok(vec![
                    Self::entry("cmdline", FileType::File),
                    Self::entry("status", FileType::File),
                    Self::entry("stat", FileType::File),
                ]),
                _ => Err(FsError::NotFound),
            },
            FsKind::Sys => match path {
                "/" => Ok(vec![
                    Self::entry("kernel", FileType::Directory),
                    Self::entry("runtime", FileType::Directory),
                ]),
                "/kernel" => Ok(vec![
                    Self::entry("osname", FileType::File),
                    Self::entry("release", FileType::File),
                    Self::entry("version", FileType::File),
                ]),
                "/runtime" => Ok(vec![
                    Self::entry("processes", FileType::File),
                    Self::entry("memory", FileType::File),
                    Self::entry("services", FileType::File),
                    Self::entry("scheduler", FileType::File),
                    Self::entry("uptime", FileType::File),
                ]),
                _ => Err(FsError::NotFound),
            },
        }
    }

    fn read(&self, path: &str) -> FsResult<Vec<u8>> {
        match self.kind {
            FsKind::Proc => {
                let now = (self.tick)();
                match path {
                    "/cpuinfo" => Ok(sysinfo::cpuinfo().into_bytes()),
                    "/knowledge" => Ok(self.knowledge_summary().into_bytes()),
                    "/meminfo" => Ok(self.memory.borrow().meminfo().into_bytes()),
                    "/uptime" => Ok(sysinfo::uptime(now).into_bytes()),
                    "/version" => Ok(sysinfo::proc_version().into_bytes()),
                    "/loadavg" => {
                        let running = self.processes.borrow().alive().len();
                        let total = self.processes.borrow().len();
                        Ok(sysinfo::loadavg(running, total).into_bytes())
                    }
                    "/self" => Err(FsError::NotFound),
                    p => {
                        let pid = parse_pid(p)?;
                        match p {
                            _ if p.ends_with("/status") => Ok(self
                                .process_status(pid)
                                .ok_or(FsError::NotFound)?
                                .into_bytes()),
                            _ if p.ends_with("/cmdline") => Ok(self
                                .processes
                                .borrow()
                                .get(pid)
                                .map(|x| x.name.clone())
                                .ok_or(FsError::NotFound)?
                                .into_bytes()),
                            _ if p.ends_with("/stat") => Ok(self
                                .process_stat_line(pid)
                                .ok_or(FsError::NotFound)?
                                .into_bytes()),
                            _ => Err(FsError::NotFound),
                        }
                    }
                }
            }
            FsKind::Sys => match path {
                "/kernel/osname" => Ok(sysinfo::OS_NAME.as_bytes().to_vec()),
                "/kernel/release" => Ok(sysinfo::RELEASE.as_bytes().to_vec()),
                "/kernel/version" => Ok(sysinfo::proc_version().into_bytes()),
                "/runtime/processes" => Ok(self.processes.borrow().to_json().into_bytes()),
                "/runtime/memory" => Ok(serde_json::to_string(&self.memory.borrow().stats())
                    .unwrap_or_default()
                    .into_bytes()),
                "/runtime/services" => Ok(self.services.borrow().to_json().into_bytes()),
                "/runtime/scheduler" => Ok(serde_json::to_string(&self.scheduler.borrow().stats())
                    .unwrap_or_default()
                    .into_bytes()),
                "/runtime/uptime" => Ok(sysinfo::uptime((self.tick)()).into_bytes()),
                _ => Err(FsError::NotFound),
            },
        }
    }

    fn stat(&self, path: &str) -> FsResult<FileStat> {
        let read = self.read(path).map(|b| b.len()).unwrap_or(0);
        Ok(FileStat {
            path: path.to_string(),
            file_type: FileType::File,
            size: read,
            perms: "r--r--r--".to_string(),
            owner: "root".to_string(),
            group: "root".to_string(),
            links: 1,
            mtime: (self.tick)(),
        })
    }
}

fn is_pid_path(path: &str) -> bool {
    parse_pid(path).is_ok()
}

fn parse_pid(path: &str) -> FsResult<u32> {
    let trimmed = path.trim_start_matches('/');
    if trimmed.contains('/') {
        return trimmed
            .split('/')
            .next()
            .unwrap_or("")
            .parse()
            .map_err(|_| FsError::NotFound);
    }
    trimmed.parse().map_err(|_| FsError::NotFound)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn booted_system() -> System {
        let mut s = System::new();
        s.boot();
        for _ in 0..50 {
            s.tick_kernel();
        }
        s
    }

    #[test]
    fn boots_into_ready_state() {
        let s = booted_system();
        assert_eq!(s.services.borrow().healthy(), 8);
        assert!(s.processes.borrow().get(1).is_some(), "init is pid 1");
        assert!(s.fs.borrow().exists("/proc/meminfo"));
        assert!(s.fs.borrow().exists("/home/nikhil"));
    }

    #[test]
    fn proc_meminfo_is_live() {
        let s = booted_system();
        let meminfo = s.fs.borrow().read_text("/proc/meminfo").unwrap();
        assert!(meminfo.contains("MemTotal:"));
    }

    #[test]
    fn shell_runs_against_real_state() {
        let s = booted_system();
        let out = s.run_command("neofetch");
        assert!(out.contains("NIKHIL//OS"));
        let ps_out = s.run_command("ps");
        assert!(ps_out.contains("init"));
    }

    #[test]
    fn scheduler_progresses() {
        let mut s = System::new();
        s.boot();
        for _ in 0..200 {
            s.tick_kernel();
        }
        assert!(s.scheduler.borrow().stats().context_switches > 0);
        assert!(s.cpu_utilization() > 0.0);
    }

    #[test]
    fn snapshot_contains_real_telemetry() {
        let s = booted_system();
        let json = s.snapshot_json();
        assert!(json.contains("\"processes\""));
        assert!(json.contains("\"memory\""));
        assert!(json.contains("\"tick\""));
    }
}
