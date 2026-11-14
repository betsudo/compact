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
        self.compiler_sha = 'c' * 64
        producer = self.root / 'tools/compact-rust-backend/check_upstream_ledger9_mixed_original_preflight.py'
        proof_checker = self.root / 'tools/compact-rust-backend/check_upstream_ledger9_mixed_original_proof.py'
        baseline_gate = self.root / 'tools/compact-rust-backend/check_upstream_ledger9_bytes_map.py'
        capture_source = self.root / 'runtime-rs/tests/fixtures/capture-ledger9-mixed-original-proofs.mjs'
        self.write(producer, b'preflight producer\n')
        self.write(proof_checker, b'proof checker\n')
        self.write(baseline_gate, b'Bytes map runtime producer\n')
        self.write(capture_source, b'capture producer\n')
        runtime_package = (b'{"name":"@midnight-ntwrk/compact-runtime",'
                           b'"version":"0.18.0","dependencies":{}}\n')
        built_runtime_package = (b'{\n  "name": "@midnight-ntwrk/compact-runtime",\n'
                                 b'  "version": "0.18.0",\n  "dependencies": {}\n}\n')
        self.write(self.root / 'runtime/package.json', runtime_package)
        self.runtime_store = Path(temporary.name) / 'nix-runtime'
        runtime = self.runtime_store / 'lib/node_modules/@midnight-ntwrk/compact-runtime'
        self.write(runtime / 'package.json', built_runtime_package)
        self.write(runtime / 'dist/index.js', b'runtime JavaScript\n')
        self.runtime_store = self.runtime_store.resolve(strict=True)
        self.baseline_path = Path(temporary.name) / 'baseline/receipt.json'
        recipe_log = self.baseline_path.parent / 'typescript-runtime-recipe.log'
        self.write(recipe_log, b'exact-head runtime realization\n')
        self.baseline = {
            'head': self.head, 'tree': self.tree, 'upstream': gate.UPSTREAM,
            'compiler_receipt_sha256': gate.sha(self.compiler_path),
            'compiler_sha256': self.compiler_sha,
            'runner_sha256': gate.sha(baseline_gate),
            'typescript_runtime_version': '0.18.0',
            'typescript_runtime_nix_store': str(self.runtime_store),
            'typescript_runtime_package_sha256': gate.sha(runtime / 'package.json'),
            'typescript_runtime_dist_files': {'index.js': gate.sha(runtime / 'dist/index.js')},
            'typescript_runtime_recipe_log_sha256': gate.sha(recipe_log),
            'result': 'Four original Bytes map cases passed generated TypeScript and ordinary/recorded Rust; checked Field-to-Uint and large-index regressions passed',
        }
        self.write(self.preflight / 'original/k1.compact', self.body)
        self.write(self.preflight / 'original/k1.callback.js', self.callback)
        artifact = self.preflight / 'k1/contract/lib.rs'
        self.write(artifact, b'generated Rust\n')
        origins_path = self.preflight / 'k1/contract/compact-rust-origins.raw.json'
        origins = {'files': [
            {'id': 0, 'path': str((self.preflight / 'original/k1.compact').resolve())},
            {'id': 1, 'path': 'compiler/standard-library.compact'},
        ], 'spans': [[0, 1]]}
        self.write(origins_path, json.dumps(origins).encode())
        canonical_origins = json.loads(origins_path.read_text())
        canonical_origins['files'][0]['path'] = '<original source>'
        canonical_sha = digest(json.dumps(
            canonical_origins, sort_keys=True, separators=(',', ':')).encode())
        extra = {}
        for name in ('index.d.ts', 'index.js.map'):
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
            'compiler_sha256': self.compiler_sha,
            'upstream': gate.UPSTREAM,
            'gate_source_sha256': gate.sha(producer),
            'capture_source_sha256': gate.sha(capture_source),
            'runtime_store': str(self.runtime_store),
            'runtime_manifest_sha256': gate.sha(runtime / 'package.json'),
            'runtime_dist_files': self.baseline['typescript_runtime_dist_files'],
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
            mock.patch.object(gate, 'PREFLIGHT_CAPTURE_SHA', gate.sha(capture_source)),
            mock.patch.object(gate, 'AGGREGATE_CONTRACT_EXTRA_SHA', {'k1': extra}),
            mock.patch.object(gate, 'AGGREGATE_ORIGINS_CANONICAL_SHA',
                              {'k1': canonical_sha}),
        ]
        for patcher in patchers:
            patcher.start()
            self.addCleanup(patcher.stop)

    @staticmethod
    def write(path: Path, data: bytes) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)

    def write_receipts(self) -> None:
        self.baseline_path.write_text(json.dumps(self.baseline))
        self.prior['runtime_baseline_sha256'] = gate.sha(self.baseline_path)
        self.preflight_path.write_text(json.dumps(self.prior))
        self.proof = {
            'format': 'ledger9-mixed-original-proofs/v1',
            'source_head': self.head, 'source_tree': self.tree,
            'preflight_receipt_sha256': gate.sha(self.preflight_path),
            'compiler_receipt_sha256': gate.sha(self.compiler_path),
            'verifier_source_sha256': gate.sha(
                self.root / 'tools/compact-rust-backend/check_upstream_ledger9_mixed_original_proof.py'),
        }
        self.proof_path.write_text(json.dumps(self.proof))

    def validate(self, *, proof_sha: str | None = None,
                 preflight_sha: str | None = None,
                 baseline_sha: str | None = None) -> None:
        gate.validate_aggregate_receipts(
            proof_path=self.proof_path, preflight_path=self.preflight_path,
            compiler_path=self.compiler_path, runtime_baseline_path=self.baseline_path,
            runtime_baseline_sha=baseline_sha or gate.sha(self.baseline_path),
            proof_sha=proof_sha or gate.sha(self.proof_path),
            preflight_sha=preflight_sha or gate.sha(self.preflight_path),
            compiler_sha=gate.sha(self.compiler_path), head=self.head, tree=self.tree)

    def test_accepts_matching_pinned_receipts_and_original_artifacts(self) -> None:
        runtime = self.runtime_store / 'lib/node_modules/@midnight-ntwrk/compact-runtime'
        self.assertNotEqual(gate.sha(self.root / 'runtime/package.json'),
                            gate.sha(runtime / 'package.json'))
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

    def test_refuses_substituted_runtime_baseline_receipt_bytes(self) -> None:
        pin = gate.sha(self.baseline_path)
        self.baseline_path.write_text('{}')
        with self.assertRaisesRegex(RuntimeError, 'independently supplied SHA-256 pin'):
            self.validate(baseline_sha=pin)

    def test_refuses_runtime_baseline_from_previous_head(self) -> None:
        self.baseline['head'] = '0' * 40
        self.baseline_path.write_text(json.dumps(self.baseline))
        with self.assertRaisesRegex(RuntimeError, 'current-head Bytes map runtime baseline'):
            self.validate()

    def test_refuses_runtime_baseline_from_other_compiler(self) -> None:
        self.baseline['compiler_receipt_sha256'] = '0' * 64
        self.baseline_path.write_text(json.dumps(self.baseline))
        with self.assertRaisesRegex(RuntimeError, 'current-head Bytes map runtime baseline'):
            self.validate()

    def test_refuses_changed_runtime_baseline_gate_source(self) -> None:
        producer = self.root / 'tools/compact-rust-backend/check_upstream_ledger9_bytes_map.py'
        self.write(producer, b'changed Bytes map runtime producer\n')
        with self.assertRaisesRegex(RuntimeError, 'current-head Bytes map runtime baseline'):
            self.validate()

    def test_refuses_changed_runtime_dist_after_baseline(self) -> None:
        runtime = self.runtime_store / 'lib/node_modules/@midnight-ntwrk/compact-runtime'
        self.write(runtime / 'dist/index.js', b'changed runtime JavaScript\n')
        with self.assertRaisesRegex(RuntimeError, 'current-head Bytes map runtime baseline'):
            self.validate()

    def test_refuses_changed_nix_runtime_package_metadata(self) -> None:
        runtime = self.runtime_store / 'lib/node_modules/@midnight-ntwrk/compact-runtime'
        self.write(runtime / 'package.json',
                   b'{"name":"@midnight-ntwrk/compact-runtime",'
                   b'"version":"0.18.0","dependencies":{"added":"1"}}\n')
        with self.assertRaisesRegex(RuntimeError, 'current-head Bytes map runtime baseline'):
            self.validate()

    def test_refuses_mixed_preflight_baseline_swap(self) -> None:
        self.prior['runtime_baseline_sha256'] = '0' * 64
        self.preflight_path.write_text(json.dumps(self.prior))
        self.proof['preflight_receipt_sha256'] = gate.sha(self.preflight_path)
        self.proof_path.write_text(json.dumps(self.proof))
        with self.assertRaisesRegex(RuntimeError, 'mixed aggregate preflight runtime differs'):
            self.validate()

    def test_refuses_changed_preflight_producer_source(self) -> None:
        producer = self.root / 'tools/compact-rust-backend/check_upstream_ledger9_mixed_original_preflight.py'
        self.write(producer, b'changed preflight producer\n')
        with self.assertRaisesRegex(RuntimeError, 'mixed aggregate preflight producer'):
            self.validate()

    def test_refuses_proof_checker_outside_current_repository(self) -> None:
        foreign = self.root.parent / 'foreign-proof-checker.py'
        self.write(foreign, b'proof checker\n')
        with self.assertRaisesRegex(RuntimeError, 'proof checker path differs'):
            gate.validate_proof_checker_source(self.proof, foreign)
        checker = self.root / 'tools/compact-rust-backend/check_upstream_ledger9_mixed_original_proof.py'
        self.write(checker, b'changed proof checker\n')
        with self.assertRaisesRegex(RuntimeError, 'proof checker differs'):
            self.validate()

    def test_refuses_external_input_mutated_after_admission(self) -> None:
        admitted = gate.snapshot_inputs([self.proof_path], [self.preflight])
        copied = self.root / 'copy'
        self.write(copied / 'lib.rs', b'admitted generated copy\n')
        copied_hashes = gate.snapshot_inputs([], [copied])
        self.write(self.preflight / 'k1/contract/lib.rs', b'mutated after admission\n')
        with mock.patch.object(gate, 'git', side_effect=['', self.head, self.tree]):
            with self.assertRaisesRegex(RuntimeError, 'input changed during qualification'):
                gate.require_final_binding(self.head, self.tree, admitted,
                                           [self.proof_path], [self.preflight],
                                           copied_hashes, [copied])

    def test_accepts_unchanged_final_binding(self) -> None:
        admitted = gate.snapshot_inputs([self.proof_path], [self.preflight])
        copied = self.root / 'copy'
        self.write(copied / 'lib.rs', b'admitted generated copy\n')
        copied_hashes = gate.snapshot_inputs([], [copied])
        with mock.patch.object(gate, 'git', side_effect=[
                '', self.head, self.tree, '', self.head, self.tree]):
            gate.require_final_binding(self.head, self.tree, admitted,
                                       [self.proof_path], [self.preflight],
                                       copied_hashes, [copied])

    def test_refuses_generated_copy_mutated_after_admission(self) -> None:
        admitted = gate.snapshot_inputs([self.proof_path], [self.preflight])
        copied = self.root / 'copy'
        self.write(copied / 'lib.rs', b'admitted generated copy\n')
        copied_hashes = gate.snapshot_inputs([], [copied])
        self.write(copied / 'lib.rs', b'mutated during build\n')
        with mock.patch.object(gate, 'git', side_effect=['', self.head, self.tree]):
            with self.assertRaisesRegex(RuntimeError, 'generated copy changed'):
                gate.require_final_binding(self.head, self.tree, admitted,
                                           [self.proof_path], [self.preflight],
                                           copied_hashes, [copied])

    def test_refuses_added_generated_copy_file(self) -> None:
        admitted = gate.snapshot_inputs([self.proof_path], [self.preflight])
        copied = self.root / 'copy'
        self.write(copied / 'lib.rs', b'admitted generated copy\n')
        copied_hashes = gate.snapshot_inputs([], [copied])
        self.write(copied / 'build.rs', b'unadmitted build script\n')
        with mock.patch.object(gate, 'git', side_effect=['', self.head, self.tree]):
            with self.assertRaisesRegex(RuntimeError, 'generated copy changed'):
                gate.require_final_binding(self.head, self.tree, admitted,
                                           [self.proof_path], [self.preflight],
                                           copied_hashes, [copied])

    def test_refuses_source_changed_after_admission(self) -> None:
        admitted = gate.snapshot_inputs([self.proof_path], [self.preflight])
        copied = self.root / 'copy'
        self.write(copied / 'lib.rs', b'admitted generated copy\n')
        copied_hashes = gate.snapshot_inputs([], [copied])
        with mock.patch.object(gate, 'git', side_effect=['', 'different-head']):
            with self.assertRaisesRegex(RuntimeError, 'source changed'):
                gate.require_final_binding(self.head, self.tree, admitted,
                                           [self.proof_path], [self.preflight],
                                           copied_hashes, [copied])

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

    def test_refuses_origin_for_another_source(self) -> None:
        path = self.preflight / 'k1/contract/compact-rust-origins.raw.json'
        origins = json.loads(path.read_text())
        origins['files'][0]['path'] = str(self.preflight / 'original/other.compact')
        path.write_text(json.dumps(origins))
        with self.assertRaisesRegex(RuntimeError, 'source origin does not name'):
            self.validate()

    def test_refuses_changed_origin_span(self) -> None:
        path = self.preflight / 'k1/contract/compact-rust-origins.raw.json'
        origins = json.loads(path.read_text())
        origins['spans'] = [[0, 2]]
        path.write_text(json.dumps(origins))
        with self.assertRaisesRegex(RuntimeError, 'source origins changed'):
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
