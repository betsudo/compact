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

"""Qualify three pinned Bytes Fold originals on generated TS and Rust APIs."""

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
FAMILY = ROOT / 'test-contracts/primitives/bytes/fold'
CASES = ('boolean_reduction', 'conditional_accumulation', 'summation')
RESULTS = ({'type-name': 'Boolean'}, {'type-name': 'Field'},
           {'type-name': 'Uint', 'maxval': 255})
MESSAGES = ('All elements should be positive',
            'Conditional accumulation should stop at 15',
            'Sum should be 15 (1+2+3+4+5)')


def fixture(case: str) -> Path:
    return FAMILY / case / f'bytes_fold_{case}.compact'


def originals() -> dict[str, str]:
    manifest = json.loads((FAMILY / 'original-source-sha256.json').read_text())
    expected = {str((fixture(case).parent / name).relative_to(ROOT))
                for case in CASES for name in
                (fixture(case).name, 'compile.pass.test.ts', 'runtime.pass.test.ts')}
    upstream = subprocess.check_output(
        ['git', 'ls-tree', '-r', '--name-only', PINNED,
         'test-contracts/primitives/bytes/fold'], cwd=ROOT, text=True).splitlines()
    require(set(upstream) == expected and manifest.get('upstream_commit') == PINNED and
            isinstance(manifest.get('files'), dict) and set(manifest['files']) == expected,
            'pinned Bytes Fold original inventory changed')
    for name, digest in manifest['files'].items():
        path = ROOT / name
        upstream_bytes = subprocess.check_output(['git', 'show', f'{PINNED}:{name}'], cwd=ROOT)
        require(path.is_file() and not path.is_symlink() and
                re.fullmatch('[0-9a-f]{64}', digest) is not None and
                sha256(path) == digest == hashlib.sha256(upstream_bytes).hexdigest(),
                f'pinned Bytes Fold original changed: {name}')
    for case, message, expected_result in zip(CASES, MESSAGES, ('true', '15n', '15n')):
        source = fixture(case).read_text()
        runtime = (fixture(case).parent / 'runtime.pass.test.ts').read_text()
        require(message in source and f'expect(result).toBe({expected_result})' in runtime,
                f'original Bytes Fold assertion changed: {case}')
    return manifest['files']


def assert_generated(contract: Path, message: str, rust: bool) -> None:
    body = contract.read_text()
    needle = json.dumps(message) if rust else repr(message)
    require(body.count(needle) == (2 if rust else 1),
            f'generated {"Rust" if rust else "TypeScript"} Fold assertion changed')


