#!/usr/bin/env python3
# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
"""Independently bind native Linux for-keyword parity and funded proof receipts."""

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
from check_upstream_ledger9_field_casts import (  # noqa: E402
    compiler_recipe_identity, runtime_identity, runtime_recipe_identity,
)
import proof_runner_input_binding as runner_inputs  # noqa: E402

PINNED = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'
FAMILY = Path('test-contracts/keywords/for')
PASS = {'generic_bound', 'return_in_lambda_in_body', 'zero_iteration'}
FAIL = {'non_constant_bound', 'return_in_body', 'return_in_body_of_constructor',
        'return_in_body_of_fold_callback', 'return_in_body_of_generic_circuit',
        'return_in_body_of_lambda', 'return_in_body_of_map_callback'}
ORIGINAL_RESULTS = {'generic_bound': 5, 'return_in_lambda_in_body': 7,
                    'zero_iteration': 0}
SYNTHETIC_RESULTS = {'repeated_values': 4, 'iterable_once': 5}
STATIC_ROSTER = {'effectful_operand': (True, False),
                 'shadowed_formal': (True, False),
                 'out_of_range': (False, False),
                 'cap_eight': (True, True),
                 'cap_nine': (True, True),
                 'cap_ten': (True, True),
                 'cap_eleven': (True, False),
                 'guarded_item': (True, False),
                 'mixed_literal': (True, False),
                 'nested_scope': (True, False),
                 'direct_dynamic': (False, False)}


def require(value, message):
    if not value:
        raise ValueError(message)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def load(path):
    require(path.is_file() and not path.is_symlink() and path.stat().st_size <= 8_000_000,
            f'missing, symlinked or oversized receipt: {path}')
    value = json.loads(path.read_text())
    require(isinstance(value, dict), f'non-object receipt: {path}')
    return value


def inventory(root):
    require(root.is_dir() and not root.is_symlink(), f'missing artifact directory: {root}')
    found = {}
    for path in sorted(root.rglob('*')):
        require(not path.is_symlink(), f'symlinked artifact: {path}')
        if path.is_file():
            found[path.relative_to(root).as_posix()] = sha(path)
    return found


def load_runner_inventory(path):
    require(path.is_file() and not path.is_symlink() and path.stat().st_size <= 64_000_000,
            f'missing, symlinked or oversized runner inventory: {path}')
    value = json.loads(path.read_text())
    require(isinstance(value, dict), 'runner inventory is not an object')
    return value


def verify_runner_inputs(receipt, output, semantic_root, semantic_receipt):
    """Independently classify every actual physical Cargo prerequisite."""
    binding = output / 'binding'
    source_path = binding / 'source-hashes.json'
    input_path = binding / 'inputs.json'
    require(sha(source_path) == receipt.get('source_hashes_sha256') and
            sha(input_path) == receipt.get('runner_inventory_sha256'),
            'retained source or runner input inventory differs')
    source = load_runner_inventory(source_path)
    declared = load_runner_inventory(input_path)
    tracked = [os.fsdecode(raw) for raw in subprocess.check_output(
        ['git', 'ls-files', '-z'], cwd=ROOT).split(b'\0') if raw]
    require(len(source) == receipt.get('source_count') == len(tracked) and
            set(source) == set(tracked), 'full committed source inventory differs')
    for name, digest in source.items():
        path = ROOT / name
        require(path.is_file() and not path.is_symlink() and sha(path) == digest,
                f'committed source hash differs: {name}')
    expected_source = {str((ROOT / name).resolve(strict=True)): digest
                       for name, digest in source.items()}
    expected_generated = {
        str((semantic_root / 'generated' / name).resolve(strict=True)): digest
        for name, digest in semantic_receipt['generated_files_sha256'].items()}
    require(declared.get('source_hashes') == expected_source and
            declared.get('generated_input_hashes') == expected_generated,
            'runner source or generated prebuild map differs')
    fields = ('source_hashes', 'generated_input_hashes',
              'dependency_source_hashes', 'generated_build_inputs_before_compile')
    inventories = {field: declared.get(field) for field in fields}
    require(all(isinstance(value, dict) and value for value in inventories.values()),
            'physical runner input inventory is incomplete')
    for field, mapping in inventories.items():
        for path, digest in mapping.items():
            require(re.fullmatch('[0-9a-f]{64}', digest) is not None and
                    Path(path).is_file() and sha(path) == digest,
                    f'runner {field} input differs: {path}')
    depfile = output / 'proof-runner.d'
    require(depfile.is_file() and not depfile.is_symlink(),
            'retained physical runner depfile is missing or symlinked')
    prerequisites = runner_inputs.depfile_prerequisites(depfile.read_text(), ROOT)
    physical, origins = runner_inputs.qualify_dependency_inputs(prerequisites,
                                                                 inventories, sha)
    require(physical == declared.get('physical_inputs_before_build') and
            origins == declared.get('physical_input_origins') and
            len(physical) == receipt.get('runner_physical_input_count') and
            origins == receipt.get('runner_physical_input_origins') and
            receipt.get('dependency_refusals') == {
                'unknown_dependency_rejected': True,
                'tampered_dependency_hash_rejected': True},
            'runner physical prerequisite closure or negative controls differ')
    both = semantic_root / 'generated/generic_bound/both'
    required = (both / 'contract/lib.rs', both / 'contract/compact-rust-ir.json',
                both / 'zkir/for_generic_bound.zkir',
                ROOT / 'tools/compact-rust-proof-smoke/src/for_generic_bound_proof.rs')
    require(all(str(path.resolve(strict=True)) in physical for path in required),
            'proof runner omitted a selected source or generated prerequisite')


