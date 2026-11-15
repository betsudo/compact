#!/usr/bin/env python3

# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0

"""Replay all pinned return originals through exact-head generated TS and Rust."""

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
FAMILY = Path('test-contracts/keywords/return')
MANIFEST = ROOT / FAMILY / 'original-source-sha256.json'
ADAPTER = ROOT / FAMILY / 'check-original-results.mjs'
PASS = ('if_else_circuit_call', 'if_else_either', 'lambda_default')
FAIL = ('missing_else_path', 'short_if_ternary',
        'unreachable_after_return_assert', 'unreachable_after_return_assignment',
        'unreachable_after_return_block', 'unreachable_after_return_const',
        'unreachable_after_return_expression', 'unreachable_after_return_for',
        'unreachable_after_return_if', 'unreachable_after_return_return',
        'unreachable_after_returning_block', 'unreachable_in_lambda_branch',
        'unreachable_statement')
PATTERN = re.compile(r'expectedError:\s*/((?:\\.|[^/])*)/')
ORIGINAL_EXPECTATIONS = {
    'if_else_circuit_call': ('return_if_else_circuit_call(ctx, 2n)',
                             'return_if_else_circuit_call(ctx, 0n)',
                             ').toBe(1n);'),
    'if_else_either': ('return_if_else_either(ctx, 2n)',
                       'return_if_else_either(ctx, 0n)',
                       'is_left: true', 'is_left: false',
                       'left: 1n', 'right: true'),
    'lambda_default': ('return_lambda_default(ctx, 5n)',
                       'expect(result).toBe(0n);'),
}


def original_inventory():
    record = json.loads(MANIFEST.read_text())
    expected = {str(FAMILY / case / name) for case in PASS
                for name in (f'return_{case}.compact', 'compile.pass.test.ts',
                             'runtime.pass.test.ts')} | {
                str(FAMILY / case / name) for case in FAIL
                for name in (f'return_{case}.compact', 'compile.fail.test.ts')}
    upstream = set(subprocess.check_output(
        ['git', 'ls-tree', '-r', '--name-only', PINNED, str(FAMILY)],
        cwd=ROOT, text=True).splitlines())
    files = record.get('files')
    require(record.get('upstream_commit') == PINNED and
            isinstance(files, dict) and set(files) == expected == upstream and
            len(files) == 35, 'return-keyword pinned roster changed')
    for name, digest in files.items():
        path = ROOT / name
        original = subprocess.check_output(['git', 'show', f'{PINNED}:{name}'], cwd=ROOT)
        require(path.is_file() and not path.is_symlink() and
                re.fullmatch('[0-9a-f]{64}', digest) is not None and
                sha256(path) == digest == hashlib.sha256(original).hexdigest(),
                f'return-keyword original changed: {name}')
    patterns = {}
    for case in FAIL:
        text = (ROOT / FAMILY / case / 'compile.fail.test.ts').read_text()
        matches = PATTERN.findall(text)
        require(len(matches) == 1, f'one original diagnostic required: {case}')
        patterns[case] = matches[0]
    for case, snippets in ORIGINAL_EXPECTATIONS.items():
        text = (ROOT / FAMILY / case / 'runtime.pass.test.ts').read_text()
        require(all(snippet in text for snippet in snippets),
                f'original return runtime assertion changed: {case}')
    return files, patterns


def generated_files(root):
    files = {}
    for path in sorted(root.rglob('*')):
        require(not path.is_symlink(), f'generated symlink: {path}')
        if path.is_file():
            files[path.relative_to(root).as_posix()] = sha256(path)
    require(files, 'no generated artifacts')
    return files


def pinned_registry():
    rows = tomllib.loads((ROOT / 'Cargo.lock').read_text())['package']
    return {(row['name'], row['version'], row['source']): row.get('checksum')
            for row in rows if 'source' in row}


