//! Virtual filesystem.
//!
//! An in-memory tree of inodes rooted at `/`. It supports:
//!   - the documented layout `/ bin dev etc home opt proc sys tmp usr var`
//!   - regular files and directories with owners, groups, and `rwx` perms
//!   - **virtual mounts**: `/proc` and `/sys` are virtual filesystems backed
//!     by live runtime state (processes, memory, services). Reading them
//!     reflects the system *right now*.
//!
//! No syscall may bypass the filesystem; all access goes through here.

use crate::permissions::Permissions;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    File,
    Directory,
}

impl std::fmt::Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileType::File => f.write_str("file"),
            FileType::Directory => f.write_str("directory"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DirEntry {
    pub name: String,
    pub file_type: FileType,
    pub size: usize,
    pub perms: String,
    pub owner: String,
    pub group: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileStat {
    pub path: String,
    pub file_type: FileType,
    pub size: usize,
    pub perms: String,
    pub owner: String,
    pub group: String,
    pub links: u32,
    pub mtime: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FsError {
    NotFound,
    NotADirectory,
    NotAFile,
    AlreadyExists,
    PermissionDenied,
    InvalidPath,
    NotMounted,
}

impl std::fmt::Display for FsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            FsError::NotFound => "no such file or directory",
            FsError::NotADirectory => "not a directory",
            FsError::NotAFile => "not a file",
            FsError::AlreadyExists => "already exists",
            FsError::PermissionDenied => "permission denied",
            FsError::InvalidPath => "invalid path",
            FsError::NotMounted => "not mounted",
        };
        f.write_str(s)
    }
}

pub type FsResult<T> = Result<T, FsError>;

/// A filesystem object.
#[derive(Debug, Clone)]
pub struct Inode {
    pub id: u64,
    pub name: String,
    pub file_type: FileType,
    pub content: Vec<u8>,
    pub permissions: Permissions,
    pub owner: String,
    pub group: String,
    pub size: usize,
    pub mtime: u64,
    pub children: BTreeMap<String, u64>,
    pub nlink: u32,
}

/// A virtual filesystem that generates its contents from live state
/// (used for `/proc` and `/sys`). Paths passed here are relative to the
/// mount point and begin with `/`.
pub trait VirtualFilesystem {
    fn list(&self, path: &str) -> FsResult<Vec<DirEntry>>;
    fn read(&self, path: &str) -> FsResult<Vec<u8>>;
    fn stat(&self, path: &str) -> FsResult<FileStat>;
}

struct Mount {
    /// Normalized mount point, e.g. `/proc`.
    path: String,
    fs: Rc<dyn VirtualFilesystem>,
}

#[derive(Default)]
pub struct FileSystem {
    inodes: HashMap<u64, Inode>,
    next_id: u64,
    root_id: u64,
    mounts: Vec<Mount>,
}

impl FileSystem {
    pub fn new() -> Self {
        let mut fs = Self {
            inodes: HashMap::new(),
            next_id: 1,
            root_id: 0,
            mounts: Vec::new(),
        };
        fs.root_id = fs.alloc_inode(
            "/",
            FileType::Directory,
            vec![],
            Permissions::dir(),
            "root",
            "root",
            0,
        );
        fs
    }

    #[allow(clippy::too_many_arguments)]
    fn alloc_inode(
        &mut self,
        name: &str,
        file_type: FileType,
        content: Vec<u8>,
        permissions: Permissions,
        owner: &str,
        group: &str,
        tick: u64,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let size = if file_type == FileType::File {
            content.len()
        } else {
            0
        };
        self.inodes.insert(
            id,
            Inode {
                id,
                name: name.to_string(),
                file_type,
                content,
                permissions,
                owner: owner.to_string(),
                group: group.to_string(),
                size,
                mtime: tick,
                children: BTreeMap::new(),
                nlink: 1,
            },
        );
        id
    }

    /// Mount a virtual filesystem at `path`.
    pub fn mount(&mut self, path: &str, fs: Rc<dyn VirtualFilesystem>) {
        let normalized = normalize_path(path);
        self.mounts.push(Mount {
            path: normalized,
            fs,
        });
    }

