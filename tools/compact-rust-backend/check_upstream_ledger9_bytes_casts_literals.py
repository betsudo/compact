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

"""Qualify pinned Bytes cast/literal originals against generated TS and Rust."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tomllib

from check_unit_mutation_initializers import validate_consumer_lock
from check_upstream_ledger9_field_casts import (
    compiler_recipe_identity, require, run, runtime_identity,
    runtime_recipe_identity, sha256, source_state,
)


ROOT = Path(__file__).resolve().parents[2]
FAMILY = ROOT / 'test-contracts/primitives/bytes'
PINNED = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'
FIELD = {'type-name': 'Field'}
BYTE = {'type-name': 'Uint', 'maxval': 255}
SUCCESS = {
    'casts/empty': ({'type-name': 'Vector', 'length': 0, 'type': BYTE}, '[]'),
    'casts/round_trip': ({'type-name': 'Vector', 'length': 4, 'type': BYTE}, '[5n, 6n, 7n, 8n]'),
    'casts/to_field': (FIELD, '42n'),
    'casts/to_tuple': ({'type-name': 'Tuple', 'types': [FIELD] * 4}, '[10n, 20n, 30n, 40n]'),
    'casts/to_uint': ({'type-name': 'Uint', 'maxval': 2**32 - 1}, '67305985n'),
    'literal/escape_sequences': ({'type-name': 'Bytes', 'length': 2}, '[13, 10]'),
    'literal/non_ascii': ({'type-name': 'Bytes', 'length': 3}, '[10, 195, 132]'),
    'literal/string': ({'type-name': 'Bytes', 'length': 8},
                       '[97, 98, 99, 100, 101, 102, 103, 104]'),
}
REFUSALS = {
    'literal/invalid_binary': 'unexpected digit 2 (expected 0 or 1)',
    'literal/invalid_hex': "unexpected character 'g'",
    'literal/invalid_octal': 'unexpected digit 8 (expected 0 through 7)',
}
CASES = {**SUCCESS, **REFUSALS}
MANIFEST = FAMILY / 'casts-literals-original-source-sha256.json'
ADAPTER = FAMILY / 'check-casts-literals.mjs'
CONSUMER = FAMILY / 'consumer-casts-literals.rs'


def case_source(case: str) -> Path:
    group, name = case.split('/')
    return FAMILY / group / name / f'bytes_{group}_{name}.compact'


def original_inventory() -> dict[str, str]:
    record = json.loads(MANIFEST.read_text())
    expected = {str((case_source(case)).relative_to(ROOT)) for case in CASES}
    expected.update(str((case_source(case).parent / 'compile.pass.test.ts').relative_to(ROOT))
                    for case in SUCCESS)
    expected.update(str((case_source(case).parent / 'runtime.pass.test.ts').relative_to(ROOT))
                    for case in SUCCESS)
    expected.update(str((case_source(case).parent / 'compile.fail.test.ts').relative_to(ROOT))
                    for case in REFUSALS)
    require(record.get('upstream_commit') == PINNED and
            isinstance(record.get('files'), dict) and set(record['files']) == expected,
            'Bytes pinned original inventory changed')
    for name, digest in record['files'].items():
        path = ROOT / name
        original = subprocess.check_output(['git', 'show', f'{PINNED}:{name}'], cwd=ROOT)
        require(path.is_file() and not path.is_symlink() and
                re.fullmatch('[0-9a-f]{64}', digest) and
                sha256(path) == digest == hashlib.sha256(original).hexdigest(),
                f'Bytes original changed: {name}')
    for case, (_, value) in SUCCESS.items():
        body = (case_source(case).parent / 'runtime.pass.test.ts').read_text()
        require(value in body, f'Bytes original runtime assertion changed: {case}')
    for case, message in REFUSALS.items():
        body = (case_source(case).parent / 'compile.fail.test.ts').read_text()
        require('expectedError:' in body and
                all(token in body for token in re.findall(r'[A-Za-z0-9]+', message)
                    if len(token) > 3),
                f'Bytes original compile refusal changed: {case}')
    return record['files']


def circuit_name(case: str) -> str:
    return 'bytes_' + case.replace('/', '_')


def verify_compilation_failure(result: subprocess.CompletedProcess[str],
                               destination: Path, message: str) -> None:
    lock = destination.parent / f'.{destination.name}.compactc.lock'
    require(result.returncode != 0 and message in result.stderr and
            not destination.exists() and
            not list(destination.parent.glob(f'.{destination.name}.compactc-stage-*')) and
            (not lock.exists() or (lock.is_file() and not lock.is_symlink() and
                                   lock.stat().st_size == 0)),
            f'Bytes compile refusal or atomic output changed: {destination}')


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
    nix = shutil.which('nix')
    require(nix is not None, 'Nix required for runtime source binding')
    require(all(json.loads((ROOT / 'runtime/package.json').read_text())[key] ==
                json.loads((runtime / 'package.json').read_text())[key]
                for key in ('name', 'version', 'dependencies')),
            'runtime package differs from committed source')
    runtime_recipe = runtime_recipe_identity(runtime, Path(nix).resolve(strict=True))
    runtime_before = runtime_identity(runtime)
    recipe = compiler_recipe_identity(args.compiler_recipe, source, compiler, scheme)
    originals = original_inventory()
    local = {str(path.relative_to(ROOT)): sha256(path) for path in (MANIFEST, ADAPTER, CONSUMER)}
    output.mkdir(parents=True)
    link = output / 'node_modules/@midnight-ntwrk/compact-runtime'
    link.parent.mkdir(parents=True)
    link.symlink_to(runtime, target_is_directory=True)
    (output / 'package.json').write_text('{"type":"module"}\n')
    env = dict(os.environ, COMPACTC_SCHEME=str(scheme))
    generated = {}
    for case in CASES:
        generated[case] = {}
        for mode in ('typescript', 'both'):
            destination = output / 'generated' / case / mode
            command = [str(compiler), '--target', 'ts']
            if mode == 'both':
                command += ['--target', 'rust', '--rust-runtime-root', str(ROOT),
                            '--rust-require-recording']
            command += ['--feature-zkir-v3', '--skip-zk', '--compact-path',
                        str(ROOT / 'compiler'), str(case_source(case)), str(destination)]
            if case in REFUSALS:
                result = subprocess.run(command, cwd=ROOT, env=env,
                                        capture_output=True, text=True)
                log = output / f'{case.replace("/", "-")}-{mode}-refusal.log'
                log.write_text(result.stdout + result.stderr)
                verify_compilation_failure(result, destination, REFUSALS[case])
                generated[case][mode] = {'refusal_sha256': sha256(log)}
                continue
            run(command, output / f'{case.replace("/", "-")}-{mode}-compile.log', env)
            info = destination / 'compiler/contract-info.json'
            require(json.loads(info.read_text())['circuits'] == [{
                'name': circuit_name(case), 'pure': True, 'proof': False,
                'arguments': [], 'result-type': SUCCESS[case][0],
            }], f'Bytes circuit capability changed: {case}')
            contract = destination / 'contract'
            runtime_log = output / f'{case.replace("/", "-")}-{mode}-runtime.log'
            result = run(['node', str(ADAPTER), str(contract / 'index.js'), case],
                         runtime_log, env)
            require(result.strip() == f'{case}: original assertion passed',
                    f'Bytes generated TypeScript result changed: {case}')
            entry = {'typescript_sha256': sha256(contract / 'index.js'),
                     'contract_info_sha256': sha256(info), 'runtime_result': result.strip(),
                     'runtime_log_sha256': sha256(runtime_log)}
            if mode == 'both':
                entry['rust_sha256'] = sha256(contract / 'lib.rs')
                entry['rust_ir_sha256'] = sha256(contract / 'compact-rust-ir.json')
            generated[case][mode] = entry
        if case in SUCCESS:
            require(all(generated[case]['typescript'][key] == generated[case]['both'][key]
                        for key in ('typescript_sha256', 'contract_info_sha256')),
                    f'paired compiler changed TypeScript output: {case}')
    consumer = output / 'rust-consumer'
    (consumer / 'tests').mkdir(parents=True)
    shutil.copyfile(CONSUMER, consumer / 'tests/bytes_casts_literals.rs')
    shutil.copyfile(ROOT / 'Cargo.lock', consumer / 'Cargo.lock')
    paths = {case: output / 'generated' / case / 'both/contract' for case in SUCCESS}
    dependencies = []
    for case, path in paths.items():
        package = tomllib.loads((path / 'Cargo.toml').read_text())['package']['name']
        dependencies.append(f'{case.split("/")[-1]} = {{ package = {json.dumps(package)}, '
                            f'path = {json.dumps(str(path))} }}')
    manifest = consumer / 'Cargo.toml'
    manifest.write_text('[workspace]\n[package]\nname = "ledger9-bytes-casts-literals-consumer"\n'
                        'version = "0.1.0"\nedition = "2024"\n[dependencies]\n' +
                        '\n'.join(dependencies) + '\n')
    require(args.cargo_target is None or
            (args.cargo_target.is_absolute() and not args.cargo_target.is_symlink()),
            'shared Cargo target must be an absolute non-symlink path')
    cargo_target = args.cargo_target.resolve() if args.cargo_target else output / 'cargo-target'
    cargo_env = dict(env, CARGO_TARGET_DIR=str(cargo_target), CARGO_INCREMENTAL='0',
                     CARGO_PROFILE_DEV_DEBUG='0')
    metadata = json.loads(run(['cargo', '+1.99.0', 'metadata', '--offline',
                               '--format-version', '1', '--manifest-path', str(manifest)],
                              output / 'cargo-metadata.log', cargo_env))
    root_lock = tomllib.loads((ROOT / 'Cargo.lock').read_text())['package']
    pinned = {(row['name'], row['version'], row['source']): row.get('checksum')
              for row in root_lock if 'source' in row}
    allowed_local = {manifest, ROOT / 'runtime-rs/Cargo.toml',
                     ROOT / 'runtime-rs-macros/Cargo.toml',
                     *(path / 'Cargo.toml' for path in paths.values())}
    validate_consumer_lock(metadata, consumer / 'Cargo.lock', allowed_local, pinned)
    run(['cargo', '+1.99.0', 'test', '--locked', '--offline', '--manifest-path', str(manifest)],
        output / 'cargo-test.log', cargo_env)
    log = (output / 'cargo-test.log').read_text()
    require('test result: ok. 8 passed; 0 failed;' in log and
            all(f'test original_{name} ... ok' in log for name in
                ('empty', 'round_trip', 'to_field', 'to_tuple', 'to_uint',
                 'escape_sequences', 'non_ascii', 'string')),
            'Bytes original Rust assertions did not pass')
    for case in CASES:
        for mode in ('typescript', 'both'):
            entry = generated[case][mode]
            if case in REFUSALS:
                log_path = output / f'{case.replace("/", "-")}-{mode}-refusal.log'
                require(sha256(log_path) == entry['refusal_sha256'] and
                        not (output / 'generated' / case / mode).exists(),
                        f'Bytes compile refusal artifact changed: {case}/{mode}')
                continue
            runtime_log = output / f'{case.replace("/", "-")}-{mode}-runtime.log'
            require(sha256(runtime_log) == entry['runtime_log_sha256'] and
                    runtime_log.read_text() == entry['runtime_result'] + '\n',
                    f'Bytes runtime evidence changed: {case}/{mode}')
            contract = output / 'generated' / case / mode / 'contract'
            info = output / 'generated' / case / mode / 'compiler/contract-info.json'
            require(sha256(contract / 'index.js') == entry['typescript_sha256'] and
                    sha256(info) == entry['contract_info_sha256'],
                    f'Bytes generated TypeScript artifact changed: {case}/{mode}')
            if mode == 'both':
                require(sha256(contract / 'lib.rs') == entry['rust_sha256'] and
                        sha256(contract / 'compact-rust-ir.json') == entry['rust_ir_sha256'],
                        f'Bytes generated Rust artifact changed: {case}')
    require(source_state() == source and runtime_identity(runtime) == runtime_before and
            compiler_recipe_identity(args.compiler_recipe, source, compiler, scheme) == recipe and
            original_inventory() == originals and
            all(sha256(ROOT / name) == digest for name, digest in local.items()),
            'Bytes source, compiler, runtime or original fixtures changed during gate')
    receipt = {**source, 'source_clean': True,
               'platform': os.uname().sysname + ' ' + os.uname().machine,
               'upstream_commit': PINNED, 'original_files': originals,
               'local_sources': local, 'checker_sha256': sha256(Path(__file__)),
               'compiler_recipe': recipe, 'compiler_sha256': sha256(compiler),
               'scheme_sha256': sha256(scheme), 'runtime': runtime_before,
               'runtime_recipe': runtime_recipe, 'generated': generated,
               'rust_consumer_lock_sha256': sha256(consumer / 'Cargo.lock'),
               'rust_test_log_sha256': sha256(output / 'cargo-test.log'),
               'cargo_target': str(cargo_target),
               'result': 'eight pinned Bytes cast/literal positive originals and three compile refusals passed both targets',
               'limitations': [('native Linux generated semantics qualified' if
                                os.uname().sysname == 'Linux' and os.uname().machine == 'x86_64'
                                else 'local Darwin arm64; native Linux qualification required'),
                               'all eight successful exports are pure/proof=false; proof and funded calls inapplicable',
                               'original Vitest harness was not run; its assertions and refusals were replayed against generated APIs']}
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': 'passed', 'head': source['head'], 'output': str(output)}))


if __name__ == '__main__':
    main()
