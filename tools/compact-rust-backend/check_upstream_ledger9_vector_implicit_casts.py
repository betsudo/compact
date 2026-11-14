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

"""Qualify pinned pure Vector implicit casts on generated TypeScript and Rust."""

import argparse
import hashlib
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
FAMILY = ROOT / 'test-contracts/primitives/vector/implicit_casts'
PINNED = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'
CASES = ('to_tuple', 'widening')
EXPECTED = {'to_tuple': [1, 2], 'widening': [1, 2, 3, 255]}
RESULT_TYPES = {
    'to_tuple': {'type-name': 'Tuple', 'types': [
        {'type-name': 'Uint', 'maxval': 255},
        {'type-name': 'Uint', 'maxval': 255},
    ]},
    'widening': {'type-name': 'Vector', 'length': 4,
                 'type': {'type-name': 'Uint', 'maxval': 18446744073709551615}},
}


def require(ok: bool, message: str) -> None:
    if not ok:
        raise RuntimeError(message)


def pinned_sha256(name: str) -> str:
    blob = subprocess.check_output(['git', 'show', f'{PINNED}:{name}'], cwd=ROOT)
    return hashlib.sha256(blob).hexdigest()


def original_inventory(path: Path) -> dict[str, str]:
    manifest = json.loads(path.read_text())
    require(manifest.get('upstream_commit') == PINNED and
            isinstance(manifest.get('files'), dict) and len(manifest['files']) == 6,
            'pinned Vector implicit-cast inventory changed')
    expected = {f'test-contracts/primitives/vector/implicit_casts/{case}/{name}'
                for case in CASES
                for name in (f'vector_implicit_casts_{case}.compact',
                             'compile.pass.test.ts', 'runtime.pass.test.ts')}
    require(set(manifest['files']) == expected, 'pinned Vector implicit-cast file set changed')
    for name, digest in manifest['files'].items():
        source = ROOT / name
        require(re.fullmatch(r'[0-9a-f]{64}', digest) is not None and
                source.is_file() and not source.is_symlink() and
                sha256(source) == digest == pinned_sha256(name),
                f'pinned original Vector implicit-cast file changed: {name}')
    return manifest['files']


def original_result(path: Path) -> list[int]:
    matches = re.findall(r'expect\(Array\.from\(result\)\)\.toEqual\(\[([^\]]+)\]\)',
                         path.read_text())
    require(len(matches) == 1, 'original Vector implicit-cast result assertion changed')
    tokens = [token.strip() for token in matches[0].split(',')]
    require(all(re.fullmatch(r'\d+n', token) for token in tokens),
            'original Vector implicit-cast result value changed')
    return [int(token[:-1]) for token in tokens]


def rust_tests_passed(log: str) -> bool:
    return (re.search(r'test result: ok\. 2 passed; 0 failed;', log) is not None and
            all(f'test original_{name} ... ok' in log for name in CASES))


