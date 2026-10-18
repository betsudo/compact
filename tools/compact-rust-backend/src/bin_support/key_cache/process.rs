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

//! Argument-array children with bounded JSON/model output, never shell commands.
use super::{
    error::{CacheError, Result, require},
    identity,
};
use std::{
    io::Read,
    process::{Command, Stdio},
};
fn captured(command: &mut Command) -> Result<(Vec<u8>, Vec<u8>)> {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| CacheError::Refused("child stdout unavailable".into()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| CacheError::Refused("child stderr unavailable".into()))?;
    let read = |stream: Box<dyn Read + Send>| -> std::io::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        stream
            .take(identity::MAX_JSON + 1)
            .read_to_end(&mut bytes)?;
        Ok(bytes)
    };
    let a = std::thread::spawn(move || read(Box::new(stdout)));
    let b = std::thread::spawn(move || read(Box::new(stderr)));
    let stdout = a
        .join()
        .map_err(|_| CacheError::Refused("child stdout reader failed".into()))??;
    let stderr = b
        .join()
        .map_err(|_| CacheError::Refused("child stderr reader failed".into()))??;
    if stdout.len() as u64 > identity::MAX_JSON || stderr.len() as u64 > identity::MAX_JSON {
        let _ = child.kill();
        let _ = child.wait();
        return Err(CacheError::Refused(
            "child output exceeds finite bound".into(),
        ));
    }
    let status = child.wait()?;
    require(
        status.success(),
        &format!(
            "artifact tool exited {status}: {}",
            String::from_utf8_lossy(&stderr)
        ),
    )?;
    Ok((stdout, stderr))
}
pub fn output(command: &mut Command) -> Result<Vec<u8>> {
    Ok(captured(command)?.0)
}
pub fn model_output(command: &mut Command) -> Result<Vec<u8>> {
    let (a, b) = captured(command)?;
    require(
        a.len() + b.len() <= identity::MAX_JSON as usize,
        "combined model output exceeds bound",
    )?;
    let mut bytes = a;
    bytes.extend(b);
    Ok(bytes)
}
