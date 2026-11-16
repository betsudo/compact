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

"""Run intact pinned import/type keyword tests through TS and paired Rust."""

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
    compiler_recipe_identity, runtime_identity, runtime_recipe_identity,
    sha256, source_state,
)


ROOT = Path(__file__).resolve().parents[2]
PINNED = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'
FAMILY = Path('test-contracts/keywords')
MANIFEST = ROOT / FAMILY / 'import-type-original-source-sha256.json'
EXPECTED_ERROR = re.compile(r'expectedError:\s*/((?:\\.|[^/])*)/')
BLOCKED = set()

# The original Vitest callbacks and assertions are copied byte for byte. These
# narrow adapters supply the older released synchronous runtime context shape.
VITEST_SHIM = (
    "import {deepStrictEqual} from 'node:assert';\n"
    "export function expect(actual) { return {toEqual(expected) "
    "{deepStrictEqual(actual, expected)}}; }\n"
)
CONTRACT_SHIM = (
    "import {createCircuitContext,createConstructorContext,dummyContractAddress} "
    "from '@midnight-ntwrk/compact-runtime';\n"
    "export function defineRuntimeTest(_url,run){return {run}};\n"
    "export async function createTestContract(Contract){"
    "const contract=new Contract({});"
    "const initial=await contract.initialState("
    "createConstructorContext(undefined,'0'.repeat(64)));"
    "const ctx=createCircuitContext(dummyContractAddress(),"
    "initial.currentZswapLocalState.coinPublicKey,"
    "initial.currentContractState,initial.currentPrivateState);"
    "return {contract,ctx}};\n"
)
RUNNER = (
    "import definition from './runtime.pass.test.ts';\n"
    "import {Contract} from './product/contract/index.js';\n"
    "await definition.run(Contract); console.log('original runtime passed');\n"
)
RUST_REPRESENTATIVES = {
    'import/module_from_file': (
        'expect(result).toEqual(2n);',
        'assert_eq!(generated::pure_circuits::import_module_from_file()'
        '.unwrap().value(), 2);',
    ),
    'import/alias_shadows_stdlib_name': (
        'expect(result).toEqual(1n);',
        'assert_eq!(generated::pure_circuits::import_alias_shadows_stdlib_name()'
        '.unwrap(), generated::runtime::Field::from(1u128));',
    ),
    'import/underscore_names': (
        'expect(result).toEqual(2n);',
        'assert_eq!(generated::pure_circuits::__compact_export_underscore()'
        '.unwrap(), generated::runtime::Field::from(1u128));'
        'assert_eq!(generated::pure_circuits::__().unwrap(), '
        'generated::runtime::Field::from(1u128));'
        'assert_eq!(generated::pure_circuits::import_underscore_names()'
        '.unwrap(), generated::runtime::Field::from(2u128));',
    ),
    'type/new_type_keeps_operations': (
        'expect(result).toEqual([true, true, 5n]);',
        'let a = generated::runtime::BoundedUint::<65535>::new(3u64.into())'
        '.unwrap(); let b = generated::runtime::BoundedUint::<65535>::new('
        '2u64.into()).unwrap(); assert_eq!(generated::pure_circuits::'
        'type_new_type_keeps_operations(a, b).unwrap(), '
        '(true, true, generated::runtime::Field::from(5u128)));',
    ),
    'type/new_type_of_adt': (
        'expect(result).toEqual(5n);',
        'use generated::{ledger_contract as contract, ledger_slots as slots};'
        'let context = || contract::initial_state('
        'generated::runtime::context::ConstructorContext::new(()))'
        '.unwrap().into_circuit_context(Default::default());'
        'let native = contract::type_new_type_of_adt(context()).unwrap();'
        'let recorded = contract::recorded::type_new_type_of_adt(context())'
        '.unwrap();'
        'assert_eq!(native.result.value(), 5);'
        'assert_eq!(recorded.execution.result.value(), 5);'
        'assert_eq!(slots::c.inspect(native.context.query.state.get_ref())'
        '.unwrap(), 5);'
        'assert_eq!(slots::c.inspect(recorded.execution.context.query.state'
        '.get_ref()).unwrap(), 5);'
        'assert_eq!(native.context.query.state, '
        'recorded.execution.context.query.state);'
        'assert_eq!(native.gas_cost, recorded.execution.gas_cost);',
    ),
}


