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
import shutil
import subprocess
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
    def foreign_fixture(self, output):
        names = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only',
                                         verifier.PINNED, *verifier.FOREIGN_DIRS],
                                        cwd=ROOT, text=True).splitlines()
        originals = {name: verifier.sha256(ROOT / name) for name in names}
        controls = {name: verifier.sha256(ROOT / name) for name in
                    ('tools/compact-rust-backend/tests/foreign_opaque_field/check.mjs',
                     'tools/compact-rust-backend/tests/foreign_opaque_field/consumer.rs')}
        logs = {}
        for name in verifier.FOREIGN_LOGS:
            value = ''
            if name.startswith('opaque-') and name.endswith('-runtime.log'):
                value = 'opaque: original Uint8Array assertion passed\n'
            if name.startswith('foreign-') and name.endswith('-runtime.log'):
                value = 'foreign: five original zero-result assertions passed\n'
            if name.startswith('field-'):
                value = 'is out of Field range\n'
            if name == 'cargo-test.log':
                value = ('test original_opaque_result_is_bytes ... ok\n'
                         'test five_original_foreign_zero_results_ordinary_and_recorded ... ok\n'
                         'test result: ok. 2 passed; 0 failed;\n')
            (output / name).write_text(value)
            logs[name] = verifier.sha256(output / name)
        rust = output / 'rust-consumer'
        (rust / 'tests').mkdir(parents=True)
        shutil.copyfile(ROOT / 'tools/compact-rust-backend/tests/foreign_opaque_field/consumer.rs',
                        rust / 'tests/foreign_opaque_field.rs')
        (rust / 'Cargo.lock').write_text('lock\n')
        generated = {}
        for kind in ('opaque', 'foreign'):
            generated[kind] = {}
            for mode in ('typescript', 'both'):
                root = output / 'generated' / kind / mode
                (root / 'compiler').mkdir(parents=True)
                (root / 'contract').mkdir()
                circuits = ([{'name': name, 'proof': True, 'pure': False}
                             for name in verifier.FOREIGN_NAMES] if kind == 'foreign' else
                            [{'name': 'opaque_casts_to_opaque', 'proof': False, 'pure': True,
                              'result-type': {'type-name': 'Opaque', 'tsType': 'Uint8Array'}}])
                (root / 'compiler/contract-info.json').write_text(json.dumps({'circuits': circuits}))
                (root / 'contract/index.js').write_text('export const contract = 1;\n')
                if mode == 'both':
                    (root / 'contract/lib.rs').write_text('pub fn contract() {}\n')
                if kind == 'foreign':
                    (root / 'zkir').mkdir()
                    for name in verifier.FOREIGN_NAMES:
                        (root / f'zkir/{name}.zkir').write_text('zkir\n')
                generated[kind][mode] = verifier.hash_tree(root)
        return {'original_files': originals, 'local_controls': controls,
                'command_logs_sha256': logs, 'generated': generated,
                'cargo_target': str(output / 'shared-target'),
                'proof_execution': False, 'funded_application': False,
                'limitations': ['native Linux generated semantics qualified',
                                'five foreign proof=true exports are unproved'],
                'rust_consumer_lock_sha256': verifier.sha256(rust / 'Cargo.lock')}

    def test_foreign_logs_artifacts_refusal_and_scope_tamper_refused(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary)
            target = output / 'shared-target'
            target.mkdir()
            receipt = self.foreign_fixture(output)
            verifier.verify_foreign(receipt, output, target)
            log = output / 'foreign-both-runtime.log'
            log.write_text('forged success\n')
            with self.assertRaisesRegex(ValueError, 'foreign command log changed'):
                verifier.verify_foreign(receipt, output, target)
            log.unlink()
            with self.assertRaisesRegex(ValueError, 'foreign command log roster differs'):
                verifier.verify_foreign(receipt, output, target)
            log.write_text('foreign: five original zero-result assertions passed\n')
            artifact = output / 'generated/foreign/both/contract/lib.rs'
            artifact.write_text('pub fn forged() {}\n')
            with self.assertRaisesRegex(ValueError, 'complete generated artifacts differ'):
                verifier.verify_foreign(receipt, output, target)
            artifact.write_text('pub fn contract() {}\n')
            refusal = output / 'field-both-compile.log'
            refusal.write_text('compiled successfully\n')
            receipt['command_logs_sha256'][refusal.name] = verifier.sha256(refusal)
            with self.assertRaisesRegex(ValueError, 'Field refusal changed'):
                verifier.verify_foreign(receipt, output, target)
            refusal.write_text('is out of Field range\n')
            receipt['command_logs_sha256'][refusal.name] = verifier.sha256(refusal)
            receipt['proof_execution'] = True
            with self.assertRaisesRegex(ValueError, 'foreign proof scope'):
                verifier.verify_foreign(receipt, output, target)

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

    def test_four_gates_share_cargo_target_and_preserve_scope(self):
        text = WORKFLOW.read_text()
        match = re.search(r'for gate in ([^;]+); do', text)
        self.assertIsNotNone(match)
        self.assertEqual(match.group(1).split(), list(verifier.CASES))
        self.assertIn('mkdir -p "$BYTES_NEXT_CARGO_TARGET"', text)
        self.assertIn('--cargo-target "$BYTES_NEXT_CARGO_TARGET"', text)
        for case in verifier.CASES:
            if case == 'foreign_opaque_field':
                path = ROOT / 'tools/compact-rust-backend/check_upstream_ledger9_foreign_opaque_field.py'
                source = ast.unparse(ast.parse(path.read_text()))
                self.assertIn('cargo-target', source)
                self.assertIn("'cargo_target'", source)
                self.assertIn("'proof_execution': False", source)
                self.assertIn('native Linux generated semantics qualified', source)
                continue
            path = ROOT / 'tools/compact-rust-backend' / f'check_upstream_ledger9_bytes_{case}.py'
            main = next(node for node in ast.parse(path.read_text()).body
                        if isinstance(node, ast.FunctionDef) and node.name == 'main')
            source = ast.unparse(main)
            self.assertIn('cargo-target', source)
            self.assertIn("'cargo_target'", source)
            self.assertIn("'proof_execution': False", source)
            self.assertIn("'native Linux generated semantics qualified'", source)
        self.assertEqual(sum(verifier.ORIGINAL_COUNT.values()), 32)
        source = VERIFIER.read_text()
        self.assertIn("'proof_required_exports_not_qualified_here': 12", source)
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
            foreign_output = base / 'foreign_opaque_field'
            foreign_output.mkdir()
            foreign_log = base / 'foreign_opaque_field.stdout.log'
            foreign_log.write_text(json.dumps({'status': 'passed', 'head': head,
                                               'output': str(foreign_output)}) + '\n')
            foreign_receipt = foreign_output / 'receipt.json'
            foreign_receipt.write_text(receipt.read_text())
            with self.assertRaisesRegex(ValueError, 'native source or result differs'):
                verifier.validate('foreign_opaque_field', foreign_receipt, foreign_log,
                                  head=head, tree=tree, compiler=base / 'compiler',
                                  recipe_path=base / 'recipe', runtime=base / 'runtime',
                                  runtime_snapshot={}, runtime_recipe={}, target=base / 'target')

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
                    if name == 'manual_update':
                        row['zkir_sha256'] = verifier.sha256(
                            root / 'zkir/bytes_manual_update_semantics.zkir')
                    modes[mode] = row
                generated[name] = modes
            receipt = {'generated': generated,
                       'generated_files_sha256': verifier.hash_tree(output / 'generated')}
            verifier.verify_generated('manual_witness', receipt, output)
            manual = receipt['generated']['manual_update']['both']
            zkir_hash = manual.pop('zkir_sha256')
            with self.assertRaisesRegex(ValueError, 'artifact fields differ'):
                verifier.verify_generated('manual_witness', receipt, output)
            manual['zkir_sha256'] = zkir_hash
            witness = receipt['generated']['witness_return']['both']
            witness['zkir_sha256'] = zkir_hash
            with self.assertRaisesRegex(ValueError, 'artifact fields differ'):
                verifier.verify_generated('manual_witness', receipt, output)
            witness.pop('zkir_sha256')
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

    def test_loop_zkir_omission_and_proof_free_control_refused(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary)
            generated = {}
            for name in verifier.GENERATED['loops']:
                modes = {}
                for mode in ('typescript', 'both'):
                    root = output / 'generated' / name / mode
                    (root / 'contract').mkdir(parents=True)
                    (root / 'compiler').mkdir()
                    (root / 'contract/index.js').write_text('export const value = 1;\n')
                    (root / 'compiler/contract-manifest.json').write_text('{}\n')
                    (root / 'compiler/contract-info.json').write_text('{}\n')
                    if name != 'zero_operand_order':
                        (root / 'zkir').mkdir()
                        (root / f'zkir/bytes_loop_{name}.zkir').write_text('zkir\n')
                    if mode == 'both':
                        (root / 'contract/lib.rs').write_text('pub fn value() {}\n')
                        (root / 'contract/compact-rust-ir.json').write_text('{}\n')
                    log = output / f'{name}-{mode}-runtime.log'
                    log.write_text(verifier.runtime_text('loops', name))
                    artifacts = {
                        'typescript_sha256': 'contract/index.js',
                        'manifest_sha256': 'compiler/contract-manifest.json',
                        'contract_info_sha256': 'compiler/contract-info.json',
                    }
                    if name != 'zero_operand_order':
                        artifacts['zkir_sha256'] = f'zkir/bytes_loop_{name}.zkir'
                    if mode == 'both':
                        artifacts.update(rust_sha256='contract/lib.rs',
                                         rust_ir_sha256='contract/compact-rust-ir.json')
                    modes[mode] = {
                        **{key: verifier.sha256(root / relative)
                           for key, relative in artifacts.items()},
                        'runtime_log_sha256': verifier.sha256(log),
                    }
                generated[name] = modes
            refusal = output / 'dynamic-refusal.log'
            refusal.write_text(
                'Bytes index did not reduce to a constant nonnegative value at compile time\n')
            receipt = {
                'generated': generated,
                'generated_files_sha256': verifier.hash_tree(output / 'generated'),
                'dynamic_index_refusal_log_sha256': verifier.sha256(refusal),
            }
            verifier.verify_generated('loops', receipt, output)
            missing = receipt['generated']['iteration']['both'].pop('zkir_sha256')
            with self.assertRaisesRegex(ValueError, 'artifact fields differ'):
                verifier.verify_generated('loops', receipt, output)
            receipt['generated']['iteration']['both']['zkir_sha256'] = missing
            control = output / 'generated/zero_operand_order/both/zkir'
            control.mkdir()
            (control / 'unexpected.zkir').write_text('forged\n')
            receipt['generated_files_sha256'] = verifier.hash_tree(output / 'generated')
            with self.assertRaisesRegex(ValueError, 'proof-free case emitted ZKIR'):
                verifier.verify_generated('loops', receipt, output)


if __name__ == '__main__':
    unittest.main()
