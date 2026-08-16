//! `nish` shell — built-in commands.
//!
//! Builtins are not fake: every command reads or writes real core state
//! through the shared references (`ShellContext`). `ps`, `free`, `cat /proc/...`,
//! and `neofetch` all reflect the live system.

use super::{ShellContext, ShellState};
use crate::filesystem::{normalize_path, FileType};
use crate::sysinfo;
use std::rc::Rc;

pub struct BuiltinResult {
    pub output: String,
    pub exit: i32,
}

impl BuiltinResult {
    pub fn ok(output: impl Into<String>) -> Self {
        Self {
            output: output.into(),
            exit: 0,
        }
    }
    pub fn err(output: impl Into<String>) -> Self {
        Self {
            output: output.into(),
            exit: 1,
        }
    }
    pub fn code(output: impl Into<String>, exit: i32) -> Self {
        Self {
            output: output.into(),
            exit,
        }
    }
}

/// Clear-screen escape sequence the terminal UI honors.
pub const CLEAR_SCREEN: &str = "\u{1b}[2J\u{1b}[H";

/// Run a builtin if `name` matches one. Returns `None` if the command is not
/// a builtin (caller reports "command not found").
pub fn run(
    name: &str,
    args: &[String],
    stdin: Option<&str>,
    ctx: &Rc<ShellContext>,
    state: &mut ShellState,
) -> Option<BuiltinResult> {
    let result = match name {
        "echo" => BuiltinResult::ok(args.join(" ")),
        "pwd" => BuiltinResult::ok(state.cwd.clone()),
        "clear" => BuiltinResult::code(CLEAR_SCREEN, 0),
        "exit" => BuiltinResult::code("", 0),
        "help" => BuiltinResult::ok(help_text()),
        "history" => BuiltinResult::ok(
            state
                .history
                .iter()
                .enumerate()
                .map(|(i, c)| format!("{:>4}  {}", i + 1, c))
                .collect::<Vec<_>>()
                .join("\n"),
        ),
        "alias" => alias(args, state),
        "unalias" => unalias(args, state),
        "export" => export(args, state),
        "env" => BuiltinResult::ok(
            state
                .env
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join("\n"),
        ),
        "which" => which(args, ctx),
        "ls" => ls(args, ctx, state),
        "cd" => cd(args, ctx, state),
        "cat" => cat(args, stdin, ctx, state),
        "mkdir" => mkdir(args, ctx, state),
        "touch" => touch(args, ctx, state),
        "rm" => rm(args, ctx, state),
        "mv" => mv(args, ctx, state),
        "cp" => cp(args, ctx, state),
        "grep" => grep(args, stdin, ctx, state),
        "find" => find(args, ctx, state),
        "ps" => ps(ctx),
        "top" => top(ctx),
        "kill" => kill(args, ctx),
        "free" => free(ctx),
        "df" => df(ctx),
        "mount" => mount(ctx),
        "uname" => BuiltinResult::ok(sysinfo::uname()),
        "neofetch" => neofetch(ctx),
        "hostname" => hostname(ctx),
        "service" => service(args, ctx),
        "pkgctl" => pkgctl(args, ctx),
        "ai" => BuiltinResult::ok(
            "AI Core: the AI runtime is part of the next build phase.\n\
             Planned intents: SEARCH_PROJECTS, SEARCH_SKILLS, OPEN_APPLICATION, JOB_ANALYSIS.",
        ),
        "graph" => BuiltinResult::ok(
            "Knowledge graph: relationships are defined in knowledge/data; the graph\n\
             visualization ships with the AI/knowledge phase.",
        ),
        "career" => BuiltinResult::ok(
            "Career Simulator is a planned application (docs/04-ML-AI-SPEC.md §10).\n\
             Inputs: target role, specialization, desired technologies.",
        ),
        _ => return None,
    };
    Some(result)
}

