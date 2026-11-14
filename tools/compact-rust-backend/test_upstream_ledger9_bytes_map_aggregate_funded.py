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

"""Focused admission checks for aggregate Bytes map funded receipts."""

import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest import mock

import check_upstream_ledger9_bytes_map_funded as gate


class AggregatePreflightTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name) / 'source'
        self.preflight = Path(self.temp.name) / 'preflight'
        self.preflight.mkdir()
        self.head = 'a' * 40
        self.tree = 'b' * 40
        self.compiler = Path(self.temp.name) / 'compiler-receipt.json'
        self.compiler.write_text('compiler receipt\n')
        self.original = (self.root /
                         'test-contracts/primitives/bytes/map/transformation/bytes_map_transformation.compact')
        self.write(self.original, 'original Compact source\n')
        self.compile_test = self.original.parent / 'compile.pass.test.ts'
        self.runtime_test = self.original.parent / 'runtime.pass.test.ts'
        self.write(self.compile_test, 'original compile assertion\n')
        self.write(self.runtime_test, 'original runtime assertion\n')
        self.write(self.root / 'tools/compact-rust-backend/check_upstream_ledger9_bytes_map.py',
                   'preflight gate\n')
        self.write(self.root / 'test-contracts/primitives/bytes/map/check.mjs', 'adapter\n')
        self.runtime = Path(self.temp.name) / 'runtime'
        self.nix_runtime = Path(self.temp.name) / 'nix-runtime/lib/node_modules/@midnight-ntwrk/compact-runtime'
        package = '{"name":"@midnight-ntwrk/compact-runtime","version":"0.18.0","dependencies":{}}\n'
        for directory in (self.root / 'runtime', self.runtime, self.nix_runtime):
            self.write(directory / 'package.json', package)
        for directory in (self.runtime, self.nix_runtime):
            self.write(directory / 'dist/index.js', 'runtime dist\n')
        self.proof = {
            'compiler_receipt_sha256': gate.sha(self.compiler),
            'compiler_sha256': 'c' * 64,
            'scheme_sha256': 'd' * 64,
            'artifacts': {},
        }
        artifacts = {}
        for kind, filename in {'typescript': 'index.js', 'rust': 'lib.rs',
                               'rust_ir': 'compact-rust-ir.json'}.items():
            path = self.preflight / 'paired/transformation/contract' / filename
            self.write(path, kind + '\n')
            digest = gate.sha(path)
            self.proof['artifacts'][f'transformation/{kind}'] = digest
            artifacts[f'transformation/paired/{kind}'] = digest
        self.write(self.preflight / 'default/transformation/contract/index.js', 'typescript\n')
        artifacts['transformation/default/typescript'] = self.proof['artifacts'][
            'transformation/typescript']
        self.write(self.preflight / 'cargo-test.log', 'Running tests/bytes_map.rs\n')
        self.write(self.preflight / 'typescript-runtime-recipe.log', 'realized runtime\n')
        for family in ('reordering', 'transformation', 'slice_composition', 'to_bytes'):
            for target in ('default', 'paired'):
                self.write(self.preflight / f'{family}-{target}-typescript.log',
                           f'bytes_map_{family} TypeScript passed\n')
        self.prior = {
            'head': self.head, 'tree': self.tree,
            'upstream': '672bd0f226f47a7d036c5c074ce71fbe1af1925e',
            'compiler_receipt_sha256': gate.sha(self.compiler),
            'compiler_sha256': self.proof['compiler_sha256'],
            'scheme_sha256': self.proof['scheme_sha256'],
            'runner_sha256': gate.sha(self.root /
                                      'tools/compact-rust-backend/check_upstream_ledger9_bytes_map.py'),
            'adapter_sha256': gate.sha(self.root / 'test-contracts/primitives/bytes/map/check.mjs'),
            'original_files': {path.relative_to(self.root).as_posix(): gate.sha(path)
                               for path in (self.original, self.compile_test, self.runtime_test)},
            'typescript_runtime_version': '0.18.0',
            'typescript_runtime_package_sha256': gate.sha(self.runtime / 'package.json'),
            'typescript_runtime_nix_store': str(Path(self.temp.name) / 'nix-runtime'),
            'typescript_runtime_dist_files': {'index.js': gate.sha(self.runtime / 'dist/index.js')},
            'result': 'Four original Bytes map cases passed generated TypeScript and ordinary/recorded Rust; checked Field-to-Uint and large-index regressions passed',
            'cargo_test_log_sha256': gate.sha(self.preflight / 'cargo-test.log'),
            'typescript_runtime_recipe_log_sha256': gate.sha(
                self.preflight / 'typescript-runtime-recipe.log'),
            'artifacts': artifacts,
        }
        self.receipt = self.preflight / 'receipt.json'
        self.patchers = [
            mock.patch.object(gate, 'ROOT', self.root),
            mock.patch.object(gate, 'ORIGINAL_SOURCE', self.original),
            mock.patch.object(gate, 'ORIGINAL_SOURCE_SHA256', gate.sha(self.original)),
            mock.patch.object(gate, 'AGGREGATE_PREFLIGHT_SOURCE_SHA256',
                              self.prior['runner_sha256']),
            mock.patch.object(gate, 'AGGREGATE_ADAPTER_SHA256', self.prior['adapter_sha256']),
            mock.patch.object(gate, 'CASES', {'transformation': 'bytes_map_transformation'}),
            mock.patch.object(gate, 'ORIGINAL_SHA256', {'transformation': {
                'bytes_map_transformation.compact': gate.sha(self.original),
                'compile.pass.test.ts': gate.sha(self.compile_test),
                'runtime.pass.test.ts': gate.sha(self.runtime_test),
            }}),
        ]
        for patcher in self.patchers:
            patcher.start()
            self.addCleanup(patcher.stop)

    @staticmethod
    def write(path: Path, content: str) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)

    def validate(self) -> str:
        self.receipt.write_text(json.dumps(self.prior))
        with (mock.patch.object(gate.shutil, 'which', return_value='/usr/bin/true'),
              mock.patch.object(gate.subprocess, 'check_output',
                                return_value=str(Path(self.temp.name) / 'nix-runtime'))):
            return gate.validate_aggregate_preflight(
                self.receipt, head=self.head, tree=self.tree,
                compiler_receipt=self.compiler, runtime=self.runtime, proof=self.proof)

    def test_accepts_matching_current_head_and_artifact_bytes(self) -> None:
        self.assertEqual(self.validate(), gate.sha(self.receipt))

    def test_refuses_ancestor_head_even_when_artifacts_match(self) -> None:
        self.prior['head'] = '0' * 40
        with self.assertRaisesRegex(RuntimeError, 'current source head and tree'):
            self.validate()

    def test_refuses_substituted_compiler_receipt(self) -> None:
        self.prior['compiler_receipt_sha256'] = '0' * 64
        with self.assertRaisesRegex(RuntimeError, 'compiler receipts differ'):
            self.validate()

    def test_accepts_equivalent_runtime_package_formatting(self) -> None:
        self.write(self.runtime / 'package.json',
                   '{\n  "name": "@midnight-ntwrk/compact-runtime",\n'
                   '  "version": "0.18.0", "dependencies": {}\n}\n')
        self.prior['typescript_runtime_package_sha256'] = gate.sha(self.runtime / 'package.json')
        self.validate()

    def test_refuses_substituted_runtime_metadata(self) -> None:
        self.write(self.runtime / 'package.json',
                   '{"name":"wrong","version":"0.18.0","dependencies":{}}\n')
        self.prior['typescript_runtime_package_sha256'] = gate.sha(self.runtime / 'package.json')
        with self.assertRaisesRegex(RuntimeError, 'runtime package metadata changed'):
            self.validate()

    def test_refuses_substituted_generated_artifact(self) -> None:
        self.write(self.preflight / 'paired/transformation/contract/lib.rs', 'changed Rust\n')
        with self.assertRaisesRegex(RuntimeError, 'generated rust differs'):
            self.validate()

    def test_refuses_receipt_without_qualification_result(self) -> None:
        del self.prior['result']
        with self.assertRaisesRegex(RuntimeError, 'qualification result or test evidence changed'):
            self.validate()

    def test_refuses_substituted_assertion_log(self) -> None:
        self.write(self.preflight / 'transformation-paired-typescript.log', 'not tested\n')
        with self.assertRaisesRegex(RuntimeError, 'assertions did not pass'):
            self.validate()


class GeneratedContractAdmissionTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.contract = Path(temporary.name) / 'contract'
        self.contract.mkdir()
        for name in gate.GENERATED_CONTRACT_FILES:
            (self.contract / name).write_text(name)

    def test_accepts_exact_generated_files(self) -> None:
        gate.validate_generated_contract_files(self.contract)

    def test_refuses_unpinned_build_script(self) -> None:
        (self.contract / 'build.rs').write_text('fn main() {}\n')
        with self.assertRaisesRegex(RuntimeError, 'unpinned or missing files'):
            gate.validate_generated_contract_files(self.contract)

    def test_refuses_symlinked_generated_file(self) -> None:
        path = self.contract / 'index.js'
        path.unlink()
        path.symlink_to(self.contract / 'lib.rs')
        with self.assertRaisesRegex(RuntimeError, 'unpinned or missing files'):
            gate.validate_generated_contract_files(self.contract)


class AggregateProofAdmissionTests(unittest.TestCase):
    def test_refuses_ancestor_and_substituted_proof_before_funded_work(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for name in ('proof', 'runner', 'consumer', 'material'):
                (root / name).mkdir()
            (root / 'proof/receipt.json').write_text(json.dumps({'head': '0' * 40}))
            source = root / 'funded.rs'
            source.write_text('pinned funded consumer\n')
            preflight = root / 'preflight.json'
            preflight.write_text('{}')
            compiler = root / 'compiler.json'
            compiler.write_text('{}')
            arguments = ['gate', '--aggregate-head', '--preflight-receipt', str(preflight),
                         '--preflight-runtime', str(root / 'material'),
                         '--aggregate-compiler-receipt-sha256', gate.sha(compiler),
                         '--aggregate-preflight-receipt-sha256', gate.sha(preflight),
                         '--aggregate-proof-receipt-sha256', gate.sha(root / 'proof/receipt.json'),
                         '--proof-gate', str(root / 'proof'), '--compiler-receipt', str(compiler),
                         '--runner-source', str(root / 'runner'), '--base-consumer',
                         str(root / 'consumer'), '--material', str(root / 'material'),
                         '--cargo-target', str(root / 'target'), '--output', str(root / 'output')]
            def fake_git(*args: str) -> str:
                return {
                    ('status', '--porcelain'): '',
                    ('rev-parse', 'HEAD'): 'a' * 40,
                    ('rev-parse', 'HEAD^{tree}'): 'b' * 40,
                    ('log', '-1', '--format=%G?'): 'G',
                    ('log', '-1', '--format=%B'): 'Signed-off-by: Test User',
                }[args]
            with (mock.patch.object(gate, 'SOURCE', source),
                  mock.patch.object(gate, 'AGGREGATE_FUNDED_SOURCE_SHA256', gate.sha(source)),
                  mock.patch.object(gate, 'git', side_effect=fake_git),
                  mock.patch.object(sys, 'argv', arguments)):
                with self.assertRaisesRegex(RuntimeError, 'proof receipt source binding changed'):
                    gate.main()
                arguments[arguments.index('--aggregate-proof-receipt-sha256') + 1] = 'f' * 64
                with self.assertRaisesRegex(RuntimeError, 'proof receipt differs'):
                    gate.main()


if __name__ == '__main__':
    unittest.main()
