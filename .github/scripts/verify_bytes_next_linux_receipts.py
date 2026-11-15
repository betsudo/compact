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
"""Fail closed on the three original-Bytes native Linux semantic receipts."""

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
CASES = ('manual_witness', 'fold', 'loops')
ORIGINAL_COUNT = {'manual_witness': 6, 'fold': 9, 'loops': 9}
GENERATED = {
    'manual_witness': {'manual_update', 'witness_return'},
    'fold': {'boolean_reduction', 'conditional_accumulation', 'summation'},
    'loops': {'empty_iteration', 'spread_iteration', 'iteration', 'zero_operand_order'},
}
MANIFESTS = {
    'manual_witness': 'test-contracts/primitives/bytes/manual-witness-original-source-sha256.json',
    'fold': 'test-contracts/primitives/bytes/fold/original-source-sha256.json',
    'loops': 'test-contracts/primitives/bytes/loops/original-source-sha256.json',
}
LOCAL = {
    'manual_witness': {
        'test-contracts/primitives/bytes/check-manual-witness.mjs',
        'test-contracts/primitives/bytes/consumer-manual-witness.rs',
        MANIFESTS['manual_witness'],
    },
    'fold': {
        'test-contracts/primitives/bytes/fold/check.mjs',
        'test-contracts/primitives/bytes/fold/consumer.rs',
        MANIFESTS['fold'],
        'tools/compact-rust-backend/tests/bytes_fold/check-control.mjs',
        'tools/compact-rust-backend/tests/bytes_fold/named.compact',
        'tools/compact-rust-backend/tests/bytes_fold/pure.compact',
    },
    'loops': {
        'compiler/rust-ir-passes.ss',
        'test-contracts/primitives/bytes/loops/check-order.mjs',
        'test-contracts/primitives/bytes/loops/check.mjs',
        'test-contracts/primitives/bytes/loops/consumer.rs',
        MANIFESTS['loops'],
        'tools/compact-rust-backend/tests/bytes_loops/dynamic_index_refusal.compact',
        'tools/compact-rust-backend/tests/bytes_loops/zero_operand_order.compact',
    },
}
RUST_TESTS = {
    'manual_witness': (
        'original_manual_update_ordinary_and_recorded',
        'original_witness_return_ordinary_and_recorded',
        'wrong_witness_bytes_are_refused_in_both_modes',
    ),
    'fold': (
        'boolean_reduction_original_result_and_replay',
        'conditional_accumulation_original_result_and_replay',
        'summation_original_result_and_replay',
        'pure_bytes_fold_source_normalization',
        'named_bytes_fold_source_normalization',
    ),
    'loops': (
        'empty_iteration_original_result_and_replay',
        'spread_iteration_original_result_and_replay',
        'iteration_original_result_and_replay',
        'zero_length_fold_evaluates_operands_in_order_and_skips_reducer',
    ),
}
RESULT = {
    'manual_witness': 'two original Bytes semantics passed generated TS and Rust ordinary/recorded APIs',
    'fold': 'three original Bytes Fold cases passed generated TS and Rust ordinary/recorded replay',
    'loops': 'three pinned Bytes loops and zero-length operand-order control passed generated TS/Rust ordinary/recorded replay',
}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def file_under(root: Path, relative: str) -> Path:
    require(isinstance(relative, str) and relative and
            not Path(relative).is_absolute() and '..' not in Path(relative).parts,
            f'invalid relative evidence path: {relative}')
    path = root / relative
    require(path.is_file() and not path.is_symlink() and
            path.resolve(strict=True).is_relative_to(root.resolve(strict=True)),
            f'missing or escaped evidence file: {path}')
    return path


def object_file(path: Path) -> dict:
    require(path.is_file() and not path.is_symlink() and path.stat().st_size <= 8_000_000,
            f'missing, symlinked or oversized object: {path}')
    result = json.loads(path.read_text())
    require(isinstance(result, dict), f'non-object JSON: {path}')
    return result


