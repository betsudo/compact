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

//! Refresh an existing frontend manifest after owned publication changes.
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::{error::Error, fs, path::Path};

fn manifest_tree(path: &Path, is_root: bool) -> Result<Value, Box<dyn Error>> {
    let mut entries = Map::new();
    entries.insert("type".into(), Value::String("directory".into()));
    let mut children = fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;
    children.sort_by_key(|entry| entry.file_name());
    for entry in children {
        let name = entry.file_name().to_string_lossy().into_owned();
        if is_root && name == "contract-manifest.json" {
            continue;
        }
        let path = entry.path();
        let kind = entry.file_type()?;
        let value = if kind.is_dir() {
            manifest_tree(&path, false)?
        } else if kind.is_file() {
            let bytes = fs::read(&path)?;
            let mut properties = Map::new();
            properties.insert("type".into(), Value::String("file".into()));
            properties.insert("size".into(), Value::from(bytes.len() as u64));
            properties.insert(
                "hash".into(),
                Value::String(format!("{:x}", Sha256::digest(bytes))),
            );
            Value::Object(properties)
        } else {
            return Err(format!("unsupported generated output entry: {}", path.display()).into());
        };
        entries.insert(name, value);
    }
    Ok(Value::Object(entries))
}

pub(crate) fn refresh_manifest(output: &Path) -> Result<(), Box<dyn Error>> {
    let path = output.join("compiler/contract-manifest.json");
    let mut manifest: Value = serde_json::from_slice(&fs::read(&path)?)?;
    let fields = manifest
        .as_object_mut()
        .ok_or("invalid contract manifest")?;
    for directory in ["compiler", "contract", "zkir", "keys"] {
        let path = output.join(directory);
        if path.is_dir() {
            fields.insert(
                directory.into(),
                manifest_tree(&path, directory == "compiler")?,
            );
        } else {
            fields.remove(directory);
        }
    }
    fs::write(
        output.join("compiler/contract-manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Root(PathBuf);
    impl Root {
        fn new() -> Self {
            let p = std::env::temp_dir().join(format!(
                "compact-output-manifest-{}-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&p).unwrap();
            fs::create_dir(p.join("compiler")).unwrap();
            fs::create_dir(p.join("contract")).unwrap();
            Self(p)
        }
    }
    impl Drop for Root {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn existing_manifest_tracks_actual_map_and_added_library_without_self_hashing() {
        let root = Root::new();
        let manifest = root.0.join("compiler/contract-manifest.json");
        fs::write(&manifest,serde_json::to_vec(&json!({"compiler-version":"fixture-version","x-kept":{"value":7},"keys":{"stale":true}})).unwrap()).unwrap();
        fs::write(root.0.join("contract/index.js.map"), b"new map bytes").unwrap();
        fs::write(root.0.join("contract/lib.rs"), b"new library bytes").unwrap();
        refresh_manifest(&root.0).unwrap();
        let value: Value = serde_json::from_slice(&fs::read(manifest).unwrap()).unwrap();
        assert_eq!(value["compiler-version"], "fixture-version");
        assert_eq!(value["x-kept"], json!({"value":7}));
        assert!(value.get("keys").is_none());
        assert!(value["compiler"].get("contract-manifest.json").is_none());
        for name in ["index.js.map", "lib.rs"] {
            let bytes = fs::read(root.0.join("contract").join(name)).unwrap();
            assert_eq!(
                value["contract"][name]["hash"],
                format!("{:x}", Sha256::digest(&bytes))
            );
            assert_eq!(value["contract"][name]["size"], bytes.len());
        }
    }
    #[test]
    fn malformed_manifest_remains_unchanged_on_refusal() {
        for bytes in [b"[]".as_slice(), b"{invalid".as_slice()] {
            let root = Root::new();
            let p = root.0.join("compiler/contract-manifest.json");
            fs::write(&p, bytes).unwrap();
            assert!(refresh_manifest(&root.0).is_err());
            assert_eq!(fs::read(p).unwrap(), bytes);
        }
    }
    #[test]
    #[cfg(unix)]
    fn generated_symbolic_links_are_refused_without_replacing_manifest() {
        let root = Root::new();
        let p = root.0.join("compiler/contract-manifest.json");
        fs::write(&p, b"{}").unwrap();
        fs::write(root.0.join("value"), b"fixture bytes").unwrap();
        std::os::unix::fs::symlink(root.0.join("value"), root.0.join("contract/link")).unwrap();
        assert!(refresh_manifest(&root.0).is_err());
        assert_eq!(fs::read(p).unwrap(), b"{}");
    }
}
