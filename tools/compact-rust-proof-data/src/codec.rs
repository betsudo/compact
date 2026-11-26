// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use anyhow::{Context, Result, ensure};
use midnight_serialize::{
    Deserializable, Serializable, Tagged, tagged_deserialize, tagged_serialize,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Cursor, Read, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Component, Path, PathBuf},
};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FileRef {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}
pub fn hash(raw: &[u8]) -> String {
    format!("{:x}", Sha256::digest(raw))
}
pub fn relative(root: &Path, name: &str) -> Result<PathBuf> {
    let rel = Path::new(name);
    ensure!(
        !name.is_empty()
            && !rel.is_absolute()
            && rel.components().all(|c| matches!(c, Component::Normal(_))),
        "contained relative input path"
    );
    let mut path = root.to_path_buf();
    for part in rel.components() {
        path.push(part);
        let m = fs::symlink_metadata(&path)?;
        ensure!(!m.file_type().is_symlink(), "symlink input refused");
    }
    Ok(path)
}
pub fn read_bytes(path: &Path, cap: usize) -> Result<Vec<u8>> {
    let before = fs::symlink_metadata(path)?;
    ensure!(
        before.is_file()
            && !before.file_type().is_symlink()
            && before.nlink() == 1
            && before.len() <= cap as u64,
        "bounded single-link regular input"
    );
    let id = |m: &fs::Metadata| {
        (
            m.dev(),
            m.ino(),
            m.len(),
            m.mode(),
            m.uid(),
            m.nlink(),
            m.mtime(),
            m.mtime_nsec(),
            m.ctime(),
            m.ctime_nsec(),
        )
    };
    let mut f = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)?;
    ensure!(id(&before) == id(&f.metadata()?), "input open identity");
    let mut raw = Vec::new();
    (&mut f).take(cap as u64 + 1).read_to_end(&mut raw)?;
    ensure!(
        raw.len() as u64 == before.len()
            && raw.len() <= cap
            && id(&before) == id(&f.metadata()?)
            && id(&before) == id(&fs::symlink_metadata(path)?),
        "input FD/path drift"
    );
    Ok(raw)
}
pub fn read_ref(root: &Path, row: &FileRef, cap: usize) -> Result<Vec<u8>> {
    ensure!(
        row.bytes <= cap as u64
            && row.sha256.len() == 64
            && row
                .sha256
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()),
        "bounded canonical input descriptor"
    );
    let raw = read_bytes(&relative(root, &row.path)?, cap)?;
    ensure!(
        raw.len() as u64 == row.bytes && hash(&raw) == row.sha256,
        "input content drift"
    );
    Ok(raw)
}
pub fn encode<T: Serializable + Tagged>(value: &T) -> Result<Vec<u8>> {
    let mut raw = Vec::new();
    tagged_serialize(value, &mut raw)?;
    Ok(raw)
}
pub fn decode<T: Serializable + Deserializable + Tagged>(raw: &[u8]) -> Result<T> {
    let mut c = Cursor::new(raw);
    let value: T = tagged_deserialize(&mut c)?;
    ensure!(
        c.position() == raw.len() as u64 && encode(&value)? == raw,
        "canonical single-object tagged EOF"
    );
    Ok(value)
}
pub fn fresh_dir(path: &Path) -> Result<PathBuf> {
    ensure!(!path.exists(), "output already exists");
    let parent = path.parent().context("output parent")?;
    ensure!(
        parent.is_dir() && !parent.is_symlink(),
        "ordinary output parent"
    );
    fs::create_dir(path)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(path.to_path_buf())
}
pub fn write_fresh(root: &Path, name: &str, raw: &[u8], cap: usize) -> Result<FileRef> {
    ensure!(raw.len() <= cap, "output cap");
    let rel = Path::new(name);
    ensure!(
        rel.components().all(|c| matches!(c, Component::Normal(_)))
            && rel.components().count() == 1,
        "single output leaf"
    );
    let path = root.join(rel);
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(&path)?;
    f.write_all(raw)?;
    f.sync_all()?;
    let a = f.metadata()?;
    let raw2 = read_bytes(&path, cap)?;
    let b = fs::symlink_metadata(&path)?;
    ensure!(
        raw2 == raw && a.dev() == b.dev() && a.ino() == b.ino() && b.mode() & 0o777 == 0o600,
        "output FD/path identity"
    );
    Ok(FileRef {
        path: name.into(),
        bytes: raw.len() as u64,
        sha256: hash(raw),
    })
}
pub fn write_json<T: Serialize>(root: &Path, name: &str, value: &T) -> Result<FileRef> {
    write_fresh(root, name, &serde_json::to_vec_pretty(value)?, 1024 * 1024)
}
