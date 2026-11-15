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

"""Qualify pinned pure Vector destructuring on generated TypeScript and Rust."""

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
FAMILY = ROOT / 'test-contracts/primitives/vector/destructuring'
PINNED = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'
FIXTURE = FAMILY / 'basic/vector_destructuring_basic.compact'
TEST = FAMILY / 'basic/runtime.pass.test.ts'
MANIFEST = FAMILY / 'original-source-sha256.json'
CONSUMER = FAMILY / 'consumer.rs'
ADAPTER = FAMILY / 'check.mjs'
RESULT_TYPE = {'type-name': 'Tuple', 'types': [
    {'type-name': 'Field'}, {'type-name': 'Field'},
]}
EXPECTED = [10, 20]


def require(ok: bool, message: str) -> None:
    if not ok:
        raise RuntimeError(message)


def pinned_sha256(name: str) -> str:
    blob = subprocess.check_output(['git', 'show', f'{PINNED}:{name}'], cwd=ROOT)
    return hashlib.sha256(blob).hexdigest()


def original_inventory(path: Path) -> dict[str, str]:
    manifest = json.loads(path.read_text())
    expected = {f'test-contracts/primitives/vector/destructuring/basic/{name}'
                for name in ('vector_destructuring_basic.compact',
                             'compile.pass.test.ts', 'runtime.pass.test.ts')}
    require(manifest.get('upstream_commit') == PINNED and
            isinstance(manifest.get('files'), dict) and
            set(manifest['files']) == expected,
            'pinned Vector destructuring inventory changed')
    for name, digest in manifest['files'].items():
        source = ROOT / name
        require(re.fullmatch(r'[0-9a-f]{64}', digest) is not None and
                source.is_file() and not source.is_symlink() and
                sha256(source) == digest == pinned_sha256(name),
                f'pinned Vector destructuring original changed: {name}')
    return manifest['files']


def original_result(path: Path) -> list[int]:
    matches = re.findall(r'expect\(Array\.from\(result\)\)\.toEqual\(\[([^\]]+)\]\)',
                         path.read_text())
    require(len(matches) == 1, 'original Vector destructuring result assertion changed')
    tokens = [token.strip() for token in matches[0].split(',')]
    require(len(tokens) == 2 and all(re.fullmatch(r'\d+n', token) for token in tokens),
            'original Vector destructuring result values changed')
    return [int(token[:-1]) for token in tokens]


def consumer_expected(path: Path) -> list[int]:
    matches = re.findall(r'const EXPECTED: \[u128; 2\] = \[([^\]]+)\];', path.read_text())
    require(len(matches) == 1, 'Rust Vector destructuring expected values changed')
    tokens = [token.strip() for token in matches[0].split(',')]
    require(len(tokens) == 2 and all(re.fullmatch(r'\d+', token) for token in tokens),
            'Rust Vector destructuring expected values changed')
    return [int(token) for token in tokens]


def assertion_messages(path: Path) -> tuple[str, ...]:
    messages = tuple(re.findall(r'assert\([^;]*?,\s*"([^"]+)"\);', path.read_text()))
    require(len(messages) == 4 and len(set(messages)) == 4,
            'pinned Vector destructuring internal assertions changed')
    return messages


def require_generated_assertions(messages: tuple[str, ...], typescript: Path,
                                 rust: Path | None = None) -> None:
    js = typescript.read_text()
    require(js.count('__compactRuntime.assert(') == len(messages) and
            all(js.count(repr(message)) == 1 for message in messages),
            'generated TypeScript lost an original destructuring assertion')
    if rust is not None:
        rs = rust.read_text()
        require(rs.count('CompactError::AssertionFailed(') == len(messages) and
                all(rs.count(json.dumps(message)) == 1 for message in messages),
                'generated Rust lost an original destructuring assertion')


def require_circuit_info(circuits: list[dict]) -> None:
    require(circuits == [{'name': 'vector_destructuring_basic', 'pure': True,
                          'proof': False, 'arguments': [], 'result-type': RESULT_TYPE}],
            'Vector destructuring pure capability or tuple result type changed')


