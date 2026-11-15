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
"""Source-only controls for the one-shot native Linux scalar workflow."""

import ast
import importlib.util
import json
from pathlib import Path
import re
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ROOT / '.github/workflows/ledger9-scalar-linux.yml'
VERIFIER = ROOT / '.github/scripts/verify_scalar_linux_receipts.py'
spec = importlib.util.spec_from_file_location('verify_scalar_linux_receipts', VERIFIER)
assert spec and spec.loader
verifier = importlib.util.module_from_spec(spec)
spec.loader.exec_module(verifier)


class ScalarLinuxWorkflowControls(unittest.TestCase):
    def test_exact_sha_trigger_and_fail_closed_graph(self):
        text = WORKFLOW.read_text()
        self.assertIn("'codex/ledger9-scalar-linux/*'", text)
        self.assertIn('refs/heads/codex/ledger9-scalar-linux/$SOURCE_SHA', text)
        self.assertIn('test "$EXPECTED_SHA" = "$SOURCE_SHA"', text)
        self.assertIn('fetch-depth: 0', text)
        self.assertEqual(text.count('bash .github/scripts/trust_vector_source_key.sh'), 2)
        self.assertIn("git cat-file -e '672bd0f226f47a7d036c5c074ce71fbe1af1925e^{commit}'", text)
        preflight = text.split('  source-preflight:\n', 1)[1].split('  compiler-linux:\n', 1)[0]
        producer = text.split('  compiler-linux:\n', 1)[1].split('  scalar-semantics:\n', 1)[0]
        consumer = text.split('  scalar-semantics:\n', 1)[1].split('  complete:\n', 1)[0]
        complete = text.split('  complete:\n', 1)[1]
        self.assertIn('test_scalar_linux_workflow.py', preflight)
        self.assertIn('needs: source-preflight', producer)
        self.assertIn('uses: ./.github/workflows/compiler-producer.yml', producer)
        self.assertIn('system: x86_64-linux', producer)
        self.assertIn('export-shell: true', producer)
        self.assertIn('needs: [source-preflight, compiler-linux]', consumer)
        for name in ('artifact-id', 'shell-artifact-id', 'producer-attempt'):
            self.assertIn('needs.compiler-linux.outputs.' + name, consumer)
        self.assertIn('compiler_closure.py import', consumer)
        self.assertIn('actions/import-compiler-shell', consumer)
        self.assertIn('toolchain: 1.99.0', consumer)
        self.assertIn('nix build --no-link .#runtime.package', consumer)
        self.assertIn('record_compiler_recipe.py', consumer)
        self.assertIn('verify_scalar_linux_receipts.py', consumer)
        self.assertIn('tee "$RUNNER_TEMP/scalar-semantic-evidence/$gate.stdout.log"', consumer)
        self.assertIn('--all "$RUNNER_TEMP/scalar-semantic-evidence"', consumer)
        self.assertIn('${{ runner.temp }}/scalar-semantic-evidence/', consumer)
        self.assertIn('if: ${{ always() }}', complete)
        for owner in ('PREFLIGHT_RESULT', 'PRODUCER_RESULT', 'SCALAR_RESULT'):
            self.assertIn('test "$' + owner + '" = success', complete)

    def test_exact_roster_and_shared_cargo_target(self):
        text = WORKFLOW.read_text()
        match = re.search(r'for gate in ([^;]+); do', text)
        self.assertIsNotNone(match)
        roster = match.group(1).split()
        self.assertEqual(roster, list(verifier.CASES))
        self.assertIn('--cargo-target "$SCALAR_CARGO_TARGET"', text)
        self.assertIn('SCALAR_CARGO_TARGET=%s/scalar-shared-cargo-target', text)
        for case in roster:
            path = ROOT / 'tools/compact-rust-backend' / f'check_upstream_ledger9_{case}.py'
            self.assertTrue(path.is_file(), case)
            parsed = ast.parse(path.read_text())
            main = next(node for node in parsed.body if isinstance(node, ast.FunctionDef)
                        and node.name == 'main')
            self.assertIn('cargo-target', ast.unparse(main), case)
            self.assertIn("'cargo_target'", ast.unparse(main), case)
        self.assertEqual(sum(count for count, _ in verifier.CASES.values()), 106)

    def test_pinned_signer_and_no_native_proof_claim(self):
        key = (ROOT / '.github/scripts/trust_vector_source_key.sh').read_text()
        self.assertIn('F8851C118C66BF98DB9B1B5DB75A9FB8C6327844', key)
        self.assertIn('git verify-commit HEAD', key)
        self.assertNotIn('recv-keys', key)
        source = VERIFIER.read_text()
        self.assertIn("'proof_required_exports_open': 9", source)
        self.assertIn("'proof_execution': False", source)
        self.assertIn("'funded_application': False", source)
        self.assertIn("'constructor_execution_proved': False", source)

    def test_generated_artifact_mutation_refused(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary)
            generated = {}
            for name in verifier.GENERATED['field_casts']:
                modes = {}
                for mode in ('typescript', 'both'):
                    directory = output / 'generated' / name / mode
                    (directory / 'contract').mkdir(parents=True)
                    (directory / 'compiler').mkdir()
                    (directory / 'contract/index.js').write_text('export const value = 1;\n')
                    (directory / 'compiler/contract-info.json').write_text('{}\n')
                    result = f'{name}: {verifier.RUNTIME_SUFFIX["field_casts"]}'
                    runtime_log = output / f'{name}-{mode}-runtime.log'
                    runtime_log.write_text(result + '\n')
                    row = {'runtime_result': result,
                           'runtime_log_sha256': verifier.sha256(runtime_log)}
                    for key in ('typescript_sha256', 'contract_info_sha256'):
                        row[key] = verifier.sha256(directory / verifier.ARTIFACTS[key])
                    if mode == 'both':
                        (directory / 'contract/lib.rs').write_text('pub fn value() {}\n')
                        (directory / 'contract/compact-rust-ir.json').write_text('{}\n')
                        for key in ('rust_sha256', 'rust_ir_sha256'):
                            row[key] = verifier.sha256(directory / verifier.ARTIFACTS[key])
                    modes[mode] = row
                generated[name] = modes
            receipt = {'generated': generated}
            verifier.verify_generated('field_casts', receipt, output)
            row = receipt['generated']['to_bytes']['both']
            row['runtime_result'] = 'forged success'
            with self.assertRaisesRegex(ValueError, 'artifact inventory differs'):
                verifier.verify_generated('field_casts', receipt, output)
            row['runtime_result'] = f'to_bytes: {verifier.RUNTIME_SUFFIX["field_casts"]}'
            runtime_log = output / 'to_bytes-both-runtime.log'
            runtime_log.write_text('forged success\n')
            with self.assertRaisesRegex(ValueError, 'Node runtime log differs'):
                verifier.verify_generated('field_casts', receipt, output)
            runtime_log.write_text(row['runtime_result'] + '\n')
            (output / 'generated/to_bytes/both/contract/lib.rs').write_text('pub fn changed() {}\n')
            with self.assertRaisesRegex(ValueError, 'lib.rs differs'):
                verifier.verify_generated('field_casts', receipt, output)
            (output / 'generated/to_bytes/both/contract/lib.rs').write_text('pub fn value() {}\n')
            receipt['generated'].pop('to_bytes')
            with self.assertRaisesRegex(ValueError, 'roster differs'):
                verifier.verify_generated('field_casts', receipt, output)

    def test_gate_status_refuses_wrong_head_or_output(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            log = directory / 'gate.log'
            source = 'a' * 40
            output = directory / 'field_casts'
            log.write_text(json.dumps({'status': 'passed', 'head': source,
                                       'output': str(output)}) + '\n')
            verifier.gate_status(log, source_sha=source, output=output)
            with self.assertRaisesRegex(ValueError, 'wrong status'):
                verifier.gate_status(log, source_sha='b' * 40, output=output)
            log.write_text(json.dumps({'status': 'failed', 'head': source,
                                       'output': str(output)}) + '\n')
            with self.assertRaisesRegex(ValueError, 'wrong status'):
                verifier.gate_status(log, source_sha=source, output=output)

    def test_manifest_swap_and_darwin_receipt_refused(self):
        manifest_path = ROOT / verifier.MANIFESTS['field_casts']
        manifest = json.loads(manifest_path.read_text())
        receipt = {'original_files': dict(manifest['files']),
                   'manifest_sha256': verifier.sha256(manifest_path)}
        self.assertEqual(verifier.verify_originals(receipt, 'field_casts'), 21)
        receipt['original_files'].pop(next(iter(receipt['original_files'])))
        with self.assertRaisesRegex(ValueError, 'original inventory differs'):
            verifier.verify_originals(receipt, 'field_casts')
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            output = base / 'field_casts'
            output.mkdir()
            log = base / 'field_casts.stdout.log'
            source = 'a' * 40
            log.write_text(json.dumps({'status': 'passed', 'head': source,
                                       'output': str(output)}) + '\n')
            receipt_path = output / 'receipt.json'
            receipt_path.write_text(json.dumps({'head': source, 'tree': 'b' * 40,
                                                'source_clean': True,
                                                'platform': 'Darwin arm64'}))
            with self.assertRaisesRegex(ValueError, 'native Linux source identity differs'):
                verifier.validate('field_casts', receipt_path, gate_log=log,
                                  source_sha=source, tree='b' * 40,
                                  compiler=base / 'compiler', recipe_path=base / 'recipe',
                                  runtime=base / 'runtime', runtime_snapshot={},
                                  runtime_recipe={}, cargo_target=base / 'target')
        with self.assertRaisesRegex(ValueError, 'local source roster differs'):
            verifier.verify_local_sources('field_casts', {'local_sources': {}})


if __name__ == '__main__':
    unittest.main()
