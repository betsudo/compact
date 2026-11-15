#!/usr/bin/env python3

# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0

"""Qualify every pinned const-keyword original with generated TS and Rust."""

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
from check_upstream_ledger9_field_casts import (
    compiler_recipe_identity, require, run, runtime_identity,
    runtime_recipe_identity, sha256, source_state,
)


ROOT = Path(__file__).resolve().parents[2]
PINNED = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'
FAMILY = Path('test-contracts/keywords/const')
MANIFEST = ROOT / FAMILY / 'original-source-sha256.json'
ADAPTER = ROOT / FAMILY / 'check-original-results.mjs'
CONSUMER = ROOT / FAMILY / 'consumer-original.rs'
PATTERN = re.compile(r'expectedError:\s*/((?:\\.|[^/])*)/')
PASS = (
    'empty_tuple_pattern', 'generic_type_annotation', 'in_braced_bodies',
    'initializer_side_effects', 'later_binding_sees_earlier',
    'multiple_bindings', 'pattern_trailing_comma', 'shadowing_in_blocks',
    'struct_destructuring', 'tuple_destructuring', 'underscore_binding',
)
FAIL = (
    'as_lambda_body', 'duplicate_in_one_block', 'duplicate_in_one_declaration',
    'duplicate_underscore', 'forward_reference_in_declaration',
    'in_else_without_block', 'in_expression_position', 'in_for_without_block',
    'in_if_without_block', 'missing_comma', 'trailing_comma',
    'use_before_declaration',
)
ORIGINAL_EXPECTATIONS = {
    'empty_tuple_pattern': ('expect(result).toEqual([])',),
    'generic_type_annotation': ('expect(result).toEqual([0n, 0n, 0n])',),
    'in_braced_bodies': ('expect(result).toEqual(3n)', ').toEqual(12n)'),
    'initializer_side_effects': ('expect(result).toEqual([1n, 2n])',),
    'later_binding_sees_earlier': ('expect(result).toEqual(2n)',),
    'multiple_bindings': ('expect(result).toEqual([true, 1n, 2n])',),
    'pattern_trailing_comma': ('expect(result).toEqual(1n)',),
    'shadowing_in_blocks': ('expect(result).toEqual(1n)',),
    'struct_destructuring': ('expect(result).toEqual([1n, 2n])',),
    'tuple_destructuring': ('expect(result).toEqual([1n, 2n, 10n, 20n])',),
    'underscore_binding': ('expect(result).toEqual(2n)',),
}


def original_inventory() -> tuple[dict[str, str], dict[str, str]]:
    record = json.loads(MANIFEST.read_text())
    pinned = subprocess.check_output(
        ['git', 'ls-tree', '-r', '--name-only', PINNED, str(FAMILY)],
        cwd=ROOT, text=True).splitlines()
    expected = {
        str(FAMILY / case / name)
        for case in PASS
        for name in (f'const_{case}.compact', 'compile.pass.test.ts',
                     'runtime.pass.test.ts')
    } | {
        str(FAMILY / case / name)
        for case in FAIL
        for name in (f'const_{case}.compact', 'compile.fail.test.ts')
    }
    files = record.get('files')
    require(record.get('upstream_commit') == PINNED and
            isinstance(files, dict) and set(files) == expected == set(pinned) and
            len(files) == 57, 'const-keyword pinned roster changed')
    for name, digest in files.items():
        path = ROOT / name
        upstream = subprocess.check_output(['git', 'show', f'{PINNED}:{name}'], cwd=ROOT)
        require(path.is_file() and not path.is_symlink() and
                re.fullmatch(r'[0-9a-f]{64}', digest) is not None and
                sha256(path) == digest == hashlib.sha256(upstream).hexdigest(),
                f'const-keyword original changed: {name}')
    patterns = {}
    for case in FAIL:
        name = FAMILY / case / 'compile.fail.test.ts'
        matches = PATTERN.findall((ROOT / name).read_text())
        require(len(matches) == 1, f'one original diagnostic required: {name}')
        patterns[case] = matches[0]
    for case, snippets in ORIGINAL_EXPECTATIONS.items():
        text = (ROOT / FAMILY / case / 'runtime.pass.test.ts').read_text()
        require(all(snippet in text for snippet in snippets),
                f'original runtime assertion changed: {case}')
    return files, patterns


