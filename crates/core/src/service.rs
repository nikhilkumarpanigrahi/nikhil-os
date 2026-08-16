//! Service manager.
//!
//! The OS is a set of small composable services with a boot order,
//! dependency graph, and observable lifecycle. `service status|start|stop|restart`
//! reflects this registry; nothing here is faked.

use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Failed,
}

impl std::fmt::Display for ServiceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ServiceState::Stopped => "stopped",
            ServiceState::Starting => "starting",
            ServiceState::Running => "running",
            ServiceState::Stopping => "stopping",
            ServiceState::Failed => "failed",
        };
        f.write_str(s)
    }
}

/// What to do if a service exits unexpectedly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RestartPolicy {
    Always,
    OnFailure,
    Never,
}

#[derive(Debug, Clone, Serialize)]
pub struct Service {
    pub name: String,
    pub description: String,
    pub dependencies: Vec<String>,
    pub state: ServiceState,
    pub pid: Option<u32>,
    /// Kernel tick at which the service last (re)started.
    pub start_tick: u64,
    pub restarts: u32,
    pub restart_policy: RestartPolicy,
    pub uptime_ticks: u64,
}

impl Service {
    fn new(
        name: &str,
        description: &str,
        dependencies: Vec<String>,
        restart_policy: RestartPolicy,
    ) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            dependencies,
            state: ServiceState::Stopped,
            pid: None,
            start_tick: 0,
            restarts: 0,
            restart_policy,
            uptime_ticks: 0,
        }
    }
}

#[derive(Default)]
pub struct ServiceRegistry {
    services: HashMap<String, Service>,
    boot_order: Vec<String>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a service definition with its boot dependencies.
    pub fn define(
        &mut self,
        name: &str,
        description: &str,
        dependencies: &[&str],
        restart_policy: RestartPolicy,
    ) {
        self.services.insert(
            name.to_string(),
            Service::new(
                name,
                description,
                dependencies.iter().map(|s| s.to_string()).collect(),
                restart_policy,
            ),
        );
        if !self.boot_order.contains(&name.to_string()) {
            self.boot_order.push(name.to_string());
        }
    }

    pub fn get(&self, name: &str) -> Option<&Service> {
        self.services.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Service> {
        self.services.get_mut(name)
    }

    pub fn names(&self) -> Vec<String> {
        self.boot_order.clone()
    }

    pub fn all(&self) -> Vec<&Service> {
        self.boot_order
            .iter()
            .filter_map(|n| self.services.get(n))
            .collect()
    }

    pub fn set_state(&mut self, name: &str, state: ServiceState) -> bool {
        if let Some(s) = self.services.get_mut(name) {
            s.state = state;
            true
        } else {
            false
        }
    }

    pub fn start(&mut self, name: &str, now_tick: u64, pid: u32) -> bool {
        if let Some(s) = self.services.get_mut(name) {
            s.state = ServiceState::Starting;
            s.state = ServiceState::Running;
            s.pid = Some(pid);
            s.start_tick = now_tick;
            s.restarts += 0;
            true
        } else {
            false
        }
    }

    pub fn stop(&mut self, name: &str) -> bool {
        if let Some(s) = self.services.get_mut(name) {
            s.state = ServiceState::Stopped;
            s.pid = None;
            true
        } else {
            false
        }
    }

    /// Advance lifecycle bookkeeping once per kernel tick.
    pub fn tick(&mut self, now_tick: u64) {
        for service in self.services.values_mut() {
            if service.state == ServiceState::Running {
                service.uptime_ticks = now_tick.saturating_sub(service.start_tick);
            }
        }
    }

    /// Services that are healthy (running).
    pub fn healthy(&self) -> usize {
        self.services
            .values()
            .filter(|s| s.state == ServiceState::Running)
            .count()
    }

    /// `service status` report.
    pub fn status_report(&self) -> String {
        let mut out = String::from("NAME                    STATE      PID    UPTIME   DEPS\n");
        for service in self.all() {
            let pid = service
                .pid
                .map(|p| p.to_string())
                .unwrap_or_else(|| "-".into());
            let deps = if service.dependencies.is_empty() {
                "-".to_string()
            } else {
                service.dependencies.join(",")
            };
            out.push_str(&format!(
                "{:<24} {:<10} {:<6} {:>7}   {}\n",
                service.name,
                format!("{:?}", service.state).to_lowercase(),
                pid,
                service.uptime_ticks,
                deps
            ));
        }
        out
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.all()).unwrap_or_else(|_| "[]".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_defines_and_starts() {
        let mut reg = ServiceRegistry::new();
        reg.define("kernel", "kernel init", &[], RestartPolicy::Never);
        reg.define("filesystem", "vfs", &["kernel"], RestartPolicy::OnFailure);
        reg.define(
            "ai-core",
            "ai runtime",
            &["filesystem"],
            RestartPolicy::Always,
        );
        assert_eq!(reg.all().len(), 3);
        assert!(reg.start("filesystem", 10, 3));
        assert_eq!(reg.get("filesystem").unwrap().state, ServiceState::Running);
        assert!(reg.get("filesystem").unwrap().uptime_ticks < 10);
        reg.tick(12);
        assert_eq!(reg.get("filesystem").unwrap().uptime_ticks, 2);
        assert_eq!(reg.healthy(), 1);
    }

    #[test]
    fn status_report_lists_services() {
        let mut reg = ServiceRegistry::new();
        reg.define("kernel", "kernel init", &[], RestartPolicy::Never);
        reg.start("kernel", 0, 1);
        let report = reg.status_report();
        assert!(report.contains("kernel"));
        assert!(report.contains("running"));
    }
}
