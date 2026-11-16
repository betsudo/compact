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

"""Replay all pinned disclose keyword originals through paired generated targets."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess

from check_upstream_ledger9_import_type_keywords import (
    EXPECTED_ERROR, VITEST_SHIM, locked_rust_check, require,
)
from check_upstream_ledger9_vector_casts import (
    compiler_recipe_identity, require_unchanged, runtime_identity,
    runtime_recipe_identity, sha256, source_state,
)

ROOT = Path(__file__).resolve().parents[2]
PINNED = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'
FAMILY = Path('test-contracts/keywords/disclose')
MANIFEST = ROOT / FAMILY / 'original-source-sha256.json'
CASES = {
    'branch_conditions', 'branch_conditions_in_constructor',
    'param_to_ledger_undisclosed', 'unexported_circuit_parameters',
    'vector_element_condition',
}
ACCEPTED = CASES - {'param_to_ledger_undisclosed', 'vector_element_condition'}
PROOF_REQUIRED = {'branch_conditions', 'branch_conditions_in_constructor'}
ORIGINAL_ASSERTIONS = {
    'branch_conditions': ('expect(first.result).toEqual(3n);',
                          'expect(second.result).toEqual(4n);'),
    'branch_conditions_in_constructor': ('expect(result).toEqual(3n);',),
    'unexported_circuit_parameters': ('expect(result).toEqual(10n);',),
}

CONTRACT_SHIM = (
    "import {createCircuitContext,createConstructorContext,dummyContractAddress} "
    "from '@midnight-ntwrk/compact-runtime';\n"
    "export function defineRuntimeTest(_url,run){return {run}};\n"
    "export async function createTestContract(Contract,witnesses={},privateState=undefined,constructorArgs=[]){"
    "const contract=new Contract(witnesses);"
    "const initial=await contract.initialState("
    "createConstructorContext(privateState,'0'.repeat(64)),...constructorArgs);"
    "const ctx=createCircuitContext(dummyContractAddress(),"
    "initial.currentZswapLocalState.coinPublicKey,"
    "initial.currentContractState,initial.currentPrivateState);"
    "return {contract,ctx}};\n"
)


def originals() -> dict[str, str]:
    manifest = json.loads(MANIFEST.read_text())
    files = manifest.get('files')
    pinned = subprocess.check_output(
        ['git', 'ls-tree', '-r', '--name-only', PINNED, str(FAMILY)],
        cwd=ROOT, text=True).splitlines()
    require(manifest.get('upstream_commit') == PINNED and
            isinstance(files, dict) and len(files) == 13 and
            sorted(files) == pinned, 'pinned disclose roster changed')
    for name in pinned:
        path = ROOT / name
        original = subprocess.check_output(['git', 'show', f'{PINNED}:{name}'], cwd=ROOT)
        digest = hashlib.sha256(original).hexdigest()
        require(path.is_file() and not path.is_symlink() and
                files[name] == digest == sha256(path),
                f'pinned disclose original changed: {name}')
    cases = {Path(name).relative_to(FAMILY).parts[0] for name in files}
    require(cases == CASES, 'disclose case roster changed')
    for case in cases:
        directory = ROOT / FAMILY / case
        require(len(list(directory.glob('*.compact'))) == 1 and
                len(list(directory.glob('compile.*.test.ts'))) == 1,
                f'disclose case incomplete: {case}')
        test = next(directory.glob('compile.*.test.ts'))
        require((test.name == 'compile.pass.test.ts') == (case in ACCEPTED),
                f'disclose classification changed: {case}')
        if case in ACCEPTED:
            callback = (directory / 'runtime.pass.test.ts').read_text()
            require(all(snippet in callback for snippet in ORIGINAL_ASSERTIONS[case]),
                    f'original callback assertions changed: {case}')
        else:
            require(len(EXPECTED_ERROR.findall(test.read_text())) == 1,
                    f'original refusal regex changed: {case}')
    return files


def run(command: list[str], log: Path, env: dict[str, str]) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True)
    log.write_text(result.stdout + result.stderr)
    return result


def adapter(folder: Path, test: Path, runtime: Path) -> dict[str, str]:
    shutil.copyfile(test, folder / 'runtime.pass.test.ts')
    (folder / 'package.json').write_text('{"type":"module"}\n')
    (folder / 'run.mjs').write_text(
        "import definition from './runtime.pass.test.ts';\n"
        "import {Contract} from './product/contract/index.js';\n"
        "await definition.run(Contract); console.log('original disclose runtime passed');\n")
    modules = folder / 'node_modules'
    link = modules / '@midnight-ntwrk/compact-runtime'
    link.parent.mkdir(parents=True)
    link.symlink_to(runtime, target_is_directory=True)
    for name, source in (('vitest', VITEST_SHIM),
                         ('@test/compact-test', CONTRACT_SHIM)):
        directory = modules / name
        directory.mkdir(parents=True)
        (directory / 'package.json').write_text(
            '{"type":"module","exports":"./index.js"}\n')
        (directory / 'index.js').write_text(source)
    return {str(path.relative_to(folder)): sha256(path)
            for path in (folder / 'package.json', folder / 'run.mjs',
                         folder / 'runtime.pass.test.ts',
                         modules / 'vitest/package.json', modules / 'vitest/index.js',
                         modules / '@test/compact-test/package.json',
                         modules / '@test/compact-test/index.js')}


def generated(product: Path) -> dict[str, str]:
    files = {}
    for path in sorted(product.rglob('*')):
        require(not path.is_symlink(), f'generated symlink: {path}')
        if path.is_file():
            files[path.relative_to(product).as_posix()] = sha256(path)
    require(files, 'generated product absent')
    return files


def rust_result(case: str, contract: Path, folder: Path,
                env: dict[str, str]) -> dict[str, str]:
    import tomllib
    package = tomllib.loads((contract / 'Cargo.toml').read_text())['package']['name']
    prefix = 'use ' + package.replace('-', '_') + ' as generated;\n'
    context = ('generated::runtime::context::ConstructorContext::new('
               + ('30u64' if case == 'unexported_circuit_parameters' else '()')
               + ')')
    if case == 'branch_conditions':
        body = (
            f'let initial=generated::ledger_contract::initial_state({context}).unwrap();'
            'let first=generated::ledger_contract::disclose_branch_conditions('
            'initial.into_circuit_context(Default::default()),true,false).unwrap();'
            'assert_eq!(first.result,generated::runtime::Field::from(3u128));'
            'let second=generated::ledger_contract::disclose_branch_conditions('
            'first.context,false,true).unwrap();'
            'assert_eq!(second.result,generated::runtime::Field::from(4u128));'
        )
    elif case == 'branch_conditions_in_constructor':
        body = (
            f'let initial=generated::ledger_contract::initial_state({context},true,false).unwrap();'
            'let result=generated::ledger_contract::disclose_branch_conditions_in_constructor('
            'initial.into_circuit_context(Default::default())).unwrap();'
            'assert_eq!(result.result,generated::runtime::Field::from(3u128));'
        )
    else:
        prefix += (
            'struct Secret;\n'
            'impl generated::ledger_contract::TryWitnesses<u64> for Secret {'
            'fn secret(&self, context: generated::runtime::context::WitnessContext<' 
            "'_, u64, generated::ledger_contract::LedgerView<'_>>)"
            ' -> Result<(u64,generated::runtime::Field),generated::runtime::CompactError> {'
            'let value=*context.private_state-10;'
            'Ok((value,generated::runtime::Field::from(value)))}}\n'
        )
        body = (
            f'let initial=generated::ledger_contract::initial_state({context}).unwrap();'
            'let result=generated::ledger_contract::disclose_unexported_circuit_parameters('
            'initial.into_circuit_context(Default::default()),&Secret).unwrap();'
            'assert_eq!(result.result,generated::runtime::Field::from(10u128));'
            'assert_eq!(result.context.private_state,10u64);'
        )
    test = contract / 'tests/original_disclose.rs'
    test.parent.mkdir(exist_ok=True)
    test.write_text(prefix + '#[test] fn original_result() {' + body + '}\n')
    result = run(['cargo', '+1.99.0', 'test', '--locked', '--offline',
                  '--manifest-path', str(contract / 'Cargo.toml'),
                  '--test', 'original_disclose'], folder / 'cargo-test.log', env)
    require(result.returncode == 0 and
            'test result: ok. 1 passed; 0 failed;' in
            (folder / 'cargo-test.log').read_text(),
            f'generated Rust original result failed: {case}: '
            f'{(result.stdout + result.stderr)[-1000:]}')
    return {'rust_test_sha256': sha256(test),
            'rust_test_log_sha256': sha256(folder / 'cargo-test.log')}


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
            'Assisted-by: AI' in commit, 'signed DCO+Assisted head required')
    compiler = args.compiler.resolve(strict=True)
    scheme = compiler.with_name('compactc-scheme').resolve(strict=True)
    runtime = args.ts_runtime.resolve(strict=True)
    output = args.output.absolute()
    require(not output.exists() and not args.output.is_symlink(),
            'gate output must be absent')
    recipe = compiler_recipe_identity(args.compiler_recipe, source, compiler, scheme)
    nix = Path(shutil.which('nix')).resolve(strict=True)
    runtime_recipe = runtime_recipe_identity(runtime, nix)
    runtime_before = runtime_identity(runtime)
    supplied = json.loads((runtime / 'package.json').read_text())
    local = json.loads((ROOT / 'runtime/package.json').read_text())
    require(all(supplied[key] == local[key]
                for key in ('name', 'version', 'dependencies')),
            'runtime package differs from source')
    files = originals()
    output.mkdir(parents=True)
    env = dict(os.environ, COMPACTC_SCHEME=str(scheme),
               CARGO_INCREMENTAL='0', CARGO_PROFILE_DEV_DEBUG='0')
    if args.cargo_target:
        env['CARGO_TARGET_DIR'] = str(args.cargo_target.resolve(strict=True))
    results = {}
    expected_registry = set()
    import tomllib
    for row in tomllib.loads((ROOT / 'Cargo.lock').read_text())['package']:
        if 'source' in row:
            expected_registry.add((row['name'], row['version'], row['source'],
                                   row.get('checksum')))
    for case in sorted(CASES):
        directory = ROOT / FAMILY / case
        contract = next(directory.glob('*.compact'))
        compile_test = next(directory.glob('compile.*.test.ts'))
        results[case] = {}
        for mode in ('typescript', 'paired'):
            folder = output / case / mode
            folder.mkdir(parents=True)
            product = folder / 'product'
            command = [str(compiler), '--target', 'ts']
            if mode == 'paired':
                command += ['--target', 'rust', '--rust-runtime-root', str(ROOT),
                            '--rust-require-recording']
            command += ['--feature-zkir-v3', '--skip-zk', '--compact-path',
                        str(ROOT / 'compiler'), str(contract), str(product)]
            result = run(command, folder / 'compile.log', env)
            row = {'command': command, 'compile_log_sha256': sha256(folder / 'compile.log')}
            if case not in ACCEPTED:
                pattern = EXPECTED_ERROR.findall(compile_test.read_text())[0]
                diagnostic = result.stdout + result.stderr
                require(result.returncode != 0 and result.stdout == '' and
                        re.search(pattern, diagnostic) is not None and
                        not product.exists() and
                        not list(folder.glob('.product.compactc-stage-*')),
                        f'original refusal changed or output leaked: {case}/{mode}')
                row.update(original_regex=pattern, output_absent=True)
            else:
                require(result.returncode == 0 and product.is_dir(),
                        f'original acceptance failed: {case}/{mode}: '
                        f'{(result.stdout + result.stderr)[-1000:]}')
                info = json.loads((product / 'compiler/contract-info.json').read_text())
                circuits = info['circuits']
                require(len(circuits) == 1 and circuits[0]['name'] ==
                        'disclose_' + case and
                        circuits[0]['proof'] == (case in PROOF_REQUIRED),
                        f'proof capability changed: {case}/{mode}')
                callback = directory / 'runtime.pass.test.ts'
                row['adapter_files'] = adapter(folder, callback, runtime)
                execute = run(['node', str(folder / 'run.mjs')],
                              folder / 'runtime.log', env)
                require(execute.returncode == 0 and
                        'original disclose runtime passed' in execute.stdout,
                        f'original callback failed: {case}/{mode}: '
                        f'{(execute.stdout + execute.stderr)[-1000:]}')
                require(sha256(folder / 'runtime.pass.test.ts') ==
                        files[str(callback.relative_to(ROOT))],
                        'original callback bytes changed')
                row['runtime_log_sha256'] = sha256(folder / 'runtime.log')
                if mode == 'paired':
                    row.update(locked_rust_check(product / 'contract', folder,
                                                 env, expected_registry))
                    row.update(rust_result(case, product / 'contract', folder, env))
                row['generated_files'] = generated(product)
            results[case][mode] = row
        if case in ACCEPTED:
            ts = results[case]['typescript']['generated_files']
            both = results[case]['paired']['generated_files']
            require(ts['contract/index.js'] == both['contract/index.js'] and
                    all(ts[name] == both.get(name) for name in ts
                        if name.startswith('zkir/') or
                        name == 'compiler/contract-info.json'),
                    f'paired target changed TypeScript or ZKIR: {case}')
    require_unchanged(source, runtime_before, recipe, runtime,
                      args.compiler_recipe, compiler, scheme)
    report = {'format': 'ledger9-pinned-disclose-keywords/v1',
              'status': 'passed', 'source': source, 'compiler_recipe': recipe,
              'runtime_recipe': runtime_recipe, 'runtime': runtime_before,
              'upstream_commit': PINNED, 'originals': files,
              'manifest_sha256': sha256(MANIFEST),
              'gate_sha256': sha256(Path(__file__)), 'cases': results,
              'qualification': 'arm64 generated TypeScript callbacks and Rust checks; fresh proofs and funded application remain separate'}
    (output / 'receipt.json').write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': 'passed', 'head': source['head'],
                      'cases': len(results), 'original_files': len(files)}))


if __name__ == '__main__':
    main()