def hash_tree(root: Path) -> dict[str, str]:
    require(root.is_dir() and not root.is_symlink(), f'missing generated tree: {root}')
    result = {}
    for path in sorted(root.rglob('*')):
        require(not path.is_symlink(), f'generated symlink: {path}')
        if path.is_file():
            result[path.relative_to(root).as_posix()] = sha256(path)
    require(result, f'empty generated tree: {root}')
    return result


def verify_gate_log(path: Path, head: str, output: Path) -> None:
    require(path.is_file() and not path.is_symlink() and path.stat().st_size <= 8_000_000,
            f'missing gate log: {path}')
    lines = path.read_text().splitlines()
    require(bool(lines) and json.loads(lines[-1]) ==
            {'status': 'passed', 'head': head, 'output': str(output)},
            f'gate did not report exact-head completion: {path}')


def verify_originals(case: str, receipt: dict) -> int:
    manifest = object_file(ROOT / MANIFESTS[case])
    originals = receipt.get('original_files')
    require(manifest.get('upstream_commit') == PINNED and
            originals == manifest.get('files') and isinstance(originals, dict) and
            len(originals) == ORIGINAL_COUNT[case], f'{case}: original roster differs')
    for relative, digest in originals.items():
        require(re.fullmatch('[0-9a-f]{64}', digest) is not None and
                relative.startswith('test-contracts/primitives/bytes/'),
                f'{case}: invalid original hash or path')
        source = file_under(ROOT, relative)
        upstream = subprocess.check_output(['git', 'show', f'{PINNED}:{relative}'], cwd=ROOT)
        require(source.read_bytes() == upstream and sha256(source) == digest,
                f'{case}: original differs from pinned upstream: {relative}')
    return len(originals)


def verify_local(case: str, receipt: dict) -> None:
    local = receipt.get('local_sources')
    require(isinstance(local, dict) and set(local) == LOCAL[case],
            f'{case}: local source roster differs')
    for relative, digest in local.items():
        require(re.fullmatch('[0-9a-f]{64}', digest) is not None and
                sha256(file_under(ROOT, relative)) == digest,
                f'{case}: local source changed: {relative}')


def runtime_text(case: str, name: str) -> str:
    if case == 'manual_witness':
        return f'{name}: original Bytes semantics passed\n'
    if case == 'fold':
        return f'{name}: original Bytes fold result and ledger passed\n'
    if name == 'zero_operand_order':
        return 'zero_operand_order: initial and source evaluated in order; reducer unreachable\n'
    return f'{name}: original Bytes loop result and ledger passed\n'