def rust_test_passed(log: str) -> bool:
    return ('test original_basic ... ok' in log and
            re.search(r'test result: ok\. 1 passed; 0 failed;', log) is not None)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'compiler-recipe', 'ts-runtime', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--cargo-target', type=Path)
    args = parser.parse_args()
    source = source_state()
    commit = subprocess.check_output(['git', 'log', '-1', '--format=%G?%n%B'],
                                     cwd=ROOT, text=True)
    require(commit.startswith('G\n') and 'Signed-off-by:' in commit and
            'Assisted-by: AI' in commit,
            'source head must be signed, DCO complete and assisted')
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
    originals = original_inventory(MANIFEST)
    values = original_result(TEST)
    require(values == EXPECTED == consumer_expected(CONSUMER),
            'Vector destructuring expected values differ from pinned original')
    messages = assertion_messages(FIXTURE)
    output.mkdir(parents=True)
    link = output / 'node_modules/@midnight-ntwrk/compact-runtime'
    link.parent.mkdir(parents=True)
    link.symlink_to(runtime, target_is_directory=True)
    (output / 'package.json').write_text('{"type":"module"}\n')
    env = os.environ.copy()
    env['COMPACTC_SCHEME'] = str(scheme)
    generated = {}
    for mode in ('typescript', 'both'):
        destination = output / 'generated' / mode
        command = [str(compiler), '--target', 'ts']
        if mode == 'both':
            command += ['--target', 'rust', '--rust-runtime-root', str(ROOT)]
        command += ['--feature-zkir-v3', '--skip-zk', '--compact-path',
                    str(ROOT / 'compiler'), str(FIXTURE), str(destination)]
        run(command, output / f'{mode}-compile.log', env=env)
        info_path = destination / 'compiler/contract-info.json'
        require_circuit_info(json.loads(info_path.read_text())['circuits'])
        contract = destination / 'contract'
        require_generated_assertions(messages, contract / 'index.js',
                                     contract / 'lib.rs' if mode == 'both' else None)
        result = run(['node', str(ADAPTER), str(contract / 'index.js'),
                      ','.join(map(str, values))],
                     output / f'{mode}-runtime.log', env=env)
        require(result.strip() == 'basic: original Vector destructuring assertion passed',
                'original Vector destructuring TypeScript assertion changed')
        generated[mode] = {'typescript_sha256': sha256(contract / 'index.js'),
                           'contract_info_sha256': sha256(info_path),
                           'runtime_result': result.strip()}
        if mode == 'both':
            generated[mode]['rust_sha256'] = sha256(contract / 'lib.rs')
            generated[mode]['rust_ir_sha256'] = sha256(contract / 'compact-rust-ir.json')
    require(generated['typescript']['typescript_sha256'] ==
            generated['both']['typescript_sha256'],
            'paired compile changed TypeScript output')
    consumer = output / 'rust-consumer'
    (consumer / 'tests').mkdir(parents=True)
    shutil.copyfile(CONSUMER, consumer / 'tests/vector_destructuring.rs')
    lock = consumer / 'Cargo.lock'
    shutil.copyfile(ROOT / 'Cargo.lock', lock)
    contract = output / 'generated/both/contract'
    package_name = tomllib.loads((contract / 'Cargo.toml').read_text())['package']['name']
    manifest = consumer / 'Cargo.toml'
    manifest.write_text('[workspace]\n[package]\nname = "ledger9-vector-destructuring-consumer"\n'
                        'version = "0.1.0"\nedition = "2024"\n[dependencies]\n'
                        f'basic = {{ package = {json.dumps(package_name)}, '
                        f'path = {json.dumps(str(contract))} }}\n')
    target = args.cargo_target.resolve() if args.cargo_target else output / 'cargo-target'
    cargo_env = dict(env, CARGO_TARGET_DIR=str(target), CARGO_INCREMENTAL='0',
                     CARGO_PROFILE_DEV_DEBUG='0')
    metadata = json.loads(run(['cargo', '+1.99.0', 'metadata', '--offline',
                               '--format-version', '1', '--manifest-path', str(manifest)],
                              output / 'cargo-metadata.log', env=cargo_env))
    workspace_lock = tomllib.loads((ROOT / 'Cargo.lock').read_text())['package']
    pinned = {(row['name'], row['version'], row['source']): row.get('checksum')
              for row in workspace_lock if 'source' in row}
    local = {manifest, contract / 'Cargo.toml', ROOT / 'runtime-rs/Cargo.toml',
             ROOT / 'runtime-rs-macros/Cargo.toml'}
    validate_consumer_lock(metadata, lock, local, pinned)
    run(['cargo', '+1.99.0', 'test', '--locked', '--offline', '--manifest-path', str(manifest)],
        output / 'cargo-test.log', env=cargo_env)
    require(rust_test_passed((output / 'cargo-test.log').read_text()),
            'original Vector destructuring Rust assertion did not pass')
    for mode in ('typescript', 'both'):
        destination = output / 'generated' / mode
        contract = destination / 'contract'
        require(sha256(contract / 'index.js') == generated[mode]['typescript_sha256'] and
                sha256(destination / 'compiler/contract-info.json') ==
                generated[mode]['contract_info_sha256'],
                'generated TypeScript or metadata changed during Rust test')
        require_generated_assertions(messages, contract / 'index.js',
                                     contract / 'lib.rs' if mode == 'both' else None)
        if mode == 'both':
            require(sha256(contract / 'lib.rs') == generated[mode]['rust_sha256'] and
                    sha256(contract / 'compact-rust-ir.json') ==
                    generated[mode]['rust_ir_sha256'],
                    'generated Rust changed during Rust test')
    require_unchanged(source, runtime_before, recipe, runtime,
                      args.compiler_recipe, compiler, scheme)
    require(original_inventory(MANIFEST) == originals and
            original_result(TEST) == EXPECTED == consumer_expected(CONSUMER) and
            assertion_messages(FIXTURE) == messages,
            'pinned Vector destructuring source changed during qualification')
    receipt = {
        **source, 'source_clean': True,
        'platform': os.uname().sysname + ' ' + os.uname().machine,
        'upstream_commit': PINNED, 'original_files': originals,
        'manifest_sha256': sha256(MANIFEST), 'checker_sha256': sha256(Path(__file__)),
        'adapter_sha256': sha256(ADAPTER), 'consumer_source_sha256': sha256(CONSUMER),
        'identity_library_sha256': sha256(ROOT / 'tools/compact-rust-backend/check_upstream_ledger9_vector_casts.py'),
        'compiler_recipe': recipe, 'compiler_sha256': sha256(compiler),
        'scheme_sha256': sha256(scheme), 'runtime': runtime_before,
        'runtime_recipe': runtime_recipe, 'generated': generated,
        'assertion_messages': list(messages), 'cargo_target': str(target),
        'rust_consumer_lock_sha256': sha256(lock),
        'rust_test_log_sha256': sha256(output / 'cargo-test.log'),
        'result': 'pinned pure Vector destructuring output and four internal assertions passed generated TypeScript and Rust',
        'limitations': ['local Darwin arm64; native Linux qualification required',
                        'pinned circuit is pure with proof=false; no recorded or funded API',
                        'original Vitest harness was not run; its pinned assertion was extracted and rerun on generated APIs'],
    }
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': 'passed', 'head': source['head'], 'output': str(output)}))


if __name__ == '__main__':
    main()
