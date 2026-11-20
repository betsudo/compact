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

"""Qualify pinned if originals on generated TypeScript and Rust targets."""

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
FAMILY = Path('test-contracts/keywords/if')
MANIFEST = ROOT / FAMILY / 'original-source-sha256.json'
ADAPTER = ROOT / FAMILY / 'check-original-results.mjs'
CONSUMER = ROOT / FAMILY / 'empty_else_branch/consumer-original.rs'
BRANCH_CONSUMER = ROOT / FAMILY / 'branches_write_ledger/consumer-original.rs'
CASES = ('empty_else_branch', 'branches_write_ledger')
RUST_REFUSAL = 'Rust backend does not yet support this nested ledger query: Set.insert'
RUST_REFUSAL_LOCATION = 'line 38 char 17:'
ORIGINAL_EXPECTATIONS = {
    'empty_else_branch': ('if_empty_else_branch(ctx, true)',
                          'if_empty_else_branch(ctx, false)',
                          ').toEqual(1n);', ').toEqual(0n);'),
    'branches_write_ledger': ('if_branches_write_ledger(',
                              'expect(first.result).toBe(true);',
                              'first.context',
                              'expect(second.result).toBe(false);'),
}


def original_inventory():
    record = json.loads(MANIFEST.read_text())
    expected = {str(FAMILY / case / name) for case in CASES
                for name in (f'if_{case}.compact', 'compile.pass.test.ts',
                             'runtime.pass.test.ts')}
    upstream = set(subprocess.check_output(
        ['git', 'ls-tree', '-r', '--name-only', PINNED, str(FAMILY)],
        cwd=ROOT, text=True).splitlines())
    files = record.get('files')
    require(record.get('upstream_commit') == PINNED and
            isinstance(files, dict) and set(files) == expected == upstream and
            len(files) == 6, 'if-keyword pinned roster changed')
    for name, digest in files.items():
        path = ROOT / name
        original = subprocess.check_output(['git', 'show', f'{PINNED}:{name}'], cwd=ROOT)
        require(path.is_file() and not path.is_symlink() and
                re.fullmatch('[0-9a-f]{64}', digest) is not None and
                sha256(path) == digest == hashlib.sha256(original).hexdigest(),
                f'if-keyword original changed: {name}')
    for case, snippets in ORIGINAL_EXPECTATIONS.items():
        text = (ROOT / FAMILY / case / 'runtime.pass.test.ts').read_text()
        require(all(snippet in text for snippet in snippets),
                f'original if runtime assertion changed: {case}')
    return files


def generated_files(root):
    files = {}
    for path in sorted(root.rglob('*')):
        require(not path.is_symlink(), f'generated symlink: {path}')
        if path.is_file():
            files[path.relative_to(root).as_posix()] = sha256(path)
    require(files, 'no generated artifacts')
    return files


def check_capability(info, case):
    circuits = info.get('circuits')
    require(isinstance(circuits, list) and len(circuits) == 1 and
            circuits[0].get('name') == f'if_{case}' and
            circuits[0].get('pure') is False and circuits[0].get('proof') is True,
            f'if original capability changed: {case}')


