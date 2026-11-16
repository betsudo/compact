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
"""Fail-closed retained evidence check for the focused secondary Linux gate."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools/compact-rust-backend'))
import proof_runner_input_binding as runner_inputs  # noqa: E402
from check_upstream_ledger9_field_casts import runtime_identity  # noqa: E402
from check_upstream_keyword_vitest_host import original_host_counts_match  # noqa: E402

PINNED = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'
FAMILIES = {'include': (7, 3, 26), 'prefix': (3, 2, 12),
            'assert': (6, 5, 17), 'module': (8, 4, 20),
            'export': (5, 1, 11), 'import_type': (39, 20, 99)}
HOST_FAMILIES = ('reserved', 'const', 'import', 'type', 'for', 'return',
                 'if', 'include', 'prefix', 'assert', 'module', 'export')


def require(ok, reason):
    if not ok:
        raise ValueError(reason)


def sha(path):
    path = Path(path)
    require(path.is_file() and not path.is_symlink(), f'missing or linked file: {path}')
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(path):
    path = Path(path)
    require(path.stat().st_size < 64_000_000, f'oversized JSON: {path}')
    value = json.loads(path.read_text())
    require(isinstance(value, dict), f'non-object JSON: {path}')
    return value


def inventory(root):
    root = Path(root)
    require(root.is_dir() and not root.is_symlink(), f'missing artifact tree: {root}')
    found = {}
    for path in sorted(root.rglob('*')):
        require(not path.is_symlink(), f'linked artifact: {path}')
        if path.is_file():
            found[path.relative_to(root).as_posix()] = sha(path)
    return found


def files(root, mapping):
    require(isinstance(mapping, dict) and mapping, f'empty inventory: {root}')
    require(inventory(root) == mapping, f'artifact inventory differs: {root}')


def original_map(mapping, count, *, local=True):
    require(isinstance(mapping, dict) and len(mapping) == count,
            'pinned original roster differs')
    for name, digest in mapping.items():
        require(name.startswith('test-contracts/') and
                re.fullmatch('[0-9a-f]{64}', digest) and
                (not local or sha(ROOT / name) == digest) and
                hashlib.sha256(subprocess.check_output(
                    ['git', 'show', f'{PINNED}:{name}'], cwd=ROOT)).hexdigest() == digest,
                f'pinned original differs: {name}')


def identity(receipt, head, tree):
    require(receipt.get('status') == 'passed' and
            receipt.get('platform') == 'Linux x86_64' and
            receipt.get('head') == head and receipt.get('tree') == tree and
            receipt.get('upstream_commit', PINNED) == PINNED,
            'Linux receipt source or status differs')


def log(root, row, key, filename):
    require(sha(root / filename) == row.get(key), f'log hash differs: {root / filename}')


def host_vitest_log(path, expected_sha):
    require(sha(path) == expected_sha and
            original_host_counts_match(Path(path).read_text()),
            'original host execution differs')


def family(name, root, head, tree, compiler, recipe, runtime):
    receipt = load(root / 'receipt.json')
    identity(receipt, head, tree)
    cases, accepted, count = FAMILIES[name]
    require(receipt.get('case_count') == cases and
            receipt.get('accepted_compile_count' if name == 'assert'
                        else 'positive_count') == accepted and
            receipt.get('compile_refusal_count' if name == 'assert'
                        else 'refusal_count') == cases - accepted and
            receipt.get('source_clean') is True and
            receipt.get('compiler_sha256') == sha(compiler) and
            receipt.get('compiler_recipe', {}).get('sha256') == sha(recipe) and
            receipt['compiler_recipe'].get('head') == head and
            receipt['compiler_recipe'].get('tree') == tree and
            receipt.get('runtime') == runtime_identity(runtime),
            f'{name} source, compiler or count differs')
    checker = ROOT / f'tools/compact-rust-backend/check_upstream_ledger9_{name}_keywords.py'
    require(receipt.get('checker_sha256') == sha(checker) and
            (name == 'import_type' or receipt.get('helper_sha256') ==
             sha(ROOT / 'tools/compact-rust-backend/check_upstream_ledger9_import_type_keywords.py')),
            f'{name} checker binding differs')
    manifest = (ROOT / 'test-contracts/keywords/import-type-original-source-sha256.json'
                if name == 'import_type' else
                ROOT / f'test-contracts/keywords/{name}/original-source-sha256.json')
    manifest_data = load(manifest)
    originals = (manifest_data['ported_files'] if name == 'import_type'
                 else manifest_data['files'])
    require(manifest_data['upstream_commit'] == PINNED and
            receipt.get('manifest_sha256') == sha(manifest) and
            receipt.get('original_files') == originals,
            f'{name} manifest or original receipt differs')
    original_map(originals, count)
    results = receipt.get('results')
    require(isinstance(results, dict) and len(results) == cases,
            f'{name} case roster differs')
    accepted_seen = 0
    seen_compile = set()
    for case, modes in results.items():
        require(set(modes) == {'typescript', 'paired'} and
                re.fullmatch(r'[a-z0-9_/]+', case) and '..' not in Path(case).parts,
                f'{name}/{case} mode roster differs')
        outcomes = set()
        for mode, row in modes.items():
            base = root / case / mode
            original = row.get('original_compile_test')
            require(original in originals and
            (original.startswith('test-contracts/keywords/import/') or
             original.startswith('test-contracts/keywords/type/') if name == 'import_type'
             else original.startswith(f'test-contracts/keywords/{name}/')),
                    f'{name}/{case} compile test differs')
            relative_case = str(Path(original).parent.relative_to(
                'test-contracts/keywords'))
            require(relative_case == (case if name == 'import_type'
                                      else f'{name}/{case}'),
                    f'{name}/{case} original case mapping differs')
            seen_compile.add(original)
            positive = original.endswith('/compile.pass.test.ts')
            outcomes.add(positive)
            product = base / 'product'
            if positive:
                require(row.get('exit_code') == 0 and product.is_dir(),
                        f'{name}/{case}/{mode} positive compilation differs')
                runtime_test = row.get('original_runtime_test')
                runtime_copy = ('runtime.test.ts' if name == 'assert'
                                else 'runtime.pass.test.ts')
                require(runtime_test in originals and
                        sha(base / runtime_copy) == originals[runtime_test] and
                        sha(base / 'runtime.log') == row.get('runtime_log_sha256'),
                        f'{name}/{case}/{mode} runtime evidence differs')
                if name == 'assert':
                    expected = ('failure' if runtime_test.endswith('/runtime.fail.test.ts')
                                else 'success')
                    require(row.get('original_outcome') == expected,
                            f'{name}/{case}/{mode} runtime outcome differs')
                if name == 'import_type':
                    require(sha(product / 'contract/index.js') == row.get('typescript_sha256') and
                            sha(product / 'compiler/contract-info.json') ==
                            row.get('contract_info_sha256'),
                            f'{name}/{case}/{mode} generated TS differs')
                    for relative, digest in row.get('zkir', {}).items():
                        require(sha(product / relative) == digest,
                                f'{name}/{case}/{mode} generated ZKIR differs')
                    if mode == 'paired':
                        require(sha(product / 'contract/lib.rs') == row.get('rust_sha256') and
                                sha(product / 'contract/compact-rust-ir.json') ==
                                row.get('rust_ir_sha256') and
                                sha(product / 'contract/tests/original_keywords.rs') ==
                                row.get('rust_execution_test_sha256') and
                                sha(base / 'cargo-test.log') ==
                                row.get('rust_execution_log_sha256'),
                                f'{name}/{case} Rust result consumer differs')
                else:
                    files(product, row.get('generated_files'))
                    if mode == 'paired':
                        require('rust_test_log_sha256' in row and
                                sha(base / 'cargo-test.log') ==
                                row['rust_test_log_sha256'],
                                f'{name}/{case} Rust consumer differs')
            else:
                require(row.get('exit_code') != 0 and
                        row.get('output_absent') is True and not product.exists() and
                        sha(base / 'diagnostic.log') == row.get('diagnostic_sha256'),
                        f'{name}/{case}/{mode} refusal differs')
        require(len(outcomes) == 1, f'{name}/{case} paired outcome differs')
        accepted_seen += True in outcomes
    require(accepted_seen == accepted, f'{name} accepted case count differs')
    require(seen_compile == {path for path in originals
                             if Path(path).name.startswith('compile.')},
            f'{name} original compile roster differs')
    if name == 'import_type':
        require(receipt.get('rust_executed_count') == 20 and
                receipt.get('proof_execution') is False,
                'import/type Rust result roster or proof scope differs')
    return sha(root / 'receipt.json')


def bytes_controls(root, head, tree, compiler, recipe, runtime):
    indexed = root / 'bytes_indexed'
    receipt = load(indexed / 'receipt.json')
    identity(receipt, head, tree)
    require(receipt.get('compiler_sha256') == sha(compiler) and
            receipt.get('compiler_recipe', {}).get('sha256') == sha(recipe) and
            receipt.get('runtime') == runtime_identity(runtime) and
            receipt.get('checker_sha256') == sha(ROOT /
                'tools/compact-rust-backend/check_upstream_ledger9_bytes_indexed_access.py') and
            receipt.get('proof_execution') is False and
            receipt.get('funded_application') is False,
            'Bytes indexed compiler or scope differs')
    original_map(receipt['original_files'], 3)
    files(indexed / 'generated', receipt['generated_files_sha256'])
    require(sha(indexed / 'cargo-test.log') == receipt.get('rust_test_log_sha256') and
            sha(indexed / 'dynamic-refusal.log') ==
            receipt['dynamic_refusal']['log_sha256'],
            'Bytes Rust result or dynamic refusal differs')
    static = root / 'bytes_static'
    static_receipt = load(static / 'receipt.json')
    require(static_receipt.get('status') == 'passed' and
            static_receipt.get('compiler_sha256') == sha(compiler) and
            static_receipt.get('pinned_originals') == receipt['original_files'] and
            set(static_receipt.get('cases', {})) == {
                'original', 'shadowed_formal', 'effectful_operand',
                'nonliteral_binding', 'out_of_bounds', 'nine_item_cap',
                'dynamic_frontend'},
            'Bytes static controls differ')
    for name, case in static_receipt['cases'].items():
        base = static / name
        require(sha(base / f'{name}.compact') == case.get('source_sha256'),
                f'Bytes static source differs: {name}')
        for mode in ('typescript', 'paired'):
            row = case[mode]
            require(sha(base / f'{mode}.log') == row['log_sha256'] and
                    (base / mode).exists() == row['output_published'],
                    f'Bytes static outcome differs: {name}/{mode}')
    require('Rust collection index occurrence controls passed' in
            (root / 'bytes-occurrences.stdout.log').read_text(),
            'Bytes compiler occurrence controls absent')
    return [sha(indexed / 'receipt.json'), sha(static / 'receipt.json')]


def host(root, head, tree, compiler, recipe):
    staged = root / 'host-stage'
    staging = load(staged / 'staging-receipt.json')
    report = load(root / 'host/receipt.json')
    identity(report, head, tree)
    require(staging.get('status') == 'staged' and
            staging.get('head') == head and staging.get('tree') == tree and
            tuple(staging.get('families', ())) == HOST_FAMILIES and
            tuple(report.get('families', ())) == HOST_FAMILIES and
            staging.get('original_count') == 418 and
            staging.get('case_count') == report['counts']['cases'] == 167 and
            staging.get('host_test_count') == report['counts']['host_tests'] == 221 and
            report.get('staging_receipt_sha256') == sha(staged / 'staging-receipt.json') and
            report.get('compiler_sha256') == sha(compiler) and
            report.get('compiler_recipe_sha256') == sha(recipe) and
            report.get('original_files_sha256') == staging.get('original_files') and
            report.get('adapter_sha256') == staging.get('adapter_sha256') ==
            sha(staged / 'runtime/index.js'),
            'twelve-family host roster, source or compiler differs')
    original_map(staging['original_files'], 418, local=False)
    for name, digest in staging['original_files'].items():
        require(sha(staged / name) == digest, f'host staged original differs: {name}')
    require(sha(root / 'host/yarn-install.log') == report['yarn_install_log_sha256'],
            'original host execution differs')
    host_vitest_log(root / 'host/original-vitest.log', report['vitest_log_sha256'])
    return sha(root / 'host/receipt.json')


def proof(root, head, tree, zkir):
    output = root / 'import_proof'
    receipt = load(output / 'receipt.json')
    identity(receipt, head, tree)
    paired = root / 'import_type/import/ledger_field/paired/product'
    generated = inventory(paired)
    require(receipt.get('import_type_head') == head and
            receipt.get('import_type_gate_sha256') ==
            sha(root / 'import_type/receipt.json') and
            receipt.get('generated_input_sha256') == generated and
            receipt.get('zkir_sha256') == sha(zkir) and
            receipt.get('proof_execution') is True and
            receipt.get('funded_application') is True,
            'import ledger field proof input or scope differs')
    require(sha(ROOT / 'tools/compact-rust-backend/tests/'
                'ledger9_import_ledger_field_wrong_verifier.compact') ==
            receipt.get('wrong_source_sha256'),
            'proof wrong-verifier source differs')
    files(output / 'wrong-generated', receipt['wrong_generated_sha256'])
    require(sha(output / 'proof-runner') == receipt['runner_sha256'] and
            sha(output / 'proof-runner.d') == receipt['runner_depfile_sha256'] and
            sha(output / 'binding/source-hashes.json') == receipt['source_hashes_sha256'] and
            sha(output / 'binding/inputs.json') == receipt['runner_inventory_sha256'],
            'proof runner or bound input inventory differs')
    source = load(output / 'binding/source-hashes.json')
    tracked = {os.fsdecode(p) for p in subprocess.check_output(
        ['git', 'ls-files', '-z'], cwd=ROOT).split(b'\0') if p}
    require(set(source) == tracked and len(source) == receipt['source_count'],
            'proof tracked source roster differs')
    for name, digest in source.items():
        require(sha(ROOT / name) == digest, f'proof source differs: {name}')
    declared = load(output / 'binding/inputs.json')
    expected_source = {str((ROOT / name).resolve(strict=True)): digest
                       for name, digest in source.items()}
    expected_generated = {str((paired / name).resolve(strict=True)): digest
                          for name, digest in generated.items()}
    require(declared.get('source_hashes') == expected_source and
            declared.get('generated_input_hashes') == expected_generated,
            'proof source or generated prebuild map differs')
    fields = ('source_hashes', 'generated_input_hashes',
              'dependency_source_hashes', 'generated_build_inputs_before_compile')
    inventories = {key: declared.get(key) for key in fields}
    require(all(isinstance(item, dict) and item for item in inventories.values()),
            'proof physical input inventory incomplete')
    for field, mapping in inventories.items():
        for name, digest in mapping.items():
            require(re.fullmatch('[0-9a-f]{64}', digest) and sha(name) == digest,
                    f'proof physical input differs: {field}/{name}')
    prereqs = runner_inputs.depfile_prerequisites(
        (output / 'proof-runner.d').read_text(), ROOT)
    physical, origins = runner_inputs.qualify_dependency_inputs(
        prereqs, inventories, sha)
    require(physical == declared.get('physical_inputs_before_build') and
            origins == declared.get('physical_input_origins') and
            len(physical) == receipt.get('runner_physical_input_count') and
            origins == receipt.get('runner_physical_input_origins') and
            receipt.get('dependency_refusals') == {
                'unknown_dependency_rejected': True,
                'tampered_dependency_hash_rejected': True},
            'proof runner physical prerequisite closure differs')
    for path in (paired / 'contract/lib.rs', paired / 'contract/compact-rust-ir.json',
                 paired / 'zkir/import_ledger_field.zkir',
                 ROOT / 'tools/compact-rust-proof-smoke/src/import_ledger_field_proof.rs'):
        require(str(path.resolve(strict=True)) in physical,
                f'proof selected source omitted: {path}')
    files(output / 'proof', receipt['artifact_hashes'])
    summary = load(output / 'proof/import-ledger-field-proof-result.json')
    require(summary == receipt.get('proof_summary') and
            summary.get('status') == 'passed' and
            summary.get('selected_source_export_proved') == 'import_ledger_field' and
            summary.get('result') == [True, 1] and
            summary.get('ledger_flag') is True and summary.get('ledger_count') == 1 and
            summary.get('public_verify_ops') == 12 and summary.get('k') == 7 and
            summary.get('strict_application') == 'default' and
            summary.get('replay_refusal') == 'IntentAlreadyExists' and
            summary.get('positive_fee_dust_spends') == 1 and
            all(summary.get(key) is True for key in (
                'standalone_verified_separately', 'wrong_verifier_rejected_at_preparation',
                'wrong_verifier_proof_rejected', 'corrupt_proof_rejected',
                'changed_binding_rejected', 'replay_unchanged')) and
            summary.get('constructor_execution_proved') is False,
            'proof verification, strict application or replay differs')
    proof_dir = output / 'proof'
    require((proof_dir / 'import-ledger-field-ledger-after.bin').read_bytes() ==
            (proof_dir / 'import-ledger-field-ledger-replay.bin').read_bytes() and
            (proof_dir / 'import-ledger-field.proof').read_bytes() !=
            (proof_dir / 'import-ledger-field-transaction.proof').read_bytes(),
            'replay ledger or distinct proof artifacts differ')
    require(sha(proof_dir / 'import_ledger_field/contract/compact-rust-ir.json') ==
            sha(paired / 'contract/compact-rust-ir.json') and
            sha(proof_dir / 'zkir/import_ledger_field.zkir') ==
            sha(paired / 'zkir/import_ledger_field.zkir'),
            'proof IR or ZKIR differs from generated gate')
    key = receipt['keygen']['import_ledger_field']
    require(key['k'] == 7 and key['rows'] == 83 and
            sha(proof_dir / 'keys/import_ledger_field.prover') == key['prover_sha256'] and
            sha(proof_dir / 'keys/import_ledger_field.verifier') == key['verifier_sha256'],
            'fresh proof keygen differs')
    require(set(receipt['keygen']) ==
            {'import_ledger_field', 'import_ledger_field_wrong_verifier'},
            'proof keygen roster differs')
    for name, row in receipt['keygen'].items():
        require(sha(proof_dir / f'keys/{name}.prover') == row['prover_sha256'] and
                sha(proof_dir / f'keys/{name}.verifier') == row['verifier_sha256'] and
                sha(proof_dir / f'zkir/{name}.bzkir') ==
                row['binary_zkir_sha256'],
                f'proof keygen artifact differs: {name}')
    material = receipt['material']
    require(material.get('mode') == 'verify-only' and
            material.get('verification') ==
            'all files checked against locked upstream SHA256 tables' and
            {6, 7} <= {row['k'] for row in material['parameters']} and
            set(receipt['material_hashes']) ==
            {row['name'] for row in material['assets'] + material['parameters']},
            'verified k6/k7 proof material absent')
    for name, digest in receipt['material_hashes'].items():
        require(not Path(name).is_absolute() and '..' not in Path(name).parts and
                sha(Path(material['cache_directory']) / name) == digest,
                f'proving material differs: {name}')
    return sha(output / 'receipt.json')


def bytes_proof(root, head, tree, zkir):
    output = root / 'bytes_proof'
    receipt = load(output / 'receipt.json')
    identity(receipt, head, tree)
    paired = root / 'bytes_indexed/generated/both'
    generated = inventory(paired)
    require(receipt.get('bytes_gate_head') == head and
            receipt.get('bytes_gate_sha256') ==
            sha(root / 'bytes_indexed/receipt.json') and
            receipt.get('generated_input_sha256') == generated and
            receipt.get('zkir_sha256') == sha(zkir) and
            receipt.get('proof_execution') is True and
            receipt.get('funded_application') is True,
            'Bytes indexed proof input or scope differs')
    require(sha(ROOT / 'tools/compact-rust-backend/tests/'
                'ledger9_bytes_indexed_access_wrong_verifier.compact') ==
            receipt.get('wrong_source_sha256'),
            'proof wrong-verifier source differs')
    files(output / 'wrong-generated', receipt['wrong_generated_sha256'])
    require(sha(output / 'proof-runner') == receipt['runner_sha256'] and
            sha(output / 'proof-runner.d') == receipt['runner_depfile_sha256'] and
            sha(output / 'binding/source-hashes.json') == receipt['source_hashes_sha256'] and
            sha(output / 'binding/inputs.json') == receipt['runner_inventory_sha256'],
            'proof runner or bound input inventory differs')
    source = load(output / 'binding/source-hashes.json')
    tracked = {os.fsdecode(p) for p in subprocess.check_output(
        ['git', 'ls-files', '-z'], cwd=ROOT).split(b'\0') if p}
    require(set(source) == tracked and len(source) == receipt['source_count'],
            'proof tracked source roster differs')
    for name, digest in source.items():
        require(sha(ROOT / name) == digest, f'proof source differs: {name}')
    declared = load(output / 'binding/inputs.json')
    expected_source = {str((ROOT / name).resolve(strict=True)): digest
                       for name, digest in source.items()}
    expected_generated = {str((paired / name).resolve(strict=True)): digest
                          for name, digest in generated.items()}
    require(declared.get('source_hashes') == expected_source and
            declared.get('generated_input_hashes') == expected_generated,
            'proof source or generated prebuild map differs')
    fields = ('source_hashes', 'generated_input_hashes',
              'dependency_source_hashes', 'generated_build_inputs_before_compile')
    inventories = {key: declared.get(key) for key in fields}
    require(all(isinstance(item, dict) and item for item in inventories.values()),
            'proof physical input inventory incomplete')
    for field, mapping in inventories.items():
        for name, digest in mapping.items():
            require(re.fullmatch('[0-9a-f]{64}', digest) and sha(name) == digest,
                    f'proof physical input differs: {field}/{name}')
    prereqs = runner_inputs.depfile_prerequisites(
        (output / 'proof-runner.d').read_text(), ROOT)
    physical, origins = runner_inputs.qualify_dependency_inputs(
        prereqs, inventories, sha)
    require(physical == declared.get('physical_inputs_before_build') and
            origins == declared.get('physical_input_origins') and
            len(physical) == receipt.get('runner_physical_input_count') and
            origins == receipt.get('runner_physical_input_origins') and
            receipt.get('dependency_refusals') == {
                'unknown_dependency_rejected': True,
                'tampered_dependency_hash_rejected': True},
            'proof runner physical prerequisite closure differs')
    for path in (paired / 'contract/lib.rs', paired / 'contract/compact-rust-ir.json',
                 paired / 'zkir/bytes_loop_indexed_access.zkir',
                 ROOT / 'tools/compact-rust-proof-smoke/src/bytes_indexed_access_proof.rs'):
        require(str(path.resolve(strict=True)) in physical,
                f'proof selected source omitted: {path}')
    files(output / 'proof', receipt['artifact_hashes'])
    summary = load(output / 'proof/bytes-indexed-access-proof-result.json')
    require(summary == receipt.get('proof_summary') and
            summary.get('status') == 'passed' and
            summary.get('selected_source_export_proved') == 'bytes_loop_indexed_access' and
            summary.get('result') == 32 and
            summary.get('ledger_result') == 32 and
            summary.get('public_verify_ops') == 54 and summary.get('k') == 8 and
            summary.get('strict_application') == 'default' and
            summary.get('replay_refusal') == 'IntentAlreadyExists' and
            summary.get('positive_fee_dust_spends') == 1 and
            all(summary.get(key) is True for key in (
                'standalone_verified_separately', 'wrong_verifier_rejected_at_preparation',
                'wrong_verifier_proof_rejected', 'corrupt_proof_rejected',
                'changed_binding_rejected', 'replay_unchanged')) and
            summary.get('constructor_execution_proved') is False,
            'proof verification, strict application or replay differs')
    proof_dir = output / 'proof'
    require((proof_dir / 'bytes-indexed-access-ledger-after.bin').read_bytes() ==
            (proof_dir / 'bytes-indexed-access-ledger-replay.bin').read_bytes() and
            (proof_dir / 'bytes-indexed-access.proof').read_bytes() !=
            (proof_dir / 'bytes-indexed-access-transaction.proof').read_bytes(),
            'replay ledger or distinct proof artifacts differ')
    require(sha(proof_dir / 'bytes_loop_indexed_access/contract/compact-rust-ir.json') ==
            sha(paired / 'contract/compact-rust-ir.json') and
            sha(proof_dir / 'zkir/bytes_loop_indexed_access.zkir') ==
            sha(paired / 'zkir/bytes_loop_indexed_access.zkir'),
            'proof IR or ZKIR differs from generated gate')
    key = receipt['keygen']['bytes_loop_indexed_access']
    require(key['k'] == 8 and key['rows'] == 223 and
            sha(proof_dir / 'keys/bytes_loop_indexed_access.prover') == key['prover_sha256'] and
            sha(proof_dir / 'keys/bytes_loop_indexed_access.verifier') == key['verifier_sha256'],
            'fresh proof keygen differs')
    require(set(receipt['keygen']) ==
            {'bytes_loop_indexed_access', 'bytes_indexed_access_wrong_verifier'},
            'proof keygen roster differs')
    for name, row in receipt['keygen'].items():
        require(sha(proof_dir / f'keys/{name}.prover') == row['prover_sha256'] and
                sha(proof_dir / f'keys/{name}.verifier') == row['verifier_sha256'] and
                sha(proof_dir / f'zkir/{name}.bzkir') ==
                row['binary_zkir_sha256'],
                f'proof keygen artifact differs: {name}')
    material = receipt['material']
    require(material.get('mode') == 'verify-only' and
            material.get('verification') ==
            'all files checked against locked upstream SHA256 tables' and
            {6, 8} <= {row['k'] for row in material['parameters']} and
            set(receipt['material_hashes']) ==
            {row['name'] for row in material['assets'] + material['parameters']},
            'verified k6/k7 proof material absent')
    for name, digest in receipt['material_hashes'].items():
        require(not Path(name).is_absolute() and '..' not in Path(name).parts and
                sha(Path(material['cache_directory']) / name) == digest,
                f'proving material differs: {name}')
    return sha(output / 'receipt.json')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--all', type=Path, required=True)
    parser.add_argument('--source-sha', required=True)
    parser.add_argument('--compiler', type=Path, required=True)
    parser.add_argument('--compiler-recipe', type=Path, required=True)
    parser.add_argument('--runtime', type=Path, required=True)
    parser.add_argument('--zkir', type=Path, required=True)
    parser.add_argument('--summary', type=Path, required=True)
    args = parser.parse_args()
    head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT,
                                   text=True).strip()
    tree = subprocess.check_output(['git', 'rev-parse', 'HEAD^{tree}'], cwd=ROOT,
                                   text=True).strip()
    require(head == args.source_sha and not subprocess.check_output(
        ['git', 'status', '--porcelain'], cwd=ROOT).strip(), 'source SHA or clean state differs')
    root = args.all
    hashes = {name: family(name, root / name, head, tree,
                           args.compiler, args.compiler_recipe, args.runtime)
              for name in FAMILIES}
    hashes['bytes_controls'] = bytes_controls(root, head, tree,
                                              args.compiler, args.compiler_recipe,
                                              args.runtime)
    hashes['host'] = host(root, head, tree, args.compiler, args.compiler_recipe)
    hashes['import_proof'] = proof(root, head, tree, args.zkir)
    hashes['bytes_proof'] = bytes_proof(root, head, tree, args.zkir)
    summary = {'status': 'passed', 'head': head, 'tree': tree,
               'platform': 'Linux x86_64', 'upstream_commit': PINNED,
               'families': list(FAMILIES), 'host_families': HOST_FAMILIES,
               'host_tests': 221, 'receipt_hashes': hashes,
               'proof_execution': True, 'funded_application': True,
               'proved_exports': ['import/ledger_field', 'bytes/loops/indexed_access'],
               'limits': ['other stateful keyword and Bytes paths have no proof claim',
                          'constructor execution unproved',
                          'cross-contract transaction outside this gate']}
    args.summary.write_text(json.dumps(summary, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': 'passed', 'head': head, 'host_tests': 221,
                      'proved_exports': ['import/ledger_field', 'bytes/loops/indexed_access']}))


if __name__ == '__main__':
    main()
