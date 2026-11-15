#!/usr/bin/env python3

# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0

"""Run the pinned for-keyword originals admitted by paired TS and Rust."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tomllib

from check_compactc_target import check_manifest
from check_unit_mutation_initializers import validate_consumer_lock
from check_upstream_ledger9_const_keywords import valid_source_signature
from check_upstream_ledger9_field_casts import (
    compiler_recipe_identity, require, run, runtime_identity,
    runtime_recipe_identity, sha256, source_state,
)


ROOT = Path(__file__).resolve().parents[2]
PINNED = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'
FAMILY = Path('test-contracts/keywords/for')
MANIFEST = ROOT / FAMILY / 'original-source-sha256.json'
ADAPTER = ROOT / FAMILY / 'check-original-results.mjs'
CONSUMER = ROOT / FAMILY / 'consumer-original.rs'
PASS = ('return_in_lambda_in_body', 'zero_iteration')
FAIL = ('non_constant_bound', 'return_in_body', 'return_in_body_of_constructor',
        'return_in_body_of_fold_callback', 'return_in_body_of_generic_circuit',
        'return_in_body_of_lambda', 'return_in_body_of_map_callback')
PATTERN = re.compile(r'expectedError:\s*/((?:\\.|[^/])*)/')
ORIGINAL_EXPECTATIONS = {'return_in_lambda_in_body': 'expect(result).toEqual(7n)',
                         'zero_iteration': 'expect(result).toEqual(0n)'}


def original_inventory():
    files = json.loads(MANIFEST.read_text())
    expected = {str(FAMILY / case / name) for case in PASS
                for name in (f'for_{case}.compact', 'compile.pass.test.ts',
                             'runtime.pass.test.ts')} | {
                str(FAMILY / case / name) for case in FAIL
                for name in (f'for_{case}.compact', 'compile.fail.test.ts')}
    require(files.get('upstream_commit') == PINNED and
            set(files.get('files', {})) == expected and len(expected) == 20,
            'for-keyword pinned roster changed')
    upstream = set(subprocess.check_output(
        ['git', 'ls-tree', '-r', '--name-only', PINNED, str(FAMILY)],
        cwd=ROOT, text=True).splitlines())
    require(upstream - expected == {
        str(FAMILY / 'generic_bound' / name) for name in (
            'for_generic_bound.compact', 'compile.pass.test.ts',
            'runtime.pass.test.ts')}, 'unexpected missing for-keyword original')
    for name, digest in files['files'].items():
        path = ROOT / name
        source = subprocess.check_output(['git', 'show', f'{PINNED}:{name}'], cwd=ROOT)
        require(path.is_file() and not path.is_symlink() and
                re.fullmatch('[0-9a-f]{64}', digest) is not None and
                sha256(path) == digest == hashlib.sha256(source).hexdigest(),
                f'for-keyword original changed: {name}')
    patterns = {}
    for case in FAIL:
        text = (ROOT / FAMILY / case / 'compile.fail.test.ts').read_text()
        matches = PATTERN.findall(text)
        require(len(matches) == 1, f'one original diagnostic required: {case}')
        patterns[case] = matches[0]
    for case, snippet in ORIGINAL_EXPECTATIONS.items():
        require(snippet in (ROOT / FAMILY / case / 'runtime.pass.test.ts').read_text(),
                f'original for runtime assertion changed: {case}')
    return files['files'], patterns


def generated_files(root):
    files = {}
    for path in sorted(root.rglob('*')):
        require(not path.is_symlink(), f'generated symlink: {path}')
        if path.is_file():
            files[path.relative_to(root).as_posix()] = sha256(path)
    require(files, 'no generated artifacts')
    return files


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'compiler-recipe', 'ts-runtime', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
    parser.add_argument('--cargo-target', type=Path)
    args = parser.parse_args()
    source = source_state()
    signature, author, body = subprocess.check_output(
        ['git', 'log', '-1', '--format=%G?%n%an <%ae>%n%B'], cwd=ROOT,
        text=True).split('\n', 2)
    require(valid_source_signature(signature, author, body),
            'signed author-matching DCO+Assisted head required')
    compiler = args.compiler.resolve(strict=True)
    scheme = compiler.with_name('compactc-scheme').resolve(strict=True)
    runtime = args.ts_runtime.resolve(strict=True)
    output = args.output.resolve()
    require(not args.output.is_symlink() and not output.exists(), 'output must be absent')
    source_package = json.loads((ROOT / 'runtime/package.json').read_text())
    installed_package = json.loads((runtime / 'package.json').read_text())
    require(all(source_package[key] == installed_package[key]
                for key in ('name', 'version', 'dependencies')),
            'runtime package differs from source')
    nix = shutil.which('nix')
    require(nix is not None, 'Nix required for runtime source binding')
    runtime_recipe = runtime_recipe_identity(runtime, Path(nix).resolve(strict=True))
    runtime_before = runtime_identity(runtime)
    recipe = compiler_recipe_identity(args.compiler_recipe, source, compiler, scheme)
    originals, patterns = original_inventory()
    local_sources = {str(path.relative_to(ROOT)): sha256(path)
                     for path in (MANIFEST, ADAPTER, CONSUMER, Path(__file__))}
    output.mkdir(parents=True)
    link = output / 'node_modules/@midnight-ntwrk/compact-runtime'
    link.parent.mkdir(parents=True)
    link.symlink_to(runtime, target_is_directory=True)
    (output / 'package.json').write_text('{"type":"module"}\n')
    env = dict(os.environ, COMPACTC_SCHEME=str(scheme), RUSTUP_TOOLCHAIN='1.99.0')
    generated = {}
    for case in PASS:
        generated[case] = {}
        contract = ROOT / FAMILY / case / f'for_{case}.compact'
        for mode in ('typescript', 'both'):
            destination = output / 'generated' / case / mode
            command = [str(compiler), '--target', 'ts']
            if mode == 'both':
                command += ['--target', 'rust', '--rust-runtime-root', str(ROOT),
                            '--rust-require-recording']
            command += ['--feature-zkir-v3', '--skip-zk', '--compact-path',
                        str(ROOT / 'compiler'), str(contract), str(destination)]
            run(command, output / f'{case}-{mode}-compile.log', env)
            proof = case == 'zero_iteration'
            check_manifest(destination, require_zkir=proof)
            info = destination / 'compiler/contract-info.json'
            circuits = json.loads(info.read_text())['circuits']
            require(len(circuits) == 1 and circuits[0]['name'] == f'for_{case}' and
                    circuits[0]['proof'] is proof and circuits[0]['pure'] is not proof,
                    f'for capability changed: {case}')
            if proof:
                require((destination / f'zkir/for_{case}.zkir').is_file(),
                        f'for proof-required ZKIR missing: {case}')
            else:
                require(not (destination / 'zkir').exists(),
                        f'proof-free for emitted ZKIR: {case}')
            target = destination / 'contract'
            runtime_log = output / f'{case}-{mode}-runtime.log'
            actual = run(['node', str(ADAPTER), str(target / 'index.js'), case],
                         runtime_log, env)
            require(actual.strip() == f'{case}: original for result passed',
                    f'for TypeScript result changed: {case}/{mode}')
            row = {'typescript_sha256': sha256(target / 'index.js'),
                   'contract_info_sha256': sha256(info),
                   'manifest_sha256': sha256(destination / 'compiler/contract-manifest.json'),
                   'runtime_log_sha256': sha256(runtime_log)}
            if proof:
                row['zkir_sha256'] = sha256(destination / f'zkir/for_{case}.zkir')
            if mode == 'both':
                row['rust_sha256'] = sha256(target / 'lib.rs')
                row['rust_ir_sha256'] = sha256(target / 'compact-rust-ir.json')
            generated[case][mode] = row
        require(all(generated[case]['typescript'][key] == generated[case]['both'][key]
                    for key in ('typescript_sha256', 'contract_info_sha256',
                                'runtime_log_sha256')),
                f'paired target altered for TypeScript: {case}')
        if case == 'zero_iteration':
            require(generated[case]['typescript']['zkir_sha256'] ==
                    generated[case]['both']['zkir_sha256'],
                    'paired target altered for ZKIR')
    refusals = {}
    for case in FAIL:
        refusals[case] = {}
        contract = ROOT / FAMILY / case / f'for_{case}.compact'
        for mode in ('typescript', 'both'):
            destination = output / 'refused' / case / mode / 'product'
            destination.parent.mkdir(parents=True)
            command = [str(compiler), '--target', 'ts']
            if mode == 'both':
                command += ['--target', 'rust', '--rust-runtime-root', str(ROOT)]
            command += ['--feature-zkir-v3', '--skip-zk', '--compact-path',
                        str(ROOT / 'compiler'), str(contract), str(destination)]
            result = subprocess.run(command, cwd=ROOT, env=env,
                                    capture_output=True, text=True)
            diagnostic = result.stdout + result.stderr
            pattern = patterns[case]
            require(result.returncode != 0 and result.stdout == '' and
                    re.search(pattern, diagnostic) is not None and
                    re.search(pattern, diagnostic.replace('return is not supported',
                                                          'return is allowed').replace(
                        'invalid context', 'valid context')) is None,
                    f'original for diagnostic or drift control failed: {case}/{mode}')
            lock = destination.parent / f'.{destination.name}.compactc.lock'
            require(not destination.exists() and
                    not list(destination.parent.glob(f'.{destination.name}.compactc-stage-*')) and
                    all(child == lock and child.is_file() and not child.is_symlink() and
                        child.stat().st_size == 0 for child in destination.parent.iterdir()),
                    f'for refusal published output: {case}/{mode}')
            log = output / 'diagnostics' / mode / f'{case}.log'
            log.parent.mkdir(parents=True, exist_ok=True)
            log.write_text(diagnostic)
            refusals[case][mode] = {'original_regex': pattern,
                                    'diagnostic_sha256': sha256(log),
                                    'exit_code': result.returncode,
                                    'output_absent': True,
                                    'drift_control_refused': True}
        require(refusals[case]['typescript']['diagnostic_sha256'] ==
                refusals[case]['both']['diagnostic_sha256'],
                f'for target diagnostic drift: {case}')
    generated_before = generated_files(output / 'generated')
    consumer = output / 'rust-consumer'
    (consumer / 'tests').mkdir(parents=True)
    shutil.copyfile(CONSUMER, consumer / 'tests/for_original.rs')
    shutil.copyfile(ROOT / 'Cargo.lock', consumer / 'Cargo.lock')
    dependencies = []
    paths = []
    for case in PASS:
        target = output / 'generated' / case / 'both/contract'
        paths.append(target)
        package = tomllib.loads((target / 'Cargo.toml').read_text())['package']['name']
        dependencies.append(f'{case} = {{ package = {json.dumps(package)}, '
                            f'path = {json.dumps(str(target))} }}')
    manifest = consumer / 'Cargo.toml'
    manifest.write_text('[workspace]\n[package]\nname = "ledger9-keyword-for-consumer"\n'
                        'version = "0.1.0"\nedition = "2024"\n[dependencies]\n'
                        'midnight-onchain-vm = "=4.0.1"\nmidnight-serialize = "=1.2.1"\n' +
                        '\n'.join(dependencies) + '\n')
    cargo_target = (args.cargo_target.resolve(strict=True) if args.cargo_target else
                    output / 'cargo-target')
    require(not args.cargo_target or (cargo_target.is_dir() and
                                     not args.cargo_target.is_symlink()),
            'shared Cargo target must be ordinary directory')
    cargo_env = dict(env, CARGO_TARGET_DIR=str(cargo_target), CARGO_INCREMENTAL='0',
                     CARGO_PROFILE_DEV_DEBUG='0')
    metadata = json.loads(run(['cargo', '+1.99.0', 'metadata', '--offline',
                               '--format-version', '1', '--manifest-path', str(manifest)],
                              output / 'cargo-metadata.log', cargo_env))
    root_lock = tomllib.loads((ROOT / 'Cargo.lock').read_text())['package']
    pinned = {(row['name'], row['version'], row['source']): row.get('checksum')
              for row in root_lock if 'source' in row}
    allowed = {manifest, ROOT / 'runtime-rs/Cargo.toml',
               ROOT / 'runtime-rs-macros/Cargo.toml',
               *(path / 'Cargo.toml' for path in paths)}
    validate_consumer_lock(metadata, consumer / 'Cargo.lock', allowed, pinned)
    run(['cargo', '+1.99.0', 'test', '--locked', '--offline',
         '--manifest-path', str(manifest), '--test', 'for_original'],
        output / 'cargo-test.log', cargo_env)
    rust_log = (output / 'cargo-test.log').read_text()
    require('test result: ok. 2 passed; 0 failed;' in rust_log and
            'test original_return_in_nested_lambda ... ok' in rust_log and
            'test original_zero_iteration_and_recorded_replay ... ok' in rust_log,
            'for Rust consumer assertions did not pass')
    require(source_state() == source and runtime_identity(runtime) == runtime_before and
            compiler_recipe_identity(args.compiler_recipe, source, compiler, scheme) == recipe and
            original_inventory() == (originals, patterns) and
            generated_files(output / 'generated') == generated_before and
            all(sha256(ROOT / name) == digest for name, digest in local_sources.items()),
            'source, compiler, runtime, originals or generated artifacts changed')
    receipt = {**source, 'status': 'passed', 'source_clean': True,
               'platform': os.uname().sysname + ' ' + os.uname().machine,
               'upstream_commit': PINNED, 'original_files': originals,
               'local_sources': local_sources, 'generated': generated,
               'generated_files_sha256': generated_before, 'refusals': refusals,
               'compiler_recipe': recipe, 'compiler_sha256': sha256(compiler),
               'scheme_sha256': sha256(scheme), 'runtime': runtime_before,
               'runtime_recipe': runtime_recipe,
               'rust_consumer_lock_sha256': sha256(consumer / 'Cargo.lock'),
               'rust_test_log_sha256': sha256(output / 'cargo-test.log'),
               'case_count': 9, 'positive_cases': 2, 'refusal_cases': 7,
               'proof_execution': False, 'funded_application': False,
               'result': 'nine pinned for-keyword diagnostics and TS/Rust results passed',
               'limitations': ['generic_bound original remains blocked by multiple Vector indices at one occurrence',
                               'native Linux qualification required',
                               'zero_iteration proof=true: fresh proof and funded strict application remain separate',
                               'original Vitest host harness was not run; pinned assertions replayed against generated APIs']}
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': 'passed', 'head': source['head'], 'cases': 9,
                      'output': str(output)}))


if __name__ == '__main__':
    main()