fn help_text() -> String {
    r#"NIKHIL//OS nish shell

Filesystem:  ls cd pwd cat mkdir touch rm mv cp grep find
System:      ps top kill free df mount uname neofetch hostname service
Packages:    pkgctl search|install|remove|update|upgrade|info|list
Shell:       echo alias unalias export env which history clear help exit
AI:          ai graph career

Pipelines, redirection, and command chaining are supported:
  ps | grep ai        cat /proc/meminfo > mem.txt        cd .. && ls
"#
    .to_string()
}

fn alias(args: &[String], state: &mut ShellState) -> BuiltinResult {
    if args.is_empty() {
        let out = state
            .aliases
            .iter()
            .map(|(k, v)| format!("alias {k}='{v}'"))
            .collect::<Vec<_>>()
            .join("\n");
        return BuiltinResult::ok(out);
    }
    let mut out = String::new();
    for arg in args {
        if let Some(eq) = arg.find('=') {
            let key = &arg[..eq];
            let val = &arg[eq + 1..];
            state.aliases.insert(key.to_string(), val.to_string());
        } else {
            if let Some(val) = state.aliases.get(arg) {
                out.push_str(&format!("alias {arg}='{val}'\n"));
            }
        }
    }
    BuiltinResult::ok(out)
}

fn unalias(args: &[String], state: &mut ShellState) -> BuiltinResult {
    for arg in args {
        state.aliases.remove(arg);
    }
    BuiltinResult::ok("")
}

fn export(args: &[String], state: &mut ShellState) -> BuiltinResult {
    for arg in args {
        if let Some(eq) = arg.find('=') {
            state
                .env
                .insert(arg[..eq].to_string(), arg[eq + 1..].to_string());
        } else {
            state.env.insert(arg.to_string(), String::new());
        }
    }
    BuiltinResult::ok("")
}

fn which(args: &[String], ctx: &Rc<ShellContext>) -> BuiltinResult {
    let builtins = [
        "echo", "pwd", "ls", "cd", "cat", "mkdir", "touch", "rm", "mv", "cp", "grep", "find", "ps",
        "top", "kill", "free", "df", "mount", "uname", "neofetch", "hostname", "service", "pkgctl",
        "alias", "unalias", "export", "env", "history", "which", "clear", "help", "exit",
    ];
    let mut out = String::new();
    for arg in args {
        if builtins.contains(&arg.as_str()) {
            out.push_str(&format!("{arg}: shell builtin\n"));
        } else if ctx.packages.borrow().get(arg).is_some() {
            out.push_str(&format!("{arg}: /usr/bin/{arg} (package)\n"));
        } else {
            out.push_str(&format!("{arg}: not found\n"));
        }
    }
    BuiltinResult::ok(out)
}

// ---- filesystem builtins -------------------------------------------------

fn path_for(state: &ShellState, path: &str) -> String {
    if path.starts_with('/') {
        normalize_path(path)
    } else {
        normalize_path(&format!("{}/{}", state.cwd, path))
    }
}

fn ls(args: &[String], ctx: &Rc<ShellContext>, state: &ShellState) -> BuiltinResult {
    let mut long = false;
    let mut all = false;
    let mut target: Option<&str> = None;
    for arg in args {
        match arg.as_str() {
            "-l" => long = true,
            "-la" | "-al" => {
                long = true;
                all = true;
            }
            "-a" => all = true,
            _ if arg.starts_with('-') => {
                return BuiltinResult::err(format!("ls: unknown option {arg}"))
            }
            _ => target = Some(arg),
        }
    }
    let path = path_for(state, target.unwrap_or("."));
    let entries = match ctx.fs.borrow().list(&path) {
        Ok(e) => e,
        Err(e) => return BuiltinResult::err(format!("ls: {path}: {e}")),
    };
    let mut out = String::new();
    for e in entries {
        if e.name.starts_with('.') && !all {
            continue;
        }
        if long {
            let kind = if e.file_type == FileType::Directory {
                'd'
            } else {
                '-'
            };
            out.push_str(&format!(
                "{kind}{} {:>8}  {:<6} {:<6} {} {}\n",
                e.perms, e.size, e.owner, e.group, "", e.name
            ));
        } else {
            let suffix = if e.file_type == FileType::Directory {
                "/"
            } else {
                ""
            };
            out.push_str(&format!("{}{}\t", e.name, suffix));
        }
    }
    if !long {
        out.push('\n');
    }
    BuiltinResult::ok(out)
}

