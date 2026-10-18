// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Shared store/output disjointness, including prospective paths and live aliases.
use std::{
    env, fs, io,
    path::{Component, Path, PathBuf},
};
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}
pub fn effective_cache(explicit: Option<&Path>) -> io::Result<PathBuf> {
    let path = if let Some(path) = explicit {
        path.to_path_buf()
    } else if let Some(path) = env::var_os("MIDNIGHT_ZKIR_FORGE_DIR") {
        path.into()
    } else {
        PathBuf::from(
            env::var_os("HOME").ok_or_else(|| invalid("HOME required for default Forge store"))?,
        )
        .join(".midnight-zkir-forge")
    };
    if path.as_os_str().is_empty() {
        return Err(invalid("Forge store path must not be empty"));
    }
    Ok(path)
}
/// Resolve each existing component, then append absent components. Resolve a
/// symlink before processing a following `..`; lexical cleanup first is unsafe.
pub fn prospective(path: &Path) -> io::Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()?.join(path)
    };
    let mut resolved = PathBuf::new();
    for part in absolute.components() {
        match part {
            Component::Prefix(p) => resolved.push(p.as_os_str()),
            Component::RootDir => resolved.push(part.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                resolved.pop();
            }
            Component::Normal(name) => {
                resolved.push(name);
                match fs::symlink_metadata(&resolved) {
                    Ok(_) => resolved = fs::canonicalize(&resolved)?,
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error),
                }
            }
        }
    }
    Ok(resolved)
}
pub fn require_disjoint(store: &Path, roots: &[&Path]) -> io::Result<()> {
    let store = prospective(store)?;
    for root in roots {
        let root = prospective(root)?;
        if store.starts_with(&root) || root.starts_with(&store) {
            return Err(invalid(&format!(
                "Forge store and compiler output must be disjoint: store={} output={}",
                store.display(),
                root.display()
            )));
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn private() -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static N: AtomicU64 = AtomicU64::new(0);
        let p = env::temp_dir().join(format!(
            "compact-store-paths-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        p
    }
    #[test]
    fn overlap_refuses_without_mutating_existing_bytes() {
        let p = private();
        let store = p.join("store with spaces");
        fs::create_dir(&store).unwrap();
        fs::write(store.join("pinned"), b"old cache and output").unwrap();
        for output in [store.clone(), store.join("missing/child"), p.clone()] {
            assert!(require_disjoint(&store, &[&output]).is_err());
            assert_eq!(
                fs::read(store.join("pinned")).unwrap(),
                b"old cache and output"
            );
        }
        assert!(require_disjoint(&store, &[&p.join("absent/parents/output")]).is_ok());
        assert!(!p.join("absent").exists());
        assert!(!p.join("ForgeNeverInvoked").exists());
        fs::remove_dir_all(p).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn aliases_and_parent_components_resolve_before_comparison() {
        let p = private();
        let store = p.join("cache");
        fs::create_dir(&store).unwrap();
        std::os::unix::fs::symlink(&store, p.join("alias")).unwrap();
        assert!(require_disjoint(&store, &[&p.join("alias/not-created")]).is_err());
        assert_eq!(
            prospective(&p.join("alias/../future/out")).unwrap(),
            fs::canonicalize(&p).unwrap().join("future/out")
        );
        let other = p.join("other");
        fs::create_dir(&other).unwrap();
        std::os::unix::fs::symlink(&other, p.join("switch")).unwrap();
        assert!(require_disjoint(&store, &[&p.join("switch/out")]).is_ok());
        fs::remove_file(p.join("switch")).unwrap();
        std::os::unix::fs::symlink(&store, p.join("switch")).unwrap();
        assert!(require_disjoint(&store, &[&p.join("switch/out")]).is_err());
        fs::remove_dir_all(p).unwrap();
    }
}
