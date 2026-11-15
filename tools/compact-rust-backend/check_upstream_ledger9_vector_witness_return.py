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

"""Qualify the intact upstream Vector witness-return semantics on generated TS/Rust."""

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tomllib

from check_compactc_target import check_manifest
from check_functional_source_sdk import cargo_configuration
from check_release_packages import compatibility_record
from check_unit_mutation_initializers import validate_consumer_lock
from check_upstream_ledger9_vector_casts import (
    compiler_recipe_identity, require_unchanged, run, runtime_identity,
    runtime_recipe_identity, sha256, source_state,
)


ROOT = Path(__file__).resolve().parents[2]
FAMILY = ROOT / 'test-contracts/primitives/vector/witness_return'
ORIGINAL = FAMILY / 'semantics'
PINNED = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'
CIRCUIT = 'vector_witness_return_semantics'
ORIGINAL_FILES = ('compile.pass.test.ts', 'runtime.pass.test.ts',
                  'vector_witness_return_semantics.compact')
ORIGINAL_HASHES = {
    'compile.pass.test.ts': '16bd80193bfe6493846325f8b1a59f9929e4059076d5a038b3fc1770b78b23a0',
    'runtime.pass.test.ts': 'a1f7f116289829ca4c24a396720811f6921d59e163cc14eb010e7fd0143c785f',
    'vector_witness_return_semantics.compact': '4305f65f6a51fc0da877da3cad5c54b5b5cb8b5c8975dad04a2aa711974bcf7c',
}


def require(value: bool, message: str) -> None:
    if not value:
        raise RuntimeError(message)