fn cd(args: &[String], ctx: &Rc<ShellContext>, state: &mut ShellState) -> BuiltinResult {
    let target = args.first().map(|s| s.as_str()).unwrap_or("~");
    let resolved = if target == "~" {
        state
            .env
            .get("HOME")
            .cloned()
            .unwrap_or_else(|| "/home/nikhil".into())
    } else if target == "-" {
        state
            .env
            .get("OLDPWD")
            .cloned()
            .unwrap_or_else(|| state.cwd.clone())
    } else {
        path_for(state, target)
    };
    let fs = ctx.fs.borrow();
    match fs.stat(&resolved) {
        Ok(st) if st.file_type == FileType::Directory => {
            state.env.insert("OLDPWD".into(), state.cwd.clone());
            state.cwd = resolved;
            BuiltinResult::ok("")
        }
        Ok(_) => BuiltinResult::err(format!("cd: {target}: not a directory")),
        Err(e) => BuiltinResult::err(format!("cd: {target}: {e}")),
    }
}

fn cat(
    args: &[String],
    stdin: Option<&str>,
    ctx: &Rc<ShellContext>,
    state: &ShellState,
) -> BuiltinResult {
    if args.is_empty() {
        return BuiltinResult::ok(stdin.unwrap_or(""));
    }
    let mut out = String::new();
    let fs = ctx.fs.borrow();
    for arg in args {
        let path = path_for(state, arg);
        match fs.read_text(&path) {
            Ok(text) => out.push_str(&text),
            Err(e) => out.push_str(&format!("cat: {arg}: {e}\n")),
        }
    }
    BuiltinResult::ok(out)
}

fn mkdir(args: &[String], ctx: &Rc<ShellContext>, state: &mut ShellState) -> BuiltinResult {
    let mut parents = false;
    let mut targets = Vec::new();
    for arg in args {
        match arg.as_str() {
            "-p" => parents = true,
            _ if arg.starts_with('-') => {
                return BuiltinResult::err(format!("mkdir: unknown option {arg}"))
            }
            _ => targets.push(arg.clone()),
        }
    }
    if targets.is_empty() {
        return BuiltinResult::err("mkdir: missing operand");
    }
    let mut fs = ctx.fs.borrow_mut();
    let tick = (ctx.tick)();
    for target in targets {
        let path = path_for(state, &target);
        let result = fs.mkdir(&path, "nikhil", "users", tick);
        if let Err(e) = result {
            if parents {
                // create each missing ancestor
                let mut acc = String::from("/");
                for part in path.trim_start_matches('/').split('/') {
                    acc = normalize_path(&format!("{acc}/{part}"));
                    if !fs.exists(&acc) {
                        let _ = fs.mkdir(&acc, "nikhil", "users", tick);
                    }
                }
            } else {
                return BuiltinResult::err(format!("mkdir: {target}: {e}"));
            }
        }
    }
    BuiltinResult::ok("")
}

fn touch(args: &[String], ctx: &Rc<ShellContext>, state: &mut ShellState) -> BuiltinResult {
    if args.is_empty() {
        return BuiltinResult::err("touch: missing operand");
    }
    let mut fs = ctx.fs.borrow_mut();
    let tick = (ctx.tick)();
    for arg in args {
        let path = path_for(state, arg);
        if let Err(e) = fs.touch(&path, "nikhil", "users", tick) {
            return BuiltinResult::err(format!("touch: {arg}: {e}"));
        }
    }
    BuiltinResult::ok("")
}