def generated_files(root: Path) -> dict[str, str]:
    files = {}
    for path in sorted(root.rglob('*')):
        require(not path.is_symlink(), f'generated artifact is a symlink: {path}')
        if path.is_file():
            files[path.relative_to(root).as_posix()] = sha256(path)
    require(files, 'generated artifact set is empty')
    return files


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
            'Assisted-by: AI' in signature, 'head must be signed, DCO and assisted')
    output = args.output.resolve()
    require(not args.output.is_symlink() and not output.exists(), 'output must be absent')
    compiler = args.compiler.resolve(strict=True)
    scheme = compiler.with_name('compactc-scheme').resolve(strict=True)
    runtime = args.ts_runtime.resolve(strict=True)
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
    pinned = originals()
    control_dir = ROOT / 'tools/compact-rust-backend/tests/bytes_fold'
    control_sources = {name: control_dir / f'{name}.compact' for name in ('pure', 'named')}
    local_paths = [FAMILY / 'original-source-sha256.json', FAMILY / 'check.mjs',
                   FAMILY / 'consumer.rs', control_dir / 'check-control.mjs',
                   *control_sources.values()]
    local = {str(path.relative_to(ROOT)): sha256(path) for path in local_paths}
    output.mkdir(parents=True)
    link = output / 'node_modules/@midnight-ntwrk/compact-runtime'
    link.parent.mkdir(parents=True)
    link.symlink_to(runtime, target_is_directory=True)
    (output / 'package.json').write_text('{"type":"module"}\n')
    env = dict(os.environ, COMPACTC_SCHEME=str(scheme), RUSTUP_TOOLCHAIN='1.99.0')
    generated = {}
    for case, result_type, message in zip(CASES, RESULTS, MESSAGES):
        generated[case] = {}
        for mode in ('typescript', 'both'):
            destination = output / 'generated' / case / mode
            command = [str(compiler), '--target', 'ts']
            if mode == 'both':
                command += ['--target', 'rust', '--rust-runtime-root', str(ROOT),
                            '--rust-require-recording']
            command += ['--feature-zkir-v3', '--skip-zk', '--compact-path',
                        str(ROOT / 'compiler'), str(fixture(case)), str(destination)]
            run(command, output / f'{case}-{mode}-compile.log', env)
            check_manifest(destination, require_zkir=True)
            info = destination / 'compiler/contract-info.json'
            require(json.loads(info.read_text())['circuits'] == [{
                'name': f'bytes_fold_{case}', 'pure': False, 'proof': True,
                'arguments': [], 'result-type': result_type}],
                f'Bytes Fold capability or result changed: {case}')
            contract = destination / 'contract'
            assert_generated(contract / 'index.js', message, rust=False)
            if mode == 'both':
                assert_generated(contract / 'lib.rs', message, rust=True)
            runtime_log = output / f'{case}-{mode}-runtime.log'
            observed = run(['node', str(FAMILY / 'check.mjs'),
                            str(contract / 'index.js'), case], runtime_log, env)
            require(observed.strip() == f'{case}: original Bytes fold result and ledger passed',
                    f'generated TypeScript Fold result changed: {case}')
            row = {'typescript_sha256': sha256(contract / 'index.js'),
                   'contract_info_sha256': sha256(info),
                   'manifest_sha256': sha256(destination / 'compiler/contract-manifest.json'),
                   'runtime_log_sha256': sha256(runtime_log),
                   'zkir_sha256': sha256(destination / f'zkir/bytes_fold_{case}.zkir')}
            if mode == 'both':
                row['rust_sha256'] = sha256(contract / 'lib.rs')
                row['rust_ir_sha256'] = sha256(contract / 'compact-rust-ir.json')
            generated[case][mode] = row
        require(all(generated[case]['typescript'][key] == generated[case]['both'][key]
                    for key in ('typescript_sha256', 'contract_info_sha256',
                                'runtime_log_sha256', 'zkir_sha256')),
                f'paired compile changed TypeScript or ZKIR: {case}')
    control_hashes = {}
    for name, control_source in control_sources.items():
        control = output / f'generated/{name}-control'
        run([str(compiler), '--target', 'ts', '--target', 'rust',
             '--rust-runtime-root', str(ROOT), '--feature-zkir-v3', '--skip-zk',
             '--compact-path', str(ROOT / 'compiler'), str(control_source), str(control)],
            output / f'{name}-control-compile.log', env)
        check_manifest(control, require_zkir=False)
        require(not (control / 'zkir').exists() and
                json.loads((control / 'compiler/contract-info.json').read_text())['circuits'] == [{
                    'name': f'bytes_fold_{name}', 'pure': True, 'proof': False,
                    'arguments': [], 'result-type': {'type-name': 'Uint', 'maxval': 255}}],
                f'{name} Bytes Fold control capability changed')
        runtime_log = output / f'{name}-control-runtime.log'
        observed = run(['node', str(control_dir / 'check-control.mjs'),
                        str(control / 'contract/index.js'), name], runtime_log, env)
        require(observed.strip() == f'{name}: pure Bytes fold result passed',
                f'{name} Bytes Fold TypeScript result changed')
        if name == 'named':
            ir = json.loads((control / 'contract/compact-rust-ir.json').read_text())
            def folds(value):
                if isinstance(value, dict):
                    if value.get('kind') == 'vector_fold_call':
                        yield value
                    for child in value.values():
                        yield from folds(child)
                elif isinstance(value, list):
                    for child in value:
                        yield from folds(child)
            named = list(folds(ir))
            require(len(named) == 1 and named[0]['name'] == 'add_byte' and
                    named[0]['source']['kind'] == 'bytes_to_vector' and
                    named[0]['source']['length'] == 3,
                    'named Bytes Fold lost checked byte source normalization')
        control_hashes[name] = {
            'typescript': sha256(control / 'contract/index.js'),
            'rust': sha256(control / 'contract/lib.rs'),
            'rust_ir': sha256(control / 'contract/compact-rust-ir.json'),
            'contract_info': sha256(control / 'compiler/contract-info.json'),
            'runtime_log': sha256(runtime_log)}
    generated_before_consumer = generated_files(output / 'generated')
    consumer = output / 'rust-consumer'
    (consumer / 'tests').mkdir(parents=True)
    shutil.copyfile(FAMILY / 'consumer.rs', consumer / 'tests/bytes_fold.rs')
    shutil.copyfile(ROOT / 'Cargo.lock', consumer / 'Cargo.lock')
    dependencies = []
    generated_paths = []
    for case in CASES:
        contract = output / 'generated' / case / 'both/contract'
        generated_paths.append(contract)
        package = tomllib.loads((contract / 'Cargo.toml').read_text())['package']['name']
        dependencies.append(f'{case} = {{ package = {json.dumps(package)}, '
                            f'path = {json.dumps(str(contract))} }}')
    for name in control_sources:
        control_contract = output / f'generated/{name}-control/contract'
        generated_paths.append(control_contract)
        control_package = tomllib.loads((control_contract / 'Cargo.toml').read_text())['package']['name']
        dependencies.append(f'{name}_control = {{ package = {json.dumps(control_package)}, '
                            f'path = {json.dumps(str(control_contract))} }}')
    manifest = consumer / 'Cargo.toml'
    manifest.write_text('[workspace]\n[package]\nname = "ledger9-bytes-fold-consumer"\n'
                        'version = "0.1.0"\nedition = "2024"\n[dependencies]\n'
                        'midnight-onchain-vm = "=4.0.1"\nmidnight-serialize = "=1.2.1"\n' +
                        '\n'.join(dependencies) + '\n')
    cargo_target = args.cargo_target.resolve(strict=True) if args.cargo_target else output / 'cargo-target'
    require(not args.cargo_target or (cargo_target.is_dir() and
                                     not args.cargo_target.is_symlink()),
            'shared Cargo target must be an ordinary directory')
    cargo_env = dict(env, CARGO_TARGET_DIR=str(cargo_target), CARGO_INCREMENTAL='0',
                     CARGO_PROFILE_DEV_DEBUG='0')
    metadata = json.loads(run(['cargo', '+1.99.0', 'metadata', '--offline',
                               '--format-version', '1', '--manifest-path', str(manifest)],
                              output / 'cargo-metadata.log', cargo_env))
    lock_packages = tomllib.loads((ROOT / 'Cargo.lock').read_text())['package']
    pinned_packages = {(row['name'], row['version'], row['source']): row.get('checksum')
                       for row in lock_packages if 'source' in row}
    allowed = {manifest, ROOT / 'runtime-rs/Cargo.toml',
               ROOT / 'runtime-rs-macros/Cargo.toml',
               *(path / 'Cargo.toml' for path in generated_paths)}
    validate_consumer_lock(metadata, consumer / 'Cargo.lock', allowed, pinned_packages)
    run(['cargo', '+1.99.0', 'test', '--locked', '--offline',
         '--manifest-path', str(manifest)], output / 'cargo-test.log', cargo_env)
    rust_log = (output / 'cargo-test.log').read_text()
    require(re.search(r'test result: ok\. 5 passed; 0 failed;', rust_log) is not None and
            all(f'test {name}_original_result_and_replay ... ok' in rust_log
                for name in CASES) and
            all(f'test {name}_bytes_fold_source_normalization ... ok' in rust_log
                for name in control_sources),
            'Bytes Fold Rust ordinary/recorded/pure tests failed')
    require(source_state() == source and runtime_identity(runtime) == runtime_before and
            compiler_recipe_identity(args.compiler_recipe, source, compiler, scheme) == recipe and
            originals() == pinned and generated_files(output / 'generated') == generated_before_consumer and
            all(sha256(ROOT / name) == digest for name, digest in local.items()),
            'source, compiler, runtime or originals changed during qualification')
    for case in CASES:
        for mode in ('typescript', 'both'):
            destination = output / 'generated' / case / mode
            contract = destination / 'contract'
            row = generated[case][mode]
            require(all(sha256(path) == row[key] for path, key in (
                (contract / 'index.js', 'typescript_sha256'),
                (destination / 'compiler/contract-info.json', 'contract_info_sha256'),
                (destination / 'compiler/contract-manifest.json', 'manifest_sha256'),
                (output / f'{case}-{mode}-runtime.log', 'runtime_log_sha256'),
                (destination / f'zkir/bytes_fold_{case}.zkir', 'zkir_sha256'))),
                f'generated TypeScript/ZKIR changed: {case}/{mode}')
            if mode == 'both':
                require(sha256(contract / 'lib.rs') == row['rust_sha256'] and
                        sha256(contract / 'compact-rust-ir.json') == row['rust_ir_sha256'],
                        f'generated Rust changed: {case}')
    for name, row in control_hashes.items():
        control = output / f'generated/{name}-control'
        require(all(sha256(path) == row[key] for path, key in (
            (control / 'contract/index.js', 'typescript'),
            (control / 'contract/lib.rs', 'rust'),
            (control / 'contract/compact-rust-ir.json', 'rust_ir'),
            (control / 'compiler/contract-info.json', 'contract_info'),
            (output / f'{name}-control-runtime.log', 'runtime_log'))),
            f'{name} Bytes Fold generated artifact or runtime log changed')
    receipt = {**source, 'status': 'passed', 'source_clean': True,
               'platform': os.uname().sysname + ' ' + os.uname().machine,
               'upstream_commit': PINNED, 'original_files': pinned,
               'local_sources': local, 'generated': generated,
               'generated_files_sha256': generated_before_consumer,
               'pure_controls': control_hashes,
               'compiler_recipe': recipe, 'compiler_sha256': sha256(compiler),
               'scheme_sha256': sha256(scheme), 'runtime': runtime_before,
               'runtime_recipe': runtime_recipe,
               'checker_sha256': sha256(Path(__file__)),
               'rust_consumer_lock_sha256': sha256(consumer / 'Cargo.lock'),
               'rust_test_log_sha256': sha256(output / 'cargo-test.log'),
               'result': 'three original Bytes Fold cases passed generated TS and Rust ordinary/recorded replay',
               'limitations': ['local Darwin arm64; native Linux qualification required',
                               'three proof=true exports require fresh independent proofs and funded strict application/replay',
                               'original Vitest harness was not run; pinned assertions were replayed against generated APIs']}
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': 'passed', 'head': source['head'], 'output': str(output)}))


if __name__ == '__main__':
    main()
