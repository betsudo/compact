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

//! Verified private copies; never mutate persistent provider exports or shared CAS.
use super::{
    error::{Result, require},
    identity::Artifact,
};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::Path,
};

pub fn regular_inside(base: &Path, path: &Path) -> Result<()> {
    require(
        base.is_absolute() && path.is_absolute(),
        "absolute export paths required",
    )?;
    require(
        fs::symlink_metadata(base)?.is_dir() && base.canonicalize()? == base,
        "export directory contains a symlink or noncanonical path",
    )?;
    require(
        path.starts_with(base) && path != base && path.canonicalize()? == path,
        "export artifact escapes its directory or follows a symlink",
    )?;
    require(
        fs::symlink_metadata(path)?.is_file(),
        "regular artifact required",
    )
}
pub fn copy_verified(source: &Path, target: &Path, expected: &Artifact) -> Result<()> {
    require(
        fs::symlink_metadata(source)?.is_file(),
        "artifact symlink/non-file refused",
    )?;
    let input = fs::File::open(source)?;
    require(
        input.metadata()?.len() == expected.size,
        "artifact length differs from declaration",
    )?;
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(target)?;
    stream_verified(input, &mut output, expected)?;
    output.sync_all()?;
    Ok(())
}
pub fn stream_verified(
    input: impl Read,
    output: &mut impl Write,
    expected: &Artifact,
) -> Result<()> {
    let mut input = input.take(expected.size + 1);
    let mut hasher = Sha256::new();
    let mut size = 0u64;
    let mut buffer = [0u8; 65536];
    loop {
        let n = input.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        size += n as u64;
        require(size <= expected.size, "artifact exceeds declared length")?;
        hasher.update(&buffer[..n]);
        output.write_all(&buffer[..n])?;
    }
    require(
        size == expected.size && format!("{:x}", hasher.finalize()) == expected.digest,
        "artifact content/length mismatch",
    )
}
pub fn hash_file(path: &Path, limit: u64) -> Result<String> {
    require(
        fs::symlink_metadata(path)?.is_file(),
        "regular artifact required",
    )?;
    let file = fs::File::open(path)?;
    require(
        file.metadata()?.len() <= limit,
        "artifact exceeds size bound",
    )?;
    let mut reader = file.take(limit + 1);
    let mut hasher = Sha256::new();
    let mut size = 0;
    let mut buffer = [0u8; 65536];
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        size += n as u64;
        require(size <= limit, "artifact grew beyond size bound")?;
        hasher.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
pub fn validate_destination(path: &Path) -> Result<()> {
    require(
        !path.try_exists()? && !path.is_symlink(),
        "refusing existing key artifact or output symlink",
    )
}
pub fn validate_targets(inputs: &Path, keys: &Path, names: &[String]) -> Result<()> {
    require(!keys.is_symlink(), "key directory symlink refused")?;
    require(
        !keys.exists() || keys.is_dir(),
        "key output must be a directory",
    )?;
    for name in names {
        for extension in ["prover", "verifier"] {
            validate_destination(&keys.join(format!("{name}.{extension}")))?;
        }
        validate_destination(&inputs.join(format!("{name}.bzkir")))?;
    }
    Ok(())
}
pub fn publish(
    stage: &Path,
    inputs: &Path,
    keys: &Path,
    artifacts: &std::collections::BTreeMap<String, Vec<Artifact>>,
) -> Result<()> {
    let names: Vec<_> = artifacts.keys().cloned().collect();
    validate_targets(inputs, keys, &names)?;
    fs::create_dir_all(keys)?;
    let keys = keys.canonicalize()?;
    // Every private copy retains the previously authenticated declaration.
    // create_new + streaming verification never re-admits a changed stage.
    for (name, expected) in artifacts {
        for artifact in expected {
            let extension = artifact.extension();
            let destination = if extension == "bzkir" { inputs } else { &keys }
                .join(format!("{name}.{extension}"));
            copy_verified(
                &stage.join(name).join(format!("artifact.{extension}")),
                &destination,
                artifact,
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn streamed_artifacts_reject_corruption_and_length_changes() {
        let a = Artifact {
            kind: "prover".into(),
            digest: super::super::identity::digest(b"abc"),
            size: 3,
        };
        assert!(stream_verified(&b"abc"[..], &mut Vec::new(), &a).is_ok());
        for input in [&b"abx"[..], &b"ab"[..], &b"abcd"[..]] {
            assert!(stream_verified(input, &mut Vec::new(), &a).is_err());
        }
    }
    #[test]
    fn persistent_outputs_are_not_overwritten() {
        let dir = super::super::Stage::new(&std::env::temp_dir()).unwrap();
        let path = dir.path().join("pinned.verifier");
        fs::write(&path, b"pinned").unwrap();
        assert!(validate_destination(&path).is_err());
        assert_eq!(fs::read(path).unwrap(), b"pinned");
    }
    #[cfg(unix)]
    #[test]
    fn symlink_exports_are_refused() {
        let dir = super::super::Stage::new(&std::env::temp_dir()).unwrap();
        let base = dir.path().canonicalize().unwrap();
        fs::write(base.join("real"), b"x").unwrap();
        std::os::unix::fs::symlink(base.join("real"), base.join("alias")).unwrap();
        assert!(regular_inside(&base, &base.join("alias")).is_err());
    }
}