def gate_log(path, expected):
    require(path.is_file() and not path.is_symlink(), f'missing gate log: {path}')
    lines = path.read_text().splitlines()
    require(lines and json.loads(lines[-1]) == expected,
            f'gate log identity or status differs: {path}')


def originals(receipt):
    manifest = load(ROOT / FAMILY / 'original-source-sha256.json')
    names = manifest.get('files')
    require(manifest.get('upstream_commit') == PINNED and
            isinstance(names, dict) and len(names) == 23 and
            receipt.get('original_files') == names,
            'pinned for original roster differs')
    upstream = set(subprocess.check_output(
        ['git', 'ls-tree', '-r', '--name-only', PINNED, str(FAMILY)],
        cwd=ROOT, text=True).splitlines())
    require(set(names) == upstream, 'pinned upstream for tree differs')
    for name, digest in names.items():
        require(name.startswith(str(FAMILY) + '/') and '..' not in Path(name).parts and
                re.fullmatch('[0-9a-f]{64}', digest), f'bad original path/hash: {name}')
        path = ROOT / name
        require(path.is_file() and not path.is_symlink() and sha(path) == digest,
                f'local original changed: {name}')
        upstream_bytes = subprocess.check_output(['git', 'show', f'{PINNED}:{name}'], cwd=ROOT)
        require(hashlib.sha256(upstream_bytes).hexdigest() == digest,
                f'upstream original changed: {name}')
    return names


