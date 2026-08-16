//! Package manager (`pkgctl`).
//!
//! Applications are installed packages with a manifest (name, version,
//! description, dependencies, permissions, entrypoint). `pkgctl` operates on
//! a real registry and an installed set.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub dependencies: Vec<String>,
    pub permissions: Vec<String>,
    pub entrypoint: String,
}

#[derive(Default)]
pub struct PackageRegistry {
    available: BTreeMap<String, PackageManifest>,
    installed: BTreeMap<String, String>, // name -> version
}

impl PackageRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, manifest: PackageManifest) {
        self.available.insert(manifest.name.clone(), manifest);
    }

    pub fn available(&self) -> Vec<&PackageManifest> {
        self.available.values().collect()
    }

    pub fn get(&self, name: &str) -> Option<&PackageManifest> {
        self.available.get(name)
    }

    pub fn install(&mut self, name: &str) -> Result<(), String> {
        let manifest = self
            .available
            .get(name)
            .ok_or_else(|| format!("package not found: {name}"))?;
        for dep in &manifest.dependencies {
            if !self.installed.contains_key(dep) {
                return Err(format!("missing dependency: {dep}"));
            }
        }
        self.installed
            .insert(name.to_string(), manifest.version.clone());
        Ok(())
    }

    pub fn remove(&mut self, name: &str) -> Result<(), String> {
        if self.installed.remove(name).is_none() {
            return Err(format!("package not installed: {name}"));
        }
        Ok(())
    }

    pub fn installed(&self) -> Vec<(&String, &String)> {
        self.installed.iter().collect()
    }

    pub fn is_installed(&self, name: &str) -> bool {
        self.installed.contains_key(name)
    }

    pub fn search(&self, query: &str) -> Vec<&PackageManifest> {
        let q = query.to_lowercase();
        self.available
            .values()
            .filter(|m| {
                m.name.to_lowercase().contains(&q) || m.description.to_lowercase().contains(&q)
            })
            .collect()
    }

    /// `pkgctl list` output.
    pub fn list_report(&self, show_available: bool) -> String {
        let mut out = String::from("NAME                    VERSION    STATUS\n");
        if show_available {
            for m in self.available() {
                let status = if self.is_installed(&m.name) {
                    "installed"
                } else {
                    "available"
                };
                out.push_str(&format!("{:<24} {:<10} {}\n", m.name, m.version, status));
            }
        } else {
            for (name, version) in self.installed() {
                out.push_str(&format!("{:<24} {:<10} installed\n", name, version));
            }
        }
        out
    }

    /// `pkgctl info` output.
    pub fn info_report(&self, name: &str) -> Result<String, String> {
        let m = self
            .available
            .get(name)
            .ok_or_else(|| format!("package not found: {name}"))?;
        let deps = if m.dependencies.is_empty() {
            "-".to_string()
        } else {
            m.dependencies.join(", ")
        };
        Ok(format!(
            "Package:    {}\nVersion:    {}\nStatus:     {}\nDescription:\n  {}\nDependencies: {}\nPermissions:  {}\nEntrypoint:   {}\n",
            m.name,
            m.version,
            if self.is_installed(name) { "installed" } else { "available" },
            m.description,
            deps,
            m.permissions.join(", "),
            m.entrypoint
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reg() -> PackageRegistry {
        let mut r = PackageRegistry::new();
        r.register(PackageManifest {
            name: "terminal".into(),
            version: "1.0.0".into(),
            description: "nish terminal".into(),
            dependencies: vec![],
            permissions: vec!["sys_ipc".into()],
            entrypoint: "apps.terminal".into(),
        });
        r.register(PackageManifest {
            name: "projects".into(),
            version: "1.0.0".into(),
            description: "projects browser".into(),
            dependencies: vec!["knowledge".into()],
            permissions: vec!["sys_ipc".into()],
            entrypoint: "apps.projects".into(),
        });
        r
    }

    #[test]
    fn install_checks_dependencies() {
        let mut r = reg();
        assert!(r.install("projects").is_err()); // needs knowledge
        assert!(r.install("terminal").is_ok());
        assert!(r.is_installed("terminal"));
    }

    #[test]
    fn search_matches_description() {
        let r = reg();
        let results = r.search("browser");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "projects");
    }

    #[test]
    fn info_report() {
        let r = reg();
        let info = r.info_report("terminal").unwrap();
        assert!(info.contains("nish terminal"));
    }
}
