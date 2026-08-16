//! `nish` — the NIKHIL//OS shell.
//!
//! ```text
//! Input → Lexer → Parser → AST → Executor → Services / syscalls
//! ```
//!
//! Commands are never handled in the UI; they execute here against real core
//! state. Pipes, redirection, environment variables, aliases, history,
//! autocomplete, and exit codes are all implemented.

pub mod builtins;
pub mod lexer;
pub mod parser;

use crate::event::EventBus;
use crate::filesystem::FileSystem;
use crate::ipc::IpcBus;
use crate::memory::MemoryManager;
use crate::package::PackageRegistry;
use crate::process::ProcessManager;
use crate::scheduler::RoundRobin;
use crate::service::ServiceRegistry;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

pub use crate::shell::parser::Node;

/// Shared references to kernel subsystems, handed to the shell (and to
/// builtins) so they operate on real state.
pub struct ShellContext {
    pub fs: Rc<RefCell<FileSystem>>,
    pub processes: Rc<RefCell<ProcessManager>>,
    pub memory: Rc<RefCell<MemoryManager>>,
    pub services: Rc<RefCell<ServiceRegistry>>,
    pub packages: Rc<RefCell<PackageRegistry>>,
    pub scheduler: Rc<RefCell<RoundRobin>>,
    pub events: Rc<EventBus>,
    pub ipc: Rc<IpcBus>,
    pub tick: Rc<dyn Fn() -> u64>,
}

/// Mutable shell state (per session).
#[derive(Default)]
pub struct ShellState {
    pub cwd: String,
    pub env: BTreeMap<String, String>,
    pub aliases: BTreeMap<String, String>,
    pub history: Vec<String>,
    pub last_exit: i32,
}

impl ShellState {
    fn new(home: &str) -> Self {
        let mut env = BTreeMap::new();
        env.insert("HOME".to_string(), home.to_string());
        env.insert("SHELL".to_string(), "/bin/nish".to_string());
        env.insert(
            "PATH".to_string(),
            "/bin:/usr/bin:/usr/local/bin".to_string(),
        );
        env.insert(
            "USER".to_string(),
            home.trim_start_matches("/home/").to_string(),
        );
        Self {
            cwd: home.to_string(),
            env,
            ..Self::default()
        }
    }
}

/// The shell session.
pub struct Shell {
    ctx: Rc<ShellContext>,
    state: ShellState,
}

pub struct RunResult {
    pub output: String,
    pub exit_code: i32,
}

impl Shell {
    pub fn new(ctx: Rc<ShellContext>, user_name: &str) -> Self {
        let home = format!("/home/{user_name}");
        let mut shell = Self {
            ctx,
            state: ShellState::new(&home),
        };
        shell
            .state
            .aliases
            .insert("ll".to_string(), "ls -l".to_string());
        shell
            .state
            .aliases
            .insert("..".to_string(), "cd ..".to_string());
        shell
            .state
            .aliases
            .insert("la".to_string(), "ls -la".to_string());
        shell
    }

    /// Run one line of input and return output + exit code.
    pub fn run_line(&mut self, input: &str) -> RunResult {
        if !parser::is_blank(input) {
            self.state.history.push(input.to_string());
            if self.state.history.len() > 500 {
                self.state.history.remove(0);
            }
        }
        let expanded = self.expand_aliases(input);
        let node = parser::parse(&expanded);
        let mut result = RunResult {
            output: String::new(),
            exit_code: 0,
        };
        self.execute(&node, None, &mut result);
        self.state.last_exit = result.exit_code;
        result
    }

    pub fn cwd(&self) -> &str {
        &self.state.cwd
    }

    pub fn history(&self) -> &[String] {
        &self.state.history
    }

    /// Candidates for autocomplete given a partial word.
    pub fn autocomplete(&self, prefix: &str) -> Vec<String> {
        let mut names: Vec<String> = BUILTIN_NAMES.iter().map(|s| s.to_string()).collect();
        for m in self.ctx.packages.borrow().available() {
            names.push(m.name.clone());
        }
        if let Ok(entries) = self.ctx.fs.borrow().list(&self.state.cwd) {
            for e in entries {
                names.push(e.name.clone());
            }
        }
        names.sort();
        names.dedup();
        names
            .into_iter()
            .filter(|n| n.starts_with(prefix))
            .collect()
    }

    // -- expansion ---------------------------------------------------------

    fn expand_aliases(&self, input: &str) -> String {
        let trimmed = input.trim_start();
        let indent = &input[..input.len() - trimmed.len()];
        let first_word: String = trimmed
            .chars()
            .take_while(|c| !c.is_whitespace() && *c != '|' && *c != '>')
            .collect();
        if let Some(expansion) = self.state.aliases.get(&first_word) {
            format!("{indent}{expansion}{}", &trimmed[first_word.len()..])
        } else {
            input.to_string()
        }
    }

