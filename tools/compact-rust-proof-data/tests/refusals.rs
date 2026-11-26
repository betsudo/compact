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

use compact_rust_proof_data::{
    codec::{self, FileRef},
    input::{self, Bundle},
};
use midnight_transient_crypto::proofs::Proof;
use serde_json::json;
use std::{
    fs,
    path::Path,
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Temp(std::path::PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "compact-proof-data-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn row(path: &str) -> FileRef {
    FileRef {
        path: path.into(),
        bytes: 1,
        sha256: "00".repeat(32),
    }
}
fn scalar(v: u8) -> String {
    let mut raw = [0; 32];
    raw[0] = v;
    raw.iter().map(|b| format!("{b:02x}")).collect()
}
#[test]
fn lane_is_measured_and_large_lane_requires_both_explicit_choices() {
    assert!(input::policy(7, 8, false).is_ok());
    assert!(input::policy(8, 8, false).is_ok());
    for (k, max, allow) in [
        (0, 8, false),
        (9, 8, true),
        (13, 13, false),
        (14, 13, true),
        (7, 13, false),
        (7, 14, true),
    ] {
        assert!(input::policy(k, max, allow).is_err());
    }
    assert!(input::policy(13, 13, true).is_ok());
}
#[test]
fn role_preflight_is_pure_and_refuses_public_private_aliases() {
    let public = row("does-not-exist/public.json");
    let secret = row("does-not-exist/private.preimage");
    assert!(
        input::disjoint_roles(&[
            (Path::new("/closed"), &public),
            (Path::new("/closed"), &secret)
        ])
        .is_ok()
    );
    assert!(
        input::disjoint_roles(&[
            (Path::new("/closed"), &secret),
            (Path::new("/closed"), &secret)
        ])
        .is_err()
    );
    for alias in ["a//b", "a/b/", "a/./b"] {
        assert!(
            input::disjoint_roles(&[
                (Path::new("/closed"), &row("a/b")),
                (Path::new("/closed"), &row(alias))
            ])
            .is_err()
        );
    }
    for name in ["../private", "/private", "a/../private", ""] {
        assert!(input::disjoint_roles(&[(Path::new("/closed"), &row(name))]).is_err());
    }
}
#[test]
fn loading_metadata_does_not_open_generation_preimage_parent_or_gate() {
    let temp = Temp::new();
    let path = temp.0.join("input.json");
    let input = json!({"schema":"compact-rust-proof-data-input/v1","profile":"map-member","sdk":{"ledger":input::LEDGER,"zk":input::ZK,"zkir":input::ZKIR,"gate":row("missing-gate")},"raw_ir":row("missing-ir"),"parent":row("missing-parent"),"binding_le":scalar(17),"randomness_le":scalar(29),"generation":{"compiler_version":"test","source_revision":"test","files":[row("missing-generation")]},"cases":[{"id":"one","supplemental":true,"input":[scalar(1)],"output":[scalar(1)],"preimage":row("missing-private"),"public_stream":row("missing-public")}]});
    fs::write(&path, serde_json::to_vec(&input).unwrap()).unwrap();
    let bundle = Bundle::load(&path).unwrap();
    assert!(bundle.authenticate_gate().is_err()); // The first explicit public reader.
    let mut aliased = input.clone();
    aliased["sdk"]["gate"] = aliased["cases"][0]["preimage"].clone();
    fs::write(&path, serde_json::to_vec(&aliased).unwrap()).unwrap();
    assert!(Bundle::load(&path).is_err());
    let mut aliased = input;
    aliased["generation"]["files"][0] = aliased["parent"].clone();
    fs::write(&path, serde_json::to_vec(&aliased).unwrap()).unwrap();
    assert!(Bundle::load(&path).is_err());
}
#[test]
fn official_tagged_single_object_codec_refuses_tag_truncation_and_tail() {
    let proof = Proof(vec![1, 2, 3, 4]);
    let raw = codec::encode(&proof).unwrap();
    assert_eq!(codec::decode::<Proof>(&raw).unwrap().0, proof.0);
    let mut wrong = raw.clone();
    wrong[0] ^= 0xff;
    assert!(codec::decode::<Proof>(&wrong).is_err());
    assert!(codec::decode::<Proof>(&raw[..raw.len() - 1]).is_err());
    let mut trailing = raw;
    trailing.push(0);
    assert!(codec::decode::<Proof>(&trailing).is_err());
}
#[test]
fn relative_reader_refuses_symlink_and_content_drift() {
    let temp = Temp::new();
    fs::write(temp.0.join("body"), b"x").unwrap();
    let descriptor = FileRef {
        path: "body".into(),
        bytes: 1,
        sha256: codec::hash(b"x"),
    };
    assert_eq!(codec::read_ref(&temp.0, &descriptor, 1).unwrap(), b"x");
    fs::write(temp.0.join("body"), b"y").unwrap();
    assert!(codec::read_ref(&temp.0, &descriptor, 1).is_err());
    std::os::unix::fs::symlink("body", temp.0.join("link")).unwrap();
    assert!(
        codec::read_ref(
            &temp.0,
            &FileRef {
                path: "link".into(),
                ..descriptor
            },
            1
        )
        .is_err()
    );
}
