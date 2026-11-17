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

"""Run all pinned assert originals through paired TS/Rust outcomes and refusals."""

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
    CONTRACT_SHIM, EXPECTED_ERROR, VITEST_SHIM, locked_rust_check,
    require, run,
)
from check_upstream_ledger9_vector_casts import (
    compiler_recipe_identity, runtime_identity, runtime_recipe_identity,
    sha256, source_state,
)


ROOT = Path(__file__).resolve().parents[2]
PINNED = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'
FAMILY = Path('test-contracts/keywords/assert')
MANIFEST = ROOT / FAMILY / 'original-source-sha256.json'
HELPER = ROOT / 'tools/compact-rust-backend/check_upstream_ledger9_import_type_keywords.py'
CASES = {'false_condition_fails', 'false_condition_in_constructor',
         'in_untaken_branch', 'old_statement_syntax', 'result_is_unit',
         'true_condition_passes'}
COMPILE_REFUSALS = {'old_statement_syntax'}
RUNTIME_REFUSALS = {'false_condition_fails', 'false_condition_in_constructor'}
ASSERTIONS = {
    'in_untaken_branch': 'expect(result).toEqual([]);',
    'result_is_unit': 'expect(result).toEqual([]);',
    'true_condition_passes': 'expect(result).toEqual([]);',
}
RUST_TESTS = {
    'false_condition_fails': (
        'let error = generated::pure_circuits::assert_false_condition_fails()'
        '.unwrap_err(); assert!(format!("{error}").contains("bob 1"));'
    ),
    'false_condition_in_constructor': (
        'let error = generated::ledger_contract::initial_state('
        'generated::runtime::context::ConstructorContext::new(()))'
        '.err().expect("constructor must fail");'
        'assert!(format!("{error}").contains("bob 1"));'
    ),
    'in_untaken_branch': (
        'let context = || generated::ledger_contract::initial_state('
        'generated::runtime::context::ConstructorContext::new(()))'
        '.unwrap().into_circuit_context(Default::default());'
        'let native = generated::ledger_contract::assert_in_untaken_branch('
        'context(), generated::runtime::Field::from(0u128)).unwrap();'
        'let recorded = generated::ledger_contract::recorded::'
        'assert_in_untaken_branch(context(), '
        'generated::runtime::Field::from(0u128)).unwrap();'
        'assert_eq!(native.result, ());'
        'assert_eq!(native.result, recorded.execution.result);'
        'assert_eq!(native.context.query.state, recorded.execution.context.query.state);'
        'assert_eq!(native.gas_cost, recorded.execution.gas_cost);'
        'assert_eq!(generated::ledger_slots::dummy.inspect('
        'native.context.query.state.get_ref()).unwrap(), '
        'generated::runtime::Field::from(1u128));'
    ),
    'result_is_unit': (
        'assert_eq!(generated::pure_circuits::assert_result_is_unit().unwrap(), ());'
    ),
    'true_condition_passes': (
        'assert_eq!(generated::pure_circuits::assert_true_condition_passes()'
        '.unwrap(), ());'
    ),
}
RUNNER = (
    "import definition from './runtime.test.ts';\n"
    "import {Contract} from './product/contract/index.js';\n"
    "let error; try { await definition.run(Contract); } catch (caught) { error=caught; }\n"
    "if (definition.expectedError) {\n"
    "  if (!error || !definition.expectedError.test(String(error))) "
    "throw new Error('original expected failure drift: '+String(error));\n"
    "  console.log('original runtime failure passed: '+String(error));\n"
    "} else {\n"
    "  if (error) throw error;\n"
    "  console.log('original runtime passed');\n"
    "}\n"
)


def originals() -> dict[str, str]:
    manifest = json.loads(MANIFEST.read_text())
    files = manifest.get('files')
    pinned = subprocess.check_output(
        ['git', 'ls-tree', '-r', '--name-only', PINNED, str(FAMILY)],
        cwd=ROOT, text=True).splitlines()
    require(manifest.get('upstream_commit') == PINNED and
            isinstance(files, dict) and len(files) == 17 and
            sorted(files) == pinned, 'pinned assert roster differs')
    for name in pinned:
        path = ROOT / name
        expected = hashlib.sha256(subprocess.check_output(
            ['git', 'show', f'{PINNED}:{name}'], cwd=ROOT)).hexdigest()
        require(path.is_file() and not path.is_symlink() and
                files[name] == expected == sha256(path),
                f'pinned assert original changed: {name}')
    cases = {Path(name).relative_to(FAMILY).parts[0] for name in files}
    require(cases == CASES and
            all(sum(1 for _ in (ROOT / FAMILY / case).glob('assert_*.compact')) == 1
                and sum(1 for _ in (ROOT / FAMILY / case).glob('compile.*.test.ts')) == 1
                for case in cases), 'assert case roster incomplete')
    return files