    fn expand_vars(&self, word: &str) -> String {
        let mut out = String::new();
        let mut chars = word.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '$' {
                if let Some(next) = chars.peek() {
                    if *next == '?' {
                        chars.next();
                        out.push_str(&self.state.last_exit.to_string());
                        continue;
                    }
                    if *next == '$' {
                        chars.next();
                        out.push('0');
                        continue;
                    }
                }
                let mut name = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_alphanumeric() || c == '_' {
                        name.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if !name.is_empty() {
                    let value = self.state.env.get(&name).cloned().unwrap_or_default();
                    out.push_str(&value);
                } else {
                    out.push('$');
                }
            } else {
                out.push(c);
            }
        }
        out
    }

    // -- execution ---------------------------------------------------------

    fn execute(&mut self, node: &Node, stdin: Option<&str>, out: &mut RunResult) {
        match node {
            Node::Simple(cmd) => self.execute_simple(cmd, stdin, out),
            Node::Pipe(left, right) => {
                let mut left_out = RunResult {
                    output: String::new(),
                    exit_code: 0,
                };
                // The left command's stdout becomes the right command's stdin.
                // Only the final stage's output reaches the caller.
                self.execute(left, stdin, &mut left_out);
                self.execute(right, Some(&left_out.output), out);
            }
            Node::Redir { cmd, dir, target } => {
                let target = self.expand_vars(target);
                match dir {
                    parser::RedirDir::In => {
                        let data = match self.ctx.fs.borrow().read_text(&target) {
                            Ok(d) => d,
                            Err(e) => {
                                out.output.push_str(&format!("nish: {target}: {e}\n"));
                                out.exit_code = 1;
                                return;
                            }
                        };
                        self.execute(cmd, Some(&data), out);
                    }
                    parser::RedirDir::Out | parser::RedirDir::Append => {
                        let mut inner = RunResult {
                            output: String::new(),
                            exit_code: 0,
                        };
                        self.execute(cmd, stdin, &mut inner);
                        let path = self.resolve_path(&target);
                        let tick = (self.ctx.tick)();
                        let user = self
                            .state
                            .env
                            .get("USER")
                            .cloned()
                            .unwrap_or_else(|| "nikhil".into());
                        let result = match dir {
                            parser::RedirDir::Append => self.ctx.fs.borrow_mut().append(
                                &path,
                                inner.output.as_bytes(),
                                &user,
                                "users",
                                tick,
                            ),
                            _ => self.ctx.fs.borrow_mut().write(
                                &path,
                                inner.output.as_bytes(),
                                &user,
                                "users",
                                tick,
                            ),
                        };
                        if let Err(e) = result {
                            out.output.push_str(&format!("nish: {target}: {e}\n"));
                            out.exit_code = 1;
                            return;
                        }
                        out.exit_code = inner.exit_code;
                    }
                }
            }
            Node::Sequence(left, right) => {
                self.execute(left, stdin, out);
                self.execute(right, None, out);
            }
            Node::And(left, right) => {
                let mut first = RunResult {
                    output: String::new(),
                    exit_code: 0,
                };
                self.execute(left, stdin, &mut first);
                out.output.push_str(&first.output);
                if first.exit_code == 0 {
                    self.execute(right, None, out);
                }
            }
            Node::Or(left, right) => {
                let mut first = RunResult {
                    output: String::new(),
                    exit_code: 0,
                };
                self.execute(left, stdin, &mut first);
                out.output.push_str(&first.output);
                if first.exit_code != 0 {
                    self.execute(right, None, out);
                }
            }
        }
    }

    fn execute_simple(
        &mut self,
        cmd: &parser::SimpleCmd,
        stdin: Option<&str>,
        out: &mut RunResult,
    ) {
        // Apply command-scoped assignments to a snapshot of the environment.
        for (k, v) in &cmd.assignments {
            let key = self.expand_vars(k);
            let val = self.expand_vars(v);
            self.state.env.insert(key, val);
        }

        let name = self.expand_vars(&cmd.name);
        let args: Vec<String> = cmd.args.iter().map(|a| self.expand_vars(a)).collect();

        if name.is_empty() {
            return;
        }

        if let Some(result) = builtins::run(&name, &args, stdin, &self.ctx, &mut self.state) {
            out.output.push_str(&result.output);
            out.exit_code = result.exit;
            return;
        }

        // Try to spawn as a package binary.
        if self.ctx.packages.borrow().get(&name).is_some() {
            out.output.push_str(&format!(
                "nish: {name}: package has no runnable entrypoint in this edition\n"
            ));
            out.exit_code = 1;
            return;
        }

        out.output
            .push_str(&format!("nish: command not found: {name}\n"));
        out.exit_code = 127;
    }

