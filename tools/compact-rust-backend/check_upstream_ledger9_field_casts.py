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

"""Qualify all pinned Field cast originals and the checked Bytes<4> boundary."""

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


ROOT = Path(__file__).resolve().parents[2]
FAMILY = ROOT / 'test-contracts/primitives/field/casts'
PINNED = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'
CASES = ('to_boolean', 'to_bytes', 'to_enum', 'to_enum_out_of_range',
         'to_uint', 'to_uint_max_field', 'to_uint_out_of_range')
ALL_CASES = (*CASES, 'bytes4_boundary')
MANIFEST = FAMILY / 'original-source-sha256.json'
ADAPTER = FAMILY / 'check.mjs'
CONSUMER = FAMILY / 'consumer.rs'
BOUNDARY = FAMILY / 'boundary.compact'
UNSUPPORTED = FAMILY / 'unsupported-width.compact'
FIELD = {'type-name': 'Field'}
COLOR = {'type-name': 'Enum', 'name': 'Color', 'elements': ['red', 'green', 'blue']}
RESULTS = {
    'to_boolean': {'type-name': 'Boolean'},
    'to_bytes': {'type-name': 'Bytes', 'length': 4},
    'to_enum': COLOR,
    'to_enum_out_of_range': COLOR,
    'to_uint': {'type-name': 'Uint', 'maxval': 2**64 - 1},
    'to_uint_max_field': {'type-name': 'Uint', 'maxval': 2**248 - 1},
    'to_uint_out_of_range': {'type-name': 'Uint', 'maxval': 2**64 - 1},
    'bytes4_boundary': {'type-name': 'Bytes', 'length': 4},
}
EXPECTED_RUNTIME_SNIPPETS = {
    'to_boolean': ('expect(result).toBe(true)',),
    'to_bytes': ('expect(Array.from(result)).toEqual([1, 0, 0, 0])',),
    'to_enum': tuple(f'expect(pure.field_casts_to_enum({i}n)).toBe({i})' for i in range(3)),
    'to_enum_out_of_range': ('pure.field_casts_to_enum_out_of_range(3n)',
                             '3 is greater than maximum enum value 2'),
    'to_uint': ('VALUES = [0n, 1n, 2n ** 64n - 1n]',
                'expect(pure.field_casts_to_uint(value)).toBe(value)'),
    'to_uint_max_field': ('pure.field_casts_to_uint_max_field(MAX_FIELD)',
                          'MAX_FIELD} is greater than ${2n ** 248n - 1n}'),
    'to_uint_out_of_range': ('pure.field_casts_to_uint_out_of_range(2n ** 64n)',
                             '18446744073709551616 is greater than 18446744073709551615'),
}


def require(value: bool, message: str) -> None:
    if not value:
        raise RuntimeError(message)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_state() -> dict[str, str]:
    def git(*args: str) -> str:
        return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()
    require(not git('status', '--porcelain'), 'clean committed source required')
    return {'head': git('rev-parse', 'HEAD'), 'tree': git('rev-parse', 'HEAD^{tree}')}


def original_inventory(path: Path) -> dict[str, str]:
    manifest = json.loads(path.read_text())
    expected = {f'test-contracts/primitives/field/casts/{case}/{name}'
                for case in CASES
                for name in (f'field_casts_{case}.compact', 'compile.pass.test.ts',
                             'runtime.fail.test.ts' if case in
                             ('to_enum_out_of_range', 'to_uint_max_field', 'to_uint_out_of_range')
                             else 'runtime.pass.test.ts')}
    require(manifest.get('upstream_commit') == PINNED and
            isinstance(manifest.get('files'), dict) and set(manifest['files']) == expected,
            'Field cast original inventory changed')
    for name, digest in manifest['files'].items():
        source = ROOT / name
        original = subprocess.check_output(['git', 'show', f'{PINNED}:{name}'], cwd=ROOT)
        require(source.is_file() and not source.is_symlink() and
                re.fullmatch('[0-9a-f]{64}', digest) is not None and
                sha256(source) == digest == hashlib.sha256(original).hexdigest(),
                f'Field cast original changed: {name}')
    for case, snippets in EXPECTED_RUNTIME_SNIPPETS.items():
        name = 'runtime.fail.test.ts' if case in ('to_enum_out_of_range',
                'to_uint_max_field', 'to_uint_out_of_range') else 'runtime.pass.test.ts'
        body = (FAMILY / case / name).read_text()
        require(all(part in body for part in snippets),
                f'Field cast original assertion changed: {case}')
    return manifest['files']