def verify_generated(case: str, receipt: dict, output: Path) -> None:
    generated = receipt.get('generated')
    require(isinstance(generated, dict) and set(generated) == GENERATED[case],
            f'{case}: generated case roster differs')
    generated_root = output / 'generated'
    require(receipt.get('generated_files_sha256') == hash_tree(generated_root),
            f'{case}: complete generated artifact inventory differs')
    for name, modes in generated.items():
        require(isinstance(modes, dict) and set(modes) == {'typescript', 'both'},
                f'{case}/{name}: target modes differ')
        for mode, row in modes.items():
            require(isinstance(row, dict), f'{case}/{name}/{mode}: missing artifacts')
            location = generated_root / name / mode
            artifacts = {
                'typescript_sha256': 'contract/index.js',
                'manifest_sha256': 'compiler/contract-manifest.json',
                'contract_info_sha256': 'compiler/contract-info.json',
            }
            if mode == 'both':
                artifacts.update(rust_sha256='contract/lib.rs',
                                 rust_ir_sha256='contract/compact-rust-ir.json')
            if 'zkir_sha256' in row:
                stem = ('bytes_fold_' if case == 'fold' else 'bytes_loop_') + name
                artifacts['zkir_sha256'] = f'zkir/{stem}.zkir'
            expected_keys = set(artifacts) | {'runtime_log_sha256'}
            require(set(row) == expected_keys, f'{case}/{name}/{mode}: artifact fields differ')
            for key, relative in artifacts.items():
                require(sha256(file_under(location, relative)) == row[key],
                        f'{case}/{name}/{mode}: {relative} differs')
            log = file_under(output, f'{name}-{mode}-runtime.log')
            require(sha256(log) == row['runtime_log_sha256'] and
                    log.read_text() == runtime_text(case, name),
                    f'{case}/{name}/{mode}: runtime result or log differs')
        require(all(modes['typescript'][key] == modes['both'][key]
                    for key in ('typescript_sha256', 'contract_info_sha256',
                                'runtime_log_sha256')), f'{case}/{name}: paired TS drift')
        if 'zkir_sha256' in modes['typescript']:
            require(modes['typescript']['zkir_sha256'] == modes['both']['zkir_sha256'],
                    f'{case}/{name}: paired ZKIR drift')
    if case == 'manual_witness':
        require(file_under(generated_root, 'manual_update/both/zkir/bytes_manual_update_semantics.zkir').is_file() and
                not (generated_root / 'witness_return/both/zkir').exists(),
                'manual/witness proof capability differs')
    if case == 'fold':
        controls = receipt.get('pure_controls')
        require(isinstance(controls, dict) and set(controls) == {'pure', 'named'},
                'Bytes Fold pure control roster differs')
        fields = {'typescript': 'contract/index.js', 'rust': 'contract/lib.rs',
                  'rust_ir': 'contract/compact-rust-ir.json',
                  'contract_info': 'compiler/contract-info.json'}
        for name, hashes in controls.items():
            require(isinstance(hashes, dict) and set(hashes) == set(fields) | {'runtime_log'},
                    f'{name}: pure control inventory differs')
            for key, relative in fields.items():
                require(sha256(file_under(generated_root / f'{name}-control', relative)) ==
                        hashes[key], f'{name}: pure control {relative} differs')
            log = file_under(output, f'{name}-control-runtime.log')
            require(sha256(log) == hashes['runtime_log'] and
                    log.read_text() == f'{name}: pure Bytes fold result passed\n',
                    f'{name}: pure TypeScript result differs')
    if case == 'loops':
        log = file_under(output, 'dynamic-refusal.log')
        require(sha256(log) == receipt.get('dynamic_index_refusal_log_sha256') and
                'Bytes index did not reduce to a constant nonnegative value at compile time' in
                log.read_text(), 'mainstream dynamic Bytes index refusal differs')


def verify_rust(case: str, receipt: dict, output: Path) -> None:
    lock = file_under(output, 'rust-consumer/Cargo.lock')
    log = file_under(output, 'cargo-test.log')
    require(sha256(lock) == receipt.get('rust_consumer_lock_sha256') and
            sha256(log) == receipt.get('rust_test_log_sha256'),
            f'{case}: Rust lock or log changed')
    text = log.read_text()
    require(f'test result: ok. {len(RUST_TESTS[case])} passed; 0 failed;' in text and
            all(f'test {name} ... ok' in text for name in RUST_TESTS[case]),
            f'{case}: original/recorded Rust tests differ')