def semantic(receipt, output, *, source, compiler, recipe, runtime):
    require(receipt.get('status') == 'passed' and receipt.get('source_clean') is True and
            receipt.get('platform') == 'Linux x86_64' and
            (receipt.get('head'), receipt.get('tree')) == (source['head'], source['tree']) and
            receipt.get('upstream_commit') == PINNED and
            (receipt.get('case_count'), receipt.get('positive_cases'),
             receipt.get('refusal_cases')) == (10, 3, 7) and
            receipt.get('proof_execution') is False and
            receipt.get('funded_application') is False,
            'native Linux for semantic identity or scope differs')
    originals(receipt)
    require(receipt.get('compiler_sha256') == sha(compiler) and
            receipt.get('scheme_sha256') == sha(compiler.with_name('compactc-scheme')) and
            receipt.get('compiler_recipe') == compiler_recipe_identity(
                recipe, source, compiler, compiler.with_name('compactc-scheme')),
            'compiler recipe or binary differs')
    require(receipt.get('runtime') == runtime_identity(runtime) and
            receipt.get('runtime_recipe') == runtime_recipe_identity(
                runtime, Path(shutil.which('nix')).resolve(strict=True)),
            'runtime package or Nix recipe differs')
    local = receipt.get('local_sources')
    expected_local = {str(FAMILY / 'original-source-sha256.json'),
                      str(FAMILY / 'check-original-results.mjs'),
                      str(FAMILY / 'consumer-original.rs'),
                      'tools/compact-rust-backend/check_upstream_ledger9_for_keywords.py'}
    require(isinstance(local, dict) and set(local) == expected_local,
            'local for adapter roster differs')
    for name, digest in local.items():
        require(sha(ROOT / name) == digest, f'local for adapter changed: {name}')
    generated = receipt.get('generated')
    require(isinstance(generated, dict) and
            set(generated) == PASS | set(SYNTHETIC_RESULTS),
            'for generated case roster differs')
    synthetic = receipt.get('synthetic_sources')
    require(isinstance(synthetic, dict) and set(synthetic) == set(SYNTHETIC_RESULTS),
            'synthetic source roster differs')
    for name, digest in synthetic.items():
        path = output / 'synthetic' / name / f'{name}.compact'
        require(path.is_file() and not path.is_symlink() and sha(path) == digest,
                f'synthetic source differs: {name}')
    require(inventory(output / 'generated') == receipt.get('generated_files_sha256'),
            'for generated artifact inventory differs')
    for name, modes in generated.items():
        require(set(modes) == {'typescript', 'both'}, f'for modes differ: {name}')
        for mode, row in modes.items():
            base = output / 'generated' / name / mode
            paths = {'typescript_sha256': base / 'contract/index.js',
                     'contract_info_sha256': base / 'compiler/contract-info.json',
                     'manifest_sha256': base / 'compiler/contract-manifest.json',
                     'runtime_log_sha256': output / f'{name}-{mode}-runtime.log'}
            if mode == 'both':
                paths.update({'rust_sha256': base / 'contract/lib.rs',
                              'rust_ir_sha256': base / 'contract/compact-rust-ir.json'})
            if name != 'return_in_lambda_in_body':
                circuit = f'for_{name}' if name in PASS else 'for_generic_bound'
                paths['zkir_sha256'] = base / f'zkir/{circuit}.zkir'
            require(set(row) == set(paths), f'for artifact keys differ: {name}/{mode}')
            for key, path in paths.items():
                require(path.is_file() and not path.is_symlink() and sha(path) == row[key],
                        f'for artifact differs: {name}/{mode}/{key}')
            runtime_log = paths['runtime_log_sha256'].read_text()
            require(runtime_log == f'{name}: original for result passed\n',
                    f'for original result log differs: {name}/{mode}')
        for key in ('typescript_sha256', 'contract_info_sha256', 'runtime_log_sha256'):
            require(modes['typescript'][key] == modes['both'][key],
                    f'for TS/paired result differs: {name}/{key}')
        if name != 'return_in_lambda_in_body':
            require(modes['typescript']['zkir_sha256'] == modes['both']['zkir_sha256'],
                    f'for TS/paired raw ZKIR differs: {name}')
        if name in PASS:
            original_test = (ROOT / FAMILY / name / 'runtime.pass.test.ts').read_text()
            require(f'expect(result).toEqual({ORIGINAL_RESULTS[name]}n)' in original_test,
                    f'original for runtime assertion differs: {name}')
    refusals = receipt.get('refusals')
    require(isinstance(refusals, dict) and set(refusals) == FAIL,
            'for original refusal roster differs')
    for name, modes in refusals.items():
        require(set(modes) == {'typescript', 'both'}, f'for refusal modes: {name}')
        original = (ROOT / FAMILY / name / 'compile.fail.test.ts').read_text()
        matches = re.findall(r'expectedError:\s*/((?:\\.|[^/])*)/', original)
        require(len(matches) == 1, f'for original regex missing: {name}')
        for mode, row in modes.items():
            log = output / 'diagnostics' / mode / f'{name}.log'
            require(log.is_file() and sha(log) == row.get('diagnostic_sha256') and
                    row.get('original_regex') == matches[0] and
                    re.search(matches[0], log.read_text()) is not None and
                    row.get('exit_code', 0) != 0 and
                    row.get('output_absent') is True and
                    row.get('drift_control_refused') is True and
                    not (output / 'refused' / name / mode / 'product').exists(),
                    f'for original refusal differs: {name}/{mode}')
        require(modes['typescript']['diagnostic_sha256'] == modes['both']['diagnostic_sha256'],
                f'for target diagnostic drift: {name}')
    cargo_log = output / 'cargo-test.log'
    require(sha(cargo_log) == receipt.get('rust_test_log_sha256') and
            'test result: ok. 5 passed; 0 failed;' in cargo_log.read_text(),
            'five generated Rust for consumers did not pass')