def runtime_identity(runtime: Path) -> dict:
    runtime = runtime.resolve(strict=True)
    dist = runtime / 'dist'
    require((dist / 'index.js').is_file(), 'TypeScript runtime dist/index.js missing')
    files = {}
    for path in sorted(dist.rglob('*')):
        require(not path.is_symlink(), 'runtime dist contains symlink')
        if path.is_dir():
            continue
        require(path.is_file(), 'runtime dist contains unsupported entry')
        files[str(path.relative_to(dist))] = sha256(path)
    return {'path': str(runtime), 'package_sha256': sha256(runtime / 'package.json'),
            'dist_files': files}


def runtime_recipe_identity(runtime: Path, nix: Path) -> dict:
    command = [str(nix), 'eval', '--offline', '--raw', '--option',
               'substitute', 'false', '.#runtime.package.outPath']
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
    require(result.returncode == 0 and result.stdout.strip() and
            '\n' not in result.stdout.strip(), 'offline runtime Nix recipe failed')
    store = Path(result.stdout.strip()).resolve(strict=True)
    expected = (store / 'lib/node_modules/@midnight-ntwrk/compact-runtime').resolve(strict=True)
    require(runtime == expected, 'runtime differs from exact-head Nix output')
    return {'command': command, 'store': str(store), 'path': str(expected),
            'nix_sha256': sha256(nix)}


def compiler_recipe_identity(path: Path, source: dict[str, str],
                             compiler: Path, scheme: Path) -> dict:
    path = path.resolve(strict=True)
    recipe = json.loads(path.read_text())
    require(recipe.get('format') == 'compact-original-existing-compiler-recipe/v1' and
            recipe.get('status') == 'passed' and
            all(recipe.get(key) == source[key] for key in ('head', 'tree')),
            'compiler recipe does not bind exact source')
    for name, binary in (('compactc', compiler), ('compactc-scheme', scheme)):
        record = recipe.get(name, {})
        require(record.get('path') == str(binary) and
                record.get('sha256') == sha256(binary),
                f'compiler recipe {name} differs')
    require(recipe.get('store') == str(compiler.parent.parent),
            'compiler recipe package differs')
    pins = recipe.get('inputs_before_use', {})
    for name in ('flake.nix', 'flake.lock', 'Cargo.lock',
                 'tools/compact-rust-backend/Cargo.lock', 'compiler/compiler-version.ss',
                 'tools/compact-rust-backend/record_compiler_recipe.py'):
        source_path = (ROOT / name).resolve(strict=True)
        require(pins.get(str(source_path)) == sha256(source_path),
                f'compiler recipe source pin differs: {name}')
    return {'path': str(path), 'sha256': sha256(path),
            **source, 'store': recipe['store']}


def circuit_info(rows: list[dict], case: str) -> None:
    name = f'field_casts_{case}'
    arguments = [] if case in ('to_boolean', 'to_bytes') else [{'name': 'value', 'type': FIELD}]
    require(rows == [{'name': name, 'pure': True, 'proof': False,
                      'arguments': arguments, 'result-type': RESULTS[case]}],
            f'Field cast capability or result type changed: {case}')