def original_inventory() -> dict[str, str]:
    inventory = {}
    for filename in ORIGINAL_FILES:
        path = ORIGINAL / filename
        relative = path.relative_to(ROOT).as_posix()
        require(path.is_file() and not path.is_symlink() and
                sha256(path) == ORIGINAL_HASHES[filename],
                f'pinned Vector witness-return original changed: {relative}')
        inventory[relative] = ORIGINAL_HASHES[filename]
    return inventory


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'compiler-recipe', 'ts-runtime', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
    parser.add_argument('--cargo-target', type=Path)
    args = parser.parse_args()
    require(__debug__, 'generated manifest validation requires Python without -O/-OO')
    source = source_state()
    require(subprocess.check_output(['git', 'log', '-1', '--format=%G?'], cwd=ROOT,
                                    text=True).strip() == 'G', 'source commit must be signed')
    require('Signed-off-by:' in subprocess.check_output(
        ['git', 'log', '-1', '--format=%B'], cwd=ROOT, text=True),
        'source commit requires DCO sign-off')
    compiler = args.compiler.resolve(strict=True)
    scheme = compiler.with_name('compactc-scheme').resolve(strict=True)
    runtime = args.ts_runtime.resolve(strict=True)
    output = args.output.resolve()
    require(not args.output.exists() and not args.output.is_symlink(), 'output must be absent')
    require(not os.environ.get('COMPACTC_SCHEME') or
            Path(os.environ['COMPACTC_SCHEME']).resolve(strict=True) == scheme,
            'compiler scheme override differs from package')
    configuration = cargo_configuration(dict(os.environ))
    require(os.environ.get('RUSTUP_TOOLCHAIN', '1.99.0') == '1.99.0',
            'Rust 1.99.0 required')
    matrix = compatibility_record()
    require(matrix['ledger_version'] == 'ledger-9.1.0.1',
            'witness-return SDK matrix differs from Ledger 9.1.0.1')
    package = json.loads((runtime / 'package.json').read_text())
    source_package = json.loads((ROOT / 'runtime/package.json').read_text())
    require(all(package[key] == source_package[key]
                for key in ('name', 'version', 'dependencies')),
            'TypeScript runtime differs from source package')
    nix = shutil.which('nix')
    require(nix is not None, 'Nix required for exact-head runtime binding')
    runtime_recipe = runtime_recipe_identity(runtime, Path(nix).resolve(strict=True))
    runtime_before = runtime_identity(runtime)
    recipe = compiler_recipe_identity(args.compiler_recipe, source, compiler, scheme)
    originals = original_inventory()

    output.mkdir(parents=True)
    runtime_link = output / 'node_modules/@midnight-ntwrk/compact-runtime'
    runtime_link.parent.mkdir(parents=True)
    runtime_link.symlink_to(runtime, target_is_directory=True)
    environment = dict(os.environ, COMPACTC_SCHEME=str(scheme), RUSTUP_TOOLCHAIN='1.99.0')
    fixture = ORIGINAL / 'vector_witness_return_semantics.compact'
    generated = {}
    for mode in ('typescript', 'both'):
        destination = output / 'generated' / mode
        command = [str(compiler), '--target', 'ts']
        if mode == 'both':
            command += ['--target', 'rust', '--rust-runtime-root', str(ROOT)]
        command += ['--feature-zkir-v3', '--skip-zk', '--compact-path',
                    str(ROOT / 'compiler'), str(fixture), str(destination)]
        run(command, output / f'{mode}-compile.log', env=environment)
        check_manifest(destination, require_zkir=False)
        manifest = json.loads((destination / 'compiler/contract-manifest.json').read_text())
        require('zkir' not in manifest and not (destination / 'zkir').exists(),
                'witness-return skip-zk unexpectedly emitted proof material')
        info = json.loads((destination / 'compiler/contract-info.json').read_text())
        circuits = info['circuits']
        require(circuits == [{'name': CIRCUIT, 'pure': False, 'proof': False,
                              'arguments': [],
                              'result-type': {'type-name': 'Tuple', 'types': []}}],
                'witness-return circuit API changed')
        adapter_result = run(['node', str(FAMILY / 'check.mjs'),
                              str(destination / 'contract/index.js'), str(runtime)],
                             output / f'{mode}-typescript.log', env=environment).strip()
        require(adapter_result == 'original Vector witness return semantics passed',
                f'{mode} original TypeScript witness assertion changed')
        row = {'typescript_sha256': sha256(destination / 'contract/index.js'),
               'manifest_sha256': sha256(destination / 'compiler/contract-manifest.json'),
               'contract_info_sha256': sha256(destination / 'compiler/contract-info.json'),
               'typescript_result': adapter_result}
        if mode == 'both':
            row['rust_sha256'] = sha256(destination / 'contract/lib.rs')
            row['rust_ir_sha256'] = sha256(destination / 'contract/compact-rust-ir.json')
        generated[mode] = row
    require(generated['typescript']['typescript_sha256'] == generated['both']['typescript_sha256'],
            'default and paired TypeScript output differ')

    contract = output / 'generated/both/contract'
    consumer = output / 'rust-consumer'
    (consumer / 'tests').mkdir(parents=True)
    shutil.copyfile(FAMILY / 'consumer.rs', consumer / 'tests/witness_return.rs')
    package_name = tomllib.loads((contract / 'Cargo.toml').read_text())['package']['name']
    manifest = consumer / 'Cargo.toml'
    manifest.write_text('[workspace]\n[package]\n'
                        'name = "ledger9-vector-witness-return-consumer"\n'
                        'version = "0.0.0"\nedition = "2024"\n[dependencies]\n'
                        f'fixture = {{ package = {json.dumps(package_name)}, '
                        f'path = {json.dumps(str(contract))} }}\n')
    cargo_env = dict(environment, CARGO_TARGET_DIR=str(
        args.cargo_target.resolve() if args.cargo_target else output / 'rust-target'))
    consumer_lock = consumer / 'Cargo.lock'
    shutil.copyfile(ROOT / 'Cargo.lock', consumer_lock)
    metadata = json.loads(run(['cargo', '+1.99.0', 'metadata', '--offline',
                               '--format-version', '1', '--manifest-path', str(manifest)],
                              output / 'cargo-metadata.log', env=cargo_env))
    workspace_lock = tomllib.loads((ROOT / 'Cargo.lock').read_text())['package']
    locked = {(row['name'], row['version'], row['source']): row.get('checksum')
              for row in workspace_lock if 'source' in row}
    local = {manifest, ROOT / 'runtime-rs/Cargo.toml', ROOT / 'runtime-rs-macros/Cargo.toml',
             contract / 'Cargo.toml'}
    lock_sha = validate_consumer_lock(metadata, consumer_lock, local, locked)
    rust_result = run(['cargo', '+1.99.0', 'test', '--locked', '--offline',
                       '--manifest-path', str(manifest)], output / 'cargo-test.log', env=cargo_env)
    require('2 passed' in rust_result, 'Rust witness-return assertions did not pass')
    require_unchanged(source, runtime_before, recipe, runtime,
                      args.compiler_recipe, compiler, scheme)
    require(original_inventory() == originals, 'upstream originals changed during qualification')
    receipt = {**source, 'status': 'passed', 'source_clean': True,
               'platform': os.uname().sysname + ' ' + os.uname().machine,
               'upstream_commit': PINNED, 'original_files': originals,
               'sdk_ledger_version': matrix['ledger_version'],
               'compiler_recipe': recipe, 'compiler_sha256': sha256(compiler),
               'scheme_sha256': sha256(scheme), 'runtime': runtime_before,
               'runtime_recipe': runtime_recipe, 'cargo_configuration': configuration,
               'generated': generated, 'adapter_sha256': sha256(FAMILY / 'check.mjs'),
               'consumer_source_sha256': sha256(FAMILY / 'consumer.rs'),
               'checker_sha256': sha256(Path(__file__)),
               'rust_consumer_lock_sha256': lock_sha,
               'rust_test_log_sha256': sha256(output / 'cargo-test.log'),
               'rust_result': 'two generated Rust witness-return tests passed',
               'limitations': ['local platform only; native Linux qualification required',
                               'original witness-return circuit has proof=false; '
                               'proof and funded transaction are inapplicable']}
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': 'passed', 'head': source['head'], 'output': str(output)}))


if __name__ == '__main__':
    main()
