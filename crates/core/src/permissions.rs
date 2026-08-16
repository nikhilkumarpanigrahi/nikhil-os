//! Users, groups, and POSIX-style permissions.
//!
//! Filesystem objects carry a permission bitmap (`rwx` for owner/group/other)
//! plus an owner and group. Syscalls that touch the filesystem check these
//! against the calling user's identity.

use std::fmt;

pub const R_BIT: u8 = 0b100;
pub const W_BIT: u8 = 0b010;
pub const X_BIT: u8 = 0b001;

/// `rwx` permission bits for one class (owner/group/other).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize)]
pub struct ClassPerm {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

impl ClassPerm {
    pub fn none() -> Self {
        Self::default()
    }
    pub fn read_only() -> Self {
        Self {
            read: true,
            write: false,
            execute: false,
        }
    }
    pub fn read_write() -> Self {
        Self {
            read: true,
            write: true,
            execute: false,
        }
    }
    pub fn all() -> Self {
        Self {
            read: true,
            write: true,
            execute: true,
        }
    }

    pub fn from_bits(bits: u8) -> Self {
        Self {
            read: bits & R_BIT != 0,
            write: bits & W_BIT != 0,
            execute: bits & X_BIT != 0,
        }
    }

    pub fn to_bits(self) -> u8 {
        let mut bits = 0;
        if self.read {
            bits |= R_BIT;
        }
        if self.write {
            bits |= W_BIT;
        }
        if self.execute {
            bits |= X_BIT;
        }
        bits
    }
}

impl fmt::Display for ClassPerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}",
            if self.read { 'r' } else { '-' },
            if self.write { 'w' } else { '-' },
            if self.execute { 'x' } else { '-' }
        )
    }
}

/// Full permission set for a filesystem object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct Permissions {
    pub owner: ClassPerm,
    pub group: ClassPerm,
    pub other: ClassPerm,
}

impl Default for Permissions {
    fn default() -> Self {
        Self {
            owner: ClassPerm::read_write(),
            group: ClassPerm::read_only(),
            other: ClassPerm::read_only(),
        }
    }
}

impl Permissions {
    pub fn new(owner_bits: u8, group_bits: u8, other_bits: u8) -> Self {
        Self {
            owner: ClassPerm::from_bits(owner_bits),
            group: ClassPerm::from_bits(group_bits),
            other: ClassPerm::from_bits(other_bits),
        }
    }

    /// Standard directory permissions: `rwxr-xr-x`.
    pub fn dir() -> Self {
        Self::new(0b111, 0b101, 0b101)
    }

    /// Standard file permissions: `rw-r--r--`.
    pub fn file() -> Self {
        Self::new(0b110, 0b100, 0b100)
    }

    pub fn to_symbolic(&self) -> String {
        format!("{}{}{}", self.owner, self.group, self.other)
    }

    pub fn to_octal(&self) -> String {
        format!(
            "{}{}{}",
            self.owner.to_bits(),
            self.group.to_bits(),
            self.other.to_bits()
        )
    }

    /// Whether the given user (by uid and groups) may read.
    pub fn can_read(&self, uid: u32, groups: &[String]) -> bool {
        self.check(uid, groups, |c| c.read)
    }

    pub fn can_write(&self, uid: u32, groups: &[String]) -> bool {
        self.check(uid, groups, |c| c.write)
    }

    pub fn can_execute(&self, uid: u32, groups: &[String]) -> bool {
        self.check(uid, groups, |c| c.execute)
    }

    fn check<F: Fn(&ClassPerm) -> bool>(&self, uid: u32, groups: &[String], f: F) -> bool {
        if uid == 0 {
            return true; // root
        }
        if groups.iter().any(|g| g == "root") {
            return true;
        }
        // `uid` param is intentionally minimal in this simulation; owner checks
        // are resolved against the filesystem's owner field by the caller.
        let _ = uid;
        f(&self.other) || f(&self.group)
    }
}

/// A user account in the simulation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct User {
    pub uid: u32,
    pub name: String,
    pub home: String,
    pub shell: String,
    pub groups: Vec<String>,
}

impl User {
    pub fn root() -> Self {
        Self {
            uid: 0,
            name: "root".into(),
            home: "/root".into(),
            shell: "/bin/nish".into(),
            groups: vec!["root".into()],
        }
    }

    /// The default non-root user.
    pub fn default_user(name: &str) -> Self {
        Self {
            uid: 1000,
            name: name.into(),
            home: format!("/home/{name}"),
            shell: "/bin/nish".into(),
            groups: vec!["users".into(), "wheel".into()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbolic_representation() {
        let p = Permissions::dir();
        assert_eq!(p.to_symbolic(), "rwxr-xr-x");
        assert_eq!(Permissions::file().to_symbolic(), "rw-r--r--");
    }

    #[test]
    fn root_can_do_anything() {
        let p = Permissions::new(0, 0, 0);
        assert!(p.can_write(0, &["root".into()]));
        assert!(!p.can_write(1000, &["users".into()]));
    }
}
