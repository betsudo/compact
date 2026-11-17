#!/usr/bin/env python3
# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
"""Source-only refusal controls for the native Linux for-keyword gate."""

import importlib.util
import json
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ROOT / '.github/workflows/ledger9-keyword-for-linux.yml'
PROOF_GATE = ROOT / 'tools/compact-rust-backend/check_upstream_ledger9_for_generic_bound_proof.py'
STATIC_GATE = ROOT / 'tools/compact-rust-backend/check_static_for_index_domain.py'
VERIFIER = ROOT / '.github/scripts/verify_keyword_for_linux_receipts.py'
spec = importlib.util.spec_from_file_location('verify_keyword_for_linux_receipts', VERIFIER)
assert spec and spec.loader
verifier = importlib.util.module_from_spec(spec)
spec.loader.exec_module(verifier)
sys.path.insert(0, str(ROOT / 'tools/compact-rust-backend'))
from check_upstream_ledger9_const_keywords import valid_source_signature

proof_spec = importlib.util.spec_from_file_location('check_for_generic_bound_proof', PROOF_GATE)
assert proof_spec and proof_spec.loader
proof_gate = importlib.util.module_from_spec(proof_spec)
proof_spec.loader.exec_module(proof_gate)
static_spec = importlib.util.spec_from_file_location('check_static_for_index_domain', STATIC_GATE)
assert static_spec and static_spec.loader
static_gate = importlib.util.module_from_spec(static_spec)
static_spec.loader.exec_module(static_gate)