def generated_assertions(source: Path, typescript: Path, rust: Path | None) -> None:
    messages = re.findall(r'assert\([^;]*?,\s*"([^"]+)"\);', source.read_text())
    js = typescript.read_text()
    require(js.count('__compactRuntime.assert(') == len(messages) and
            all(js.count(repr(message)) == messages.count(message)
                for message in set(messages)),
            'generated TypeScript lost a Field cast assertion')
    if rust is not None:
        rs = rust.read_text()
        require(rs.count('CompactError::AssertionFailed(') == len(messages) and
                all(rs.count(json.dumps(message)) == messages.count(message)
                    for message in set(messages)),
                'generated Rust lost a Field cast assertion')


def run(command: list[str], log: Path, env: dict[str, str]) -> str:
    result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True)
    log.write_text(result.stdout + result.stderr)
    require(result.returncode == 0,
            f'{command[0]} exited {result.returncode}: {log}\n{result.stderr[-1800:]}')
    return result.stdout


def source_for(case: str) -> Path:
    return BOUNDARY if case == 'bytes4_boundary' else FAMILY / case / f'field_casts_{case}.compact'


def checked_boundary_sources(boundary: Path, unsupported: Path,
                             adapter: Path, consumer: Path) -> None:
    require('return value as Bytes<4>;' in boundary.read_text() and
            'return value as Bytes<16>;' in unsupported.read_text() and
            all(token in adapter.read_text() for token in
                ('0x01020304n', '0xffffffffn', '0x100000000n', '2n ** 40n')) and
            all(token in consumer.read_text() for token in
                ('0x01020304u128', '0xffffffffu128', '0x100000000u128', '1u128 << 40')),
            'checked Field-to-Bytes<4> boundary or Bytes<16> refusal source changed')