def check_rust_refusal(result, destination, location=RUST_REFUSAL_LOCATION):
    diagnostic = result.stdout + result.stderr
    require(result.returncode != 0 and result.stdout == '' and
            location in diagnostic and RUST_REFUSAL in diagnostic,
            'nested Set.insert diagnostic changed or Rust accepted without qualification')
    lock = destination.parent / f'.{destination.name}.compactc.lock'
    require(not destination.exists() and
            not list(destination.parent.glob(f'.{destination.name}.compactc-stage-*')) and
            all(child == lock and child.is_file() and not child.is_symlink() and
                child.stat().st_size == 0 for child in destination.parent.iterdir()),
            'nested Set.insert refusal published output')
    return diagnostic


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
            'if gate output must be absent')
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
    originals = original_inventory()
    local_paths = (MANIFEST, ADAPTER, CONSUMER, BRANCH_CONSUMER, Path(__file__))
    local_sources = {str(path.relative_to(ROOT)): sha256(path) for path in local_paths}
    output.mkdir(parents=True)
    link = output / 'node_modules/@midnight-ntwrk/compact-runtime'
    link.parent.mkdir(parents=True)
    link.symlink_to(runtime, target_is_directory=True)
    (output / 'package.json').write_text('{"type":"module"}\n')
    env = dict(os.environ, COMPACTC_SCHEME=str(scheme), RUSTUP_TOOLCHAIN='1.99.0',
               PYTHONDONTWRITEBYTECODE='1')
    generated = {}
    for case in CASES:
        generated[case] = {}
        original = ROOT / FAMILY / case / f'if_{case}.compact'
        for mode in ('typescript', 'both'):
            destination = output / 'generated' / case / mode
            command = [str(compiler), '--target', 'ts']
            if mode == 'both':
                command += ['--target', 'rust', '--rust-runtime-root', str(ROOT),
                            '--rust-require-recording']
            command += ['--feature-zkir-v3', '--skip-zk', '--compact-path',
                        str(ROOT / 'compiler'), str(original), str(destination)]
            run(command, output / f'{case}-{mode}-compile.log', env)
            check_manifest(destination, require_zkir=True)
            info = destination / 'compiler/contract-info.json'
            check_capability(json.loads(info.read_text()), case)
            contract = destination / 'contract'
            runtime_log = output / f'{case}-{mode}-runtime.log'
            actual = run(['node', str(ADAPTER), str(contract / 'index.js'), case],
                         runtime_log, env)
            require(actual.strip() == f'{case}: original if result passed',
                    f'original if TS result changed: {case}/{mode}')
            row = {'typescript_sha256': sha256(contract / 'index.js'),
                   'contract_info_sha256': sha256(info),
                   'manifest_sha256': sha256(destination / 'compiler/contract-manifest.json'),
                   'runtime_log_sha256': sha256(runtime_log),
                   'zkir_sha256': sha256(destination / f'zkir/if_{case}.zkir')}
            if mode == 'both':
                row['rust_sha256'] = sha256(contract / 'lib.rs')
                row['rust_ir_sha256'] = sha256(contract / 'compact-rust-ir.json')
            generated[case][mode] = row
    for case, modes in generated.items():
        require(all(modes['typescript'][key] == modes['both'][key]
                    for key in ('typescript_sha256', 'contract_info_sha256',
                                'runtime_log_sha256', 'zkir_sha256')),
                f'paired {case} target changed TS result or raw ZKIR')
    branch_source = (ROOT / FAMILY /
                     'branches_write_ledger/if_branches_write_ledger.compact').read_text()
    mutations = {
        'ledger_argument': branch_source.replace(
            'set.insert(b as Field)', 'set.insert(set.size() as Field)', 1),
        'other_action': branch_source.replace(
            'set.insert(b as Field);', 'set.remove(b as Field);', 1),
        'ledger_condition': branch_source.replace(
            'return true ?', 'return set.isEmpty() ?', 1),
    }
    refusals = {}
    for name, mutated in mutations.items():
        require(mutated != branch_source, f'empty if refusal mutation: {name}')
        folder = output / 'refused' / name
        folder.mkdir(parents=True)
        variant = folder / 'if_branches_write_ledger.compact'
        variant.write_text(mutated)
        ts = folder / 'typescript'
        run([str(compiler), '--target', 'ts', '--feature-zkir-v3', '--skip-zk',
             '--compact-path', str(ROOT / 'compiler'), str(variant), str(ts)],
            folder / 'typescript.log', env)
        check_manifest(ts, require_zkir=True)
        refused = folder / 'paired' / 'product'
        refused.parent.mkdir()
        command = [str(compiler), '--target', 'ts', '--target', 'rust',
                   '--rust-runtime-root', str(ROOT), '--rust-require-recording',
                   '--feature-zkir-v3', '--skip-zk', '--compact-path',
                   str(ROOT / 'compiler'), str(variant), str(refused)]
        result = subprocess.run(command, cwd=ROOT, env=env,
                                capture_output=True, text=True)
        diagnostic = check_rust_refusal(
            result, refused, 'line 38 char 26:' if name == 'ledger_condition'
            else RUST_REFUSAL_LOCATION)
        refusal_log = folder / 'paired.log'
        refusal_log.write_text(diagnostic)
        refusals[name] = {'source_sha256': sha256(variant),
                          'typescript_manifest_sha256': sha256(
                              ts / 'compiler/contract-manifest.json'),
                          'diagnostic_sha256': sha256(refusal_log),
                          'output_absent': True}
    before = generated_files(output / 'generated')
    targets = {case: output / 'generated' / case / 'both/contract'
               for case in CASES}
    consumer = output / 'rust-consumer'
    (consumer / 'tests').mkdir(parents=True)
    shutil.copyfile(ROOT / 'Cargo.lock', consumer / 'Cargo.lock')
    shutil.copyfile(CONSUMER, consumer / 'tests/if_empty.rs')
    shutil.copyfile(BRANCH_CONSUMER, consumer / 'tests/if_branches.rs')
    manifest = consumer / 'Cargo.toml'
    dependencies = []
    for case, target in targets.items():
        package = tomllib.loads((target / 'Cargo.toml').read_text())['package']['name']
        crate_name = 'if_branches_write_ledger' if case == 'branches_write_ledger' else case
        dependencies.append(f'{crate_name} = {{ package = {json.dumps(package)}, '
                            f'path = {json.dumps(str(target))} }}')
    manifest.write_text('[workspace]\n[package]\nname = "ledger9-if-original-consumer"\n'
                        'version = "0.1.0"\nedition = "2024"\n[dependencies]\n'
                        + '\n'.join(dependencies) + '\n'
                        'midnight-onchain-vm = "=4.0.1"\n'
                        'midnight-serialize = "=1.2.1"\n')
    cargo_env = dict(env, CARGO_TARGET_DIR=str(cargo_target), CARGO_INCREMENTAL='0',
                     CARGO_PROFILE_DEV_DEBUG='0')
    metadata = json.loads(run(['cargo', '+1.99.0', 'metadata', '--offline',
                               '--format-version', '1', '--manifest-path', str(manifest)],
                              output / 'cargo-metadata.log', cargo_env))
    root_lock = tomllib.loads((ROOT / 'Cargo.lock').read_text())['package']
    pinned = {(row['name'], row['version'], row['source']): row.get('checksum')
              for row in root_lock if 'source' in row}
    allowed = {manifest, *(target / 'Cargo.toml' for target in targets.values()),
               ROOT / 'runtime-rs/Cargo.toml', ROOT / 'runtime-rs-macros/Cargo.toml'}
    validate_consumer_lock(metadata, consumer / 'Cargo.lock', allowed, pinned)
    run(['cargo', '+1.99.0', 'test', '--locked', '--offline',
         '--manifest-path', str(manifest), '--tests'],
        output / 'cargo-test.log', cargo_env)
    rust_log = (output / 'cargo-test.log').read_text()
    require(rust_log.count('test result: ok. 1 passed; 0 failed;') >= 2 and
            'test original_true_and_false_results_with_recorded_replay ... ok' in rust_log and
            'test original_constructor_and_sequential_calls_with_recorded_replay ... ok' in rust_log,
            'generated if original Rust assertions did not pass')
    for case, target in targets.items():
        shutil.copyfile(ROOT / 'Cargo.lock', target / 'Cargo.lock')
        metadata = json.loads(run(['cargo', '+1.99.0', 'metadata', '--offline',
                                   '--format-version', '1', '--manifest-path',
                                   str(target / 'Cargo.toml')],
                                  output / f'{case}-generated-cargo-metadata.log', cargo_env))
        validate_consumer_lock(metadata, target / 'Cargo.lock',
                               {target / 'Cargo.toml', ROOT / 'runtime-rs/Cargo.toml',
                                ROOT / 'runtime-rs-macros/Cargo.toml'}, pinned)
        # The pinned source deliberately has identical Set.insert branches.
        # Keep every other generated-crate warning denied without rewriting
        # its conditional or changing the manifest for unrelated contracts.
        clippy_exception = (['-A', 'clippy::if_same_then_else']
                            if case == 'branches_write_ledger' else [])
        run(['cargo', '+1.99.0', 'clippy', '--locked', '--offline', '--all-targets',
             '--manifest-path', str(target / 'Cargo.toml'), '--', '-D', 'warnings',
             *clippy_exception],
            output / f'{case}-generated-clippy.log', cargo_env)
    after = generated_files(output / 'generated')
    lock_names = {f'{case}/both/contract/Cargo.lock' for case in CASES}
    require(set(after) == set(before) | lock_names and
            all(after[name] == digest for name, digest in before.items()),
            'Rust consumer or Clippy modified compiler-generated if artifacts')
    require(source_state() == source and runtime_identity(runtime) == runtime_before and
            compiler_recipe_identity(args.compiler_recipe, source, compiler, scheme) == recipe and
            original_inventory() == originals and
            generated_files(output / 'generated') == after and
            all(sha256(ROOT / name) == digest for name, digest in local_sources.items()),
            'if source, compiler, runtime or generated artifact changed')
    receipt = {**source, 'status': 'passed',
               'source_clean': True, 'platform': os.uname().sysname + ' ' + os.uname().machine,
               'upstream_commit': PINNED, 'original_files': originals,
               'local_sources': local_sources, 'generated': generated,
               'compiler_generated_files_sha256': before,
               'generated_files_sha256': after,
               'generated_cargo_lock_sha256': {name: after[name] for name in lock_names},
               'bounded_refusals': refusals,
               'compiler_recipe': recipe, 'compiler_sha256': sha256(compiler),
               'scheme_sha256': sha256(scheme), 'runtime': runtime_before,
               'runtime_recipe': runtime_recipe,
               'rust_consumer_lock_sha256': sha256(consumer / 'Cargo.lock'),
               'rust_test_log_sha256': sha256(output / 'cargo-test.log'),
               'clippy_log_sha256': {case: sha256(output / f'{case}-generated-clippy.log')
                                     for case in CASES},
               'clippy_source_exception': {'branches_write_ledger':
                                           'clippy::if_same_then_else (pinned identical Set.insert branches)'},
               'case_count': 2, 'rust_accepted_cases': 2, 'rust_refusal_cases': 0,
               'proof_execution': False, 'funded_application': False,
               'limitations': ['native Linux qualification required',
                               'fresh proof and funded strict application remain separate',
                               'selected Set insertion excludes effectful argument and ledger-read condition shapes',
                               'original Vitest host harness not run; pinned assertions replayed']}
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': receipt['status'], 'head': source['head'],
                      'cases': 2, 'output': str(output)}))


if __name__ == '__main__':
    main()