def generated_files(root: Path) -> dict[str, str]:
    files = {}
    for path in sorted(root.rglob('*')):
        require(not path.is_symlink(), f'generated symlink: {path}')
        if path.is_file():
            files[path.relative_to(root).as_posix()] = sha256(path)
    require(files, 'no generated artifacts')
    return files


def refused(result: subprocess.CompletedProcess[str], destination: Path,
            pattern: str) -> str:
    diagnostic = result.stdout + result.stderr
    require(result.returncode != 0 and result.stdout == '' and
            re.search(pattern, diagnostic) is not None and
            re.search(pattern, diagnostic.replace('parse error:', 'changed diagnostic:')
                      .replace('found multiple bindings', 'changed multiple bindings')
                      .replace('might be referenced', 'cannot be referenced')
                      .replace('const binding found', 'changed binding found')) is None,
            f'original const diagnostic or drift control failed: {destination}')
    lock = destination.parent / f'.{destination.name}.compactc.lock'
    require(not destination.exists() and
            not list(destination.parent.glob(f'.{destination.name}.compactc-stage-*')) and
            all(child == lock and child.is_file() and not child.is_symlink() and
                child.stat().st_size == 0 for child in destination.parent.iterdir()),
            f'refused const published output: {destination}')
    return diagnostic


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'compiler-recipe', 'ts-runtime', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
    parser.add_argument('--cargo-target', type=Path)
    args = parser.parse_args()
    source = source_state()
    signature = subprocess.check_output(['git', 'log', '-1', '--format=%G?%n%B'],
                                        cwd=ROOT, text=True)
    require(signature.startswith('G\n') and 'Signed-off-by:' in signature and
            'Assisted-by: AI' in signature, 'signed DCO+Assisted head required')
    compiler = args.compiler.resolve(strict=True)
    scheme = compiler.with_name('compactc-scheme').resolve(strict=True)
    runtime = args.ts_runtime.resolve(strict=True)
    output = args.output.resolve()
    require(not args.output.is_symlink() and not output.exists(), 'output must be absent')
    source_package = json.loads((ROOT / 'runtime/package.json').read_text())
    installed_package = json.loads((runtime / 'package.json').read_text())
    require(all(source_package[k] == installed_package[k]
                for k in ('name', 'version', 'dependencies')),
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
        contract = ROOT / FAMILY / case / f'const_{case}.compact'
        for mode in ('typescript', 'both'):
            destination = output / 'generated' / case / mode
            command = [str(compiler), '--target', 'ts']
            if mode == 'both':
                command += ['--target', 'rust', '--rust-runtime-root', str(ROOT),
                            '--rust-require-recording']
            command += ['--feature-zkir-v3', '--skip-zk', '--compact-path',
                        str(ROOT / 'compiler'), str(contract), str(destination)]
            run(command, output / f'{case}-{mode}-compile.log', env)
            proof = case == 'initializer_side_effects'
            check_manifest(destination, require_zkir=proof)
            info = destination / 'compiler/contract-info.json'
            circuits = json.loads(info.read_text())['circuits']
            require(len(circuits) == 1 and circuits[0]['name'] == f'const_{case}' and
                    circuits[0]['proof'] is proof and circuits[0]['pure'] is not proof,
                    f'const capability changed: {case}')
            if proof:
                zkir = destination / f'zkir/const_{case}.zkir'
                require(zkir.is_file(), f'proof-required const ZKIR missing: {case}')
            else:
                require(not (destination / 'zkir').exists(),
                        f'proof-free const emitted ZKIR: {case}')
            target = destination / 'contract'
            runtime_log = output / f'{case}-{mode}-runtime.log'
            actual = run(['node', str(ADAPTER), str(target / 'index.js'), case],
                         runtime_log, env)
            require(actual.strip() == f'{case}: original const result passed',
                    f'const TypeScript result changed: {case}/{mode}')
            row = {'typescript_sha256': sha256(target / 'index.js'),
                   'contract_info_sha256': sha256(info),
                   'manifest_sha256': sha256(destination / 'compiler/contract-manifest.json'),
                   'runtime_log_sha256': sha256(runtime_log)}
            if proof:
                row['zkir_sha256'] = sha256(zkir)
            if mode == 'both':
                row['rust_sha256'] = sha256(target / 'lib.rs')
                row['rust_ir_sha256'] = sha256(target / 'compact-rust-ir.json')
            generated[case][mode] = row
        require(all(generated[case]['typescript'][key] == generated[case]['both'][key]
                    for key in ('typescript_sha256', 'contract_info_sha256',
                                'runtime_log_sha256')),
                f'paired target altered TypeScript: {case}')
        if case == 'initializer_side_effects':
            require(generated[case]['typescript']['zkir_sha256'] ==
                    generated[case]['both']['zkir_sha256'],
                    'paired target altered const ZKIR')
    refusals = {}
    for case in FAIL:
        refusals[case] = {}
        contract = ROOT / FAMILY / case / f'const_{case}.compact'
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
            diagnostic = refused(result, destination, patterns[case])
            log = output / 'diagnostics' / mode / f'{case}.log'
            log.parent.mkdir(parents=True, exist_ok=True)
            log.write_text(diagnostic)
            refusals[case][mode] = {'original_regex': patterns[case],
                                    'diagnostic_sha256': sha256(log),
                                    'exit_code': result.returncode,
                                    'output_absent': True,
                                    'drift_control_refused': True}
        require(refusals[case]['typescript']['diagnostic_sha256'] ==
                refusals[case]['both']['diagnostic_sha256'],
                f'const target diagnostic drift: {case}')
    generated_before = generated_files(output / 'generated')
    consumer = output / 'rust-consumer'
    (consumer / 'tests').mkdir(parents=True)
    shutil.copyfile(CONSUMER, consumer / 'tests/const_original.rs')
    shutil.copyfile(ROOT / 'Cargo.lock', consumer / 'Cargo.lock')
    dependencies = []
    generated_paths = []
    for case in PASS:
        target = output / 'generated' / case / 'both/contract'
        generated_paths.append(target)
        package = tomllib.loads((target / 'Cargo.toml').read_text())['package']['name']
        dependencies.append(f'{case} = {{ package = {json.dumps(package)}, '
                            f'path = {json.dumps(str(target))} }}')
    manifest = consumer / 'Cargo.toml'
    manifest.write_text('[workspace]\n[package]\nname = "ledger9-keyword-const-consumer"\n'
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
               *(path / 'Cargo.toml' for path in generated_paths)}
    validate_consumer_lock(metadata, consumer / 'Cargo.lock', allowed, pinned)
    run(['cargo', '+1.99.0', 'test', '--locked', '--offline',
         '--manifest-path', str(manifest), '--test', 'const_original'],
        output / 'cargo-test.log', cargo_env)
    rust_log = (output / 'cargo-test.log').read_text()
    require('test result: ok. 2 passed; 0 failed;' in rust_log and
            'test original_pure_const_results ... ok' in rust_log and
            'test original_const_initializer_order_and_recorded_replay ... ok' in rust_log,
            'const Rust consumer assertions did not pass')
    require(source_state() == source and runtime_identity(runtime) == runtime_before and
            compiler_recipe_identity(args.compiler_recipe, source, compiler, scheme) == recipe and
            original_inventory() == (originals, patterns) and
            generated_files(output / 'generated') == generated_before and
            all(sha256(ROOT / name) == digest for name, digest in local_sources.items()),
            'source, compiler, runtime, originals or generated artifacts changed during gate')
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
               'case_count': 23, 'positive_cases': 11, 'refusal_cases': 12,
               'proof_execution': False, 'funded_application': False,
               'result': 'all pinned const diagnostics and original TS/Rust results passed',
               'limitations': ['native Linux qualification required',
                               'initializer_side_effects proof=true: fresh proof and funded strict application remain separate',
                               'original Vitest host harness was not run; pinned assertions replayed against generated APIs']}
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': 'passed', 'head': source['head'], 'cases': 23,
                      'output': str(output)}))


if __name__ == '__main__':
    main()