def require_refusal(result: subprocess.CompletedProcess[str], destination: Path) -> str:
    message = result.stdout + result.stderr
    lock = destination.parent / f'.{destination.name}.compactc.lock'
    require(result.returncode != 0 and result.stdout == '' and
            'Rust backend supports Field-to-Bytes<32> casts only' in result.stderr and
            not destination.exists() and
            not list(destination.parent.glob(f'.{destination.name}.compactc-stage-*')) and
            (not lock.exists() or
             (lock.is_file() and not lock.is_symlink() and lock.stat().st_size == 0)),
            'direct Field-to-Bytes<16> refusal or atomic output changed')
    return message


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
            'Assisted-by: AI' in commit, 'head must be signed, DCO and assisted')
    compiler = args.compiler.resolve(strict=True)
    scheme = compiler.with_name('compactc-scheme').resolve(strict=True)
    runtime = args.ts_runtime.resolve(strict=True)
    output = args.output.resolve()
    require(not args.output.is_symlink() and not output.exists(), 'output must be absent')
    source_package = json.loads((ROOT / 'runtime/package.json').read_text())
    supplied_package = json.loads((runtime / 'package.json').read_text())
    require(all(source_package[k] == supplied_package[k]
                for k in ('name', 'version', 'dependencies')),
            'runtime package differs from source')
    nix = shutil.which('nix')
    require(nix is not None, 'Nix required for runtime source binding')
    runtime_recipe = runtime_recipe_identity(runtime, Path(nix).resolve(strict=True))
    runtime_before = runtime_identity(runtime)
    recipe = compiler_recipe_identity(args.compiler_recipe, source, compiler, scheme)
    originals = original_inventory(MANIFEST)
    checked_boundary_sources(BOUNDARY, UNSUPPORTED, ADAPTER, CONSUMER)
    local_sources = {name: sha256(path) for name, path in
                     (('boundary', BOUNDARY), ('unsupported_width', UNSUPPORTED),
                      ('adapter', ADAPTER), ('consumer', CONSUMER))}
    output.mkdir(parents=True)
    link = output / 'node_modules/@midnight-ntwrk/compact-runtime'
    link.parent.mkdir(parents=True)
    link.symlink_to(runtime, target_is_directory=True)
    (output / 'package.json').write_text('{"type":"module"}\n')
    env = dict(os.environ, COMPACTC_SCHEME=str(scheme))
    generated = {}
    for case in ALL_CASES:
        result = {}
        for mode in ('typescript', 'both'):
            destination = output / 'generated' / case / mode
            command = [str(compiler), '--target', 'ts']
            if mode == 'both':
                command += ['--target', 'rust', '--rust-runtime-root', str(ROOT),
                            '--rust-require-recording']
            command += ['--feature-zkir-v3', '--skip-zk', '--compact-path',
                        str(ROOT / 'compiler'), str(source_for(case)), str(destination)]
            run(command, output / f'{case}-{mode}-compile.log', env)
            info = destination / 'compiler/contract-info.json'
            circuit_info(json.loads(info.read_text())['circuits'], case)
            contract = destination / 'contract'
            rust = contract / 'lib.rs' if mode == 'both' else None
            generated_assertions(source_for(case), contract / 'index.js', rust)
            runtime_log = output / f'{case}-{mode}-runtime.log'
            value = run(['node', str(ADAPTER), str(contract / 'index.js'), case],
                        runtime_log, env)
            require(value.strip() == f'{case}: original or boundary assertions passed',
                    f'Field cast adapter result changed: {case}')
            entry = {'typescript_sha256': sha256(contract / 'index.js'),
                     'contract_info_sha256': sha256(info), 'runtime_result': value.strip(),
                     'runtime_log_sha256': sha256(runtime_log)}
            if rust is not None:
                entry['rust_sha256'] = sha256(rust)
                entry['rust_ir_sha256'] = sha256(contract / 'compact-rust-ir.json')
            result[mode] = entry
        require(result['typescript']['typescript_sha256'] ==
                result['both']['typescript_sha256'] and
                result['typescript']['contract_info_sha256'] ==
                result['both']['contract_info_sha256'],
                f'paired compiler changed TypeScript output: {case}')
        generated[case] = result
    control = output / 'unsupported-typescript'
    run([str(compiler), '--target', 'ts', '--feature-zkir-v3', '--skip-zk',
         '--compact-path', str(ROOT / 'compiler'), str(UNSUPPORTED), str(control)],
        output / 'unsupported-typescript-compile.log', env)
    require(json.loads((control / 'compiler/contract-info.json').read_text())['circuits'] ==
            [{'name': 'field_casts_bytes16_refusal', 'pure': True, 'proof': False,
              'arguments': [{'name': 'value', 'type': FIELD}],
              'result-type': {'type-name': 'Bytes', 'length': 16}}],
            'Field-to-Bytes<16> valid TS control changed')
    refusal = output / 'unsupported-paired'
    command = [str(compiler), '--target', 'ts', '--target', 'rust',
               '--rust-runtime-root', str(ROOT), '--feature-zkir-v3', '--skip-zk',
               '--compact-path', str(ROOT / 'compiler'), str(UNSUPPORTED), str(refusal)]
    failed = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True)
    (output / 'unsupported-paired-refusal.log').write_text(failed.stdout + failed.stderr)
    require_refusal(failed, refusal)
    consumer = output / 'rust-consumer'
    (consumer / 'tests').mkdir(parents=True)
    shutil.copyfile(CONSUMER, consumer / 'tests/field_casts.rs')
    shutil.copyfile(ROOT / 'Cargo.lock', consumer / 'Cargo.lock')
    paths = {case: output / 'generated' / case / 'both/contract' for case in ALL_CASES}
    manifest = consumer / 'Cargo.toml'
    dependencies = []
    for case, path in paths.items():
        package = tomllib.loads((path / 'Cargo.toml').read_text())['package']['name']
        alias = 'boundary' if case == 'bytes4_boundary' else case
        dependencies.append(f'{alias} = {{ package = {json.dumps(package)}, '
                            f'path = {json.dumps(str(path))} }}')
    manifest.write_text('[workspace]\n[package]\nname = "ledger9-field-casts-consumer"\n'
                        'version = "0.1.0"\nedition = "2024"\n[dependencies]\n' +
                        '\n'.join(dependencies) + '\n')
    require(args.cargo_target is None or
            (args.cargo_target.is_absolute() and not args.cargo_target.is_symlink()),
            'shared Cargo target must be an absolute non-symlink path')
    target = args.cargo_target.resolve() if args.cargo_target else output / 'cargo-target'
    cargo_env = dict(env, CARGO_TARGET_DIR=str(target), CARGO_INCREMENTAL='0',
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
    test_log = (output / 'cargo-test.log').read_text()
    require(re.search(r'test result: ok\. 8 passed; 0 failed;', test_log) is not None and
            all(f'test original_{case} ... ok' in test_log for case in CASES) and
            'test checked_bytes4_boundaries ... ok' in test_log,
            'Field cast original/boundary Rust tests did not pass')
    for case in ALL_CASES:
        for mode in ('typescript', 'both'):
            destination = output / 'generated' / case / mode
            contract = destination / 'contract'
            entry = generated[case][mode]
            runtime_log = output / f'{case}-{mode}-runtime.log'
            require(sha256(runtime_log) == entry['runtime_log_sha256'] and
                    runtime_log.read_text() == entry['runtime_result'] + '\n',
                    f'Field cast runtime evidence changed: {case}/{mode}')
            require(sha256(contract / 'index.js') == entry['typescript_sha256'] and
                    sha256(destination / 'compiler/contract-info.json') ==
                    entry['contract_info_sha256'],
                    f'Field cast generated output changed during Rust test: {case}/{mode}')
            rust = contract / 'lib.rs' if mode == 'both' else None
            generated_assertions(source_for(case), contract / 'index.js', rust)
            if rust is not None:
                require(sha256(rust) == entry['rust_sha256'] and
                        sha256(contract / 'compact-rust-ir.json') == entry['rust_ir_sha256'],
                        f'Field cast generated Rust changed: {case}')
    require(source_state() == source and runtime_identity(runtime) == runtime_before and
            compiler_recipe_identity(args.compiler_recipe, source, compiler, scheme) == recipe and
            original_inventory(MANIFEST) == originals and
            checked_boundary_sources(BOUNDARY, UNSUPPORTED, ADAPTER, CONSUMER) is None and
            all(sha256(path) == local_sources[name] for name, path in
                (('boundary', BOUNDARY), ('unsupported_width', UNSUPPORTED),
                 ('adapter', ADAPTER), ('consumer', CONSUMER))),
            'Field cast source, compiler or runtime changed during qualification')
    receipt = {**source, 'source_clean': True, 'platform': os.uname().sysname + ' ' + os.uname().machine,
               'upstream_commit': PINNED, 'original_files': originals,
               'manifest_sha256': sha256(MANIFEST), 'local_sources': local_sources,
               'checker_sha256': sha256(Path(__file__)),
               'compiler_recipe': recipe, 'compiler_sha256': sha256(compiler),
               'scheme_sha256': sha256(scheme), 'runtime': runtime_before,
               'runtime_recipe': runtime_recipe, 'generated': generated,
               'unsupported_typescript_sha256': sha256(control / 'contract/index.js'),
               'unsupported_refusal_log_sha256': sha256(output / 'unsupported-paired-refusal.log'),
               'rust_consumer_lock_sha256': sha256(consumer / 'Cargo.lock'),
               'rust_test_log_sha256': sha256(output / 'cargo-test.log'),
               'cargo_target': str(target),
               'result': 'seven pinned Field cast cases and checked Bytes<4> boundaries passed both targets; Bytes<16> Rust refusal preserved',
               'limitations': [('native Linux generated semantics qualified' if
                                os.uname().sysname == 'Linux' and os.uname().machine == 'x86_64'
                                else 'local Darwin arm64; native Linux qualification required'),
                               'all seven originals are pure/proof=false; no proof or funded call applies',
                               'original Vitest harness was not run; pinned assertions and failures were replayed against generated APIs']}
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': 'passed', 'head': source['head'], 'output': str(output)}))


if __name__ == '__main__':
    main()
