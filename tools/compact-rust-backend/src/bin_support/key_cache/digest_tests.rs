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

//! Independent SHA answers exercise the actual identity/file/streaming paths.
use super::{identity, publication, Stage};
use sha2::{Digest, Sha512};
use std::io::{Cursor, Read};

#[derive(serde::Deserialize)]
struct Vector {
    name: String,
    bytes: usize,
    sha256: String,
    sha512: String,
}

struct Chunked<'a> {
    input: Cursor<&'a [u8]>,
    chunk: usize,
}
impl Read for Chunked<'_> {
    fn read(&mut self, output: &mut [u8]) -> std::io::Result<usize> {
        let n = output.len().min(self.chunk);
        self.input.read(&mut output[..n])
    }
}

#[test]
fn independent_sha_answers_cover_padding_streaming_and_file_guards() {
    let vectors: Vec<Vector> = serde_json::from_str(include_str!("digest_vectors.json")).unwrap();
    assert_eq!(vectors.len(), 18);
    let stage = Stage::new(&std::env::temp_dir()).unwrap();
    for vector in vectors {
        let bytes = match vector.name.as_str() {
            "empty" => Vec::new(),
            "abc" => b"abc".to_vec(),
            "million-a" => vec![b'a'; 1_000_000],
            _ => (0..vector.bytes).map(|i| (i * 131 + 17) as u8).collect(),
        };
        assert_eq!(bytes.len(), vector.bytes);
        assert_eq!(identity::digest(&bytes), vector.sha256, "{}", vector.name);
        assert_eq!(
            format!("{:x}", Sha512::digest(&bytes)),
            vector.sha512,
            "{}",
            vector.name
        );
        let path = stage.path().join(&vector.name);
        std::fs::write(&path, &bytes).unwrap();
        assert_eq!(
            publication::hash_file(&path, bytes.len() as u64).unwrap(),
            vector.sha256
        );
        if !bytes.is_empty() {
            assert!(publication::hash_file(&path, bytes.len() as u64 - 1).is_err());
        }
        let artifact = identity::Artifact {
            kind: "prover".into(),
            digest: vector.sha256,
            size: bytes.len() as u64,
        };
        for chunk in [1, 7, 63, 64, 65, 65_536] {
            let input = Chunked {
                input: Cursor::new(bytes.as_slice()),
                chunk,
            };
            let mut copied = Vec::new();
            publication::stream_verified(input, &mut copied, &artifact).unwrap();
            assert_eq!(copied, bytes);
        }
    }
}
