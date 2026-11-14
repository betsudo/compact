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

"""Qualify three pinned stateful Vector loops on generated TS and Rust."""

import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tomllib

from check_upstream_ledger9_vector_casts import (
    compiler_recipe_identity, require_unchanged, run, runtime_identity,
    runtime_recipe_identity, sha256, source_state,
)
from check_unit_mutation_initializers import validate_consumer_lock


ROOT = Path(__file__).resolve().parents[2]
FAMILY = ROOT / 'test-contracts/primitives/vector/loops'
PINNED = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'
CASES = ('empty_iteration', 'iteration', 'spread_iteration')
K = {'empty_iteration': 6, 'iteration': 8, 'spread_iteration': 8}


def require(ok: bool, message: str) -> None:
    if not ok:
        raise RuntimeError(message)


def original_inventory(path: Path) -> dict[str, str]:
    manifest = json.loads(path.read_text())
    require(manifest.get('upstream_commit') == PINNED and
            isinstance(manifest.get('files'), dict) and len(manifest['files']) == 9,
            'pinned Vector loop inventory changed')
    expected = {f'test-contracts/primitives/vector/loops/{name}/{filename}'
                for name in CASES
                for filename in (f'vector_loop_{name}.compact',
                                 'compile.pass.test.ts', 'runtime.pass.test.ts')}
    require(set(manifest['files']) == expected, 'pinned Vector loop file set changed')
    for name, digest in manifest['files'].items():
        source = ROOT / name
        require(re.fullmatch(r'[0-9a-f]{64}', digest) is not None and
                source.is_file() and not source.is_symlink() and sha256(source) == digest,
                f'pinned original Vector loop file changed: {name}')
    return manifest['files']


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'compiler-recipe', 'ts-runtime', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    source = source_state()
    require(subprocess.check_output(['git', 'log', '-1', '--format=%G?'], cwd=ROOT,
                                    text=True).strip() == 'G' and
            'Signed-off-by:' in subprocess.check_output(
                ['git', 'log', '-1', '--format=%B'], cwd=ROOT, text=True),
            'source head must be signed and DCO complete')
    compiler = args.compiler.resolve(strict=True)
    scheme = compiler.with_name('compactc-scheme').resolve(strict=True)
    runtime = args.ts_runtime.resolve(strict=True)
    output = args.output.resolve()
    require(not args.output.is_symlink() and not output.exists(), 'output must be absent')
    source_package = json.loads((ROOT / 'runtime/package.json').read_text())
    runtime_package = json.loads((runtime / 'package.json').read_text())
    require(all(source_package[key] == runtime_package[key]
                for key in ('name', 'version', 'dependencies')),
            'TypeScript runtime package differs from source')
    nix = shutil.which('nix')
    require(nix is not None, 'Nix required for exact-head runtime binding')
    runtime_recipe = runtime_recipe_identity(runtime, Path(nix).resolve(strict=True))
    runtime_before = runtime_identity(runtime)
    recipe = compiler_recipe_identity(args.compiler_recipe, source, compiler, scheme)
    manifest_path = FAMILY / 'original-source-sha256.json'
    originals = original_inventory(manifest_path)
    output.mkdir(parents=True)
    link = output / 'node_modules/@midnight-ntwrk/compact-runtime'
    link.parent.mkdir(parents=True)
    link.symlink_to(runtime, target_is_directory=True)
    (output / 'package.json').write_text('{"type":"module"}\n')
    env = os.environ.copy()
    env['COMPACTC_SCHEME'] = str(scheme)
    generated = {}
    for name in CASES:
        fixture = FAMILY / name / f'vector_loop_{name}.compact'
        cases = {}
        for mode in ('typescript', 'both'):
            destination = output / 'generated' / mode / name
            command = [str(compiler), '--target', 'ts']
            if mode == 'both':
                command += ['--target', 'rust', '--rust-runtime-root', str(ROOT),
                            '--rust-require-recording']
            command += ['--feature-zkir-v3', '--skip-zk', '--compact-path',
                        str(ROOT / 'compiler'), str(fixture), str(destination)]
            run(command, output / f'{name}-{mode}.log', env=env)
            info_path = destination / 'compiler/contract-info.json'
            circuits = json.loads(info_path.read_text())['circuits']
            require(len(circuits) == 1 and circuits[0]['name'] == f'vector_loop_{name}'
                    and not circuits[0]['pure'] and circuits[0]['proof'] and
                    not circuits[0]['arguments'],
                    f'{name} stateful provability metadata changed')
            contract = destination / 'contract'
            result = run(['node', str(FAMILY / 'check.mjs'),
                          str(contract / 'index.js'), name],
                         output / f'{name}-{mode}-runtime.log', env=env)
            require(result.strip() == f'{name}: original loop result and state passed',
                    f'{name} original TypeScript assertion changed')
            raw = destination / 'zkir' / f'vector_loop_{name}.zkir'
            cases[mode] = {'typescript_sha256': sha256(contract / 'index.js'),
                           'contract_info_sha256': sha256(info_path),
                           'zkir_sha256': sha256(raw),
                           'runtime_result': result.strip()}
            if mode == 'both':
                cases[mode]['rust_sha256'] = sha256(contract / 'lib.rs')
                cases[mode]['rust_ir_sha256'] = sha256(contract / 'compact-rust-ir.json')
        require(cases['typescript']['typescript_sha256'] ==
                cases['both']['typescript_sha256'] and
                cases['typescript']['zkir_sha256'] == cases['both']['zkir_sha256'],
                f'{name} TS or ZKIR changed in paired compile')
        generated[name] = cases
    consumer = output / 'rust-consumer'
    (consumer / 'tests').mkdir(parents=True)
    shutil.copyfile(FAMILY / 'consumer.rs', consumer / 'tests/vector_loops.rs')
    lock = consumer / 'Cargo.lock'
    shutil.copyfile(ROOT / 'Cargo.lock', lock)
    manifest = consumer / 'Cargo.toml'
    cargo = ['[workspace]', '[package]', 'name = "ledger9-vector-loops-consumer"',
             'version = "0.1.0"', 'edition = "2024"', '[dependencies]']
    contracts = []
    for name in CASES:
        contract = output / 'generated/both' / name / 'contract'
        contracts.append(contract)
        package = tomllib.loads((contract / 'Cargo.toml').read_text())['package']['name']
        cargo.append(f'{name} = {{ package = {json.dumps(package)}, '
                     f'path = {json.dumps(str(contract))} }}')
    manifest.write_text('\n'.join(cargo) + '\n')
    cargo_target = output / 'cargo-target'
    cargo_env = dict(env, CARGO_TARGET_DIR=str(cargo_target))
    metadata = json.loads(run(['cargo', '+1.99.0', 'metadata', '--offline',
                               '--format-version', '1', '--manifest-path', str(manifest)],
                              output / 'cargo-metadata.log', env=cargo_env))
    workspace_lock = tomllib.loads((ROOT / 'Cargo.lock').read_text())['package']
    pinned = {(row['name'], row['version'], row['source']): row.get('checksum')
              for row in workspace_lock if 'source' in row}
    local = {manifest, ROOT / 'runtime-rs/Cargo.toml', ROOT / 'runtime-rs-macros/Cargo.toml'}
    local.update(contract / 'Cargo.toml' for contract in contracts)
    validate_consumer_lock(metadata, lock, local, pinned)
    run(['cargo', '+1.99.0', 'test', '--locked', '--offline', '--manifest-path', str(manifest)],
        output / 'cargo-test.log', env=cargo_env)
    require('3 passed' in (output / 'cargo-test.log').read_text(),
            'three original Rust ordinary/recorded assertions did not pass')
    require_unchanged(source, runtime_before, recipe, runtime,
                      args.compiler_recipe, compiler, scheme)
    require(original_inventory(manifest_path) == originals,
            'original Vector loop files changed during qualification')
    receipt = {
        **source, 'source_clean': True,
        'platform': os.uname().sysname + ' ' + os.uname().machine,
        'upstream_commit': PINNED, 'original_files': originals,
        'manifest_sha256': sha256(manifest_path),
        'checker_sha256': sha256(Path(__file__)),
        'adapter_sha256': sha256(FAMILY / 'check.mjs'),
        'consumer_source_sha256': sha256(FAMILY / 'consumer.rs'),
        'identity_library_sha256': sha256(ROOT / 'tools/compact-rust-backend/check_upstream_ledger9_vector_casts.py'),
        'compiler_recipe': recipe, 'compiler_sha256': sha256(compiler),
        'scheme_sha256': sha256(scheme), 'runtime': runtime_before,
        'runtime_recipe': runtime_recipe, 'generated': generated,
        'raw_zkir_k': K, 'cargo_target': str(cargo_target),
        'rust_consumer_lock_sha256': sha256(lock),
        'rust_test_log_sha256': sha256(output / 'cargo-test.log'),
        'result': 'three original stateful Vector loops passed TypeScript and ordinary/recorded Rust',
        'limitations': ['local Darwin arm64; native Linux qualification required',
                        'fresh proof and funded strict application/replay are separate gates'],
    }
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': 'passed', 'head': source['head'], 'output': str(output)}))


if __name__ == '__main__':
    main()
