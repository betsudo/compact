#!/usr/bin/env python3
# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
"""Source-only refusal controls for the native Linux for-keyword gate."""

import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ROOT / '.github/workflows/ledger9-keyword-for-linux.yml'
PROOF_GATE = ROOT / 'tools/compact-rust-backend/check_upstream_ledger9_for_generic_bound_proof.py'
VERIFIER = ROOT / '.github/scripts/verify_keyword_for_linux_receipts.py'
spec = importlib.util.spec_from_file_location('verify_keyword_for_linux_receipts', VERIFIER)
assert spec and spec.loader
verifier = importlib.util.module_from_spec(spec)
spec.loader.exec_module(verifier)


class KeywordForLinuxWorkflowControls(unittest.TestCase):
    def test_selected_sha_and_fail_closed_graph(self):
        text = WORKFLOW.read_text()
        self.assertIn("'codex/ledger9-keyword-for-linux/*'", text)
        self.assertIn('refs/heads/codex/ledger9-keyword-for-linux/$SOURCE_SHA', text)
        self.assertIn('test "$EXPECTED_SHA" = "$SOURCE_SHA"', text)
        self.assertEqual(text.count('fetch-depth: 0'), 2)
        pinned = ('git fetch --no-tags --depth=1 '
                  'https://github.com/LFDT-Minokawa/compact.git '
                  '672bd0f226f47a7d036c5c074ce71fbe1af1925e')
        self.assertEqual(text.count(pinned), 2)
        self.assertEqual(text.count('bash .github/scripts/trust_vector_source_key.sh'), 2)
        preflight = text.split('  source-preflight:\n', 1)[1].split('  compiler-linux:\n', 1)[0]
        producer = text.split('  compiler-linux:\n', 1)[1].split('  keyword-for:\n', 1)[0]
        consumer = text.split('  keyword-for:\n', 1)[1].split('  complete:\n', 1)[0]
        complete = text.split('  complete:\n', 1)[1]
        self.assertIn('test_keyword_for_linux_workflow.py', preflight)
        self.assertIn('test_compiler_closure.py', preflight)
        self.assertIn('ledger-9.1.0.1', preflight)
        self.assertIn('needs: source-preflight', producer)
        self.assertIn('uses: ./.github/workflows/compiler-producer.yml', producer)
        self.assertIn('system: x86_64-linux', producer)
        self.assertIn('export-shell: true', producer)
        self.assertIn('needs: [source-preflight, compiler-linux]', consumer)
        for field in ('artifact-id', 'shell-artifact-id', 'producer-attempt'):
            self.assertIn('needs.compiler-linux.outputs.' + field, consumer)
        self.assertIn('compiler_closure.py import', consumer)
        self.assertIn('actions/import-compiler-shell', consumer)
        self.assertIn('toolchain: 1.99.0', consumer)
        self.assertIn('record_compiler_recipe.py', consumer)
        self.assertIn('nix build --no-link .#runtime.package', consumer)
        self.assertIn('nix build --no-link --print-out-paths .#zkir-v3', consumer)
        self.assertIn('if: ${{ always() }}', complete)
        for owner in ('PREFLIGHT_RESULT', 'PRODUCER_RESULT', 'KEYWORD_FOR_RESULT'):
            self.assertIn('test "$' + owner + '" = success', complete)

    def test_exact_original_proof_and_refusal_order(self):
        text = WORKFLOW.read_text()
        semantic = text.index('check_upstream_ledger9_for_keywords.py')
        refusals = text.index('check_static_for_index_domain.py')
        occurrence = text.index('compiler/test-rust-vector-indices.ss')
        prepare = text.index('--prepare-proof-material --parameter-k 6 --parameter-k 7')
        proof = text.index('check_upstream_ledger9_for_generic_bound_proof.py')
        self.assertLess(semantic, refusals)
        self.assertLess(refusals, occurrence)
        self.assertLess(occurrence, prepare)
        self.assertLess(prepare, proof)
        self.assertIn('--expected-source-sha "$SOURCE_SHA"', text)
        self.assertIn('--cargo-target "$KEYWORD_FOR_CARGO_TARGET"', text)
        self.assertIn('--grouped-gate "$RUNNER_TEMP/keyword-for-semantic"', text)
        self.assertEqual(text.count('verify_keyword_for_linux_receipts.py'), 2)
        self.assertIn('--proof "$RUNNER_TEMP/keyword-for-proof"', text)
        self.assertIn('${{ runner.temp }}/keyword-for-proof/', text)
        proof_source = PROOF_GATE.read_text()
        self.assertIn('PROOF_BODY_SHA256', proof_source)
        self.assertIn("'dedicated signed generic-bound proof body changed'", proof_source)
        self.assertIn("gate['head'] == source['head']", proof_source)
        self.assertIn("gate['tree'] == source['tree']", proof_source)
        self.assertIn("files(generated) == gate['generated_files_sha256']", proof_source)
        self.assertIn("'strict_application'] == 'default'", proof_source)
        self.assertIn("'replay_refusal'] == 'IntentAlreadyExists'", proof_source)

    def test_gate_log_refuses_wrong_head_or_status(self):
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            log = base / 'gate.log'
            expected = {'status':'passed', 'head':'a' * 40, 'cases':10,
                        'output':str(base / 'semantic')}
            log.write_text(json.dumps(expected) + '\n')
            verifier.gate_log(log, expected)
            changed = dict(expected, head='b' * 40)
            with self.assertRaisesRegex(ValueError, 'gate log identity'):
                verifier.gate_log(log, changed)
            log.write_text(json.dumps(dict(expected, status='failed')) + '\n')
            with self.assertRaisesRegex(ValueError, 'gate log identity'):
                verifier.gate_log(log, expected)

    def test_artifact_inventory_refuses_mutation_and_symlink(self):
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            original = base / 'original'
            original.write_text('pinned\n')
            snapshot = verifier.inventory(base)
            original.write_text('changed\n')
            self.assertNotEqual(verifier.inventory(base), snapshot)
            original.write_text('pinned\n')
            (base / 'borrowed').symlink_to(original)
            with self.assertRaisesRegex(ValueError, 'symlinked artifact'):
                verifier.inventory(base)

    def test_darwin_and_wrong_proof_source_refused(self):
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            source = {'head':'a' * 40, 'tree':'b' * 40}
            receipt = {'status':'passed', 'source_clean':True,
                       'platform':'Darwin arm64', 'head':source['head'],
                       'tree':source['tree']}
            with self.assertRaisesRegex(ValueError, 'native Linux for semantic'):
                verifier.semantic(receipt, base, source=source,
                                  compiler=base/'compiler', recipe=base/'recipe',
                                  runtime=base/'runtime')
            proof = dict(receipt, platform='Linux x86_64', grouped_head='c' * 40)
            with self.assertRaisesRegex(ValueError, 'native Linux proof identity'):
                verifier.proof(proof, base, source=source, semantic_receipt={},
                               semantic_path=base/'semantic.json', zkir=base/'zkir')


if __name__ == '__main__':
    unittest.main()