    fn mount_for<'a>(&'a self, path: &str) -> Option<(String, &'a dyn VirtualFilesystem)> {
        for m in &self.mounts {
            if path == m.path {
                return Some(("/".to_string(), m.fs.as_ref()));
            }
            if let Some(rest) = path.strip_prefix(&format!("{}/", m.path)) {
                return Some((format!("/{rest}"), m.fs.as_ref()));
            }
        }
        None
    }

    // ---- path resolution -------------------------------------------------

    /// Resolve a normalized path to an inode id.
    fn resolve(&self, path: &str) -> FsResult<u64> {
        if let Some((rel, vfs)) = self.mount_for(path) {
            // Virtual mounts resolve to nothing in the inode tree; callers
            // that need inodes (e.g. write) reject virtual paths.
            let _ = (rel, vfs);
            return Err(FsError::NotMounted);
        }
        if path == "/" {
            return Ok(self.root_id);
        }
        let mut id = self.root_id;
        for part in path
            .trim_start_matches('/')
            .split('/')
            .filter(|p| !p.is_empty())
        {
            let inode = self.inodes.get(&id).ok_or(FsError::NotFound)?;
            if inode.file_type != FileType::Directory {
                return Err(FsError::NotADirectory);
            }
            let child = inode.children.get(part).ok_or(FsError::NotFound)?;
            id = *child;
        }
        Ok(id)
    }

    fn get(&self, id: u64) -> Option<&Inode> {
        self.inodes.get(&id)
    }

    fn get_mut(&mut self, id: u64) -> Option<&mut Inode> {
        self.inodes.get_mut(&id)
    }

    // ---- high-level operations ------------------------------------------

    /// Whether a path exists (including under virtual mounts).
    pub fn exists(&self, path: &str) -> bool {
        let p = normalize_path(path);
        if let Some((rel, vfs)) = self.mount_for(&p) {
            return vfs.stat(&rel).is_ok();
        }
        self.resolve(&p).is_ok()
    }

    pub fn mkdir(&mut self, path: &str, owner: &str, group: &str, tick: u64) -> FsResult<()> {
        let p = normalize_path(path);
        if self.resolve(&p).is_ok() {
            return Err(FsError::AlreadyExists);
        }
        let (parent, name) = split_parent(&p);
        let parent_id = self.resolve(&parent)?;
        {
            let parent_inode = self.get_mut(parent_id).ok_or(FsError::NotFound)?;
            if parent_inode.file_type != FileType::Directory {
                return Err(FsError::NotADirectory);
            }
        }
        let id = self.alloc_inode(
            name,
            FileType::Directory,
            vec![],
            Permissions::dir(),
            owner,
            group,
            tick,
        );
        self.get_mut(parent_id)
            .unwrap()
            .children
            .insert(name.to_string(), id);
        Ok(())
    }

    /// Create an empty file (or truncate if it exists) — `touch`.
    pub fn touch(&mut self, path: &str, owner: &str, group: &str, tick: u64) -> FsResult<()> {
        let p = normalize_path(path);
        if let Ok(id) = self.resolve(&p) {
            if self.get(id).unwrap().file_type == FileType::Directory {
                return Err(FsError::AlreadyExists);
            }
            self.get_mut(id).unwrap().mtime = tick;
            return Ok(());
        }
        let (parent, name) = split_parent(&p);
        let parent_id = self.resolve(&parent)?;
        let id = self.alloc_inode(
            name,
            FileType::File,
            vec![],
            Permissions::file(),
            owner,
            group,
            tick,
        );
        self.get_mut(parent_id)
            .unwrap()
            .children
            .insert(name.to_string(), id);
        Ok(())
    }

    /// Write bytes to a file, creating it if absent.
    pub fn write(
        &mut self,
        path: &str,
        data: &[u8],
        owner: &str,
        group: &str,
        tick: u64,
    ) -> FsResult<()> {
        let p = normalize_path(path);
        let id = match self.resolve(&p) {
            Ok(id) => id,
            Err(FsError::NotFound) => {
                let (parent, name) = split_parent(&p);
                let parent_id = self.resolve(&parent)?;
                let id = self.alloc_inode(
                    name,
                    FileType::File,
                    vec![],
                    Permissions::file(),
                    owner,
                    group,
                    tick,
                );
                self.get_mut(parent_id)
                    .unwrap()
                    .children
                    .insert(name.to_string(), id);
                id
            }
            Err(e) => return Err(e),
        };
        {
            let inode = self.get_mut(id).ok_or(FsError::NotFound)?;
            if inode.file_type == FileType::Directory {
                return Err(FsError::NotAFile);
            }
            inode.content = data.to_vec();
            inode.size = data.len();
            inode.mtime = tick;
        }
        Ok(())
    }

    /// Append bytes to a file (used for `>>` redirection).
    pub fn append(
        &mut self,
        path: &str,
        data: &[u8],
        owner: &str,
        group: &str,
        tick: u64,
    ) -> FsResult<()> {
        let existing = self.read(path).unwrap_or_default();
        let mut combined = existing;
        combined.extend_from_slice(data);
        self.write(path, &combined, owner, group, tick)
    }

    /// Read a file's bytes. Virtual mounts read live state.
    pub fn read(&self, path: &str) -> FsResult<Vec<u8>> {
        let p = normalize_path(path);
        if let Some((rel, vfs)) = self.mount_for(&p) {
            return vfs.read(&rel);
        }
        let id = self.resolve(&p)?;
        let inode = self.get(id).ok_or(FsError::NotFound)?;
        if inode.file_type != FileType::File {
            return Err(FsError::NotAFile);
        }
        Ok(inode.content.clone())
    }

    /// Read a file as text.
    pub fn read_text(&self, path: &str) -> FsResult<String> {
        Ok(String::from_utf8_lossy(&self.read(path)?).to_string())
    }

    /// List a directory. Virtual mounts list live state.
    pub fn list(&self, path: &str) -> FsResult<Vec<DirEntry>> {
        let p = normalize_path(path);
        if let Some((rel, vfs)) = self.mount_for(&p) {
            return vfs.list(&rel);
        }
        let id = self.resolve(&p)?;
        let inode = self.get(id).ok_or(FsError::NotFound)?;
        if inode.file_type != FileType::Directory {
            return Err(FsError::NotADirectory);
        }
        let mut entries = Vec::new();
        for (name, child_id) in &inode.children {
            if let Some(child) = self.get(*child_id) {
                entries.push(DirEntry {
                    name: name.clone(),
                    file_type: child.file_type,
                    size: child.size,
                    perms: child.permissions.to_symbolic(),
                    owner: child.owner.clone(),
                    group: child.group.clone(),
                });
            }
        }
        entries.sort_by(|a, b| {
            let a_dir = a.file_type == FileType::Directory;
            let b_dir = b.file_type == FileType::Directory;
            b_dir.cmp(&a_dir).then_with(|| a.name.cmp(&b.name))
        });
        Ok(entries)
    }

    pub fn stat(&self, path: &str) -> FsResult<FileStat> {
        let p = normalize_path(path);
        if let Some((rel, vfs)) = self.mount_for(&p) {
            let mut st = vfs.stat(&rel)?;
            st.path = p;
            return Ok(st);
        }
        let id = self.resolve(&p)?;
        let inode = self.get(id).ok_or(FsError::NotFound)?;
        Ok(FileStat {
            path: p,
            file_type: inode.file_type,
            size: inode.size,
            perms: inode.permissions.to_symbolic(),
            owner: inode.owner.clone(),
            group: inode.group.clone(),
            links: inode.nlink,
            mtime: inode.mtime,
        })
    }

    pub fn remove(&mut self, path: &str) -> FsResult<()> {
        let p = normalize_path(path);
        if p == "/" {
            return Err(FsError::PermissionDenied);
        }
        let (parent, name) = split_parent(&p);
        let parent_id = self.resolve(&parent)?;
        let child_id = {
            let parent_inode = self.get(parent_id).ok_or(FsError::NotFound)?;
            parent_inode
                .children
                .get(name)
                .copied()
                .ok_or(FsError::NotFound)?
        };
        {
            let child = self.get(child_id).ok_or(FsError::NotFound)?;
            if child.file_type == FileType::Directory && !child.children.is_empty() {
                return Err(FsError::PermissionDenied); // non-empty dir: use rm -r
            }
        }
        self.get_mut(parent_id).unwrap().children.remove(name);
        self.inodes.remove(&child_id);
        Ok(())
    }

    /// Remove a directory tree recursively.
    pub fn remove_all(&mut self, path: &str) -> FsResult<()> {
        let p = normalize_path(path);
        if p == "/" {
            return Err(FsError::PermissionDenied);
        }
        let (parent, name) = split_parent(&p);
        let parent_id = self.resolve(&parent)?;
        let child_id = {
            let parent_inode = self.get(parent_id).ok_or(FsError::NotFound)?;
            parent_inode
                .children
                .get(name)
                .copied()
                .ok_or(FsError::NotFound)?
        };
        self.collect_remove(child_id);
        self.get_mut(parent_id).unwrap().children.remove(name);
        Ok(())
    }

    fn collect_remove(&mut self, id: u64) {
        let children: Vec<u64> = {
            match self.get(id) {
                Some(inode) => inode.children.values().copied().collect(),
                None => return,
            }
        };
        for child in children {
            self.collect_remove(child);
        }
        self.inodes.remove(&id);
    }

    pub fn rename(&mut self, from: &str, to: &str) -> FsResult<()> {
        let fp = normalize_path(from);
        let tp = normalize_path(to);
        if fp == "/" || tp == "/" {
            return Err(FsError::PermissionDenied);
        }
        let (f_parent, f_name) = split_parent(&fp);
        let f_parent_id = self.resolve(&f_parent)?;
        let child_id = {
            let pin = self.get(f_parent_id).ok_or(FsError::NotFound)?;
            pin.children.get(f_name).copied().ok_or(FsError::NotFound)?
        };
        let (t_parent, t_name) = split_parent(&tp);
        let t_parent_id = self.resolve(&t_parent)?;
        {
            let pin = self.get_mut(t_parent_id).ok_or(FsError::NotFound)?;
            if pin.children.contains_key(t_name) {
                return Err(FsError::AlreadyExists);
            }
            pin.children.insert(t_name.to_string(), child_id);
        }
        self.get_mut(f_parent_id).unwrap().children.remove(f_name);
        if let Some(inode) = self.get_mut(child_id) {
            inode.name = t_name.to_string();
        }
        Ok(())
    }

    /// Number of inodes (for `df`).
    pub fn inode_count(&self) -> usize {
        self.inodes.len()
    }

    /// List of virtual mount points (for `mount`).
    pub fn mount_points(&self) -> Vec<String> {
        self.mounts.iter().map(|m| m.path.clone()).collect()
    }

    /// Serialize a directory listing to JSON (for the Files application).
    pub fn list_json(&self, path: &str) -> String {
        match self.list(path) {
            Ok(entries) => serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_string()),
            Err(e) => format!(r#"{{"error": "{}"}}"#, e),
        }
    }

    /// Serialize a stat result to JSON.
    pub fn stat_json(&self, path: &str) -> String {
        match self.stat(path) {
            Ok(st) => serde_json::to_string(&st).unwrap_or_else(|_| "{}".to_string()),
            Err(e) => format!(r#"{{"error": "{}"}}"#, e),
        }
    }

    /// Tree dump for Developer Mode.
    pub fn tree(&self, path: &str) -> String {
        let p = normalize_path(path);
        let mut out = String::new();
        self.tree_into(&p, 0, &mut out);
        out
    }

    fn tree_into(&self, path: &str, depth: usize, out: &mut String) {
        let indent = "  ".repeat(depth);
        let label = path.rsplit('/').next().unwrap_or(path);
        if let Ok(st) = self.stat(path) {
            let kind = if st.file_type == FileType::Directory {
                "dir"
            } else {
                "file"
            };
            out.push_str(&format!("{indent}{label}/ [{kind}] {}\n", st.size));
            if st.file_type == FileType::Directory {
                if let Ok(entries) = self.list(path) {
                    for e in entries {
                        self.tree_into(&format!("{}/{}", trim_slash(path), e.name), depth + 1, out);
                    }
                }
            }
        }
    }
}

