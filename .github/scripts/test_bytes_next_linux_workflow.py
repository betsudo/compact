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
"""Source and tamper controls for the native Linux Bytes workflow."""

import ast
import importlib.util
import json
from pathlib import Path
import re
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ROOT / '.github/workflows/ledger9-bytes-next-linux.yml'
VERIFIER = ROOT / '.github/scripts/verify_bytes_next_linux_receipts.py'
spec = importlib.util.spec_from_file_location('verify_bytes_next_linux_receipts', VERIFIER)
assert spec and spec.loader
verifier = importlib.util.module_from_spec(spec)
spec.loader.exec_module(verifier)


class BytesNextLinuxWorkflowControls(unittest.TestCase):
    def test_exact_sha_and_fail_closed_job_graph(self):
        text = WORKFLOW.read_text()
        self.assertIn("'codex/ledger9-bytes-next-linux/*'", text)
        self.assertIn('refs/heads/codex/ledger9-bytes-next-linux/$SOURCE_SHA', text)
        self.assertIn('test "$EXPECTED_SHA" = "$SOURCE_SHA"', text)
        self.assertEqual(text.count('bash .github/scripts/trust_vector_source_key.sh'), 2)
        self.assertIn('fetch-depth: 0', text)
        self.assertIn("git cat-file -e '672bd0f226f47a7d036c5c074ce71fbe1af1925e^{commit}'", text)
        preflight = text.split('  source-preflight:\n', 1)[1].split('  compiler-linux:\n', 1)[0]
        producer = text.split('  compiler-linux:\n', 1)[1].split('  bytes-next-semantics:\n', 1)[0]
        consumer = text.split('  bytes-next-semantics:\n', 1)[1].split('  complete:\n', 1)[0]
        complete = text.split('  complete:\n', 1)[1]
        self.assertIn('test_bytes_next_linux_workflow.py', preflight)
        self.assertIn('needs: source-preflight', producer)
        self.assertIn('uses: ./.github/workflows/compiler-producer.yml', producer)
        self.assertIn('system: x86_64-linux', producer)
        self.assertIn('export-shell: true', producer)
        self.assertIn('needs: [source-preflight, compiler-linux]', consumer)
        for name in ('artifact-id', 'shell-artifact-id', 'producer-attempt'):
            self.assertIn('needs.compiler-linux.outputs.' + name, consumer)
        for name in ('compiler_closure.py import', 'actions/import-compiler-shell',
                     'record_compiler_recipe.py', 'nix build --no-link .#runtime.package',
                     'verify_bytes_next_linux_receipts.py', 'toolchain: 1.99.0'):
            self.assertIn(name, consumer)
        self.assertIn('always()', complete)
        for owner in ('PREFLIGHT_RESULT', 'PRODUCER_RESULT', 'BYTES_NEXT_RESULT'):
            self.assertIn('test "$' + owner + '" = success', complete)

    def test_three_gates_share_cargo_target_and_preserve_scope(self):
        text = WORKFLOW.read_text()
        match = re.search(r'for gate in ([^;]+); do', text)
        self.assertIsNotNone(match)
        self.assertEqual(match.group(1).split(), list(verifier.CASES))
        self.assertIn('mkdir -p "$BYTES_NEXT_CARGO_TARGET"', text)
        self.assertIn('--cargo-target "$BYTES_NEXT_CARGO_TARGET"', text)
        for case in verifier.CASES:
            path = ROOT / 'tools/compact-rust-backend' / f'check_upstream_ledger9_bytes_{case}.py'
            main = next(node for node in ast.parse(path.read_text()).body
                        if isinstance(node, ast.FunctionDef) and node.name == 'main')
            source = ast.unparse(main)
            self.assertIn('cargo-target', source)
            self.assertIn("'cargo_target'", source)
            self.assertIn("'proof_execution': False", source)
            self.assertIn("'native Linux generated semantics qualified'", source)
        self.assertEqual(sum(verifier.ORIGINAL_COUNT.values()), 24)
        source = VERIFIER.read_text()
        self.assertIn("'proof_required_exports_not_qualified_here': 7", source)
        self.assertIn("'funded_application': False", source)
        self.assertIn("'constructor_execution_proved': False", source)

    def test_gate_log_and_darwin_receipt_refused(self):
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            output = base / 'manual_witness'
            output.mkdir()
            head, tree = 'a' * 40, 'b' * 40
            log = base / 'manual_witness.stdout.log'
            log.write_text(json.dumps({'status': 'passed', 'head': head,
                                       'output': str(output)}) + '\n')
            verifier.verify_gate_log(log, head, output)
            with self.assertRaisesRegex(ValueError, 'exact-head completion'):
                verifier.verify_gate_log(log, 'c' * 40, output)
            receipt = output / 'receipt.json'
            receipt.write_text(json.dumps({'status': 'passed', 'head': head, 'tree': tree,
                                           'source_clean': True, 'platform': 'Darwin arm64'}))
            with self.assertRaisesRegex(ValueError, 'native source or result differs'):
                verifier.validate('manual_witness', receipt, log, head=head, tree=tree,
                                  compiler=base / 'compiler', recipe_path=base / 'recipe',
                                  runtime=base / 'runtime', runtime_snapshot={},
                                  runtime_recipe={}, target=base / 'target')

    def test_original_and_local_inventory_tamper_refused(self):
        manifest = json.loads((ROOT / verifier.MANIFESTS['loops']).read_text())
        receipt = {'original_files': dict(manifest['files'])}
        self.assertEqual(verifier.verify_originals('loops', receipt), 9)
        receipt['original_files'].pop(next(iter(receipt['original_files'])))
        with self.assertRaisesRegex(ValueError, 'original roster differs'):
            verifier.verify_originals('loops', receipt)
        with self.assertRaisesRegex(ValueError, 'local source roster differs'):
            verifier.verify_local('loops', {'local_sources': {}})

    def test_forged_runtime_and_unlisted_generated_file_refused(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary)
            generated = {}
            for name in verifier.GENERATED['manual_witness']:
                modes = {}
                for mode in ('typescript', 'both'):
                    root = output / 'generated' / name / mode
                    (root / 'contract').mkdir(parents=True)
                    (root / 'compiler').mkdir()
                    (root / 'contract/index.js').write_text('export const value = 1;\n')
                    (root / 'compiler/contract-manifest.json').write_text('{}\n')
                    (root / 'compiler/contract-info.json').write_text('{}\n')
                    if name == 'manual_update':
                        (root / 'zkir').mkdir()
                        (root / 'zkir/bytes_manual_update_semantics.zkir').write_text('zkir\n')
                    if mode == 'both':
                        (root / 'contract/lib.rs').write_text('pub fn value() {}\n')
                        (root / 'contract/compact-rust-ir.json').write_text('{}\n')
                    log = output / f'{name}-{mode}-runtime.log'
                    log.write_text(verifier.runtime_text('manual_witness', name))
                    row = {'runtime_log_sha256': verifier.sha256(log)}
                    artifacts = {
                        'typescript_sha256': 'contract/index.js',
                        'manifest_sha256': 'compiler/contract-manifest.json',
                        'contract_info_sha256': 'compiler/contract-info.json',
                    }
                    if mode == 'both':
                        artifacts.update(rust_sha256='contract/lib.rs',
                                         rust_ir_sha256='contract/compact-rust-ir.json')
                    row.update({key: verifier.sha256(root / relative)
                                for key, relative in artifacts.items()})
                    modes[mode] = row
                generated[name] = modes
            receipt = {'generated': generated,
                       'generated_files_sha256': verifier.hash_tree(output / 'generated')}
            verifier.verify_generated('manual_witness', receipt, output)
            log = output / 'manual_update-both-runtime.log'
            log.write_text('forged success\n')
            receipt['generated']['manual_update']['both']['runtime_log_sha256'] = verifier.sha256(log)
            with self.assertRaisesRegex(ValueError, 'runtime result or log differs'):
                verifier.verify_generated('manual_witness', receipt, output)
            log.write_text(verifier.runtime_text('manual_witness', 'manual_update'))
            receipt['generated']['manual_update']['both']['runtime_log_sha256'] = verifier.sha256(log)
            (output / 'generated/manual_update/both/contract/unlisted.rs').write_text('x')
            with self.assertRaisesRegex(ValueError, 'complete generated artifact inventory differs'):
                verifier.verify_generated('manual_witness', receipt, output)


if __name__ == '__main__':
    unittest.main()
