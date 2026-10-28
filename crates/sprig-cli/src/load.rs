//! Turning paths on the command line into workspaces for the core.
//!
//! The core reads nothing, so this is where files are found. A directory
//! argument is a workspace rooted there; a file argument is rooted at its own
//! directory. Grafted and referenced files are read only when a plan names
//! them, by asking the core what it still wants, so pointing `sprig` at one
//! file never walks a whole home directory.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use sprig_core::Workspace;

/// A path that could not be read: a usage error, exit 2.
#[derive(Debug)]
pub struct LoadError {
    pub path: PathBuf,
    pub error: io::Error,
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let why = match self.error.kind() {
            io::ErrorKind::NotFound => "no such file or directory".to_string(),
            io::ErrorKind::PermissionDenied => "permission denied".to_string(),
            io::ErrorKind::InvalidData => "not UTF-8 text".to_string(),
            _ => self.error.to_string(),
        };
        write!(f, "cannot read {}: {why}", self.path.display())
    }
}

impl std::error::Error for LoadError {}

/// Files read from under one root.
pub struct Loaded {
    pub root: PathBuf,
    pub workspace: Workspace,
    /// The files named on the command line, or `None` when a directory was:
    /// then every top-level plan under it is an entry.
    pub entries: Option<Vec<String>>,
}

impl Loaded {
    /// The path to show a person for a workspace path: the root as they
    /// typed it, then the path with `/`, on every platform, so output is the
    /// same wherever it is produced.
    pub fn display(&self, key: &str) -> String {
        if self.root == Path::new(".") || self.root.as_os_str().is_empty() {
            key.to_string()
        } else {
            let root = self.root.display().to_string();
            format!("{}/{key}", root.trim_end_matches(['/', '\\']))
        }
    }
}

pub fn load(paths: &[PathBuf]) -> Result<Vec<Loaded>, LoadError> {
    let default = [PathBuf::from(".")];
    let paths = if paths.is_empty() {
        &default[..]
    } else {
        paths
    };

    // Arguments under the same root share a workspace, so a graft between two
    // files named on the command line resolves.
    let mut groups: BTreeMap<PathBuf, Loaded> = BTreeMap::new();
    for path in paths {
        let meta = fs::metadata(path).map_err(|error| LoadError {
            path: path.clone(),
            error,
        })?;
        let (root, entry) = if meta.is_dir() {
            (path.clone(), None)
        } else {
            let root = match path.parent() {
                Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
                _ => PathBuf::from("."),
            };
            let name = path
                .file_name()
                .expect("a file has a name")
                .to_string_lossy()
                .into_owned();
            (root, Some(name))
        };
        let canonical = root.canonicalize().map_err(|error| LoadError {
            path: root.clone(),
            error,
        })?;
        let group = groups.entry(canonical.clone()).or_insert_with(|| Loaded {
            root: root.clone(),
            workspace: Workspace::new(),
            entries: Some(Vec::new()),
        });
        match entry {
            Some(name) => {
                let src = read(path, &canonical)?.ok_or_else(|| LoadError {
                    path: path.clone(),
                    error: io::Error::new(io::ErrorKind::NotFound, "outside"),
                })?;
                group.workspace.insert(name.clone(), src);
                if let Some(e) = &mut group.entries
                    && !e.contains(&name)
                {
                    e.push(name);
                }
            }
            None => {
                let mut found = Vec::new();
                walk(&root, &root, &mut found)
                    .map_err(|(path, error)| LoadError { path, error })?;
                for (key, file) in found {
                    if let Some(src) = read(&file, &canonical)? {
                        group.workspace.insert(key, src);
                    }
                }
                group.entries = None;
            }
        }
    }

    // Then whatever the plans graft or refer to, until nothing more is named.
    for (canonical, group) in &mut groups {
        loop {
            let wanted = group.workspace.wanted();
            if wanted.is_empty() {
                break;
            }
            for key in wanted {
                let file = group.root.join(&key);
                match read(&file, canonical) {
                    Ok(Some(src)) => group.workspace.insert(key, src),
                    Ok(None) => group.workspace.mark_absent(key),
                    Err(e)
                        if matches!(
                            e.error.kind(),
                            io::ErrorKind::NotFound | io::ErrorKind::IsADirectory
                        ) =>
                    {
                        group.workspace.mark_absent(key)
                    }
                    Err(e) => return Err(e),
                }
            }
        }
    }
    Ok(groups.into_values().collect())
}

/// Reads a file, or `None` if it resolves outside the root: a symlink is
/// untrusted input exactly as a graft path is.
fn read(path: &Path, root: &Path) -> Result<Option<String>, LoadError> {
    let err = |error| LoadError {
        path: path.to_path_buf(),
        error,
    };
    let real = path.canonicalize().map_err(err)?;
    if !real.starts_with(root) {
        return Ok(None);
    }
    if fs::metadata(&real).map_err(err)?.is_dir() {
        return Err(err(io::Error::new(
            io::ErrorKind::IsADirectory,
            "a directory",
        )));
    }
    let bytes = fs::read(&real).map_err(err)?;
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|_| err(io::Error::new(io::ErrorKind::InvalidData, "not UTF-8")))
}

/// Every `.sprig` file under `dir`, skipping hidden directories and not
/// following directory links, with its workspace path.
fn walk(
    root: &Path,
    dir: &Path,
    out: &mut Vec<(String, PathBuf)>,
) -> Result<(), (PathBuf, io::Error)> {
    let entries = fs::read_dir(dir).map_err(|e| (dir.to_path_buf(), e))?;
    let mut entries: Vec<_> = entries
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| (dir.to_path_buf(), e))?;
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let path = e.path();
        let name = e.file_name();
        let kind = e.file_type().map_err(|err| (path.clone(), err))?;
        if kind.is_dir() {
            if !name.to_string_lossy().starts_with('.') {
                walk(root, &path, out)?;
            }
        } else if path.extension().is_some_and(|x| x == "sprig") {
            let key: Vec<String> = path
                .strip_prefix(root)
                .expect("walked from the root")
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect();
            out.push((key.join("/"), path));
        }
    }
    Ok(())
}