def validate(case: str, path: Path, gate_log: Path, *,
             head: str, tree: str, compiler: Path, recipe_path: Path, runtime: Path,
             runtime_snapshot: dict, runtime_recipe: dict, target: Path) -> dict:
    require(case in CASES and path.parent.name == case, 'unknown or misplaced Bytes gate')
    output = path.parent
    verify_gate_log(gate_log, head, output)
    receipt = object_file(path)
    require(receipt.get('status') == 'passed' and receipt.get('head') == head and
            receipt.get('tree') == tree and receipt.get('source_clean') is True and
            receipt.get('platform') == 'Linux x86_64' and
            receipt.get('upstream_commit') == PINNED and
            receipt.get('result') == RESULT[case], f'{case}: native source or result differs')
    require(receipt.get('cargo_target') == str(target) and
            receipt.get('proof_execution') is False and
            receipt.get('funded_application') is False and
            receipt.get('constructor_execution_proved') is False and
            'native Linux generated semantics qualified' in receipt.get('limitations', []),
            f'{case}: semantic proof scope or shared Cargo target differs')
    recipe = object_file(recipe_path)
    require(recipe.get('status') == 'passed' and recipe.get('head') == head and
            recipe.get('tree') == tree, f'{case}: exact-head compiler recipe differs')
    scheme = compiler.with_name('compactc-scheme')
    require(receipt.get('compiler_sha256') == sha256(compiler) and
            receipt.get('scheme_sha256') == sha256(scheme) and
            receipt.get('compiler_recipe') ==
            {'path': str(recipe_path), 'sha256': sha256(recipe_path),
             'head': head, 'tree': tree, 'store': str(compiler.parent.parent)},
            f'{case}: compiler material differs')
    recorded = receipt.get('runtime_recipe')
    require(receipt.get('runtime') == runtime_snapshot and
            isinstance(recorded, dict) and recorded.get('store') == runtime_recipe['store'] and
            recorded.get('path') == runtime_recipe['path'] and
            recorded.get('command', [])[1:] == runtime_recipe['command'][1:],
            f'{case}: exact Runtime 0.18 differs')
    require(receipt.get('checker_sha256') == sha256(
                ROOT / f'tools/compact-rust-backend/check_upstream_ledger9_bytes_{case}.py'),
            f'{case}: gate script changed')
    count = verify_originals(case, receipt)
    verify_local(case, receipt)
    verify_generated(case, receipt, output)
    verify_rust(case, receipt, output)
    return {'case': case, 'head': head, 'tree': tree, 'status': 'passed',
            'original_files': count, 'receipt_sha256': sha256(path)}


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
    require(re.fullmatch('[0-9a-f]{40}', args.source_sha) is not None,
            'invalid selected source SHA')
    head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    tree = subprocess.check_output(['git', 'rev-parse', 'HEAD^{tree}'], cwd=ROOT, text=True).strip()
    require(head == args.source_sha and not subprocess.check_output(
        ['git', 'status', '--porcelain'], cwd=ROOT), 'source changed during Bytes qualification')
    compiler = args.compiler.resolve(strict=True)
    recipe_path = args.compiler_recipe.resolve(strict=True)
    runtime = args.runtime.resolve(strict=True)
    selected = os.environ.get('BYTES_NEXT_CARGO_TARGET')
    require(selected and Path(selected).is_absolute() and
            not Path(selected).is_symlink() and Path(selected).is_dir(),
            'existing shared Bytes Cargo target not selected')
    target = Path(selected).resolve(strict=True)
    nix = shutil.which('nix')
    require(nix is not None, 'Nix required for exact-head runtime receipt')
    runtime_snapshot = runtime_identity(runtime)
    runtime_recipe = runtime_recipe_identity(runtime, Path(nix).resolve(strict=True))
    options = dict(head=head, tree=tree, compiler=compiler, recipe_path=recipe_path,
                   runtime=runtime, runtime_snapshot=runtime_snapshot,
                   runtime_recipe=runtime_recipe, target=target)
    if args.case:
        require(args.receipt is not None and args.gate_log is not None and
                args.summary is None, 'single Bytes gate needs receipt and status log')
        print(json.dumps(validate(args.case, args.receipt, args.gate_log, **options),
                         sort_keys=True))
    else:
        require(args.receipt is None and args.gate_log is None and
                args.summary is not None, 'group needs a new summary path')
        directory = args.all.resolve(strict=True)
        require(not args.summary.exists() and args.summary.parent == directory,
                'summary must be new in evidence directory')
        rows = [validate(case, directory / case / 'receipt.json',
                         directory / f'{case}.stdout.log', **options)
                for case in CASES]
        summary = {'status': 'passed', 'head': head, 'tree': tree,
                   'platform': 'Linux x86_64', 'upstream_commit': PINNED,
                   'original_files': sum(row['original_files'] for row in rows),
                   'cases': rows, 'compiler_recipe_sha256': sha256(recipe_path),
                   'runtime_package_sha256': sha256(runtime / 'package.json'),
                   'proof_execution': False, 'funded_application': False,
                   'constructor_execution_proved': False,
                   'proof_required_exports_not_qualified_here': 7}
        require(summary['original_files'] == 24 and len(rows) == 3,
                'Bytes original inventory incomplete')
        args.summary.write_text(json.dumps(summary, indent=2, sort_keys=True) + '\n')
        print(json.dumps({'status': 'passed', 'head': head, 'cases': len(rows)}))


if __name__ == '__main__':
    main()
