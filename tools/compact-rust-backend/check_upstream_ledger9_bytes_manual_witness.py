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

"""Qualify pinned Bytes manual update and witness return on generated TS/Rust."""

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
FAMILY = ROOT / 'test-contracts/primitives/bytes'
PINNED = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'
CASES = ('manual_update', 'witness_return')
MANIFEST = FAMILY / 'manual-witness-original-source-sha256.json'
ADAPTER = FAMILY / 'check-manual-witness.mjs'
CONSUMER = FAMILY / 'consumer-manual-witness.rs'
RESULTS = {
    'manual_update': {'type-name': 'Bytes', 'length': 4},
    'witness_return': {'type-name': 'Tuple', 'types': []},
}
MESSAGES = {
    'manual_update': ('First byte should be 1', 'Third byte should be 14 (64-50)',
                      'Last byte should be 50 (100-50)'),
    'witness_return': ('Witness byte 0 should be 10', 'Witness byte 1 should be 20',
                       'Witness byte 3 should be 40'),
}


def fixture(case: str) -> Path:
    return FAMILY / case / 'semantics' / f'bytes_{case}_semantics.compact'


def original_inventory(path: Path) -> dict[str, str]:
    record = json.loads(path.read_text())
    expected = {str((fixture(case).parent / name).relative_to(ROOT))
                for case in CASES
                for name in (fixture(case).name, 'compile.pass.test.ts',
                             'runtime.pass.test.ts')}
    require(record.get('upstream_commit') == PINNED and
            isinstance(record.get('files'), dict) and set(record['files']) == expected,
            'Bytes manual/witness original inventory changed')
    for name, digest in record['files'].items():
        file = ROOT / name
        original = subprocess.check_output(['git', 'show', f'{PINNED}:{name}'], cwd=ROOT)
        require(file.is_file() and not file.is_symlink() and
                re.fullmatch('[0-9a-f]{64}', digest) is not None and
                sha256(file) == digest == hashlib.sha256(original).hexdigest(),
                f'pinned Bytes original changed: {name}')
    manual = (fixture('manual_update').parent / 'runtime.pass.test.ts').read_text()
    witness = (fixture('witness_return').parent / 'runtime.pass.test.ts').read_text()
    require('expect(result).toEqual(Uint8Array.from([1, 4, 14, 50]))' in manual and
            'Uint8Array.from([10, 20, 30, 40])' in witness and
            'expect(result).toEqual([])' in witness,
            'original Bytes assertions or witness bytes changed')
    for case in CASES:
        source = fixture(case).read_text()
        require(tuple(re.findall(r'assert\([^;]*?,\s*"([^"]+)"\);', source)) ==
                MESSAGES[case], f'original Bytes internal assertions changed: {case}')
    return record['files']


def circuit_info(circuits: list[dict], case: str) -> None:
    require(circuits == [{'name': f'bytes_{case}_semantics', 'pure': False,
                          'proof': case == 'manual_update', 'arguments': [],
                          'result-type': RESULTS[case]}],
            f'Bytes {case} capability or result type changed')


def local_controls(adapter: Path, consumer: Path) -> None:
    js = adapter.read_text()
    rs = consumer.read_text()
    require(all(token in js for token in
                ('[1, 4, 14, 50]', 'Uint8Array.from([10, 20, 30, 40])',
                 '[0, 11, /Witness byte 0 should be 10/]',
                 '[1, 21, /Witness byte 1 should be 20/]',
                 '[3, 41, /Witness byte 3 should be 40/]')) and
            all(token in rs for token in
                ('const EXPECTED: [u8; 4] = [1, 4, 14, 50];',
                 'bytes: [10, 20, 30, 40]',
                 '(0, 11, "Witness byte 0 should be 10")',
                 '(1, 21, "Witness byte 1 should be 20")',
                 '(3, 41, "Witness byte 3 should be 40")')),
            'Bytes manual/witness original result or refusal controls changed')


def generated_assertions(contract: Path, case: str, *, rust: bool) -> None:
    body = contract.read_text()
    marker = 'CompactError::AssertionFailed(' if rust else '__compactRuntime.assert('
    copies = 2 if rust else 1
    # The TypeScript manual-update circuit also emits checked-subtraction
    # assertions. Count the original message-bearing assertions separately.
    require(body.count(marker) >= copies * len(MESSAGES[case]) and
            all(body.count(json.dumps(message) if rust else repr(message)) == copies
                for message in MESSAGES[case]),
            f'generated {"Rust" if rust else "TypeScript"} lost original Bytes assertions: {case}')