def require(ok: bool, message: str) -> None:
    if not ok:
        raise RuntimeError(message)


def run(command: list[str], path: Path, env: dict[str, str]) -> str:
    result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True,
                            text=True)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(result.stdout + result.stderr)
    require(result.returncode == 0, f'command failed: {command}: {path}')
    return result.stdout


def original_inventory() -> tuple[dict[str, str], dict[str, str]]:
    manifest = json.loads(MANIFEST.read_text())
    ported = manifest.get('ported_files')
    blocked = manifest.get('blocked_files')
    cases = manifest.get('blocked_cases')
    expected = subprocess.check_output(
        ['git', 'ls-tree', '-r', '--name-only', PINNED,
         str(FAMILY / 'import'), str(FAMILY / 'type')], cwd=ROOT,
        text=True).splitlines()
    require(manifest.get('upstream_commit') == PINNED and
            isinstance(ported, dict) and isinstance(blocked, dict) and
            isinstance(cases, dict) and set(cases) == BLOCKED and
            len(ported) == 99 and len(blocked) == 0 and
            sorted(set(ported) | set(blocked)) == expected and
            set(ported).isdisjoint(blocked),
            'pinned import/type coverage roster differs')
    for name in expected:
        pin = subprocess.check_output(['git', 'show', f'{PINNED}:{name}'], cwd=ROOT)
        digest = hashlib.sha256(pin).hexdigest()
        mapped = ported if name in ported else blocked
        path = ROOT / name
        require(mapped[name] == digest and re.fullmatch(r'[0-9a-f]{64}', digest)
                is not None and (path.is_file() and not path.is_symlink()
                                 and sha256(path) == digest if name in ported
                                 else not path.exists()),
                f'pinned original changed or blocked case unexpectedly staged: {name}')
    selected_cases = {'/'.join(Path(name).relative_to(FAMILY).parts[:2])
                      for name in ported}
    require(len(selected_cases) == 39 and selected_cases.isdisjoint(BLOCKED),
            'selected case roster differs')
    for case in selected_cases:
        directory = ROOT / FAMILY / case
        require(len(list(directory.glob('*.compact'))) == 1 and
                len(list(directory.glob('compile.*.test.ts'))) == 1,
                f'original case incomplete: {case}')
    return ported, blocked


def error_pattern(test: Path) -> str:
    matches = EXPECTED_ERROR.findall(test.read_text())
    require(len(matches) == 1 and matches[0].startswith('line '),
            f'expected one pinned diagnostic regex: {test}')
    return matches[0]


def refuse(result: subprocess.CompletedProcess[str], destination: Path,
           pattern: str) -> str:
    diagnostic = result.stdout + result.stderr
    require(result.returncode != 0 and result.stdout == '' and
            re.search(pattern, diagnostic) is not None and
            re.search(pattern, diagnostic.replace('line ', 'row ', 1)) is None and
            not destination.exists() and
            not list(destination.parent.glob(f'.{destination.name}.compactc-stage-*')),
            f'original failure or drift control changed: {destination}')
    return diagnostic


def make_adapter(destination: Path, test: Path, runtime: Path) -> None:
    shutil.copyfile(test, destination / 'runtime.pass.test.ts')
    (destination / 'package.json').write_text('{"type":"module"}\n')
    (destination / 'run.mjs').write_text(RUNNER)
    modules = destination / 'node_modules'
    package = modules / '@midnight-ntwrk/compact-runtime'
    package.parent.mkdir(parents=True)
    package.symlink_to(runtime, target_is_directory=True)
    for name, source in (('vitest', VITEST_SHIM),
                         ('@test/compact-test', CONTRACT_SHIM)):
        directory = modules / name
        directory.mkdir(parents=True)
        (directory / 'package.json').write_text(
            '{"type":"module","exports":"./index.js"}\n')
        (directory / 'index.js').write_text(source)


