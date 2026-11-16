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
"""Fail-closed static checks for the dedicated exact-head native Linux gate."""

from pathlib import Path
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ROOT / '.github/workflows/ledger9-grouped-linux.yml'
PIN = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'


def require_fragment(text: str, fragment: str) -> None:
    if fragment not in text:
        raise ValueError(f'missing required workflow fragment: {fragment}')


def section(text: str, start: str, end: str | None = None) -> str:
    require_fragment(text, start)
    tail = text.split(start, 1)[1]
    if end is None:
        return tail
    require_fragment(tail, end)
    return tail.split(end, 1)[0]


def validate_workflow(text: str) -> None:
    preflight = section(text, '  source-preflight:\n', '  compiler-linux:\n')
    producer = section(text, '  compiler-linux:\n', '  grouped-originals:\n')
    consumer = section(text, '  grouped-originals:\n', '  complete:\n')
    complete = section(text, '  complete:\n')
    require_fragment(text, "'codex/ledger9-grouped-linux/*'")
    require_fragment(preflight, 'refs/heads/codex/ledger9-grouped-linux/$SOURCE_SHA')
    require_fragment(preflight, 'test "$EXPECTED_SHA" = "$SOURCE_SHA"')
    require_fragment(preflight, 'git verify-commit HEAD')
    require_fragment(preflight, 'test_ledger9_grouped_linux_workflow.py')
    require_fragment(preflight, "compatibility_record()['ledger_version'] == 'ledger-9.1.0.1'")
    require_fragment(preflight, "['version'] == '0.18.0'")
    require_fragment(producer, 'needs: source-preflight')
    require_fragment(producer, 'uses: ./.github/workflows/compiler-producer.yml')
    require_fragment(producer, 'system: x86_64-linux')
    require_fragment(producer, 'export-shell: true')
    require_fragment(consumer, 'needs: [source-preflight, compiler-linux]')
    require_fragment(consumer, "needs.source-preflight.result == 'success'")
    require_fragment(consumer, "needs.compiler-linux.result == 'success'")
    for name in ('artifact-id', 'shell-artifact-id', 'producer-attempt'):
        require_fragment(consumer, 'needs.compiler-linux.outputs.' + name)
    for name in ('compiler_closure.py import', 'actions/import-compiler-shell',
                 'record_compiler_recipe.py', 'nix build --no-link .#runtime.package',
                 'nix build --no-link --print-out-paths .#zkir-v3'):
        require_fragment(consumer, name)
    require_fragment(consumer, 'test "$COMPACT_RUNTIME_PKG" = "$GROUPED_RUNTIME"')
    require_fragment(consumer, 'grep -Fq "compiler=$COMPACTC" "$wrapper"')
    require_fragment(consumer, 'bash test-contracts/test.sh')
    expected = {
        'runtime': ('npm run test', '358'),
        'root-host': ('bash test-contracts/test.sh', '540'),
        'keccak': ('check_upstream_ledger9_keccak.py', '23'),
        'sha256': ('check_upstream_ledger9_sha256.py', '6'),
    }
    for owner, (command, count) in expected.items():
        require_fragment(consumer, command)
        require_fragment(consumer, f"{owner}.log")
        require_fragment(consumer, f"{count} passed \\({count}\\)")
    require_fragment(consumer, '--allow-pragma-probe')
    require_fragment(consumer, 'test -f test-contracts/node_modules/vitest/vitest.mjs')
    require_fragment(consumer, 'ln -s test-contracts/node_modules node_modules')
    require_fragment(consumer, 'ln -s ../test-contracts/node_modules/vitest node_modules/vitest')
    require_fragment(consumer, 'test -f node_modules/vitest/vitest.mjs')
    for owner in ('check_upstream_ledger9_bytes_large_reconstruction_gap.py',
                  'check_upstream_ledger9_bytes_large_reconstruction_proof.py',
                  'audit_upstream_ledger9_bytes_large_reconstruction_proof.py'):
        require_fragment(consumer, owner)
    for fragment in ('--expected-source-sha "$GITHUB_SHA"',
                     '--compiler-recipe "$RUNNER_TEMP/grouped-compiler-recipe/receipt.json"',
                     '--semantic-gate "$RUNNER_TEMP/grouped-evidence/bytes-semantic"',
                     "proof['proof_summary']['public_verify_ops'] == 315",
                     "proof['proof_summary']['strict_application'] == 'default'",
                     "proof['proof_summary']['replay_refusal'] == 'IntentAlreadyExists'",
                     "proof['proof_summary']['standalone_verified_separately'] is True",
                     "audit['proof_execution'] is True and audit['funded_application'] is True",
                     "'scope': 'Keccak and SHA-256; SHA-512 is not claimed'"):
        require_fragment(consumer, fragment)
    require_fragment(consumer, 'test "$(uname -s)" = Linux')
    require_fragment(consumer, 'test "$(uname -m)" = x86_64')
    require_fragment(consumer, 'test -z "$(git status --porcelain)"')
    require_fragment(consumer, 'include-hidden-files: true')
    require_fragment(consumer, "'native_linux': True")
    require_fragment(complete, 'needs: [source-preflight, compiler-linux, grouped-originals]')
    require_fragment(complete, 'if: ${{ always() }}')
    for owner in ('PREFLIGHT_RESULT', 'PRODUCER_RESULT', 'GROUPED_RESULT'):
        require_fragment(complete, f'test "${owner}" = success')
    fetch = ('git fetch --no-tags --depth=1 '
             f'https://github.com/LFDT-Minokawa/compact.git {PIN}')
    if text.count(fetch) != 2 or text.count('trust_vector_source_key.sh') != 2:
        raise ValueError('pinned upstream fetch/trust owner missing')
    if consumer.index('compiler_closure.py import') > consumer.index('Run full original Runtime'):
        raise ValueError('compiler import must precede test owners')
    ordered = ('Run full original Runtime', 'Run intact root host',
               'Run 23 intact Keccak', 'Run ten-step Bytes',
               'Prove k16', 'Refuse missing owners')
    positions = [consumer.index(name) for name in ordered]
    if positions != sorted(positions):
        raise ValueError('owner order changed')


