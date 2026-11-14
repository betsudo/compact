#!/usr/bin/env python3

# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Receipt and runtime-path refusals for aggregate mixed funded qualification."""

import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock

import check_upstream_ledger9_mixed_original_funded as gate


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


class AggregateMixedAdmissionTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name) / 'source'
        self.preflight = Path(temporary.name) / 'preflight'
        self.preflight.mkdir()
        self.head, self.tree = 'a' * 40, 'b' * 40
        self.body = b'original Compact body\n'
        self.callback = b'original callback\n'
        self.honest = b'honest preimage'
        self.changed = b'changed-key preimage'
        self.compiler_path = Path(temporary.name) / 'compiler-receipt.json'
        self.compiler_path.write_text('compiler receipt\n')
        producer = self.root / 'tools/compact-rust-backend/check_upstream_ledger9_mixed_original_preflight.py'
        capture_source = self.root / 'runtime-rs/tests/fixtures/capture-ledger9-mixed-original-proofs.mjs'
        self.write(producer, b'preflight producer\n')
        self.write(capture_source, b'capture producer\n')
        self.write(self.preflight / 'original/k1.compact', self.body)
        self.write(self.preflight / 'original/k1.callback.js', self.callback)
        artifact = self.preflight / 'k1/contract/lib.rs'
        self.write(artifact, b'generated Rust\n')
        extra = {}
        for name in ('compact-rust-origins.raw.json', 'index.d.ts', 'index.js.map'):
            path = self.preflight / 'k1/contract' / name
            self.write(path, name.encode() + b'\n')
            extra[name] = gate.sha(path)
        for name in ('Cargo.toml', 'compact-rust-ir.json', 'index.js',
                     'rust-capabilities.json'):
            self.write(self.preflight / 'k1/contract' / name, name.encode() + b'\n')
        self.write(self.preflight / 'k1-capture/honest.preimage', self.honest)
        self.write(self.preflight / 'k1-capture/changed-key.preimage', self.changed)
        capture = {
            'checks': ['all original callback assertions and nested Map point',
                       'honest ZKIR accepted', 'changed nested Map key refused'],
            'honestPreimageSha256': digest(self.honest),
            'changedKeyPreimageSha256': digest(self.changed),
        }
        self.write(self.preflight / 'k1-capture/capture.json',
                   json.dumps(capture).encode())
        self.prior = {
            'format': 'ledger9-mixed-original-preflight/v1',
            'source_head': self.head, 'source_tree': self.tree,
            'compiler_receipt_sha256': gate.sha(self.compiler_path),
            'upstream': gate.UPSTREAM,
            'gate_source_sha256': gate.sha(producer),
            'capture_source_sha256': gate.sha(capture_source),
            'runtime_baseline_sha256':
                '34bee302b30f1d87a4e04e3904891a452865d36f20ddd61613e39a36a02b4426',
            'artifacts': {'k1/contract/lib.rs': gate.sha(artifact)},
            'captures': {'k1': capture},
        }
        self.preflight_path = self.preflight / 'receipt.json'
        self.proof_path = Path(temporary.name) / 'proof-receipt.json'
        self.write_receipts()
        patchers = [
            mock.patch.object(gate, 'ROOT', self.root),
            mock.patch.object(gate, 'ORIGINAL_CASES', {'k1': {
                'source': digest(self.body), 'callback': digest(self.callback),
                'artifacts': {'contract/lib.rs': gate.sha(artifact)},
                'preimage': digest(self.honest),
                'changed_key_preimage': digest(self.changed),
            }}),
            mock.patch.object(gate, 'original_cases', return_value={
                'k1': (self.body, self.callback),
            }),
            mock.patch.object(gate, 'PREFLIGHT_GATE_SHA', gate.sha(producer)),
            mock.patch.object(gate, 'PREFLIGHT_CAPTURE_SHA', gate.sha(capture_source)),
            mock.patch.object(gate, 'AGGREGATE_CONTRACT_EXTRA_SHA', {'k1': extra}),
        ]
        for patcher in patchers:
            patcher.start()
            self.addCleanup(patcher.stop)

    @staticmethod
    def write(path: Path, data: bytes) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)

    def write_receipts(self) -> None:
        self.preflight_path.write_text(json.dumps(self.prior))
        self.proof = {
            'format': 'ledger9-mixed-original-proofs/v1',
            'source_head': self.head, 'source_tree': self.tree,
            'preflight_receipt_sha256': gate.sha(self.preflight_path),
            'compiler_receipt_sha256': gate.sha(self.compiler_path),
        }
        self.proof_path.write_text(json.dumps(self.proof))

    def validate(self, *, proof_sha: str | None = None,
                 preflight_sha: str | None = None) -> None:
        gate.validate_aggregate_receipts(
            proof_path=self.proof_path, preflight_path=self.preflight_path,
            compiler_path=self.compiler_path,
            proof_sha=proof_sha or gate.sha(self.proof_path),
            preflight_sha=preflight_sha or gate.sha(self.preflight_path),
            compiler_sha=gate.sha(self.compiler_path), head=self.head, tree=self.tree)

    def test_accepts_matching_pinned_receipts_and_original_artifacts(self) -> None:
        self.validate()

    def test_refuses_ancestor_proof_head(self) -> None:
        self.proof['source_head'] = '0' * 40
        self.proof_path.write_text(json.dumps(self.proof))
        with self.assertRaisesRegex(RuntimeError, 'current head, tree'):
            self.validate()

    def test_refuses_substituted_proof_receipt_bytes(self) -> None:
        pin = gate.sha(self.proof_path)
        self.proof_path.write_text('{}')
        with self.assertRaisesRegex(RuntimeError, 'independently supplied SHA-256 pin'):
            self.validate(proof_sha=pin)

    def test_refuses_substituted_preflight_receipt_bytes(self) -> None:
        pin = gate.sha(self.preflight_path)
        self.preflight_path.write_text('{}')
        with self.assertRaisesRegex(RuntimeError, 'independently supplied SHA-256 pin'):
            self.validate(preflight_sha=pin)

    def test_refuses_changed_original_callback(self) -> None:
        self.write(self.preflight / 'original/k1.callback.js', b'changed callback\n')
        with self.assertRaisesRegex(RuntimeError, 'original mixed source or callback changed'):
            self.validate()

    def test_refuses_changed_generated_rust(self) -> None:
        self.write(self.preflight / 'k1/contract/lib.rs', b'changed Rust\n')
        with self.assertRaisesRegex(RuntimeError, 'generated mixed artifact changed'):
            self.validate()

    def test_refuses_extra_generated_build_script(self) -> None:
        self.write(self.preflight / 'k1/contract/build.rs', b'fn main() {}\n')
        with self.assertRaisesRegex(RuntimeError, 'unpinned or missing files'):
            self.validate()

    def test_current_worktree_runtime_path_needs_no_text_change(self) -> None:
        runtime = str(self.root / 'runtime-rs')
        manifest = ('[dependencies.midnight-compact-runtime]\n'
                    f'path = "{runtime}"\n')
        self.assertEqual(gate.rebound_runtime_manifest(
            manifest, runtime, aggregate=True), manifest)
        with self.assertRaisesRegex(RuntimeError, 'runtime rebinding failed'):
            gate.rebound_runtime_manifest(manifest, runtime, aggregate=False)

    def test_same_head_other_worktree_runtime_path_is_rebound(self) -> None:
        other_runtime = '/another-clean-checkout/runtime-rs'
        manifest = ('[dependencies.midnight-compact-runtime]\n'
                    f'path = "{other_runtime}"\n')
        rebound = gate.rebound_runtime_manifest(
            manifest, other_runtime, aggregate=True)
        self.assertIn(f'path = "{self.root / "runtime-rs"}"', rebound)
        self.assertNotIn(other_runtime, rebound)


if __name__ == '__main__':
    unittest.main()