def original_error(test: Path) -> str:
    found = EXPECTED_ERROR.findall(test.read_text())
    require(len(found) == 1, f'one original regex required: {test}')
    return found[0]


def stage_runtime(output: Path, test: Path, runtime: Path) -> None:
    shutil.copyfile(test, output / 'runtime.test.ts')
    (output / 'package.json').write_text('{"type":"module"}\n')
    (output / 'run.mjs').write_text(RUNNER)
    modules = output / 'node_modules'
    package = modules / '@midnight-ntwrk/compact-runtime'
    package.parent.mkdir(parents=True)
    package.symlink_to(runtime, target_is_directory=True)
    shim = CONTRACT_SHIM.replace(
        'defineRuntimeTest(_url,run){return {run}}',
        'defineRuntimeTest(_url,run,options={})'
        '{return {run,expectedError:options.expectedError}}')
    require(shim != CONTRACT_SHIM, 'released context adapter shape changed')
    for name, source in (('vitest', VITEST_SHIM),
                         ('@test/compact-test', shim)):
        directory = modules / name
        directory.mkdir(parents=True)
        (directory / 'package.json').write_text(
            '{"type":"module","exports":"./index.js"}\n')
        (directory / 'index.js').write_text(source)


def generated_files(product: Path) -> dict[str, str]:
    return {str(path.relative_to(product)): sha256(path)
            for path in sorted(product.rglob('*')) if path.is_file() and
            not path.is_symlink()}