def check_capability(info, case):
    circuits = info.get('circuits')
    require(isinstance(circuits, list) and len(circuits) == 1 and
            circuits[0].get('name') == f'return_{case}' and
            circuits[0].get('pure') is True and circuits[0].get('proof') is False,
            f'original return circuit capability changed: {case}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'compiler-recipe', 'ts-runtime', 'cargo-target', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
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
    cargo_target = args.cargo_target.resolve(strict=True)
    require(cargo_target.is_dir() and not args.cargo_target.is_symlink(),
            'shared Cargo target must be an ordinary existing directory')
    output = args.output.resolve()
    require(not args.output.is_symlink() and not output.exists(),
            'return gate output must be absent')
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
    consumers = {case: ROOT / FAMILY / case / 'consumer-original.rs' for case in PASS}
    local_paths = (MANIFEST, ADAPTER, Path(__file__), *consumers.values())
    local_sources = {str(path.relative_to(ROOT)): sha256(path) for path in local_paths}
    output.mkdir(parents=True)
    link = output / 'node_modules/@midnight-ntwrk/compact-runtime'
    link.parent.mkdir(parents=True)
    link.symlink_to(runtime, target_is_directory=True)
    (output / 'package.json').write_text('{"type":"module"}\n')
    env = dict(os.environ, COMPACTC_SCHEME=str(scheme), RUSTUP_TOOLCHAIN='1.99.0',
               PYTHONDONTWRITEBYTECODE='1')
    generated = {}
    for case in PASS:
        generated[case] = {}
        original = ROOT / FAMILY / case / f'return_{case}.compact'
        for mode in ('typescript', 'both'):
            destination = output / 'generated' / case / mode
            command = [str(compiler), '--target', 'ts']
            if mode == 'both':
                command += ['--target', 'rust', '--rust-runtime-root', str(ROOT),
                            '--rust-require-recording']
            command += ['--feature-zkir-v3', '--skip-zk', '--compact-path',
                        str(ROOT / 'compiler'), str(original), str(destination)]
            run(command, output / f'{case}-{mode}-compile.log', env)
            check_manifest(destination, require_zkir=False)
            require(not (destination / 'zkir').exists(),
                    f'proof-free return generated ZKIR: {case}/{mode}')
            info = destination / 'compiler/contract-info.json'
            check_capability(json.loads(info.read_text()), case)
            contract = destination / 'contract'
            runtime_log = output / f'{case}-{mode}-runtime.log'
            actual = run(['node', str(ADAPTER), str(contract / 'index.js'), case],
                         runtime_log, env)
            require(actual.strip() == f'{case}: original return result passed',
                    f'original return TS result changed: {case}/{mode}')
            row = {'typescript_sha256': sha256(contract / 'index.js'),
                   'contract_info_sha256': sha256(info),
                   'manifest_sha256': sha256(destination / 'compiler/contract-manifest.json'),
                   'runtime_log_sha256': sha256(runtime_log)}
            if mode == 'both':
                row['rust_sha256'] = sha256(contract / 'lib.rs')
                row['rust_ir_sha256'] = sha256(contract / 'compact-rust-ir.json')
            generated[case][mode] = row
        require(all(generated[case]['typescript'][key] == generated[case]['both'][key]
                    for key in ('typescript_sha256', 'contract_info_sha256',
                                'runtime_log_sha256')),
                f'paired return target changed TS result or source: {case}')
    refusals = {}
    for case in FAIL:
        refusals[case] = {}
        original = ROOT / FAMILY / case / f'return_{case}.compact'
        for mode in ('typescript', 'both'):
            destination = output / 'refused' / case / mode / 'product'
            destination.parent.mkdir(parents=True)
            command = [str(compiler), '--target', 'ts']
            if mode == 'both':
                command += ['--target', 'rust', '--rust-runtime-root', str(ROOT)]
            command += ['--feature-zkir-v3', '--skip-zk', '--compact-path',
                        str(ROOT / 'compiler'), str(original), str(destination)]
            result = subprocess.run(command, cwd=ROOT, env=env,
                                    capture_output=True, text=True)
            diagnostic = result.stdout + result.stderr
            pattern = patterns[case]
            require(result.returncode != 0 and result.stdout == '' and
                    re.search(pattern, diagnostic) is not None,
                    f'original return diagnostic changed: {case}/{mode}')
            lock = destination.parent / f'.{destination.name}.compactc.lock'
            require(not destination.exists() and
                    not list(destination.parent.glob(f'.{destination.name}.compactc-stage-*')) and
                    all(child == lock and child.is_file() and not child.is_symlink() and
                        child.stat().st_size == 0 for child in destination.parent.iterdir()),
                    f'return refusal published output: {case}/{mode}')
            log = output / 'diagnostics' / mode / f'{case}.log'
            log.parent.mkdir(parents=True, exist_ok=True)
            log.write_text(diagnostic)
            refusals[case][mode] = {'original_regex': pattern,
                                    'diagnostic_sha256': sha256(log),
                                    'exit_code': result.returncode,
                                    'output_absent': True}
        require(refusals[case]['typescript']['diagnostic_sha256'] ==
                refusals[case]['both']['diagnostic_sha256'],
                f'return target diagnostic drift: {case}')
    generated_before = generated_files(output / 'generated')
    cargo_env = dict(env, CARGO_TARGET_DIR=str(cargo_target), CARGO_INCREMENTAL='0',
                     CARGO_PROFILE_DEV_DEBUG='0')
    pinned = pinned_registry()
    consumer = output / 'rust-consumer'
    (consumer / 'tests').mkdir(parents=True)
    shutil.copyfile(ROOT / 'Cargo.lock', consumer / 'Cargo.lock')
    dependencies = []
    paths = []
    for case in PASS:
        contract = output / 'generated' / case / 'both/contract'
        paths.append(contract)
        package = tomllib.loads((contract / 'Cargo.toml').read_text())['package']['name']
        dependencies.append(f'{package} = {{ path = {json.dumps(str(contract))} }}')
        shutil.copyfile(consumers[case], consumer / 'tests' / f'{case}.rs')
    manifest = consumer / 'Cargo.toml'
    manifest.write_text('[workspace]\n[package]\nname = "ledger9-keyword-return-consumer"\n'
                        'version = "0.1.0"\nedition = "2024"\n[dependencies]\n' +
                        '\n'.join(dependencies) + '\n')
    metadata = json.loads(run(['cargo', '+1.99.0', 'metadata', '--offline',
                               '--format-version', '1', '--manifest-path', str(manifest)],
                              output / 'cargo-metadata.log', cargo_env))
    allowed = {manifest, ROOT / 'runtime-rs/Cargo.toml',
               ROOT / 'runtime-rs-macros/Cargo.toml',
               *(path / 'Cargo.toml' for path in paths)}
    validate_consumer_lock(metadata, consumer / 'Cargo.lock', allowed, pinned)
    run(['cargo', '+1.99.0', 'test', '--locked', '--offline',
         '--manifest-path', str(manifest), '--tests'],
        output / 'cargo-test.log', cargo_env)
    rust_log = (output / 'cargo-test.log').read_text()
    require(rust_log.count('test result: ok. 1 passed; 0 failed;') == 3 and
            'test original_branches ... ok' in rust_log and
            'test original_lambda_scope ... ok' in rust_log,
            'three original generated Rust return consumers did not pass')
    clippy = {}
    generated_locks = {}
    for case in PASS:
        target = output / 'generated' / case / 'both/contract'
        target_manifest = target / 'Cargo.toml'
        # A generated crate is its own Cargo root. Seed the workspace's
        # released dependency cohort before resolving its local package.
        shutil.copyfile(ROOT / 'Cargo.lock', target / 'Cargo.lock')
        metadata = json.loads(run(['cargo', '+1.99.0', 'metadata', '--offline',
                                   '--format-version', '1', '--manifest-path',
                                   str(target_manifest)],
                                  output / f'{case}-cargo-metadata.log', cargo_env))
        allowed = {target_manifest, ROOT / 'runtime-rs/Cargo.toml',
                   ROOT / 'runtime-rs-macros/Cargo.toml'}
        validate_consumer_lock(metadata, target / 'Cargo.lock', allowed, pinned)
        run(['cargo', '+1.99.0', 'clippy', '--locked', '--offline', '--all-targets',
             '--manifest-path', str(target_manifest), '--', '-D', 'warnings'],
            output / f'{case}-clippy.log', cargo_env)
        clippy[case] = sha256(output / f'{case}-clippy.log')
        generated_locks[case] = sha256(target / 'Cargo.lock')
    after = generated_files(output / 'generated')
    lock_names = {f'{case}/both/contract/Cargo.lock' for case in PASS}
    require(set(after) == set(generated_before) | lock_names and
            all(after[name] == digest for name, digest in generated_before.items()) and
            all(after[f'{case}/both/contract/Cargo.lock'] == digest
                for case, digest in generated_locks.items()),
            'Clippy changed compiler-generated return artifacts')
    require(source_state() == source and runtime_identity(runtime) == runtime_before and
            compiler_recipe_identity(args.compiler_recipe, source, compiler, scheme) == recipe and
            original_inventory() == (originals, patterns) and
            all(sha256(ROOT / name) == digest for name, digest in local_sources.items()) and
            generated_files(output / 'generated') == after,
            'return source, compiler, runtime or artifacts changed')
    receipt = {**source, 'status': 'passed', 'source_clean': True,
               'platform': os.uname().sysname + ' ' + os.uname().machine,
               'upstream_commit': PINNED, 'original_files': originals,
               'local_sources': local_sources, 'generated': generated,
               'generated_files_sha256': after, 'compiler_generated_files_sha256': generated_before,
               'generated_cargo_locks_sha256': generated_locks,
               'refusals': refusals, 'compiler_recipe': recipe,
               'compiler_sha256': sha256(compiler), 'scheme_sha256': sha256(scheme),
               'runtime': runtime_before, 'runtime_recipe': runtime_recipe,
               'rust_consumer_lock_sha256': sha256(consumer / 'Cargo.lock'),
               'rust_test_log_sha256': sha256(output / 'cargo-test.log'),
               'clippy_logs_sha256': clippy,
               'case_count': 16, 'positive_cases': 3, 'refusal_cases': 13,
               'proof_execution': False, 'funded_application': False,
               'result': 'all sixteen pinned return results and diagnostics passed',
               'limitations': ['native Linux qualification required',
                               'all three accepted return circuits are pure and proof-free',
                               'original Vitest host harness not run; pinned assertions and refusals replayed']}
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': 'passed', 'head': source['head'], 'cases': 16,
                      'output': str(output)}))


if __name__ == '__main__':
    main()