class KeywordForLinuxWorkflowControls(unittest.TestCase):
    def test_head_signature_matches_semantic_gate(self):
        identity = subprocess.check_output(
            ['git', 'log', '-1', '--format=%G?%n%an <%ae>%n%B'],
            cwd=ROOT, text=True,
        )
        signature, author, body = identity.split('\n', 2)
        self.assertTrue(valid_source_signature(signature, author, body))

    def test_static_producer_and_verifier_rosters_match(self):
        original = static_gate.ORIGINAL.read_text()
        variants = static_gate.variants(original)
        self.assertEqual(set(variants), set(verifier.STATIC_ROSTER))
        for name, (_, ts_passes, refusal) in variants.items():
            self.assertEqual(verifier.STATIC_ROSTER[name],
                             (ts_passes, ts_passes and refusal is None))

    def test_selected_sha_and_fail_closed_graph(self):
        text = WORKFLOW.read_text()
        for ref in re.findall(r'^\s+uses:\s+([^\s#]+)', text, re.MULTILINE):
            if not ref.startswith('./'):
                self.assertRegex(ref, r'^[^@]+@[0-9a-f]{40}$')
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

    def test_occurrence_log_requires_current_108_controls(self):
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            log = base / 'occurrence.log'
            log.write_text('108 Rust collection index occurrence controls passed\n')
            verifier.occurrence_log(log)
            log.write_text('64 Rust collection index occurrence controls passed\n')
            with self.assertRaisesRegex(ValueError, 'source occurrence controls'):
                verifier.occurrence_log(log)
            log.unlink()
            log.symlink_to(base / 'other.log')
            with self.assertRaisesRegex(ValueError, 'symlinked source occurrence log'):
                verifier.occurrence_log(log)

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
                               semantic_path=base/'semantic.json',
                               semantic_root=base, zkir=base/'zkir')

    def test_physical_dependency_controls_refuse_unknown_and_tampered(self):
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            source = base / 'source.rs'
            source.write_text('fn pinned() {}\n')
            path = str(source.resolve())
            inventory = {'source_hashes': {path: verifier.sha(source)}}
            self.assertEqual(proof_gate.dependency_refusals({path}, inventory, base), {
                'unknown_dependency_rejected': True,
                'tampered_dependency_hash_rejected': True})

    def test_linux_verifier_refuses_unadmitted_or_changed_depfile_inputs(self):
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            root = base / 'repo'
            root.mkdir()
            body = root / 'tools/compact-rust-proof-smoke/src/for_generic_bound_proof.rs'
            body.parent.mkdir(parents=True)
            body.write_text('fn proof() {}\n')
            source = root / 'pinned.rs'
            source.write_text('fn source() {}\n')
            subprocess.run(['git', 'init', '-q', str(root)], check=True)
            subprocess.run(['git', 'add', 'pinned.rs',
                            'tools/compact-rust-proof-smoke/src/for_generic_bound_proof.rs'],
                           cwd=root, check=True)
            semantic = base / 'semantic'
            both = semantic / 'generated/generic_bound/both'
            generated = [both / 'contract/lib.rs',
                         both / 'contract/compact-rust-ir.json',
                         both / 'zkir/for_generic_bound.zkir']
            for member in generated:
                member.parent.mkdir(parents=True, exist_ok=True)
                member.write_text(member.name)
            dependency = base / 'cargo/src/lib.rs'
            build_input = base / 'target/out/generated.rs'
            for member in (dependency, build_input):
                member.parent.mkdir(parents=True, exist_ok=True)
                member.write_text(member.name)
            output = base / 'proof'
            binding = output / 'binding'
            binding.mkdir(parents=True)
            source_map = {str(path.relative_to(root)): verifier.sha(path)
                          for path in (source, body)}
            source_path = binding / 'source-hashes.json'
            source_path.write_text(json.dumps(source_map))
            semantic_map = {str(path.relative_to(semantic / 'generated')): verifier.sha(path)
                            for path in generated}
            inventories = {
                'source_hashes': {str(path.resolve()): verifier.sha(path)
                                  for path in (source, body)},
                'generated_input_hashes': {str(path.resolve()): verifier.sha(path)
                                           for path in generated},
                'dependency_source_hashes': {str(dependency.resolve()): verifier.sha(dependency)},
                'generated_build_inputs_before_compile': {
                    str(build_input.resolve()): verifier.sha(build_input)},
            }
            depfile = output / 'proof-runner.d'
            paths = [str(path.resolve()) for path in
                     (source, body, *generated, dependency, build_input)]
            depfile.write_text('runner: ' + ' '.join(paths) + '\n')
            physical, origins = proof_gate.runner_inputs.qualify_dependency_inputs(
                set(paths), inventories, verifier.sha)
            inputs = binding / 'inputs.json'
            inputs.write_text(json.dumps({**inventories,
                                          'physical_inputs_before_build': physical,
                                          'physical_input_origins': origins}))
            receipt = {
                'source_count': len(source_map),
                'source_hashes_sha256': verifier.sha(source_path),
                'runner_inventory_sha256': verifier.sha(inputs),
                'runner_physical_input_count': len(physical),
                'runner_physical_input_origins': origins,
                'dependency_refusals': {
                    'unknown_dependency_rejected': True,
                    'tampered_dependency_hash_rejected': True},
            }
            previous_root = verifier.ROOT
            verifier.ROOT = root.resolve()
            try:
                verifier.verify_runner_inputs(
                    receipt, output, semantic, {'generated_files_sha256': semantic_map})
                unknown = base / 'unknown.rs'
                unknown.write_text('fn unknown() {}\n')
                depfile.write_text('runner: ' + ' '.join([*paths, str(unknown)]) + '\n')
                with self.assertRaisesRegex(ValueError, 'missing or inconsistent'):
                    verifier.verify_runner_inputs(
                        receipt, output, semantic, {'generated_files_sha256': semantic_map})
                depfile.write_text('runner: ' + ' '.join(paths) + '\n')
                dependency.write_text('fn tampered() {}\n')
                with self.assertRaisesRegex(ValueError, 'input differs'):
                    verifier.verify_runner_inputs(
                        receipt, output, semantic, {'generated_files_sha256': semantic_map})
            finally:
                verifier.ROOT = previous_root


if __name__ == '__main__':
    unittest.main()