def proof(receipt, output, *, source, semantic_receipt, semantic_path,
          semantic_root, zkir):
    require(receipt.get('status') == 'passed' and receipt.get('source_clean') is True and
            receipt.get('platform') == 'Linux x86_64' and
            (receipt.get('head'), receipt.get('tree')) == (source['head'], source['tree']) and
            receipt.get('grouped_head') == source['head'] and
            receipt.get('grouped_gate_sha256') == sha(semantic_path) and
            receipt.get('generated_input_sha256') == semantic_receipt['generated_files_sha256'] and
            receipt.get('zkir_sha256') == sha(zkir) and
            receipt.get('proof_execution') is True and receipt.get('funded_application') is True,
            'native Linux proof identity, source or scope differs')
    runner = output / 'proof-runner'
    require(runner.is_file() and not runner.is_symlink() and
            sha(runner) == receipt.get('runner_sha256') and
            sha(output / 'proof-runner.d') == receipt.get('runner_depfile_sha256'),
            'proof runner or build provenance differs')
    verify_runner_inputs(receipt, output, semantic_root, semantic_receipt)
    root = output / 'proof'
    require(inventory(root) == receipt.get('artifact_hashes'),
            'proof artifact inventory differs')
    require(sha(root / 'for_generic_bound/contract/compact-rust-ir.json') ==
            sha(semantic_root / 'generated/generic_bound/both/contract/compact-rust-ir.json') and
            sha(root / 'zkir/for_generic_bound.zkir') ==
            sha(semantic_root / 'generated/generic_bound/both/zkir/for_generic_bound.zkir') and
            sha(root / 'zkir/for_zero_iteration.zkir') ==
            sha(semantic_root / 'generated/zero_iteration/both/zkir/for_zero_iteration.zkir'),
            'proof IR or raw ZKIR differs from the exact generated semantic gate')
    summary = load(root / 'for_generic_bound-proof-result.json')
    require(summary == receipt.get('proof_summary') and
            summary.get('status') == 'passed' and
            summary.get('selected_source_export_proved') == 'for_generic_bound' and
            summary.get('result') == summary.get('ledger_total') == 5 and
            summary.get('public_verify_ops') == 18 and
            summary.get('private_transcript_outputs') == 0 and
            summary.get('k') == 7 and
            summary.get('strict_application') == 'default' and
            summary.get('replay_refusal') == 'IntentAlreadyExists' and
            summary.get('positive_fee_dust_spends') == 1 and
            summary.get('standalone_verified_separately') is True and
            summary.get('wrong_verifier_rejected_at_preparation') is True and
            summary.get('wrong_verifier_proof_rejected') is True and
            summary.get('corrupt_proof_rejected') is True and
            summary.get('changed_binding_rejected') is True and
            summary.get('replay_unchanged') is True,
            'standalone proof, strict application or replay result differs')
    for name in ('for-generic-bound.proof', 'for-generic-bound-transaction.proof',
                 'for-generic-bound-ledger-after.bin', 'for-generic-bound-ledger-replay.bin'):
        require((root / name).is_file() and (root / name).stat().st_size > 0,
                f'proof/ledger artifact absent: {name}')
    require((root / 'for-generic-bound-ledger-after.bin').read_bytes() ==
            (root / 'for-generic-bound-ledger-replay.bin').read_bytes(),
            'replay changed ledger bytes')
    require((root / 'for-generic-bound.proof').read_bytes() !=
            (root / 'for-generic-bound-transaction.proof').read_bytes(),
            'standalone and transaction proofs not distinct')
    keygen = receipt.get('keygen')
    require(isinstance(keygen, dict) and set(keygen) ==
            {'for_generic_bound', 'for_zero_iteration'} and
            keygen['for_generic_bound']['k'] == 7 and
            keygen['for_generic_bound']['rows'] == 97,
            'fresh circuit keygen model differs')
    for name, row in keygen.items():
        for key, suffix in (('prover_sha256', '.prover'), ('verifier_sha256', '.verifier')):
            require(sha(root / 'keys' / f'{name}{suffix}') == row[key],
                    f'keygen artifact differs: {name}/{key}')
        require(sha(root / 'zkir' / f'{name}.bzkir') == row['binary_zkir_sha256'],
                f'binary ZKIR differs: {name}')
    material = receipt.get('material')
    require(material.get('mode') == 'verify-only' and
            material.get('verification') ==
            'all files checked against locked upstream SHA256 tables' and
            {6, 7} <= {row['k'] for row in material['parameters']},
            'k6/k7 verified proof material missing')
    cache = Path(material['cache_directory'])
    expected_material = [row['name'] for row in
                         material['assets'] + material['parameters']]
    material_hashes = receipt.get('material_hashes')
    require(isinstance(material_hashes, dict) and expected_material and
            len(expected_material) == len(set(expected_material)) and
            set(material_hashes) == set(expected_material),
            'verified proving-material inventory differs')
    for name, digest in material_hashes.items():
        member = cache / name
        require(not Path(name).is_absolute() and '..' not in Path(name).parts and
                member.is_file() and not member.is_symlink() and
                member.resolve(strict=True).is_relative_to(cache.resolve(strict=True)) and
                sha(member) == digest,
                f'proof material differs: {name}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('semantic', 'semantic-log', 'refusals', 'refusals-log',
                 'occurrence-log', 'source-sha', 'compiler', 'compiler-recipe',
                 'runtime', 'zkir', 'summary'):
        parser.add_argument('--' + name, required=True)
    parser.add_argument('--proof')
    parser.add_argument('--proof-log')
    args = parser.parse_args()
    require((args.proof is None) == (args.proof_log is None),
            'proof and proof log must be supplied together')
    source = {'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT,
                                              text=True).strip(),
              'tree': subprocess.check_output(['git', 'rev-parse', 'HEAD^{tree}'], cwd=ROOT,
                                              text=True).strip()}
    require(re.fullmatch('[0-9a-f]{40}', args.source_sha) and
            args.source_sha == source['head'] and
            subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT) == b'',
            'selected source head or clean tree differs')
    semantic_root = Path(args.semantic).resolve(strict=True)
    semantic_path = semantic_root / 'receipt.json'
    gate_log(Path(args.semantic_log), {'status':'passed', 'head':source['head'],
                                      'cases':10, 'output':str(semantic_root)})
    receipt = load(semantic_path)
    semantic(receipt, semantic_root, source=source,
             compiler=Path(args.compiler).resolve(strict=True),
             recipe=Path(args.compiler_recipe).resolve(strict=True),
             runtime=Path(args.runtime).resolve(strict=True))
    refusals_root = Path(args.refusals).resolve(strict=True)
    gate_log(Path(args.refusals_log), {'status':'passed', 'cases':len(STATIC_ROSTER)})
    negative = load(refusals_root / 'receipt.json')
    require(negative.get('status') == 'passed' and
            len(negative.get('cases', {})) == len(STATIC_ROSTER) and
            negative.get('compiler_sha256') == sha(args.compiler) and
            negative.get('upstream_commit') == PINNED and
            negative.get('original_sha256') ==
            receipt['original_files'][str(FAMILY / 'generic_bound/for_generic_bound.compact')],
            'static-domain refusal receipt differs')
    require(set(negative['cases']) == set(STATIC_ROSTER), 'static-domain probe roster differs')
    for name, (ts_passes, paired_passes) in STATIC_ROSTER.items():
        row = negative['cases'][name]
        folder = refusals_root / name
        require(sha(folder / f'{name}.compact') == row['source_sha256'],
                f'static-domain probe source differs: {name}')
        for mode, success in (('typescript', ts_passes), ('paired', paired_passes)):
            result = row[mode]
            require((result['exit'] == 0) == success and
                    result['output_published'] is success and
                    (folder / mode).exists() is success and
                    sha(folder / f'{mode}.log') == result['log_sha256'],
                    f'static-domain probe outcome differs: {name}/{mode}')
    require(Path(args.occurrence_log).read_text().splitlines()[-1] ==
            '64 Rust collection index occurrence controls passed',
            'merged source occurrence controls did not pass')
    proof_sha = None
    if args.proof:
        proof_root = Path(args.proof).resolve(strict=True)
        gate_log(Path(args.proof_log), {'status':'passed', 'head':source['head'],
                                       'output':str(proof_root)})
        proof_path = proof_root / 'receipt.json'
        proof(load(proof_path), proof_root, source=source,
              semantic_receipt=receipt, semantic_path=semantic_path,
              semantic_root=semantic_root,
              zkir=Path(args.zkir).resolve(strict=True))
        proof_sha = sha(proof_path)
    summary_path = Path(args.summary)
    require(not summary_path.exists() and not summary_path.is_symlink(),
            'summary output must be new')
    summary_path.write_text(json.dumps({
        'status':'passed', 'head':source['head'], 'tree':source['tree'],
        'platform':'Linux x86_64', 'semantic_receipt_sha256':sha(semantic_path),
        'refusal_receipt_sha256':sha(refusals_root / 'receipt.json'),
        'proof_receipt_sha256':proof_sha, 'proof_execution':bool(args.proof),
        'funded_application':bool(args.proof), 'cases':10, 'original_files':23,
    }, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status':'passed', 'head':source['head'],
                      'proof':bool(args.proof), 'summary':str(summary_path)}))


if __name__ == '__main__':
    main()
