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

"""Replay the pinned constructor, ledger, pragma, sealed and struct keyword cases."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tomllib

from check_upstream_ledger9_import_type_keywords import (
    EXPECTED_ERROR, locked_rust_check, require, run,
)
from check_upstream_ledger9_vector_casts import (
    compiler_recipe_identity, runtime_identity, runtime_recipe_identity,
    sha256, source_state,
)


ROOT = Path(__file__).resolve().parents[2]
PINNED = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'
FAMILIES = ('constructor', 'ledger', 'pragma', 'sealed', 'struct')
CASES = {
    'constructor': {'parameters'},
    'ledger': {'duplicate_binding'},
    'pragma': {'compiler_version_negation_needs_version_atom',
               'negation_needs_version_atom'},
    'sealed': {'exported_circuit_cannot_mutate'},
    'struct': {'spread_not_first'},
}
HELPER = ROOT / 'tools/compact-rust-backend/check_upstream_ledger9_import_type_keywords.py'
ASSERTION = 'expect(result).toEqual(45n);'
CONTRACT_SHIM = (
    "import {createCircuitContext,createConstructorContext,dummyContractAddress} "
    "from '@midnight-ntwrk/compact-runtime';\n"
    "export function defineRuntimeTest(_url,run){return {run}};\n"
    "export async function createTestContract(Contract,witnesses,_private,args){"
    "const contract=new Contract(witnesses);"
    "const initial=await contract.initialState("
    "createConstructorContext(undefined,'0'.repeat(64)),...args);"
    "const ctx=createCircuitContext(dummyContractAddress(),"
    "initial.currentZswapLocalState.coinPublicKey,"
    "initial.currentContractState,initial.currentPrivateState);"
    "return {contract,ctx}};\n"
)
RUST_TEST = '''use compact_contract_constructor_parameters as generated;
use generated::{ledger_contract as contract, ledger_slots as slots};
use generated::runtime::{BoundedUint, FixedVector, Field};

#[test]
fn pinned_constructor_parameters() {
    let values = FixedVector(std::array::from_fn(|i| BoundedUint::<9>::new(i as u128).unwrap()));
    let initial = contract::initial_state(generated::runtime::context::ConstructorContext::new(()), values).unwrap();
    let context = initial.into_circuit_context(Default::default());
    let native = contract::constructor_parameters(context).unwrap();
    assert_eq!(native.result, Field::from(45u128));
    assert_eq!(slots::sum.inspect(native.context.query.state.get_ref()).unwrap(), Field::from(45u128));
    let values = FixedVector(std::array::from_fn(|i| BoundedUint::<9>::new(i as u128).unwrap()));
    let initial = contract::initial_state(generated::runtime::context::ConstructorContext::new(()), values).unwrap();
    let recorded = contract::recorded::constructor_parameters(initial.into_circuit_context(Default::default())).unwrap();
    assert_eq!(recorded.execution.result, native.result);
    assert_eq!(recorded.execution.context.query.state, native.context.query.state);
    assert_eq!(recorded.execution.gas_cost, native.gas_cost);
}
'''


def originals() -> dict[str, str]:
    files = {}
    for family in FAMILIES:
        folder = Path('test-contracts/keywords') / family
        manifest = json.loads((ROOT / folder / 'original-source-sha256.json').read_text())
        pinned = subprocess.check_output(
            ['git', 'ls-tree', '-r', '--name-only', PINNED, str(folder)],
            cwd=ROOT, text=True).splitlines()
        roster = manifest.get('files')
        require(manifest.get('upstream_commit') == PINNED and
                isinstance(roster, dict) and sorted(roster) == pinned,
                f'pinned {family} roster changed')
        cases = {Path(name).relative_to(folder).parts[0] for name in pinned}
        require(cases == CASES[family], f'pinned {family} cases changed')
        for name in pinned:
            path = ROOT / name
            digest = hashlib.sha256(subprocess.check_output(
                ['git', 'show', f'{PINNED}:{name}'], cwd=ROOT)).hexdigest()
            require(path.is_file() and not path.is_symlink() and
                    roster[name] == digest == sha256(path),
                    f'pinned source changed: {name}')
            files[name] = digest
    require(len(files) == 13, 'small keyword original count changed')
    return files


def generated_files(product: Path) -> dict[str, str]:
    files = {}
    for path in sorted(product.rglob('*')):
        require(not path.is_symlink(), f'generated symlink: {path}')
        if path.is_file():
            files[str(path.relative_to(product))] = sha256(path)
    require(files, 'missing generated files')
    return files


def runtime_adapter(folder: Path, original: Path, runtime: Path) -> dict[str, str]:
    shutil.copyfile(original, folder / original.name)
    (folder / 'package.json').write_text('{"type":"module"}\n')
    (folder / 'run.mjs').write_text(
        "import definition from './runtime.pass.test.ts';\n"
        "import {Contract} from './product/contract/index.js';\n"
        "await definition.run(Contract); console.log('original callback passed');\n")
    modules = folder / 'node_modules'
    package = modules / '@midnight-ntwrk/compact-runtime'
    package.parent.mkdir(parents=True)
    package.symlink_to(runtime, target_is_directory=True)
    for name, source in (
        ('vitest', "import {deepStrictEqual} from 'node:assert';\n"
         "export function expect(actual){return {toEqual(expected){"
         "deepStrictEqual(actual,expected)}}};\n"),
        ('@test/compact-test', CONTRACT_SHIM),
    ):
        directory = modules / name
        directory.mkdir(parents=True)
        (directory / 'package.json').write_text(
            '{"type":"module","exports":"./index.js"}\n')
        (directory / 'index.js').write_text(source)
    return {str(path.relative_to(folder)): sha256(path)
            for path in sorted(folder.rglob('*')) if path.is_file() and
            'product' not in path.relative_to(folder).parts and
            'node_modules/@midnight-ntwrk/compact-runtime' not in str(path)}


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
            'Assisted-by: AI' in commit, 'head requires signature/DCO/AI')
    compiler = args.compiler.resolve(strict=True)
    scheme = compiler.with_name('compactc-scheme').resolve(strict=True)
    runtime = args.ts_runtime.resolve(strict=True)
    destination = args.output.resolve()
    require(not args.output.is_symlink() and not destination.exists(),
            'output must be absent')
    recipe = compiler_recipe_identity(args.compiler_recipe, source,
                                      compiler, scheme)
    nix = Path(subprocess.check_output(['which', 'nix'], text=True).strip()).resolve()
    runtime_recipe = runtime_recipe_identity(runtime, nix)
    runtime_before = runtime_identity(runtime)
    package = json.loads((ROOT / 'runtime/package.json').read_text())
    supplied = json.loads((runtime / 'package.json').read_text())
    require(all(package[key] == supplied[key]
                for key in ('name', 'version', 'dependencies')),
            'released runtime differs from source')
    files = originals()
    source_hashes = {str(path.relative_to(ROOT)): sha256(path)
                     for family in FAMILIES
                     for path in [(ROOT / 'test-contracts/keywords' / family /
                                   'original-source-sha256.json')]}
    checker_hash = sha256(Path(__file__))
    helper_hash = sha256(HELPER)
    expected_registry = {(row['name'], row['version'], row['source'],
                          row.get('checksum'))
                         for row in tomllib.loads((ROOT / 'Cargo.lock').read_text())['package']
                         if 'source' in row}
    destination.mkdir(parents=True)
    env = dict(os.environ, COMPACTC_SCHEME=str(scheme))
    cargo_env = dict(env, CARGO_INCREMENTAL='0', CARGO_PROFILE_DEV_DEBUG='0',
                     CARGO_TARGET_DIR=str(args.cargo_target.resolve()
                                          if args.cargo_target else destination / 'cargo-target'))
    results = {}
    for family in FAMILIES:
        for case in sorted(CASES[family]):
            key = f'{family}/{case}'
            directory = ROOT / 'test-contracts/keywords' / key
            contract = next(directory.glob('*.compact'))
            test = next(directory.glob('compile.*.test.ts'))
            accepted = family == 'constructor'
            require((test.name == 'compile.pass.test.ts') == accepted,
                    f'original test classification changed: {key}')
            pattern = None
            if not accepted:
                matches = EXPECTED_ERROR.findall(test.read_text())
                require(len(matches) == 1, f'original regex count changed: {key}')
                pattern = matches[0]
            results[key] = {}
            for mode in ('typescript', 'paired'):
                output = destination / key / mode
                product = output / 'product'
                output.mkdir(parents=True)
                command = [str(compiler), '--target', 'ts']
                if mode == 'paired':
                    command += ['--target', 'rust', '--rust-runtime-root', str(ROOT),
                                '--rust-require-recording']
                command += ['--feature-zkir-v3', '--skip-zk', '--compact-path',
                            str(ROOT / 'compiler'), str(contract), str(product)]
                result = subprocess.run(command, cwd=ROOT, env=env,
                                        capture_output=True, text=True)
                row = {'command': command, 'original_compile_test':
                       str(test.relative_to(ROOT)), 'exit_code': result.returncode}
                if not accepted:
                    diagnostic = result.stdout + result.stderr
                    require(result.returncode != 0 and result.stdout == '' and
                            re.search(pattern, diagnostic) is not None and
                            re.search(pattern, diagnostic.replace('line ', 'row ', 1)) is None and
                            not product.exists(),
                            f'original diagnostic drift or output leak: {key}/{mode}')
                    log = output / 'diagnostic.log'
                    log.write_text(diagnostic)
                    row.update(original_regex=pattern,
                               diagnostic_sha256=sha256(log), output_absent=True)
                else:
                    require(result.returncode == 0 and product.is_dir(),
                            f'original positive failed: {key}/{mode}: '
                            f'{(result.stdout + result.stderr)[-1000:]}')
                    info = json.loads((product / 'compiler/contract-info.json').read_text())
                    circuits = info.get('circuits')
                    require(len(circuits) == 1 and circuits[0]['name'] ==
                            'constructor_parameters' and circuits[0]['pure'] is False and
                            circuits[0]['proof'] is True and
                            info['ledger'][0]['name'] == 'sum',
                            'constructor capability or ledger changed')
                    original = directory / 'runtime.pass.test.ts'
                    require(ASSERTION in original.read_text() and
                            'const v = [0n, 1n, 2n, 3n, 4n, 5n, 6n, 7n, 8n, 9n];'
                            in original.read_text(), 'original runtime assertion changed')
                    adapter = runtime_adapter(output, original, runtime)
                    run(['node', str(output / 'run.mjs')], output / 'runtime.log', env)
                    require(sha256(output / original.name) ==
                            files[str(original.relative_to(ROOT))],
                            'original runtime callback changed')
                    row.update(original_runtime_test=str(original.relative_to(ROOT)),
                               original_assertion=ASSERTION,
                               adapter_files=adapter,
                               runtime_log_sha256=sha256(output / 'runtime.log'))
                    if mode == 'paired':
                        rust = product / 'contract'
                        row.update(locked_rust_check(rust, output, cargo_env,
                                                     expected_registry))
                        consumer = rust / 'tests/original_constructor.rs'
                        consumer.parent.mkdir(exist_ok=True)
                        consumer.write_text(RUST_TEST)
                        log = output / 'cargo-test.log'
                        run(['cargo', '+1.99.0', 'test', '--locked', '--offline',
                             '--manifest-path', str(rust / 'Cargo.toml'),
                             '--test', 'original_constructor'], log, cargo_env)
                        require('test result: ok. 1 passed; 0 failed;' in log.read_text(),
                                'generated Rust constructor result failed')
                        row.update(rust_test_sha256=sha256(consumer),
                                   rust_test_log_sha256=sha256(log))
                    row['generated_files'] = generated_files(product)
                results[key][mode] = row
            a, b = results[key]['typescript'], results[key]['paired']
            if accepted:
                require(a['generated_files']['contract/index.js'] ==
                        b['generated_files']['contract/index.js'] and
                        a['generated_files']['compiler/contract-info.json'] ==
                        b['generated_files']['compiler/contract-info.json'] and
                        all(digest == b['generated_files'].get(name)
                            for name, digest in a['generated_files'].items()
                            if name.startswith('zkir/')),
                        'paired compilation changed TS/ZKIR')
            else:
                require(a['diagnostic_sha256'] == b['diagnostic_sha256'],
                        f'paired compilation changed original refusal: {key}')
    require(source_state() == source and originals() == files and
            all(sha256(ROOT / name) == digest for name, digest in source_hashes.items()) and
            sha256(Path(__file__)) == checker_hash and sha256(HELPER) == helper_hash and
            runtime_identity(runtime) == runtime_before and
            runtime_recipe_identity(runtime, nix) == runtime_recipe and
            compiler_recipe_identity(args.compiler_recipe, source, compiler, scheme) == recipe,
            'source, compiler, original, or runtime changed during gate')
    receipt = {**source, 'status': 'passed', 'source_clean': True,
               'platform': os.uname().sysname + ' ' + os.uname().machine,
               'upstream_commit': PINNED, 'original_files': files,
               'manifest_files': source_hashes, 'checker_sha256': checker_hash,
               'helper_sha256': helper_hash, 'compiler_recipe': recipe,
               'compiler_sha256': sha256(compiler), 'scheme_sha256': sha256(scheme),
               'runtime_recipe': runtime_recipe, 'runtime_identity': runtime_before,
               'case_count': 6, 'mode_count': 2, 'results': results,
               'limitations': ['local Darwin arm64; native Linux qualification required',
                               'constructor proof and funded strict application not covered by this gate',
                               'compile-fail originals have no runtime or proof']}
    (destination / 'receipt.json').write_text(json.dumps(receipt, indent=2,
                                                         sort_keys=True) + '\n')
    print(json.dumps({'status': 'passed', 'head': source['head'],
                      'cases': 6, 'output': str(destination)}))


if __name__ == '__main__':
    main()