    fn resolve_path(&self, path: &str) -> String {
        if path.starts_with('/') {
            crate::filesystem::normalize_path(path)
        } else {
            crate::filesystem::normalize_path(&format!("{}/{}", self.state.cwd, path))
        }
    }
}

/// Builtin command names (also used for autocomplete).
pub const BUILTIN_NAMES: &[&str] = &[
    "ai", "alias", "cat", "cd", "clear", "cp", "df", "echo", "env", "exit", "export", "find",
    "free", "graph", "grep", "help", "history", "hostname", "kill", "ls", "mkdir", "mount", "mv",
    "neofetch", "pkgctl", "ps", "pwd", "rm", "service", "top", "touch", "unalias", "uname",
    "which", "career",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filesystem::FileSystem;

    fn ctx() -> Rc<ShellContext> {
        let fs = Rc::new(RefCell::new(FileSystem::new()));
        fs.borrow_mut().mkdir("/home", "root", "root", 0).unwrap();
        fs.borrow_mut()
            .mkdir("/home/nikhil", "nikhil", "users", 0)
            .unwrap();
        fs.borrow_mut()
            .mkdir("/home/nikhil/projects", "nikhil", "users", 0)
            .unwrap();
        fs.borrow_mut()
            .touch("/home/nikhil/projects/ai.rs", "nikhil", "users", 0)
            .unwrap();
        fs.borrow_mut()
            .write(
                "/home/nikhil/projects/ai.rs",
                b"// ai core",
                "nikhil",
                "users",
                0,
            )
            .unwrap();

        let processes = Rc::new(RefCell::new(ProcessManager::new()));
        let memory = Rc::new(RefCell::new(MemoryManager::new(16 * 1024)));
        let services = Rc::new(RefCell::new(ServiceRegistry::new()));
        let packages = Rc::new(RefCell::new(PackageRegistry::new()));
        let scheduler = Rc::new(RefCell::new(RoundRobin::new(5)));
        let events = Rc::new(EventBus::new());
        let ipc = Rc::new(IpcBus::new());
        let tick: Rc<dyn Fn() -> u64> = Rc::new(|| 0);
        Rc::new(ShellContext {
            fs,
            processes,
            memory,
            services,
            packages,
            scheduler,
            events,
            ipc,
            tick,
        })
    }

    #[test]
    fn simple_command_and_pwd() {
        let mut shell = Shell::new(ctx(), "nikhil");
        let r = shell.run_line("pwd");
        assert_eq!(r.output.trim(), "/home/nikhil");
        assert_eq!(r.exit_code, 0);
    }

    #[test]
    fn pipeline_ps_grep() {
        let mut shell = Shell::new(ctx(), "nikhil");
        // spawn a couple of processes
        {
            let mut pm = shell.ctx.processes.borrow_mut();
            pm.spawn("ai-worker", "nikhil", 1, 128, &[], 0, 1);
            pm.spawn("desktop", "nikhil", 1, 256, &[], 0, 1);
        }
        let r = shell.run_line("ps | grep ai");
        assert!(r.output.contains("ai-worker"), "output was: {}", r.output);
        assert!(!r.output.contains("desktop"));
    }

    #[test]
    fn redirection_writes_file() {
        let mut shell = Shell::new(ctx(), "nikhil");
        let r = shell.run_line("echo hello > note.txt");
        assert_eq!(r.exit_code, 0);
        let content = shell
            .ctx
            .fs
            .borrow()
            .read_text("/home/nikhil/note.txt")
            .unwrap();
        assert_eq!(content, "hello");
        let r2 = shell.run_line("cat note.txt");
        assert_eq!(r2.output, "hello");
    }

    #[test]
    fn variables_and_exit_codes() {
        let mut shell = Shell::new(ctx(), "nikhil");
        shell.run_line("export GREETING=hi");
        let r = shell.run_line("echo $GREETING");
        assert_eq!(r.output.trim(), "hi");
        let r2 = shell.run_line("ls /nonexistent-path-xyz");
        assert_ne!(r2.exit_code, 0);
        let r3 = shell.run_line("echo $?");
        assert_ne!(r3.output.trim(), "0");
    }

    #[test]
    fn unknown_command_is_127() {
        let mut shell = Shell::new(ctx(), "nikhil");
        let r = shell.run_line("definitely-not-a-command");
        assert_eq!(r.exit_code, 127);
    }

    #[test]
    fn autocomplete_suggests() {
        let shell = Shell::new(ctx(), "nikhil");
        let names = shell.autocomplete("pk");
        assert!(names.iter().any(|n| n == "pkgctl"));
    }
}