def rust_passed(log: str) -> None:
    require(re.search(r'test result: ok\. 3 passed; 0 failed;', log) is not None and
            all(f'test {name} ... ok' in log for name in
                ('original_manual_update_ordinary_and_recorded',
                 'original_witness_return_ordinary_and_recorded',
                 'wrong_witness_bytes_are_refused_in_both_modes')),
            'Bytes manual/witness Rust ordinary/recorded tests did not pass')


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'compiler-recipe', 'ts-runtime', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
    args = parser.parse_args()
    source = source_state()
    signature = subprocess.check_output(['git', 'log', '-1', '--format=%G?%n%B'],
                                        cwd=ROOT, text=True)
    require(signature.startswith('G\n') and 'Signed-off-by:' in signature and
            'Assisted-by: AI' in signature, 'head must be signed, DCO and assisted')
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
    originals = original_inventory(MANIFEST)
    local_controls(ADAPTER, CONSUMER)
    local = {str(path.relative_to(ROOT)): sha256(path)
             for path in (MANIFEST, ADAPTER, CONSUMER)}
    output.mkdir(parents=True)
    link = output / 'node_modules/@midnight-ntwrk/compact-runtime'
    link.parent.mkdir(parents=True)
    link.symlink_to(runtime, target_is_directory=True)
    (output / 'package.json').write_text('{"type":"module"}\n')
    env = dict(os.environ, COMPACTC_SCHEME=str(scheme), RUSTUP_TOOLCHAIN='1.99.0')
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
                        str(ROOT / 'compiler'), str(fixture(case)), str(destination)]
            run(command, output / f'{case}-{mode}-compile.log', env)
            check_manifest(destination, require_zkir=False)
            require(not (destination / 'zkir').exists(), 'skip-zk emitted proof material')
            info = destination / 'compiler/contract-info.json'
            circuit_info(json.loads(info.read_text())['circuits'], case)
            contract = destination / 'contract'
            generated_assertions(contract / 'index.js', case, rust=False)
            if mode == 'both':
                generated_assertions(contract / 'lib.rs', case, rust=True)
            runtime_log = output / f'{case}-{mode}-runtime.log'
            result = run(['node', str(ADAPTER), str(contract / 'index.js'), case],
                         runtime_log, env)
            require(result.strip() == f'{case}: original Bytes semantics passed',
                    f'Bytes {case} TypeScript assertion changed')
            row = {'typescript_sha256': sha256(contract / 'index.js'),
                   'manifest_sha256': sha256(destination / 'compiler/contract-manifest.json'),
                   'contract_info_sha256': sha256(info),
                   'runtime_log_sha256': sha256(runtime_log)}
            if mode == 'both':
                row['rust_sha256'] = sha256(contract / 'lib.rs')
                row['rust_ir_sha256'] = sha256(contract / 'compact-rust-ir.json')
            generated[case][mode] = row
        require(all(generated[case]['typescript'][key] == generated[case]['both'][key]
                    for key in ('typescript_sha256', 'contract_info_sha256',
                                'runtime_log_sha256')),
                f'paired compilation changed TypeScript behavior: {case}')
    consumer = output / 'rust-consumer'
    (consumer / 'tests').mkdir(parents=True)
    shutil.copyfile(CONSUMER, consumer / 'tests/bytes_manual_witness.rs')
    shutil.copyfile(ROOT / 'Cargo.lock', consumer / 'Cargo.lock')
    dependencies = []
    generated_paths = []
    for case in CASES:
        contract = output / 'generated' / case / 'both/contract'
        generated_paths.append(contract)
        package = tomllib.loads((contract / 'Cargo.toml').read_text())['package']['name']
        dependencies.append(f'{case} = {{ package = {json.dumps(package)}, '
                            f'path = {json.dumps(str(contract))} }}')
    manifest = consumer / 'Cargo.toml'
    manifest.write_text('[workspace]\n[package]\nname = "ledger9-bytes-manual-witness-consumer"\n'
                        'version = "0.1.0"\nedition = "2024"\n[dependencies]\n'
                        'midnight-onchain-vm = "=4.0.1"\nmidnight-serialize = "=1.2.1"\n' +
                        '\n'.join(dependencies) + '\n')
    cargo_env = dict(env, CARGO_TARGET_DIR=str(output / 'cargo-target'), CARGO_INCREMENTAL='0',
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
    run(['cargo', '+1.99.0', 'test', '--locked', '--offline', '--manifest-path', str(manifest)],
        output / 'cargo-test.log', cargo_env)
    rust_passed((output / 'cargo-test.log').read_text())
    require(source_state() == source and runtime_identity(runtime) == runtime_before and
            compiler_recipe_identity(args.compiler_recipe, source, compiler, scheme) == recipe and
            original_inventory(MANIFEST) == originals and
            local_controls(ADAPTER, CONSUMER) is None and
            all(sha256(ROOT / name) == digest for name, digest in local.items()),
            'source, compiler, runtime or originals changed during qualification')
    for case in CASES:
        for mode in ('typescript', 'both'):
            destination = output / 'generated' / case / mode
            contract = destination / 'contract'
            row = generated[case][mode]
            require(sha256(contract / 'index.js') == row['typescript_sha256'] and
                    sha256(destination / 'compiler/contract-info.json') == row['contract_info_sha256'] and
                    sha256(destination / 'compiler/contract-manifest.json') == row['manifest_sha256'] and
                    sha256(output / f'{case}-{mode}-runtime.log') == row['runtime_log_sha256'],
                    f'generated TypeScript artifact changed: {case}/{mode}')
            if mode == 'both':
                require(sha256(contract / 'lib.rs') == row['rust_sha256'] and
                        sha256(contract / 'compact-rust-ir.json') == row['rust_ir_sha256'],
                        f'generated Rust artifact changed: {case}')
    receipt = {**source, 'status': 'passed', 'source_clean': True,
               'platform': os.uname().sysname + ' ' + os.uname().machine,
               'upstream_commit': PINNED, 'original_files': originals,
               'local_sources': local, 'generated': generated,
               'compiler_recipe': recipe, 'compiler_sha256': sha256(compiler),
               'scheme_sha256': sha256(scheme), 'runtime': runtime_before,
               'runtime_recipe': runtime_recipe,
               'checker_sha256': sha256(Path(__file__)),
               'rust_consumer_lock_sha256': sha256(consumer / 'Cargo.lock'),
               'rust_test_log_sha256': sha256(output / 'cargo-test.log'),
               'result': 'two original Bytes semantics passed generated TS and Rust ordinary/recorded APIs',
               'limitations': ['local Darwin arm64; native Linux qualification required',
                               'manual update has proof=true: fresh independent proof and funded strict application/replay remain separate gates',
                               'witness return has proof=false: no proof/funded call applies',
                               'original Vitest harness was not run; pinned assertions were replayed against generated APIs']}
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': 'passed', 'head': source['head'], 'output': str(output)}))


if __name__ == '__main__':
    main()
