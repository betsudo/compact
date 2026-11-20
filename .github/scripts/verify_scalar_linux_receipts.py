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
"""Fail closed on the four original-scalar native Linux semantic receipts."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools/compact-rust-backend'))
from check_upstream_ledger9_field_casts import runtime_identity, runtime_recipe_identity  # noqa: E402

PINNED = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'
CASES = {
    'field_casts': (21, 8),
    'uint_casts_ranges': (20, 4),
    'bytes_casts_literals': (30, 8),
    'bytes_remaining_scalar': (35, 11),
}
GENERATED = {
    'field_casts': {'bytes4_boundary', 'to_boolean', 'to_bytes', 'to_enum',
                    'to_enum_out_of_range', 'to_uint', 'to_uint_max_field',
                    'to_uint_out_of_range'},
    'uint_casts_ranges': {'casts/to_bytes', 'casts/to_uint_different_width',
                          'ranges/addition', 'ranges/incompatible_with_plain_uint',
                          'ranges/invalid_lower_bound', 'ranges/inverted_bounds',
                          'ranges/multiplication', 'ranges/zero_upper_bound'},
    'bytes_casts_literals': {'casts/empty', 'casts/round_trip', 'casts/to_field',
                             'casts/to_tuple', 'casts/to_uint', 'literal/escape_sequences',
                             'literal/invalid_binary', 'literal/invalid_hex',
                             'literal/invalid_octal', 'literal/non_ascii', 'literal/string'},
    'bytes_remaining_scalar': {'default/semantics', 'equality/empty',
                               'equality/semantics', 'indexing/basic', 'pad/semantics',
                               'slice/basic', 'slice/empty', 'spread/basic',
                               'spread/empty', 'spread/multiple', 'spread/nested',
                               'spread/rejects_non_bytes'},
}
REFUSALS = {
    'field_casts': set(),
    'uint_casts_ranges': {'ranges/incompatible_with_plain_uint',
                          'ranges/invalid_lower_bound', 'ranges/inverted_bounds',
                          'ranges/zero_upper_bound'},
    'bytes_casts_literals': {'literal/invalid_binary', 'literal/invalid_hex',
                             'literal/invalid_octal'},
    'bytes_remaining_scalar': {'spread/rejects_non_bytes'},
}
ARTIFACTS = {
    'typescript_sha256': 'contract/index.js',
    'contract_info_sha256': 'compiler/contract-info.json',
    'rust_sha256': 'contract/lib.rs',
    'rust_ir_sha256': 'contract/compact-rust-ir.json',
}
RUNTIME_SUFFIX = {
    'field_casts': 'original or boundary assertions passed',
    'uint_casts_ranges': 'original assertion passed',
    'bytes_casts_literals': 'original assertion passed',
    'bytes_remaining_scalar': 'original assertion and state passed',
}
FIELD_LOCAL = {
    'adapter': 'test-contracts/primitives/field/casts/check.mjs',
    'boundary': 'test-contracts/primitives/field/casts/boundary.compact',
    'consumer': 'test-contracts/primitives/field/casts/consumer.rs',
    'unsupported_width': 'test-contracts/primitives/field/casts/unsupported-width.compact',
}
MANIFESTS = {
    'field_casts': 'test-contracts/primitives/field/casts/original-source-sha256.json',
    'uint_casts_ranges': 'test-contracts/primitives/uint/original-source-sha256.json',
    'bytes_casts_literals':
        'test-contracts/primitives/bytes/casts-literals-original-source-sha256.json',
    'bytes_remaining_scalar':
        'test-contracts/primitives/bytes/remaining-scalar-original-source-sha256.json',
}
LOCAL_SOURCES = {
    'field_casts': set(FIELD_LOCAL),
    'uint_casts_ranges': {
        'test-contracts/primitives/uint/check.mjs',
        'test-contracts/primitives/uint/consumer.rs',
        MANIFESTS['uint_casts_ranges'],
    },
    'bytes_casts_literals': {
        'test-contracts/primitives/bytes/check-casts-literals.mjs',
        'test-contracts/primitives/bytes/consumer-casts-literals.rs',
        MANIFESTS['bytes_casts_literals'],
    },
    'bytes_remaining_scalar': {
        'test-contracts/primitives/bytes/check-remaining-scalar.mjs',
        'test-contracts/primitives/bytes/consumer-remaining-scalar.rs',
        MANIFESTS['bytes_remaining_scalar'],
    },
}


def require(value: bool, message: str) -> None:
    if not value:
        raise ValueError(message)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_receipt(path: Path) -> dict:
    require(path.is_file() and not path.is_symlink() and path.stat().st_size <= 8_000_000,
            f'missing, symlinked or oversized receipt: {path}')
    value = json.loads(path.read_text())
    require(isinstance(value, dict), f'non-object receipt: {path}')
    return value


def gate_status(path: Path, *, source_sha: str, output: Path) -> None:
    require(path.is_file() and not path.is_symlink() and path.stat().st_size <= 8_000_000,
            f'missing, symlinked or oversized gate log: {path}')
    lines = path.read_text().splitlines()
    require(bool(lines), f'empty gate log: {path}')
    require(json.loads(lines[-1]) == {'status': 'passed', 'head': source_sha,
                                     'output': str(output)},
            f'gate reported wrong status, head or output: {path}')


def verify_originals(receipt: dict, case: str) -> int:
    originals = receipt.get('original_files')
    require(isinstance(originals, dict) and len(originals) == CASES[case][0],
            f'{case}: original inventory differs')
    manifest_path = ROOT / MANIFESTS[case]
    manifest = load_receipt(manifest_path)
    require(manifest.get('upstream_commit') == PINNED and
            originals == manifest.get('files'),
            f'{case}: original manifest differs')
    if case == 'field_casts':
        require(receipt.get('manifest_sha256') == sha256(manifest_path),
                'Field original manifest hash differs')
    for name, digest in originals.items():
        require(isinstance(name, str) and name.startswith('test-contracts/primitives/') and
                '..' not in Path(name).parts and
                isinstance(digest, str) and re.fullmatch('[0-9a-f]{64}', digest),
                f'{case}: invalid original path/hash')
        path = ROOT / name
        require(path.is_file() and not path.is_symlink() and sha256(path) == digest,
                f'{case}: original source changed: {name}')
        upstream = subprocess.check_output(['git', 'show', f'{PINNED}:{name}'], cwd=ROOT)
        require(hashlib.sha256(upstream).hexdigest() == digest,
                f'{case}: original differs from pinned upstream: {name}')
    return len(originals)


def verify_generated(case: str, receipt: dict, output: Path) -> None:
    generated = receipt.get('generated')
    require(isinstance(generated, dict) and set(generated) == GENERATED[case],
            f'{case}: generated case roster differs')
    for name, modes in generated.items():
        require(isinstance(modes, dict) and set(modes) == {'typescript', 'both'},
                f'{case}/{name}: generated modes differ')
        for mode, row in modes.items():
            require(isinstance(row, dict), f'{case}/{name}/{mode}: generated row missing')
            destination = output / 'generated' / name / mode
            if name in REFUSALS[case]:
                log = output / f'{name.replace("/", "-")}-{mode}-refusal.log'
                require(set(row) == {'refusal_sha256'} and log.is_file() and
                        not log.is_symlink() and sha256(log) == row['refusal_sha256'] and
                        not destination.exists(),
                        f'{case}/{name}/{mode}: original refusal artifact differs')
                continue
            expected = {'typescript_sha256', 'contract_info_sha256',
                        'runtime_result', 'runtime_log_sha256'}
            if mode == 'both':
                expected.update(('rust_sha256', 'rust_ir_sha256'))
            result = f'{name}: {RUNTIME_SUFFIX[case]}'
            require(set(row) == expected and row['runtime_result'] == result,
                    f'{case}/{name}/{mode}: generated artifact inventory differs')
            runtime_log = output / f'{name.replace("/", "-")}-{mode}-runtime.log'
            require(runtime_log.is_file() and not runtime_log.is_symlink() and
                    sha256(runtime_log) == row['runtime_log_sha256'] and
                    runtime_log.read_text() == result + '\n',
                    f'{case}/{name}/{mode}: Node runtime log differs')
            for key, relative in ARTIFACTS.items():
                if key in row:
                    path = destination / relative
                    require(path.is_file() and not path.is_symlink() and
                            sha256(path) == row[key],
                            f'{case}/{name}/{mode}: {relative} differs')
        if name not in REFUSALS[case]:
            for key in ('typescript_sha256', 'contract_info_sha256'):
                require(modes['typescript'][key] == modes['both'][key],
                        f'{case}/{name}: TS-only/paired drift')


def verify_local_sources(case: str, receipt: dict) -> None:
    local = receipt.get('local_sources')
    require(isinstance(local, dict) and set(local) == LOCAL_SOURCES[case],
            f'{case}: local source roster differs')
    for name, digest in local.items():
        relative = FIELD_LOCAL.get(name) if case == 'field_casts' else name
        require(relative is not None and isinstance(digest, str) and
                re.fullmatch('[0-9a-f]{64}', digest) and
                '..' not in Path(relative).parts and
                Path(relative).is_relative_to('test-contracts/primitives'),
                f'{case}: invalid local source entry')
        path = ROOT / relative
        require(path.is_file() and not path.is_symlink() and sha256(path) == digest,
                f'{case}: local source changed: {relative}')


def validate(case: str, path: Path, *, gate_log: Path, source_sha: str, tree: str,
             compiler: Path, recipe_path: Path, runtime: Path,
             runtime_snapshot: dict, runtime_recipe: dict, cargo_target: Path) -> dict:
    require(case in CASES and path.parent.name == case, 'unknown or misplaced scalar gate')
    gate_status(gate_log, source_sha=source_sha, output=path.parent)
    receipt = load_receipt(path)
    require(receipt.get('head') == source_sha and receipt.get('tree') == tree and
            receipt.get('source_clean') is True and receipt.get('platform') == 'Linux x86_64',
            f'{case}: native Linux source identity differs')
    require(receipt.get('upstream_commit') == PINNED,
            f'{case}: pinned upstream differs')
    require(receipt.get('cargo_target') == str(cargo_target),
            f'{case}: shared Cargo target differs')
    require('native Linux generated semantics qualified' in receipt.get('limitations', []) and
            not receipt.get('proof_execution') and not receipt.get('funded_application') and
            not receipt.get('constructor_execution_proved'),
            f'{case}: semantic gate misstates native proof scope')
    if case == 'bytes_remaining_scalar':
        require(any('nine stateful proof-required exports need fresh proof' in item
                    for item in receipt['limitations']),
                'nine Bytes proof-required exports not labelled open')
    compiler_recipe = load_receipt(recipe_path)
    require(compiler_recipe.get('status') == 'passed' and
            compiler_recipe.get('head') == source_sha and
            compiler_recipe.get('tree') == tree,
            f'{case}: compiler recipe differs')
    scheme = compiler.with_name('compactc-scheme')
    require(receipt.get('compiler_sha256') == sha256(compiler) and
            receipt.get('scheme_sha256') == sha256(scheme) and
            receipt.get('compiler_recipe') == {
                'path': str(recipe_path), 'sha256': sha256(recipe_path),
                'head': source_sha, 'tree': tree,
                'store': str(compiler.parent.parent),
            }, f'{case}: compiler material differs')
    recorded_runtime = receipt.get('runtime_recipe', {})
    require(receipt.get('runtime') == runtime_snapshot and
            isinstance(recorded_runtime, dict) and
            recorded_runtime.get('store') == runtime_recipe['store'] and
            recorded_runtime.get('path') == runtime_recipe['path'] and
            recorded_runtime.get('command', [])[1:] == runtime_recipe['command'][1:],
            f'{case}: exact-head Runtime 0.18 differs')
    require(receipt.get('checker_sha256') == sha256(
                ROOT / f'tools/compact-rust-backend/check_upstream_ledger9_{case}.py'),
            f'{case}: gate script differs')
    original_count = verify_originals(receipt, case)
    verify_local_sources(case, receipt)
    verify_generated(case, receipt, path.parent)
    consumer = path.parent / 'rust-consumer'
    test_log = path.parent / 'cargo-test.log'
    require(consumer.joinpath('Cargo.lock').is_file() and
            not consumer.joinpath('Cargo.lock').is_symlink() and
            sha256(consumer / 'Cargo.lock') == receipt.get('rust_consumer_lock_sha256') and
            test_log.is_file() and not test_log.is_symlink() and
            sha256(test_log) == receipt.get('rust_test_log_sha256') and
            f'test result: ok. {CASES[case][1]} passed; 0 failed;' in test_log.read_text(),
            f'{case}: generated Rust consumer evidence differs')
    if case == 'field_casts':
        require(sha256(path.parent / 'unsupported-typescript/contract/index.js') ==
                receipt.get('unsupported_typescript_sha256') and
                sha256(path.parent / 'unsupported-paired-refusal.log') ==
                receipt.get('unsupported_refusal_log_sha256'),
                'Field-to-Bytes<16> refusal evidence differs')
    return {'case': case, 'head': source_sha, 'tree': tree, 'status': 'passed',
            'original_files': original_count, 'receipt_sha256': sha256(path)}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    selection = parser.add_mutually_exclusive_group(required=True)
    selection.add_argument('--case', choices=CASES)
    selection.add_argument('--all', type=Path)
    parser.add_argument('--receipt', type=Path)
    parser.add_argument('--gate-log', type=Path)
    parser.add_argument('--source-sha', required=True)
    parser.add_argument('--compiler', type=Path, required=True)
    parser.add_argument('--compiler-recipe', type=Path, required=True)
    parser.add_argument('--runtime', type=Path, required=True)
    parser.add_argument('--summary', type=Path)
    args = parser.parse_args()
    require(re.fullmatch('[0-9a-f]{40}', args.source_sha), 'invalid source SHA')
    head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    tree = subprocess.check_output(['git', 'rev-parse', 'HEAD^{tree}'], cwd=ROOT, text=True).strip()
    require(head == args.source_sha and not subprocess.check_output(
        ['git', 'status', '--porcelain'], cwd=ROOT),
        'source changed during scalar qualification')
    compiler = args.compiler.resolve(strict=True)
    recipe_path = args.compiler_recipe.resolve(strict=True)
    runtime = args.runtime.resolve(strict=True)
    selected_target = os.environ.get('SCALAR_CARGO_TARGET')
    require(selected_target and Path(selected_target).is_absolute() and
            not Path(selected_target).is_symlink(),
            'shared scalar Cargo target not selected')
    target = Path(selected_target).resolve()
    nix = shutil.which('nix')
    require(nix is not None, 'Nix required for exact-head runtime receipt')
    runtime_snapshot = runtime_identity(runtime)
    runtime_recipe = runtime_recipe_identity(runtime, Path(nix).resolve(strict=True))
    options = dict(source_sha=head, tree=tree, compiler=compiler, recipe_path=recipe_path,
                   runtime=runtime, runtime_snapshot=runtime_snapshot,
                   runtime_recipe=runtime_recipe, cargo_target=target)
    if args.case:
        require(args.receipt is not None and args.gate_log is not None and
                args.summary is None, 'single gate needs receipt and status log')
        print(json.dumps(validate(args.case, args.receipt, gate_log=args.gate_log,
                                  **options), sort_keys=True))
    else:
        require(args.receipt is None and args.gate_log is None and
                args.summary is not None, 'group needs summary path')
        directory = args.all.resolve(strict=True)
        require(not args.summary.exists() and args.summary.parent == directory,
                'summary must be new in evidence directory')
        rows = [validate(case, directory / case / 'receipt.json',
                         gate_log=directory / f'{case}.stdout.log', **options)
                for case in CASES]
        summary = {'status': 'passed', 'head': head, 'tree': tree,
                   'platform': 'Linux x86_64', 'upstream_commit': PINNED,
                   'original_files': sum(row['original_files'] for row in rows),
                   'cases': rows, 'compiler_recipe_sha256': sha256(recipe_path),
                   'runtime_package_sha256': sha256(runtime / 'package.json'),
                   'proof_execution': False, 'funded_application': False,
                   'constructor_execution_proved': False,
                   'proof_required_exports_open': 9}
        require(summary['original_files'] == 106 and len(rows) == 4,
                'scalar original inventory incomplete')
        args.summary.write_text(json.dumps(summary, indent=2, sort_keys=True) + '\n')
        print(json.dumps({'status': 'passed', 'head': head, 'cases': len(rows)}))


if __name__ == '__main__':
    main()