def rust_result(case: str, product: Path, output: Path,
                env: dict[str, str]) -> dict[str, str]:
    contract = product / 'contract'
    package = tomllib.loads((contract / 'Cargo.toml').read_text())['package']['name']
    consumer = contract / 'tests/original_assert.rs'
    consumer.parent.mkdir(exist_ok=True)
    consumer.write_text(
        'use ' + package.replace('-', '_') + ' as generated;\n'
        '#[test] fn pinned_assert_result() { ' + RUST_TESTS[case] + ' }\n')
    log = output / 'cargo-test.log'
    run(['cargo', '+1.99.0', 'test', '--locked', '--offline',
         '--manifest-path', str(contract / 'Cargo.toml'),
         '--test', 'original_assert'], log, env)
    require(re.search(r'test result: ok\. 1 passed; 0 failed;',
                      log.read_text()) is not None,
            f'generated Rust original result failed: {case}')
    return {'rust_test_sha256': sha256(consumer),
            'rust_test_log_sha256': sha256(log)}


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
            'Assisted-by: AI' in commit, 'source head requires signature/DCO/AI')
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
    manifest_hash = sha256(MANIFEST)
    script_hash = sha256(Path(__file__))
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
    for case in sorted(CASES):
        directory = ROOT / FAMILY / case
        contract = next(directory.glob('assert_*.compact'))
        test = next(directory.glob('compile.*.test.ts'))
        compile_refusal = case in COMPILE_REFUSALS
        require((test.name == 'compile.fail.test.ts') == compile_refusal,
                f'original compile classification changed: {case}')
        pattern = original_error(test) if compile_refusal else None
        results[case] = {}
        for mode in ('typescript', 'paired'):
            output = destination / case / mode
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
            if compile_refusal:
                diagnostic = result.stdout + result.stderr
                require(result.returncode != 0 and result.stdout == '' and
                        re.search(pattern, diagnostic) is not None and
                        re.search(pattern, diagnostic.replace('line ', 'row ', 1)) is None and
                        not product.exists(),
                        f'original diagnostic drift or output leak: {case} {mode}')
                log = output / 'diagnostic.log'
                log.write_text(diagnostic)
                row.update(original_regex=pattern,
                           diagnostic_sha256=sha256(log), output_absent=True)
            else:
                require(result.returncode == 0 and product.is_dir(),
                        f'accepted original failed: {case} {mode}: '
                        f'{(result.stdout + result.stderr)[-1000:]}')
                runtime_test = next(directory.glob('runtime.*.test.ts'))
                runtime_refusal = case in RUNTIME_REFUSALS
                require((runtime_test.name == 'runtime.fail.test.ts') == runtime_refusal,
                        f'original runtime classification changed: {case}')
                if runtime_refusal:
                    require(original_error(runtime_test) == 'bob 1',
                            f'original assert failure message changed: {case}')
                else:
                    require(ASSERTIONS[case] in runtime_test.read_text(),
                            f'original assertion changed: {case}')
                stage_runtime(output, runtime_test, runtime)
                run(['node', str(output / 'run.mjs')], output / 'runtime.log', env)
                log_text = (output / 'runtime.log').read_text()
                require(('original runtime failure passed: ' if runtime_refusal
                         else 'original runtime passed') in log_text,
                        f'original runtime outcome changed: {case} {mode}')
                require(sha256(output / 'runtime.test.ts') ==
                        files[str(runtime_test.relative_to(ROOT))],
                        f'original runtime callback changed: {case}')
                row.update(original_runtime_test=str(runtime_test.relative_to(ROOT)),
                           original_outcome='failure' if runtime_refusal else 'success',
                           runtime_log_sha256=sha256(output / 'runtime.log'))
                if mode == 'paired':
                    row.update(locked_rust_check(product / 'contract', output,
                                                 cargo_env, expected_registry))
                    row.update(rust_result(case, product, output, cargo_env))
                row['generated_files'] = generated_files(product)
            results[case][mode] = row
        if compile_refusal:
            require(results[case]['typescript']['diagnostic_sha256'] ==
                    results[case]['paired']['diagnostic_sha256'],
                    f'paired target changed original refusal: {case}')
        else:
            ts = results[case]['typescript']['generated_files']
            paired = results[case]['paired']['generated_files']
            shared = [name for name in ts if name.startswith('zkir/') or
                      name == 'compiler/contract-info.json']
            require(all(ts[name] == paired.get(name) for name in shared) and
                    ts['contract/index.js'] == paired['contract/index.js'],
                    f'paired target changed generated TS/ZKIR: {case}')
    for case, modes in results.items():
        for mode, row in modes.items():
            output = destination / case / mode
            if 'diagnostic_sha256' in row:
                require(sha256(output / 'diagnostic.log') == row['diagnostic_sha256']
                        and not (output / 'product').exists(),
                        f'original refusal evidence changed: {case} {mode}')
            else:
                require(generated_files(output / 'product') == row['generated_files']
                        and sha256(output / 'runtime.log') == row['runtime_log_sha256']
                        and sha256(output / 'runtime.test.ts') ==
                        files[row['original_runtime_test']],
                        f'generated or runtime evidence changed: {case} {mode}')
    require(source_state() == source and originals() == files and
            sha256(MANIFEST) == manifest_hash and
            sha256(Path(__file__)) == script_hash and sha256(HELPER) == helper_hash and
            runtime_identity(runtime) == runtime_before and
            runtime_recipe_identity(runtime, nix) == runtime_recipe and
            compiler_recipe_identity(args.compiler_recipe, source,
                                     compiler, scheme) == recipe,
            'source, original, runtime, helper or compiler changed during gate')
    receipt = {**source, 'status': 'passed', 'source_clean': True,
               'platform': os.uname().sysname + ' ' + os.uname().machine,
               'upstream_commit': PINNED, 'manifest_sha256': manifest_hash,
               'checker_sha256': script_hash, 'helper_sha256': helper_hash,
               'original_files': files, 'compiler_recipe': recipe,
               'compiler_sha256': sha256(compiler), 'scheme_sha256': sha256(scheme),
               'runtime_recipe': runtime_recipe, 'runtime': runtime_before,
               'case_count': len(CASES), 'accepted_compile_count': len(CASES)-len(COMPILE_REFUSALS),
               'compile_refusal_count': len(COMPILE_REFUSALS),
               'runtime_refusal_count': len(RUNTIME_REFUSALS), 'results': results,
               'limitations': ['Darwin arm64 local run; native Linux, original '
                               'Vitest host package and fresh proof/funded '
                               'qualification remain separate.']}
    (destination / 'receipt.json').write_text(json.dumps(receipt, indent=2,
                                                      sort_keys=True) + '\n')
    print(json.dumps({'status': 'passed', 'head': source['head'],
                      'cases': len(CASES), 'output': str(destination)}))


if __name__ == '__main__':
    main()
