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
use compact_rust_proof_data::{
    codec,
    input::{Bundle, PARENT_SHA},
};
use midnight_proofs::{
    poly::commitment::Params,
    utils::{SerdeFormat, helpers::ProcessedSerdeObject},
};
use midnight_transient_crypto::proofs::{ParamsProver, ParamsProverProvider};
use std::{
    io::{self, Cursor},
    sync::Arc,
};
pub struct Own {
    pub params: ParamsProver,
    pub k: u8,
}
impl ParamsProverProvider for Own {
    async fn get_params(&self, k: u8) -> io::Result<ParamsProver> {
        if k == self.k {
            Ok(self.params.clone())
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "only measured own k; no fallback",
            ))
        }
    }
}
pub fn raw(p: &ParamsProver) -> Result<Vec<u8>> {
    let mut b = Vec::new();
    p.0.write_custom(&mut b, SerdeFormat::RawBytesUnchecked)?;
    Ok(b)
}
fn checked<T: ProcessedSerdeObject + PartialEq>(p: &T) -> Result<Vec<u8>> {
    let mut b = Vec::new();
    ProcessedSerdeObject::write(p, &mut b, SerdeFormat::RawBytesUnchecked)?;
    let mut c = Cursor::new(b.as_slice());
    let q = T::read(&mut c, SerdeFormat::RawBytes)?;
    ensure!(
        c.position() == b.len() as u64 && &q == p,
        "checked RAW point EOF"
    );
    Ok(b)
}
pub fn derive(bundle: &Bundle, k: u8) -> Result<Own> {
    let row = bundle
        .input
        .parent
        .as_ref()
        .context("explicit current static14 DATA required")?;
    ensure!(
        row.sha256 == PARENT_SHA && row.bytes == 3146116,
        "current SDK parent DATA identity"
    );
    let bytes = bundle.read(row, 3146116)?;
    ensure!(
        bytes[..4] == 14u32.to_le_bytes(),
        "parent14 header BEFORE raw reader"
    );
    let mut c = Cursor::new(bytes.as_slice());
    let parent = ParamsProver::read(&mut c)?;
    ensure!(
        c.position() == bytes.len() as u64 && parent.0.max_k() == 14 && raw(&parent)? == bytes,
        "canonical parent RAW"
    );
    let g2 = checked(&parent.0.g2())?;
    let sg2 = checked(&parent.0.s_g2())?;
    let mut clone = parent.0.as_ref().clone();
    clone.downsize(u32::from(k));
    let own = ParamsProver(Arc::new(clone));
    let b = raw(&own)?;
    let mut sample = Vec::new();
    ProcessedSerdeObject::write(
        parent.0.g_lagrange().first().context("G1 width")?,
        &mut sample,
        SerdeFormat::RawBytesUnchecked,
    )?;
    let end = 4usize
        .checked_add(
            (1usize << k)
                .checked_mul(sample.len())
                .context("prefix width")?,
        )
        .context("prefix end")?;
    ensure!(
        !sample.is_empty()
            && end <= b.len()
            && end <= bytes.len()
            && b[4..end] == bytes[4..end]
            && checked(&own.0.g2())? == g2
            && checked(&own.0.s_g2())? == sg2,
        "same raw monomial G1 prefix/G2 family"
    );
    let mut c = Cursor::new(b.as_slice());
    let reload = ParamsProver::read(&mut c)?;
    ensure!(
        c.position() == b.len() as u64
            && reload.0.max_k() == u32::from(k)
            && raw(&reload)? == b
            && raw(&parent)? == bytes
            && bundle.read(row, 3146116)? == bytes,
        "own RAW canonical/parent unchanged"
    );
    Ok(Own { params: reload, k })
}