fn rm(args: &[String], ctx: &Rc<ShellContext>, state: &mut ShellState) -> BuiltinResult {
    let mut recursive = false;
    let mut targets = Vec::new();
    for arg in args {
        match arg.as_str() {
            "-r" | "-rf" | "-fr" => recursive = true,
            _ => targets.push(arg.clone()),
        }
    }
    if targets.is_empty() {
        return BuiltinResult::err("rm: missing operand");
    }
    let mut fs = ctx.fs.borrow_mut();
    for target in targets {
        let path = path_for(state, &target);
        let result = if recursive {
            fs.remove_all(&path)
        } else {
            fs.remove(&path)
        };
        if let Err(e) = result {
            return BuiltinResult::err(format!("rm: {target}: {e}"));
        }
    }
    BuiltinResult::ok("")
}

fn mv(args: &[String], ctx: &Rc<ShellContext>, state: &mut ShellState) -> BuiltinResult {
    if args.len() < 2 {
        return BuiltinResult::err("mv: missing operand");
    }
    let src = path_for(state, &args[0]);
    let dst = path_for(state, &args[1]);
    let mut fs = ctx.fs.borrow_mut();
    match fs.rename(&src, &dst) {
        Ok(()) => BuiltinResult::ok(""),
        Err(e) => BuiltinResult::err(format!("mv: {e}")),
    }
}

fn cp(args: &[String], ctx: &Rc<ShellContext>, state: &mut ShellState) -> BuiltinResult {
    if args.len() < 2 {
        return BuiltinResult::err("cp: missing operand");
    }
    let src = path_for(state, &args[0]);
    let dst = path_for(state, &args[1]);
    let data = match ctx.fs.borrow().read(&src) {
        Ok(d) => d,
        Err(e) => return BuiltinResult::err(format!("cp: {src}: {e}")),
    };
    let mut fs = ctx.fs.borrow_mut();
    let tick = (ctx.tick)();
    match fs.write(&dst, &data, "nikhil", "users", tick) {
        Ok(()) => BuiltinResult::ok(""),
        Err(e) => BuiltinResult::err(format!("cp: {e}")),
    }
}

fn grep(
    args: &[String],
    stdin: Option<&str>,
    ctx: &Rc<ShellContext>,
    state: &ShellState,
) -> BuiltinResult {
    if args.is_empty() {
        return BuiltinResult::err("grep: missing pattern");
    }
    let pattern = &args[0];
    let mut out = String::new();
    let text: String = if args.len() > 1 {
        let path = path_for(state, &args[1]);
        match ctx.fs.borrow().read_text(&path) {
            Ok(t) => t,
            Err(e) => return BuiltinResult::err(format!("grep: {}: {}", args[1], e)),
        }
    } else if let Some(stdin) = stdin {
        stdin.to_string()
    } else {
        return BuiltinResult::err("grep: no input");
    };
    for line in text.lines() {
        if line.contains(pattern) {
            out.push_str(line);
            out.push('\n');
        }
    }
    let exit = if out.is_empty() { 1 } else { 0 };
    BuiltinResult::code(out, exit)
}

fn find(args: &[String], ctx: &Rc<ShellContext>, state: &ShellState) -> BuiltinResult {
    let root = path_for(state, args.first().map(|s| s.as_str()).unwrap_or("."));
    let pattern = args.get(1).map(|s| s.as_str()).unwrap_or("");
    let mut out = String::new();
    walk(ctx, &root, pattern, &mut out);
    BuiltinResult::ok(out)
}

fn walk(ctx: &Rc<ShellContext>, path: &str, pattern: &str, out: &mut String) {
    let fs = ctx.fs.borrow();
    let name = path.rsplit('/').next().unwrap_or(path);
    if pattern.is_empty() || name.contains(pattern) {
        out.push_str(path);
        out.push('\n');
    }
    if let Ok(entries) = fs.list(path) {
        for e in entries {
            if e.file_type == FileType::Directory {
                walk(ctx, &format!("{}/{}", path, e.name), pattern, out);
            } else if pattern.is_empty() || e.name.contains(pattern) {
                out.push_str(&format!("{}/{}\n", path, e.name));
            }
        }
    }
}

// ---- system builtins -----------------------------------------------------