def consumer_expected(path: Path) -> dict[str, list[int]]:
    source = path.read_text()
    result = {}
    for name in CASES:
        constant = name.upper()
        matches = re.findall(rf'const {constant}_EXPECTED: \[u128; \d+\] = \[([^\]]+)\];',
                             source)
        require(len(matches) == 1, f'{name} Rust expected values changed')
        tokens = [token.strip() for token in matches[0].split(',')]
        require(all(re.fullmatch(r'\d+', token) for token in tokens),
                f'{name} Rust expected values changed')
        result[name] = [int(token) for token in tokens]
    return result


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
    package = json.loads((ROOT / 'runtime/package.json').read_text())
    supplied = json.loads((runtime / 'package.json').read_text())
    require(all(package[key] == supplied[key] for key in ('name', 'version', 'dependencies')),
            'TypeScript runtime package differs from source')
    nix = shutil.which('nix')
    require(nix is not None, 'Nix required for exact-head runtime binding')
    runtime_recipe = runtime_recipe_identity(runtime, Path(nix).resolve(strict=True))
    runtime_before = runtime_identity(runtime)
    recipe = compiler_recipe_identity(args.compiler_recipe, source, compiler, scheme)
    manifest = FAMILY / 'original-source-sha256.json'
    originals = original_inventory(manifest)
    require(consumer_expected(FAMILY / 'consumer.rs') == EXPECTED,
            'Rust expected values differ from original assertions')
    output.mkdir(parents=True)
    link = output / 'node_modules/@midnight-ntwrk/compact-runtime'
    link.parent.mkdir(parents=True)
    link.symlink_to(runtime, target_is_directory=True)
    (output / 'package.json').write_text('{"type":"module"}\n')
    env = os.environ.copy()
    env['COMPACTC_SCHEME'] = str(scheme)
    generated = {}
    for name in CASES:
        fixture = FAMILY / name / f'vector_implicit_casts_{name}.compact'
        values = original_result(FAMILY / name / 'runtime.pass.test.ts')
        require(values == EXPECTED[name], f'{name} pinned result assertion changed')
        modes = {}
        for mode in ('typescript', 'both'):
            destination = output / 'generated' / mode / name
            command = [str(compiler), '--target', 'ts']
            if mode == 'both':
                command += ['--target', 'rust', '--rust-runtime-root', str(ROOT)]
            command += ['--feature-zkir-v3', '--skip-zk', '--compact-path',
                        str(ROOT / 'compiler'), str(fixture), str(destination)]
            run(command, output / f'{name}-{mode}.log', env=env)
            info_path = destination / 'compiler/contract-info.json'
            circuits = json.loads(info_path.read_text())['circuits']
            require(len(circuits) == 1 and circuits[0]['name'] == f'vector_implicit_casts_{name}'
                    and circuits[0]['pure'] and not circuits[0]['proof'] and
                    not circuits[0]['arguments'] and
                    circuits[0]['result-type'] == RESULT_TYPES[name],
                    f'{name} pure capability or implicit-cast result type changed')
            contract = destination / 'contract'
            result = run(['node', str(FAMILY / 'check.mjs'), str(contract / 'index.js'),
                          name, ','.join(map(str, values))],
                         output / f'{name}-{mode}-runtime.log', env=env)
            require(result.strip() == f'{name}: original implicit-cast result passed',
                    f'{name} original TypeScript result changed')
            modes[mode] = {'typescript_sha256': sha256(contract / 'index.js'),
                           'contract_info_sha256': sha256(info_path),
                           'runtime_result': result.strip()}
            if mode == 'both':
                modes[mode]['rust_sha256'] = sha256(contract / 'lib.rs')
                modes[mode]['rust_ir_sha256'] = sha256(contract / 'compact-rust-ir.json')
        require(modes['typescript']['typescript_sha256'] == modes['both']['typescript_sha256'],
                f'{name} paired TypeScript output changed')
        generated[name] = modes
    consumer = output / 'rust-consumer'
    (consumer / 'tests').mkdir(parents=True)
    shutil.copyfile(FAMILY / 'consumer.rs', consumer / 'tests/vector_implicit_casts.rs')
    lock = consumer / 'Cargo.lock'
    shutil.copyfile(ROOT / 'Cargo.lock', lock)
    cargo = ['[workspace]', '[package]', 'name = "ledger9-vector-implicit-casts-consumer"',
             'version = "0.1.0"', 'edition = "2024"', '[dependencies]']
    contracts = []
    for name in CASES:
        contract = output / 'generated/both' / name / 'contract'
        contracts.append(contract)
        package_name = tomllib.loads((contract / 'Cargo.toml').read_text())['package']['name']
        cargo.append(f'{name} = {{ package = {json.dumps(package_name)}, '
                     f'path = {json.dumps(str(contract))} }}')
    cargo_path = consumer / 'Cargo.toml'
    cargo_path.write_text('\n'.join(cargo) + '\n')
    target = output / 'cargo-target'
    cargo_env = dict(env, CARGO_TARGET_DIR=str(target), CARGO_INCREMENTAL='0',
                     CARGO_PROFILE_DEV_DEBUG='0')
    metadata = json.loads(run(['cargo', '+1.99.0', 'metadata', '--offline',
                               '--format-version', '1', '--manifest-path', str(cargo_path)],
                              output / 'cargo-metadata.log', env=cargo_env))
    workspace_lock = tomllib.loads((ROOT / 'Cargo.lock').read_text())['package']
    pinned = {(row['name'], row['version'], row['source']): row.get('checksum')
              for row in workspace_lock if 'source' in row}
    local = {cargo_path, ROOT / 'runtime-rs/Cargo.toml', ROOT / 'runtime-rs-macros/Cargo.toml'}
    local.update(contract / 'Cargo.toml' for contract in contracts)
    validate_consumer_lock(metadata, lock, local, pinned)
    run(['cargo', '+1.99.0', 'test', '--locked', '--offline', '--manifest-path', str(cargo_path)],
        output / 'cargo-test.log', env=cargo_env)
    require(rust_tests_passed((output / 'cargo-test.log').read_text()),
            'two original Rust assertions did not pass')
    for name in CASES:
        for mode in ('typescript', 'both'):
            destination = output / 'generated' / mode / name
            recorded = generated[name][mode]
            contract = destination / 'contract'
            require(sha256(contract / 'index.js') == recorded['typescript_sha256'] and
                    sha256(destination / 'compiler/contract-info.json') ==
                    recorded['contract_info_sha256'],
                    f'{name} generated output changed during Rust test')
            if mode == 'both':
                require(sha256(contract / 'lib.rs') == recorded['rust_sha256'] and
                        sha256(contract / 'compact-rust-ir.json') ==
                        recorded['rust_ir_sha256'], f'{name} generated Rust changed')
    require_unchanged(source, runtime_before, recipe, runtime,
                      args.compiler_recipe, compiler, scheme)
    require(original_inventory(manifest) == originals,
            'original Vector implicit-cast files changed during qualification')
    require(consumer_expected(FAMILY / 'consumer.rs') == EXPECTED,
            'Rust expected values changed during qualification')
    receipt = {
        **source, 'source_clean': True,
        'platform': os.uname().sysname + ' ' + os.uname().machine,
        'upstream_commit': PINNED, 'original_files': originals,
        'manifest_sha256': sha256(manifest), 'checker_sha256': sha256(Path(__file__)),
        'adapter_sha256': sha256(FAMILY / 'check.mjs'),
        'consumer_source_sha256': sha256(FAMILY / 'consumer.rs'),
        'identity_library_sha256': sha256(ROOT / 'tools/compact-rust-backend/check_upstream_ledger9_vector_casts.py'),
        'compiler_recipe': recipe, 'compiler_sha256': sha256(compiler),
        'scheme_sha256': sha256(scheme), 'runtime': runtime_before,
        'runtime_recipe': runtime_recipe, 'generated': generated,
        'cargo_target': str(target), 'rust_consumer_lock_sha256': sha256(lock),
        'rust_test_log_sha256': sha256(output / 'cargo-test.log'),
        'result': 'two pinned pure Vector implicit-cast assertions restated on generated TypeScript and Rust',
        'limitations': ['local Darwin arm64; native Linux qualification required',
                        'both pinned circuits are pure with proof=false; no recorded or funded API',
                        'original Vitest harness was not run; its pinned assertions were extracted and rerun on generated APIs'],
    }
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': 'passed', 'head': source['head'], 'output': str(output)}))


if __name__ == '__main__':
    main()
