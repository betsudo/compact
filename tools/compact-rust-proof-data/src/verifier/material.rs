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

//! Exact current public SDK parameter/key codecs; no keygen or proving.
use crate::{
    codec::{self, decode},
    input::PARENT_SHA,
};
use anyhow::{Result, ensure};
use midnight_curves::Bls12;
use midnight_proofs::{
    poly::{commitment::Params, kzg::params::ParamsKZG},
    utils::{SerdeFormat, helpers::ProcessedSerdeObject},
};
use midnight_serialize::tagged_serialize;
use midnight_transient_crypto::proofs::{ParamsVerifier, VerifierKey};
use midnight_zk_stdlib::MidnightVK;
use std::io::Cursor;

fn parameters(bytes: &[u8], k: u32) -> Result<ParamsKZG<Bls12>> {
    ensure!(
        bytes.get(..4) == Some(k.to_le_bytes().as_slice()),
        "bound parameter header BEFORE unchecked reader"
    );
    let mut input = Cursor::new(bytes);
    let p = ParamsKZG::<Bls12>::read_custom(&mut input, SerdeFormat::RawBytesUnchecked)?;
    ensure!(
        input.position() == bytes.len() as u64 && p.max_k() == k,
        "current RAW parameters EOF and actual k"
    );
    let mut again = Vec::new();
    p.write_custom(&mut again, SerdeFormat::RawBytesUnchecked)?;
    ensure!(again == bytes, "current RAW parameter canonical roundtrip");
    Ok(p)
}
/// The only unchecked parent decoder is dominated by the compiled-in static
/// parent hash; a producer FileRef digest alone does not establish SRS trust.
pub fn trusted_parent(bytes: &[u8]) -> Result<ParamsKZG<Bls12>> {
    ensure!(
        codec::hash(bytes) == PARENT_SHA,
        "authenticated current static parent14 BEFORE unchecked reader"
    );
    parameters(bytes, 14)
}
fn with_exact_derived<T, F: FnOnce(&[u8]) -> Result<T>>(
    candidate: &[u8],
    expected: &[u8],
    reader: F,
) -> Result<T> {
    ensure!(
        candidate == expected,
        "own RAW bytes must exactly equal trusted parent clone/downsize, including lagrange"
    );
    reader(expected)
}
pub fn derived_own(
    parent: &ParamsKZG<Bls12>,
    k: u32,
    candidate: &[u8],
) -> Result<(ParamsKZG<Bls12>, ParamsVerifier)> {
    ensure!(
        (1..=13).contains(&k) && k <= parent.max_k(),
        "bounded measured own degree"
    );
    let mut own = parent.clone();
    own.downsize(k);
    let mut expected = Vec::new();
    own.write_custom(&mut expected, SerdeFormat::RawBytesUnchecked)?;
    // Never decode producer-supplied own bytes. Exact equality authenticates
    // both bases/G2 family; only trusted derived bytes reach the SDK reader.
    let verifier = with_exact_derived(candidate, &expected, own_verifier)?;
    Ok((own, verifier))
}
pub fn same_family(
    own: &ParamsKZG<Bls12>,
    parent: &ParamsKZG<Bls12>,
    a: &[u8],
    b: &[u8],
) -> Result<()> {
    ensure!(
        own.g2() == parent.g2() && own.s_g2() == parent.s_g2(),
        "same own/default actual G2 and sG2 family"
    );
    // Measure the CURRENT SDK RAW G1 writer, never compressed/default point size.
    let mut g1 = Vec::new();
    own.g_lagrange()[0].write(&mut g1, SerdeFormat::RawBytesUnchecked)?;
    let mut g2 = Vec::new();
    own.g2().write(&mut g2, SerdeFormat::RawBytesUnchecked)?;
    let own_n = 1usize << own.max_k();
    let parent_n = 1usize << parent.max_k();
    ensure!(
        !g1.is_empty() && !g2.is_empty() && own_n <= parent_n,
        "actual bounded RAW codec dimensions"
    );
    ensure!(
        a.len() == 4 + 2 * own_n * g1.len() + 2 * g2.len()
            && b.len() == 4 + 2 * parent_n * g1.len() + 2 * g2.len(),
        "exact current RAW layout"
    );
    ensure!(
        a[4..4 + own_n * g1.len()] == b[4..4 + own_n * g1.len()],
        "same actual monomial G1 prefix"
    );
    Ok(())
}
fn own_verifier(bytes: &[u8]) -> Result<ParamsVerifier> {
    let mut input = Cursor::new(bytes);
    let p = ParamsVerifier::read(&mut input)?;
    ensure!(
        input.position() == bytes.len() as u64,
        "public own verifier reader exact EOF"
    );
    Ok(p)
}
pub fn verifier(bytes: &[u8], k: u32) -> Result<VerifierKey> {
    let vk: VerifierKey = decode(bytes)?;
    vk.init()?;
    let original = vk.original_bytes();
    let mut input = Cursor::new(original.as_slice());
    let native = MidnightVK::read(&mut input, SerdeFormat::Processed)?;
    ensure!(
        input.position() == original.len() as u64 && u32::from(native.k()) == k,
        "native initialized VK same own-k and EOF"
    );
    let mut canonical = Vec::new();
    native.write(&mut canonical, SerdeFormat::Processed)?;
    ensure!(
        canonical == original,
        "native initialized VK canonical bytes"
    );
    canonical.clear();
    tagged_serialize(&vk, &mut canonical)?;
    ensure!(canonical == bytes, "outer initialized VK canonical bytes");
    Ok(vk)
}

#[cfg(test)]
mod trust_tests {
    use super::with_exact_derived;
    #[test]
    fn altered_or_truncated_own_bytes_never_enter_the_decoder() {
        // Pure reader-entry spy, not malformed curve material sent to the SDK.
        let expected = [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        for index in [0, 4, 8, 11] {
            let mut candidate = expected;
            candidate[index] ^= 1;
            let mut calls = 0;
            assert!(
                with_exact_derived(&candidate, &expected, |_| {
                    calls += 1;
                    Ok(())
                })
                .is_err()
            );
            assert_eq!(calls, 0);
        }
        let mut calls = 0;
        assert!(
            with_exact_derived(&expected[..11], &expected, |_| {
                calls += 1;
                Ok(())
            })
            .is_err()
        );
        assert_eq!(calls, 0);
    }
    #[test]
    fn exact_candidate_passes_only_the_trusted_expected_slice_once() {
        let expected = [1u8, 2, 3];
        let candidate = expected;
        let mut calls = 0;
        with_exact_derived(&candidate, &expected, |raw| {
            calls += 1;
            assert!(std::ptr::eq(raw.as_ptr(), expected.as_ptr()));
            Ok(())
        })
        .unwrap();
        assert_eq!(calls, 1);
    }
}
