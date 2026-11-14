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

"""Focused exact-head and refusal admission checks for original mixed proofs."""

import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock

import check_upstream_ledger9_mixed_original_proof as gate
from collection_gate_binding import GENERATED_CONTRACT_FILES


class MixedProofAdmissionTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name) / 'source'
        self.preflight = Path(temporary.name) / 'preflight'
        self.preflight.mkdir()
        self.head, self.tree = 'a' * 40, 'b' * 40
        self.compiler = self.write(Path(temporary.name) / 'compactc', 'compiler\n')
        self.compiler_receipt = self.write(Path(temporary.name) / 'compiler.json', '{}\n')
        self.baseline = self.write(Path(temporary.name) / 'baseline.json', '{}\n')
        self.gate_source = self.write(self.root / 'tools/compact-rust-backend/check_upstream_ledger9_mixed_original_preflight.py', 'gate\n')
        self.capture_source = self.write(self.root / 'runtime-rs/tests/fixtures/capture-ledger9-mixed-original-proofs.mjs', 'capture\n')
        self.runtime = Path(temporary.name) / 'runtime/lib/node_modules/@midnight-ntwrk/compact-runtime'
        self.write(self.runtime / 'package.json', '{"version":"0.18.0"}\n')
        self.write(self.preflight / 'runtime-recipe.log', 'runtime\n')
        self.body, self.callback = b'original\n', b'assertions\n'
        self.write(self.preflight / 'original/k1.compact', self.body)
        self.write(self.preflight / 'original/k1.callback.js', self.callback)
        artifact_bytes = {'contract/lib.rs': b'rust\n', 'zkir/test.zkir': b'zkir\n'}
        self.artifacts = {}
        for relative, content in artifact_bytes.items():
            path = self.write(self.preflight / 'k1' / relative, content)
            self.artifacts[relative] = gate.sha(path)
        for filename in GENERATED_CONTRACT_FILES:
            path = self.preflight / 'k1/contract' / filename
            if not path.exists():
                self.write(path, filename + '\n')
        honest = self.write(self.preflight / 'k1-capture/honest.preimage', b'honest\n')
        changed = self.write(self.preflight / 'k1-capture/changed-key.preimage', b'changed\n')
        pins = {'source': gate.sha(self.preflight / 'original/k1.compact'),
                'callback': gate.sha(self.preflight / 'original/k1.callback.js'),
                'artifacts': self.artifacts,
                'preimage': gate.sha(honest),
                'changed_key_preimage': gate.sha(changed)}
        self.capture = {'honestPreimageSha256': pins['preimage'],
                        'changedKeyPreimageSha256': pins['changed_key_preimage'],
                        'callbackSha256': pins['callback'],
                        'rawZkirSha256': pins['artifacts']['zkir/test.zkir'],
                        'checks': ['all original callback assertions and nested Map point',
                                   'honest ZKIR accepted', 'changed nested Map key refused']}
        self.write(self.preflight / 'k1-capture/capture.json', json.dumps(self.capture))
        self.prior = {
            'format': 'ledger9-mixed-original-preflight/v1',
            'source_head': self.head, 'source_tree': self.tree,
            'upstream': gate.UPSTREAM,
            'compiler_receipt_sha256': gate.sha(self.compiler_receipt),
            'compiler_sha256': gate.sha(self.compiler),
            'gate_source_sha256': gate.sha(self.gate_source),
            'capture_source_sha256': gate.sha(self.capture_source),
            'runtime_baseline_sha256': gate.sha(self.baseline),
            'runtime_recipe_log_sha256': gate.sha(self.preflight / 'runtime-recipe.log'),
            'runtime_store': str(self.runtime.parents[3]),
            'runtime_manifest_sha256': gate.sha(self.runtime / 'package.json'),
            'runtime_dist_files': {'index.js': 'a' * 64},
            'zkir_wasm_tgz_sha256': gate.WASM_TGZ_SHA256,
            'artifacts': {f'k1/{name}': digest for name, digest in self.artifacts.items()},
            'captures': {'k1': self.capture},
        }
        self.path = self.preflight / 'receipt.json'
        self.patchers = [
            mock.patch.object(gate, 'CASES', {'k1': gate.CASES['k1']}),
            mock.patch.object(gate, 'ORIGINAL_CASES', {'k1': pins}),
            mock.patch.object(gate, 'PREFLIGHT', self.gate_source),
            mock.patch.object(gate, 'CAPTURE', self.capture_source),
            mock.patch.object(gate, 'ROOT', self.root),
            mock.patch.object(gate, 'validate_runtime_baseline', return_value={
                'typescript_runtime_package_sha256': gate.sha(self.runtime / 'package.json'),
                'typescript_runtime_dist_files': self.prior['runtime_dist_files'],
            }),
        ]
        for patcher in self.patchers:
            patcher.start()
            self.addCleanup(patcher.stop)

    @staticmethod
    def write(path: Path, content: str | bytes) -> Path:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content.encode() if isinstance(content, str) else content)
        return path

    def validate(self, pin: str | None = None) -> dict:
        self.path.write_text(json.dumps(self.prior))
        return gate.validate_preflight(
            self.path, pin or gate.sha(self.path), head=self.head, tree=self.tree,
            compiler_receipt=self.compiler_receipt, compiler=self.compiler,
            runtime_baseline=self.baseline, baseline_pin=gate.sha(self.baseline),
            extracted={'k1': (self.body, self.callback)})

    def test_accepts_bound_preflight(self) -> None:
        self.assertEqual(self.validate()['source_head'], self.head)

    def test_refuses_wrong_independent_receipt_pin(self) -> None:
        with self.assertRaisesRegex(RuntimeError, 'receipt SHA-256 pin changed'):
            self.validate('0' * 64)

    def test_refuses_stale_head_even_when_receipt_is_rehashed(self) -> None:
        self.prior['source_head'] = '0' * 40
        with self.assertRaisesRegex(RuntimeError, 'source head/tree'):
            self.validate()

    def test_refuses_other_compiler_receipt(self) -> None:
        self.prior['compiler_receipt_sha256'] = '0' * 64
        with self.assertRaisesRegex(RuntimeError, 'compiler or producer source'):
            self.validate()

    def test_refuses_other_runtime_baseline(self) -> None:
        self.prior['runtime_baseline_sha256'] = '0' * 64
        with self.assertRaisesRegex(RuntimeError, 'runtime baseline or recipe'):
            self.validate()

    def test_refuses_changed_generated_artifact(self) -> None:
        self.write(self.preflight / 'k1/zkir/test.zkir', 'different\n')
        with self.assertRaisesRegex(RuntimeError, 'generated artifact changed'):
            self.validate()

    def test_refuses_changed_original_callback_capture(self) -> None:
        self.capture['checks'] = ['honest ZKIR accepted']
        self.write(self.preflight / 'k1-capture/capture.json', json.dumps(self.capture))
        with self.assertRaisesRegex(RuntimeError, 'callback or refusal capture changed'):
            self.validate()

    def test_refuses_changed_runner_source_with_pinned_receipt(self) -> None:
        source = Path(self.root.parent) / 'runner'
        for name, digest in gate.RUNNER_SOURCE_SHA256.items():
            self.write(source / name, name + '\n')
        receipt = self.write(self.root.parent / 'runner-receipt.json', json.dumps({
            'runner_source_sha256': {name: gate.sha(source / name)
                                     for name in gate.RUNNER_SOURCE_SHA256}}))
        with (mock.patch.object(gate, 'RUNNER_SOURCE_SHA256',
                                json.loads(receipt.read_text())['runner_source_sha256']),
              mock.patch.object(gate, 'RUNNER_RECEIPT_SHA256', gate.sha(receipt))):
            self.write(source / 'src/main.rs', 'changed\n')
            with self.assertRaisesRegex(RuntimeError, 'proof runner source'):
                gate.validate_runner_source(source, receipt)

    def test_refuses_changed_parameters_or_wrong_verifier_k(self) -> None:
        material = self.root.parent / 'material-root/material'
        params = self.write(material / 'bls_midnight_2p17', b'params\n')
        wrong = self.root.parent / 'wrong-root'
        verifier = self.write(wrong / 'two/statement.verifier', b'wrong verifier\n')
        material_receipt = self.write(material.parent / 'material-receipt.json',
                                      json.dumps({'files': {params.name: {
                                          'sha256': gate.sha(params),
                                          'bytes': params.stat().st_size}}}))
        wrong_data = {'two': {'k': 17, 'pins': {
            'statement.verifier': gate.sha(verifier)}}}
        wrong_receipt = self.write(wrong / 'receipt.json', json.dumps(wrong_data))
        case = {'k': 17, 'wrong': 'two', 'params': gate.sha(params),
                'wrong_verifier': gate.sha(verifier)}
        with (mock.patch.object(gate, 'CASES', {'k1': case}),
              mock.patch.object(gate, 'MATERIAL_RECEIPT_SHA256', gate.sha(material_receipt)),
              mock.patch.object(gate, 'WRONG_RECEIPT_SHA256', gate.sha(wrong_receipt))):
            gate.validate_material_and_wrong(material, wrong)
            self.write(params, b'changed params\n')
            with self.assertRaisesRegex(RuntimeError, 'parameters or same-k'):
                gate.validate_material_and_wrong(material, wrong)
        self.write(params, b'params\n')
        wrong_data['two']['k'] = 18
        self.write(wrong_receipt, json.dumps(wrong_data))
        with (mock.patch.object(gate, 'CASES', {'k1': case}),
              mock.patch.object(gate, 'MATERIAL_RECEIPT_SHA256', gate.sha(material_receipt)),
              mock.patch.object(gate, 'WRONG_RECEIPT_SHA256', gate.sha(wrong_receipt))):
            with self.assertRaisesRegex(RuntimeError, 'parameters or same-k'):
                gate.validate_material_and_wrong(material, wrong)

    def test_refuses_missing_independent_verification_controls(self) -> None:
        gate.validate_verification_output(gate.VERIFY_RESULT, case='k1')
        with self.assertRaisesRegex(RuntimeError, 'verification or refusal'):
            gate.validate_verification_output('verified', case='k1')

    def test_refuses_preimage_mutated_after_initial_admission(self) -> None:
        artifacts = {}
        for family in ('field_to_uint', 'large_index', 'reordering',
                       'transformation', 'slice_composition', 'to_bytes'):
            for mode in ('default', 'paired'):
                path = self.write(self.baseline.parent / mode / family / 'contract/index.js',
                                  f'{family} {mode} TypeScript\n')
                artifacts[f'{family}/{mode}/typescript'] = gate.sha(path)
            for kind, filename in (('rust', 'lib.rs'),
                                   ('rust_ir', 'compact-rust-ir.json')):
                path = self.write(self.baseline.parent / 'paired' / family / 'contract' / filename,
                                  f'{family} {kind}\n')
                artifacts[f'{family}/paired/{kind}'] = gate.sha(path)
        cargo_log = self.write(self.baseline.parent / 'cargo-test.log', 'tests passed\n')
        lock = self.write(self.baseline.parent / 'rust-consumer/Cargo.lock', 'lock\n')
        self.write(self.baseline, json.dumps({
            'artifacts': artifacts, 'cargo_test_log_sha256': gate.sha(cargo_log),
            'rust_consumer_lock_sha256': gate.sha(lock)}))
        self.prior['runtime_baseline_sha256'] = gate.sha(self.baseline)
        self.validate()
        scheme = self.write(self.root.parent / 'compactc-scheme', 'scheme\n')
        zkir = self.write(self.root.parent / 'zkir', 'zkir\n')
        runner_source = self.root.parent / 'runner-source'
        for name in gate.RUNNER_SOURCE_SHA256:
            self.write(runner_source / name, name + '\n')
        runner_receipt = self.write(self.root.parent / 'runner-receipt.json', '{}\n')
        material = self.root.parent / 'material-root/material'
        self.write(material.parent / 'material-receipt.json', '{}\n')
        self.write(material / 'bls_midnight_2p17', 'params\n')
        wrong = self.root.parent / 'wrong'
        self.write(wrong / 'receipt.json', '{}\n')
        self.write(wrong / 'two/statement.verifier', 'verifier\n')
        self.write(self.baseline.parent / 'typescript-runtime-recipe.log', 'runtime\n')
        self.write(self.runtime / 'dist/index.js', 'runtime dist\n')
        paths = gate.external_input_paths(
            preflight_path=self.path, baseline_path=self.baseline,
            compiler_path=self.compiler_receipt, compiler=self.compiler, scheme=scheme,
            runner_source=runner_source, runner_receipt=runner_receipt,
            material=material, wrong_root=wrong, zkir=zkir, prior=self.prior)
        self.assertIn(self.preflight / 'k1-capture/honest.preimage', paths)
        admitted = gate.fingerprint_inputs(paths)
        self.write(self.preflight / 'k1-capture/honest.preimage', 'mutated after admission\n')
        with self.assertRaisesRegex(RuntimeError, 'input changed during qualification'):
            gate.require_inputs_unchanged(paths, admitted)


if __name__ == '__main__':
    unittest.main()