fn ps(ctx: &Rc<ShellContext>) -> BuiltinResult {
    let pm = ctx.processes.borrow();
    let mut out = String::from("  PID  PPID  STATE      CPU%   MEM(kB)  NAME\n");
    for p in pm.alive() {
        out.push_str(&format!(
            "{:>5} {:>5}  {:<10} {:>4.1} {:>8}  {}\n",
            p.pid,
            p.parent_pid,
            format!("{:?}", p.state).to_lowercase(),
            p.cpu_usage,
            p.memory_kb,
            p.name
        ));
    }
    BuiltinResult::ok(out)
}

fn top(ctx: &Rc<ShellContext>) -> BuiltinResult {
    let pm = ctx.processes.borrow();
    let mem = ctx.memory.borrow().stats();
    let mut procs: Vec<_> = pm.alive().into_iter().collect();
    procs.sort_by(|a, b| {
        b.cpu_usage
            .partial_cmp(&a.cpu_usage)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mem_pct = mem
        .used_kb
        .saturating_mul(100)
        .checked_div(mem.total_kb)
        .unwrap_or(0);
    let mut out = format!(
        "top - simulated runtime    processes: {}    memory: {}% used\n\n  PID  STATE      CPU%   MEM(kB)  NAME\n",
        procs.len(),
        mem_pct
    );
    for p in procs.iter().take(12) {
        out.push_str(&format!(
            "{:>5}  {:<10} {:>4.1} {:>8}  {}\n",
            p.pid,
            format!("{:?}", p.state).to_lowercase(),
            p.cpu_usage,
            p.memory_kb,
            p.name
        ));
    }
    BuiltinResult::ok(out)
}

fn kill(args: &[String], ctx: &Rc<ShellContext>) -> BuiltinResult {
    if args.is_empty() {
        return BuiltinResult::err("kill: missing operand");
    }
    let mut pm = ctx.processes.borrow_mut();
    let mut mem = ctx.memory.borrow_mut();
    let mut out = String::new();
    for arg in args {
        let signal = arg.strip_prefix('-');
        if let Some(sig) = signal {
            out.push_str(&format!(
                "kill: signal {sig} (not delivered in simulation)\n"
            ));
            continue;
        }
        match arg.parse::<u32>() {
            Ok(pid) => {
                if pm.kill(pid) {
                    mem.free(pid);
                    out.push_str(&format!("killed pid={pid}\n"));
                } else {
                    out.push_str(&format!("kill: no such process: {pid}\n"));
                }
            }
            Err(_) => out.push_str(&format!("kill: invalid pid: {arg}\n")),
        }
    }
    BuiltinResult::ok(out)
}

fn free(ctx: &Rc<ShellContext>) -> BuiltinResult {
    BuiltinResult::ok(ctx.memory.borrow().free_report())
}

fn df(ctx: &Rc<ShellContext>) -> BuiltinResult {
    let fs = ctx.fs.borrow();
    let used = fs.inode_count();
    let total = 4096usize; // simulated disk: 4096 inodes
    let avail = total.saturating_sub(used);
    let pct = used.saturating_mul(100).checked_div(total).unwrap_or(0);
    BuiltinResult::ok(format!(
        "Filesystem     1K-blocks      Used  Available  Use%  Mounted on\n\
         sim-vfs          {:>8} {:>8}  {:>9}  {:>3}%  /\n\
         inodes used     {:>8} of {}",
        total, used, avail, pct, used, total
    ))
}

fn mount(ctx: &Rc<ShellContext>) -> BuiltinResult {
    let fs = ctx.fs.borrow();
    let mut out = String::from("/ on sim-vfs type vfs\n");
    for m in fs.mount_points() {
        out.push_str(&format!("{m} on sim-vfs type proc (virtual)\n"));
    }
    BuiltinResult::ok(out)
}

fn neofetch(ctx: &Rc<ShellContext>) -> BuiltinResult {
    BuiltinResult::ok(sysinfo::neofetch(
        &ctx.processes.borrow(),
        &ctx.memory.borrow(),
        &ctx.services.borrow(),
    ))
}

fn hostname(ctx: &Rc<ShellContext>) -> BuiltinResult {
    match ctx.fs.borrow().read_text("/etc/hostname") {
        Ok(h) => BuiltinResult::ok(h.trim_end().to_string()),
        Err(_) => BuiltinResult::ok("nikhil-os"),
    }
}

fn service(args: &[String], ctx: &Rc<ShellContext>) -> BuiltinResult {
    let action = args.first().map(|s| s.as_str()).unwrap_or("status");
    let mut services = ctx.services.borrow_mut();
    let tick = (ctx.tick)();
    match action {
        "status" => BuiltinResult::ok(services.status_report()),
        "start" => {
            let name = args.get(1);
            match name {
                Some(name) if services.get(name).is_some() => {
                    let pid = {
                        let mut pm = ctx.processes.borrow_mut();
                        pm.spawn(name, "root", 1, 128, &["sys_service"], tick, 1)
                    };
                    services.start(name, tick, pid);
                    BuiltinResult::ok(format!("service {name}: started (pid={pid})"))
                }
                Some(_) => BuiltinResult::err(format!("service: unknown service: {}", args[1])),
                None => BuiltinResult::err("service start: missing service name"),
            }
        }
        "stop" => {
            let name = args.get(1);
            match name {
                Some(name) if services.get(name).is_some() => {
                    services.stop(name);
                    BuiltinResult::ok(format!("service {name}: stopped"))
                }
                Some(_) => BuiltinResult::err(format!("service: unknown service: {}", args[1])),
                None => BuiltinResult::err("service stop: missing service name"),
            }
        }
        "restart" => {
            let name = args.get(1);
            match name {
                Some(name) if services.get(name).is_some() => {
                    services.stop(name);
                    let pid = {
                        let mut pm = ctx.processes.borrow_mut();
                        pm.spawn(name, "root", 1, 128, &["sys_service"], tick, 1)
                    };
                    services.start(name, tick, pid);
                    BuiltinResult::ok(format!("service {name}: restarted (pid={pid})"))
                }
                Some(_) => BuiltinResult::err(format!("service: unknown service: {}", args[1])),
                None => BuiltinResult::err("service restart: missing service name"),
            }
        }
        _ => BuiltinResult::err(format!("service: unknown action: {action}")),
    }
}

fn pkgctl(args: &[String], ctx: &Rc<ShellContext>) -> BuiltinResult {
    let action = args.first().map(|s| s.as_str()).unwrap_or("list");
    let mut packages = ctx.packages.borrow_mut();
    match action {
        "list" => {
            let all = args
                .get(1)
                .map(|s| s == "-a" || s == "--all")
                .unwrap_or(false);
            BuiltinResult::ok(packages.list_report(all))
        }
        "search" => {
            let q = args.get(1).map(|s| s.as_str()).unwrap_or("");
            let results = packages.search(q);
            let mut out = String::from("NAME                    VERSION\n");
            for m in results {
                out.push_str(&format!(
                    "{:<24} {:<10} {}\n",
                    m.name, m.version, m.description
                ));
            }
            BuiltinResult::ok(out)
        }
        "info" => match args.get(1) {
            Some(name) => match packages.info_report(name) {
                Ok(info) => BuiltinResult::ok(info),
                Err(e) => BuiltinResult::err(e),
            },
            None => BuiltinResult::err("pkgctl info: missing package name"),
        },
        "install" => match args.get(1) {
            Some(name) => match packages.install(name) {
                Ok(()) => BuiltinResult::ok(format!("installed {name}")),
                Err(e) => BuiltinResult::err(e),
            },
            None => BuiltinResult::err("pkgctl install: missing package name"),
        },
        "remove" => match args.get(1) {
            Some(name) => match packages.remove(name) {
                Ok(()) => BuiltinResult::ok(format!("removed {name}")),
                Err(e) => BuiltinResult::err(e),
            },
            None => BuiltinResult::err("pkgctl remove: missing package name"),
        },
        "update" => BuiltinResult::ok("pkgctl: package index is up to date"),
        "upgrade" => BuiltinResult::ok("pkgctl: all packages are up to date"),
        _ => BuiltinResult::err(format!("pkgctl: unknown action: {action}")),
    }
}