/// Normalize a path: collapse repeated slashes, `.`, `..`.
pub fn normalize_path(path: &str) -> String {
    let mut parts: Vec<String> = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            p => parts.push(p.to_string()),
        }
    }
    let mut result = format!("/{}", parts.join("/"));
    if result.len() > 1 {
        result = result.trim_end_matches('/').to_string();
    }
    result
}

/// Split a normalized path into (parent, name).
pub fn split_parent(path: &str) -> (String, &str) {
    let p = path.trim_end_matches('/');
    match p.rfind('/') {
        Some(0) => ("/".to_string(), &p[1..]),
        Some(idx) => (p[..idx].to_string(), &p[idx + 1..]),
        None => ("/".to_string(), p),
    }
}

fn trim_slash(path: &str) -> &str {
    if path.len() > 1 {
        path.trim_end_matches('/')
    } else {
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    fn test_fs() -> FileSystem {
        let mut fs = FileSystem::new();
        fs.mkdir("/home", "root", "root", 0).unwrap();
        fs.mkdir("/home/nikhil", "nikhil", "users", 0).unwrap();
        fs.touch("/home/nikhil/README.md", "nikhil", "users", 0)
            .unwrap();
        fs.write("/home/nikhil/README.md", b"hello os", "nikhil", "users", 1)
            .unwrap();
        fs
    }

    #[test]
    fn normalize_and_resolve() {
        assert_eq!(normalize_path("/home//nikhil/../nikhil"), "/home/nikhil");
        assert_eq!(normalize_path("/a/b/../../c"), "/c");
    }

    #[test]
    fn mkdir_write_read() {
        let fs = test_fs();
        assert_eq!(fs.read_text("/home/nikhil/README.md").unwrap(), "hello os");
        assert!(fs.exists("/home/nikhil/README.md"));
        assert!(!fs.exists("/home/nikhil/missing"));
    }

    #[test]
    fn list_sorts_dirs_first() {
        let mut fs = test_fs();
        fs.mkdir("/home/nikhil/projects", "nikhil", "users", 0)
            .unwrap();
        let entries = fs.list("/home/nikhil").unwrap();
        assert_eq!(entries[0].name, "projects");
        assert_eq!(entries[0].file_type, FileType::Directory);
        assert!(entries.iter().any(|e| e.name == "README.md"));
    }

    #[test]
    fn remove_and_rename() {
        let mut fs = test_fs();
        fs.touch("/home/nikhil/a.txt", "nikhil", "users", 0)
            .unwrap();
        fs.rename("/home/nikhil/a.txt", "/home/nikhil/b.txt")
            .unwrap();
        assert!(!fs.exists("/home/nikhil/a.txt"));
        assert!(fs.exists("/home/nikhil/b.txt"));
        fs.remove("/home/nikhil/b.txt").unwrap();
        assert!(!fs.exists("/home/nikhil/b.txt"));
    }

    #[test]
    fn non_empty_dir_rejects_remove() {
        let mut fs = test_fs();
        assert!(matches!(
            fs.remove("/home/nikhil"),
            Err(FsError::PermissionDenied)
        ));
        fs.remove_all("/home/nikhil").unwrap();
        assert!(!fs.exists("/home/nikhil"));
    }

    #[test]
    fn virtual_mount_reads_live() {
        struct Fake {
            value: RefCell<i32>,
        }
        impl VirtualFilesystem for Fake {
            fn list(&self, _path: &str) -> FsResult<Vec<DirEntry>> {
                Ok(vec![DirEntry {
                    name: "live".into(),
                    file_type: FileType::File,
                    size: 4,
                    perms: "r--r--r--".into(),
                    owner: "root".into(),
                    group: "root".into(),
                }])
            }
            fn read(&self, _path: &str) -> FsResult<Vec<u8>> {
                Ok(format!("value={}", self.value.borrow()).into_bytes())
            }
            fn stat(&self, _path: &str) -> FsResult<FileStat> {
                Ok(FileStat {
                    path: "/live".into(),
                    file_type: FileType::File,
                    size: 4,
                    perms: "r--r--r--".into(),
                    owner: "root".into(),
                    group: "root".into(),
                    links: 1,
                    mtime: 0,
                })
            }
        }
        let fake = Rc::new(Fake {
            value: RefCell::new(7),
        });
        let mut fs = FileSystem::new();
        fs.mount("/proc", fake.clone() as Rc<dyn VirtualFilesystem>);
        assert_eq!(fs.read_text("/proc/live").unwrap(), "value=7");
        *fake.value.borrow_mut() = 99;
        assert_eq!(fs.read_text("/proc/live").unwrap(), "value=99");
        assert!(fs.exists("/proc/live"));
    }
}