class GroupedLinuxWorkflowControls(unittest.TestCase):
    def test_exact_head_graph_and_all_owners(self):
        validate_workflow(WORKFLOW.read_text())

    def test_each_owner_omission_is_refused(self):
        text = WORKFLOW.read_text()
        for owner in ('358 passed', '540 passed', '23 passed', '6 passed',
                      'check_upstream_ledger9_bytes_large_reconstruction_gap.py',
                      'check_upstream_ledger9_bytes_large_reconstruction_proof.py',
                      'audit_upstream_ledger9_bytes_large_reconstruction_proof.py',
                      'test "$GROUPED_RESULT" = success'):
            with self.subTest(owner=owner):
                with self.assertRaises(ValueError):
                    validate_workflow(text.replace(owner, 'removed', 1))

    def test_vitest_setup_preserves_existing_trees_and_refuses_invalid_ones(self):
        text = WORKFLOW.read_text()
        script = section(
            text,
            '# Begin root Vitest path setup: retain every existing dependency tree.\n',
            '          # End root Vitest path setup.',
        )
        self.assertNotIn('rm ', script)
        self.assertNotIn('ln -sfn', script)
        for case in ('absent', 'existing-empty', 'existing-valid',
                     'existing-invalid', 'dangling-root-link'):
            with self.subTest(case=case), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                test_vitest = root / 'test-contracts/node_modules/vitest'
                test_vitest.mkdir(parents=True)
                (test_vitest / 'vitest.mjs').write_text('test fixture\n')
                node_modules = root / 'node_modules'
                if case.startswith('existing-'):
                    node_modules.mkdir()
                if case == 'existing-valid':
                    current = node_modules / 'vitest'
                    current.mkdir()
                    (current / 'vitest.mjs').write_text('retain existing\n')
                if case == 'existing-invalid':
                    (node_modules / 'vitest').mkdir()
                if case == 'dangling-root-link':
                    node_modules.symlink_to('missing')
                result = subprocess.run(['bash', '-euo', 'pipefail', '-c', script],
                                        cwd=root, capture_output=True, text=True)
                if case in ('existing-invalid', 'dangling-root-link'):
                    self.assertNotEqual(result.returncode, 0)
                    self.assertTrue((node_modules / 'vitest').is_dir()
                                    if case == 'existing-invalid' else node_modules.is_symlink())
                    continue
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertTrue((node_modules / 'vitest/vitest.mjs').is_file())
                if case == 'absent':
                    self.assertTrue(node_modules.is_symlink())
                elif case == 'existing-empty':
                    self.assertTrue((node_modules / 'vitest').is_symlink())
                else:
                    self.assertFalse((node_modules / 'vitest').is_symlink())
                    self.assertEqual((node_modules / 'vitest/vitest.mjs').read_text(),
                                     'retain existing\n')

    def test_root_host_wrapper_and_proof_material_are_owned(self):
        flake = (ROOT / 'flake.nix').read_text()
        host = (ROOT / 'test-contracts/test.sh').read_text()
        proof = (ROOT / 'tools/compact-rust-backend/'
                 'check_upstream_ledger9_bytes_large_reconstruction_proof.py').read_text()
        audit = (ROOT / 'tools/compact-rust-backend/'
                 'audit_upstream_ledger9_bytes_large_reconstruction_proof.py').read_text()
        for fragment in ('packages.compactc-test-contracts', 'devShells.test-contracts',
                         'COMPACT_RUNTIME_PKG =', 'sha256sum "$arg"',
                         'exec "$compiler" --feature-zkir-v3 "$@"'):
            self.assertIn(fragment, flake)
        self.assertIn('COMPACT_BINARY=compactc corepack yarn lint', host)
        self.assertIn('COMPACT_BINARY=compactc corepack yarn test', host)
        self.assertIn('MIN_START_FREE_GIB = 22', proof)
        self.assertIn('K16_PARAMETER_BYTES = 12583300', proof)
        self.assertIn("'replay_unchanged'", proof)
        self.assertIn('independent-verify.log', audit)
        self.assertIn('wrong-verifier.log', audit)
        self.assertIn('tampered-proof.log', audit)


if __name__ == '__main__':
    unittest.main()