def locked_rust_check(contract: Path, output: Path, env: dict[str, str],
                      expected_registry: set[tuple]) -> dict[str, str]:
    lock = contract / 'Cargo.lock'
    shutil.copyfile(ROOT / 'Cargo.lock', lock)
    manifest = contract / 'Cargo.toml'
    run(['cargo', '+1.99.0', 'metadata', '--offline', '--format-version', '1',
         '--manifest-path', str(manifest)],
        output / 'cargo-metadata.log', env)
    actual = tomllib.loads(lock.read_text())['package']
    registry = {(row['name'], row['version'], row['source'],
                 row.get('checksum')) for row in actual if 'source' in row}
    require(registry <= expected_registry,
            f'generated Rust lock left workspace registry graph: {contract}')
    run(['cargo', '+1.99.0', 'check', '--locked', '--offline',
         '--manifest-path', str(manifest)], output / 'cargo-check.log', env)
    return {'manifest_sha256': sha256(manifest), 'lock_sha256': sha256(lock),
            'cargo_check_sha256': sha256(output / 'cargo-check.log')}


def representative_rust_execution(case: str, contract: Path, test: Path,
                                  output: Path, env: dict[str, str]) -> dict[str, str]:
    expected, body = RUST_REPRESENTATIVES[case]
    require(expected in test.read_text(),
            f'Rust representative lost original assertion: {case}')
    package = tomllib.loads((contract / 'Cargo.toml').read_text())['package']['name']
    crate = package.replace('-', '_')
    consumer = contract / 'tests/original_keywords.rs'
    consumer.parent.mkdir()
    consumer.write_text('use ' + crate + ' as generated;\n'
                        '#[test] fn pinned_original_result() { ' + body + ' }\n')
    log = output / 'cargo-test.log'
    run(['cargo', '+1.99.0', 'test', '--locked', '--offline',
         '--manifest-path', str(contract / 'Cargo.toml'),
         '--test', 'original_keywords'], log, env)
    require(re.search(r'test result: ok\. 1 passed; 0 failed;',
                      log.read_text()) is not None,
            f'Rust representative test count changed: {case}')
    return {'rust_execution_test_sha256': sha256(consumer),
            'rust_execution_log_sha256': sha256(log),
            'original_assertion': expected}


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
            'Assisted-by: AI' in commit, 'source head must be signed and DCO complete')
    compiler = args.compiler.resolve(strict=True)
    scheme = compiler.with_name('compactc-scheme').resolve(strict=True)
    runtime = args.ts_runtime.resolve(strict=True)
    output = args.output.resolve()
    require(not args.output.is_symlink() and not output.exists(),
            'output must be absent')
    recipe = compiler_recipe_identity(args.compiler_recipe, source,
                                      compiler, scheme)
    nix = shutil.which('nix')
    require(nix is not None, 'Nix required for exact-head runtime binding')
    runtime_recipe = runtime_recipe_identity(runtime, Path(nix).resolve())
    runtime_before = runtime_identity(runtime)
    package = json.loads((ROOT / 'runtime/package.json').read_text())
    supplied = json.loads((runtime / 'package.json').read_text())
    require(all(package[k] == supplied[k]
                for k in ('name', 'version', 'dependencies')),
            'runtime package differs from source')
    originals, blocked = original_inventory()
    manifest_hash = sha256(MANIFEST)
    checker_hash = sha256(Path(__file__))
    cases = sorted({'/'.join(Path(name).relative_to(FAMILY).parts[:2])
                    for name in originals})
    expected_registry = {(row['name'], row['version'], row['source'],
                          row.get('checksum'))
                         for row in tomllib.loads((ROOT / 'Cargo.lock').read_text())['package']
                         if 'source' in row}
    output.mkdir(parents=True)
    env = dict(os.environ, COMPACTC_SCHEME=str(scheme))
    cargo_env = dict(env, CARGO_INCREMENTAL='0', CARGO_PROFILE_DEV_DEBUG='0',
                     CARGO_TARGET_DIR=str(args.cargo_target.resolve()
                                          if args.cargo_target else output / 'cargo-target'))
    results = {}
    for case in cases:
        directory = ROOT / FAMILY / case
        contract = next(directory.glob('*.compact'))
        test = next(directory.glob('compile.*.test.ts'))
        positive = test.name == 'compile.pass.test.ts'
        pattern = None if positive else error_pattern(test)
        results[case] = {}
        for mode in ('typescript', 'paired'):
            destination = output / case / mode / 'product'
            destination.parent.mkdir(parents=True)
            command = [str(compiler), '--target', 'ts']
            if mode == 'paired':
                command += ['--target', 'rust', '--rust-runtime-root', str(ROOT),
                            '--rust-require-recording']
            command += ['--feature-zkir-v3', '--skip-zk', '--compact-path',
                        str(ROOT / 'compiler'), str(contract), str(destination)]
            result = subprocess.run(command, cwd=ROOT, env=env,
                                    capture_output=True, text=True)
            row = {'original_compile_test': str(test.relative_to(ROOT)),
                   'command': command, 'exit_code': result.returncode}
            if not positive:
                diagnostic = refuse(result, destination, pattern)
                log = output / case / mode / 'diagnostic.log'
                log.write_text(diagnostic)
                row.update(original_regex=pattern, diagnostic_sha256=sha256(log),
                           diagnostic_drift_refused=True, output_absent=True)
            else:
                require(result.returncode == 0 and destination.is_dir(),
                        f'original positive did not compile: {case} {mode}: '
                        f'{(result.stdout + result.stderr)[-1200:]}')
                info = destination / 'compiler/contract-info.json'
                entry = destination / 'contract/index.js'
                require(info.is_file() and entry.is_file(),
                        f'generated TypeScript product missing: {case} {mode}')
                if case == 'import/underscore_names':
                    names = [item['name'] for item in
                             json.loads(info.read_text())['circuits']]
                    require(names == ['_', '__', 'import_underscore_names'],
                            'underscore alias contract-info lost Compact names')
                if case == 'type/new_type_of_adt':
                    info_data = json.loads(info.read_text())
                    require(info_data['ledger'] == [
                        {'name': 'c', 'index': 0, 'exported': True,
                         'storage': 'Counter'}] and
                        info_data['circuits'] == [
                            {'name': 'type_new_type_of_adt', 'pure': False,
                             'proof': True, 'arguments': [], 'result-type':
                             {'type-name': 'Uint', 'maxval': 18446744073709551615}}],
                            'nominal Counter ledger declaration or capability changed')
                row.update(typescript_sha256=sha256(entry),
                           contract_info_sha256=sha256(info),
                           zkir={str(p.relative_to(destination)): sha256(p)
                                 for p in sorted((destination / 'zkir').glob('*.zkir'))})
                runtime_test = directory / 'runtime.pass.test.ts'
                require(runtime_test.is_file(), f'original runtime test missing: {case}')
                make_adapter(destination.parent, runtime_test, runtime)
                run(['node', str(destination.parent / 'run.mjs')],
                    output / case / mode / 'runtime.log', env)
                require(sha256(destination.parent / 'runtime.pass.test.ts') ==
                        originals[str(runtime_test.relative_to(ROOT))],
                        f'original runtime test copy changed: {case}')
                row.update(original_runtime_test=str(runtime_test.relative_to(ROOT)),
                           runtime_log_sha256=sha256(output / case / mode / 'runtime.log'),
                           runtime_assertions='passed')
                if mode == 'paired':
                    rust = destination / 'contract/lib.rs'
                    ir = destination / 'contract/compact-rust-ir.json'
                    require(rust.is_file() and ir.is_file(),
                            f'paired Rust product missing: {case}')
                    if case == 'import/underscore_names':
                        source_text = rust.read_text()
                        require(all('pub fn ' + name + '(' in source_text
                                    for name in ('__compact_export_underscore', '__',
                                                 'import_underscore_names')),
                                'underscore alias generated Rust API changed')
                    if case == 'type/new_type_of_adt':
                        require(all(fragment in rust.read_text() for fragment in
                                    ('CounterSlot', 'record_increment', 'record_read')),
                                'nominal Counter generated Rust actions changed')
                    row.update(rust_sha256=sha256(rust), rust_ir_sha256=sha256(ir))
                    row.update(locked_rust_check(destination / 'contract',
                                                 output / case / mode,
                                                 cargo_env, expected_registry))
                    if case in RUST_REPRESENTATIVES:
                        row.update(representative_rust_execution(
                            case, destination / 'contract', runtime_test,
                            output / case / mode, cargo_env))
            results[case][mode] = row
        ts, paired = results[case]['typescript'], results[case]['paired']
        if positive:
            require(ts['typescript_sha256'] == paired['typescript_sha256'] and
                    ts['zkir'] == paired['zkir'] and
                    ts['contract_info_sha256'] == paired['contract_info_sha256'],
                    f'paired target changed generated TS or ZKIR: {case}')
        else:
            require(ts['diagnostic_sha256'] == paired['diagnostic_sha256'],
                    f'paired target changed original diagnostic: {case}')
    for case in cases:
        for mode, row in results[case].items():
            base = output / case / mode
            if 'diagnostic_sha256' in row:
                require(sha256(base / 'diagnostic.log') == row['diagnostic_sha256'] and
                        not (base / 'product').exists(),
                        f'refusal artifact changed during gate: {case} {mode}')
                continue
            product = base / 'product'
            require(sha256(product / 'contract/index.js') == row['typescript_sha256'] and
                    sha256(product / 'compiler/contract-info.json') ==
                    row['contract_info_sha256'] and
                    {str(p.relative_to(product)): sha256(p)
                     for p in sorted((product / 'zkir').glob('*.zkir'))} == row['zkir'] and
                    sha256(base / 'runtime.pass.test.ts') ==
                    originals[row['original_runtime_test']] and
                    sha256(base / 'runtime.log') == row['runtime_log_sha256'] and
                    (base / 'node_modules/@midnight-ntwrk/compact-runtime').resolve() ==
                    runtime,
                    f'generated TS, ZKIR or original runtime changed: {case} {mode}')
            if mode == 'paired':
                require(sha256(product / 'contract/lib.rs') == row['rust_sha256'] and
                        sha256(product / 'contract/compact-rust-ir.json') ==
                        row['rust_ir_sha256'] and
                        sha256(product / 'contract/Cargo.lock') == row['lock_sha256'] and
                        sha256(base / 'cargo-check.log') == row['cargo_check_sha256'],
                        f'generated Rust or lock changed: {case}')
                if case in RUST_REPRESENTATIVES:
                    require(sha256(product / 'contract/tests/original_keywords.rs') ==
                            row['rust_execution_test_sha256'] and
                            sha256(base / 'cargo-test.log') ==
                            row['rust_execution_log_sha256'],
                            f'Rust representative evidence changed: {case}')
    require(source_state() == source and original_inventory() ==
            (originals, blocked) and sha256(MANIFEST) == manifest_hash and
            sha256(Path(__file__)) == checker_hash and
            runtime_identity(runtime) == runtime_before and
            runtime_recipe_identity(runtime, Path(nix).resolve()) == runtime_recipe and
            compiler_recipe_identity(args.compiler_recipe, source,
                                     compiler, scheme) == recipe,
            'source, original, runtime or compiler changed during gate')
    receipt = {**source, 'status': 'passed', 'source_clean': True,
               'platform': os.uname().sysname + ' ' + os.uname().machine,
               'upstream_commit': PINNED, 'manifest_sha256': manifest_hash,
               'checker_sha256': checker_hash, 'original_files': originals,
               'blocked_files': blocked, 'blocked_cases': sorted(BLOCKED),
               'compiler_recipe': recipe, 'compiler_sha256': sha256(compiler),
               'scheme_sha256': sha256(scheme), 'runtime_recipe': runtime_recipe,
               'runtime': runtime_before, 'case_count': len(cases),
               'positive_count': sum('runtime.pass.test.ts' in x for x in originals),
               'refusal_count': sum('compile.fail.test.ts' in x for x in originals),
               'results': results,
               'limitations': ['Original Vitest harness is adapted to runtime 0.18.0; '
                               'native Linux, wider Rust behavioral execution and proofs remain.']}
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2,
                                                    sort_keys=True) + '\n')
    print(json.dumps({'status': 'passed', 'head': source['head'],
                      'cases': len(cases), 'output': str(output)}))


if __name__ == '__main__':
    main()
