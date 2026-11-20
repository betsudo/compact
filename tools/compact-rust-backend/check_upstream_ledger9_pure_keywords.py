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

"""Replay the complete pinned pure-keyword family through generated TS/Rust."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tomllib

from check_upstream_ledger9_import_type_keywords import (
    EXPECTED_ERROR, locked_rust_check, make_adapter, require, run,
)
from check_upstream_ledger9_vector_casts import (
    compiler_recipe_identity, runtime_identity, runtime_recipe_identity,
    sha256, source_state,
)


ROOT = Path(__file__).resolve().parents[2]
PINNED = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'
FAMILY = Path('test-contracts/keywords/pure')
MANIFEST = ROOT / FAMILY / 'original-source-sha256.json'
HELPER = ROOT / 'tools/compact-rust-backend/check_upstream_ledger9_import_type_keywords.py'
CASES = {'uses_ledger', 'without_body', 'without_ledger', 'writes_ledger'}
ASSERTIONS = ('expect(pureCircuits.pure_without_ledger(1n)).toEqual(2n);',
              'expect(result).toEqual(2n);')


def originals() -> dict[str, str]:
    manifest = json.loads(MANIFEST.read_text())
    files = manifest.get('files')
    pinned = subprocess.check_output(
        ['git', 'ls-tree', '-r', '--name-only', PINNED, str(FAMILY)],
        cwd=ROOT, text=True).splitlines()
    require(manifest.get('upstream_commit') == PINNED and
            isinstance(files, dict) and len(files) == 9 and
            sorted(files) == pinned, 'pinned pure roster differs')
    for name in pinned:
        path = ROOT / name
        expected = hashlib.sha256(subprocess.check_output(
            ['git', 'show', f'{PINNED}:{name}'], cwd=ROOT)).hexdigest()
        require(path.is_file() and not path.is_symlink() and
                files[name] == expected == sha256(path),
                f'pinned pure original changed: {name}')
    cases = {Path(name).relative_to(FAMILY).parts[0] for name in files}
    require(cases == CASES and
            all(len(list((ROOT / FAMILY / case).glob('*.compact'))) == 1 and
                len(list((ROOT / FAMILY / case).glob('compile.*.test.ts'))) == 1
                for case in cases), 'pure case roster incomplete')
    runtime = (ROOT / FAMILY / 'without_ledger/runtime.pass.test.ts').read_text()
    require(all(assertion in runtime for assertion in ASSERTIONS),
            'original pure runtime assertions changed')
    return files


def generated_files(product: Path) -> dict[str, str]:
    files = {}
    for path in sorted(product.rglob('*')):
        require(not path.is_symlink(), f'generated symlink: {path}')
        if path.is_file():
            files[str(path.relative_to(product))] = sha256(path)
    require(files, 'missing generated files')
    return files


def rust_result(product: Path, output: Path, env: dict[str, str]) -> dict[str, str]:
    contract = product / 'contract'
    package = tomllib.loads((contract / 'Cargo.toml').read_text())['package']['name']
    consumer = contract / 'tests/original_pure.rs'
    consumer.parent.mkdir(exist_ok=True)
    consumer.write_text(
        'use ' + package.replace('-', '_') + ' as generated;\n'
        '#[test] fn pinned_pure_result() { '
        'assert_eq!(generated::pure_circuits::pure_without_ledger('
        'generated::runtime::Field::from(1u128)).unwrap(), '
        'generated::runtime::Field::from(2u128)); }\n')
    log = output / 'cargo-test.log'
    run(['cargo', '+1.99.0', 'test', '--locked', '--offline',
         '--manifest-path', str(contract / 'Cargo.toml'), '--test', 'original_pure'],
        log, env)
    require('test result: ok. 1 passed; 0 failed;' in log.read_text(),
            'generated Rust pure result failed')
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
        contract = next(directory.glob('*.compact'))
        test = next(directory.glob('compile.*.test.ts'))
        accepted = case == 'without_ledger'
        require((test.name == 'compile.pass.test.ts') == accepted,
                f'original classification changed: {case}')
        pattern = None
        if not accepted:
            matches = EXPECTED_ERROR.findall(test.read_text())
            require(len(matches) == 1, f'one pinned diagnostic required: {case}')
            pattern = matches[0]
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
            if not accepted:
                diagnostic = result.stdout + result.stderr
                require(result.returncode != 0 and result.stdout == '' and
                        re.search(pattern, diagnostic) is not None and
                        re.search(pattern, diagnostic.replace('line ', 'row ', 1)) is None and
                        not product.exists(),
                        f'original diagnostic drift or output leak: {case}/{mode}')
                log = output / 'diagnostic.log'
                log.write_text(diagnostic)
                row.update(original_regex=pattern,
                           diagnostic_sha256=sha256(log), output_absent=True)
            else:
                require(result.returncode == 0 and product.is_dir(),
                        f'positive original failed: {case}/{mode}: '
                        f'{(result.stdout + result.stderr)[-1000:]}')
                info = json.loads((product / 'compiler/contract-info.json').read_text())
                circuits = info.get('circuits')
                require(isinstance(circuits, list) and len(circuits) == 1 and
                        circuits[0].get('name') == 'pure_without_ledger' and
                        circuits[0].get('pure') is True and
                        circuits[0].get('proof') is False,
                        f'original pure capability changed: {mode}')
                runtime_test = directory / 'runtime.pass.test.ts'
                make_adapter(output, runtime_test, runtime)
                (output / 'run.mjs').write_text(
                    "import definition from './runtime.pass.test.ts';\n"
                    "import {Contract,pureCircuits} from './product/contract/index.js';\n"
                    "await definition.run(Contract,pureCircuits); "
                    "console.log('original pure runtime passed');\n")
                adapter_files = {str(path.relative_to(output)): sha256(path)
                                 for directory_ in (output / 'node_modules/vitest',
                                                    output / 'node_modules/@test/compact-test')
                                 for path in sorted(directory_.glob('*')) if path.is_file()}
                for name in ('package.json', 'run.mjs'):
                    adapter_files[name] = sha256(output / name)
                run(['node', str(output / 'run.mjs')], output / 'runtime.log', env)
                require(sha256(output / 'runtime.pass.test.ts') ==
                        files[str(runtime_test.relative_to(ROOT))],
                        'original runtime callback changed')
                row.update(original_runtime_test=str(runtime_test.relative_to(ROOT)),
                           original_assertions=ASSERTIONS,
                           runtime_log_sha256=sha256(output / 'runtime.log'),
                           adapter_files=adapter_files)
                if mode == 'paired':
                    row.update(locked_rust_check(product / 'contract', output,
                                                 cargo_env, expected_registry))
                    row.update(rust_result(product, output, cargo_env))
                row['generated_files'] = generated_files(product)
            results[case][mode] = row
        if accepted:
            ts = results[case]['typescript']['generated_files']
            paired = results[case]['paired']['generated_files']
            shared = [name for name in ts if name.startswith('zkir/') or
                      name == 'compiler/contract-info.json']
            require(all(ts[name] == paired.get(name) for name in shared) and
                    ts['contract/index.js'] == paired['contract/index.js'],
                    'paired target changed generated TS/ZKIR')
        else:
            require(results[case]['typescript']['diagnostic_sha256'] ==
                    results[case]['paired']['diagnostic_sha256'],
                    f'paired target changed original refusal: {case}')
    for case, modes in results.items():
        for mode, row in modes.items():
            output = destination / case / mode
            if 'diagnostic_sha256' in row:
                require(sha256(output / 'diagnostic.log') == row['diagnostic_sha256']
                        and not (output / 'product').exists(),
                        f'original refusal evidence changed: {case}/{mode}')
            else:
                require(generated_files(output / 'product') == row['generated_files'] and
                        sha256(output / 'runtime.log') == row['runtime_log_sha256'] and
                        sha256(output / 'runtime.pass.test.ts') ==
                        files[row['original_runtime_test']] and
                        all(sha256(output / name) == digest
                            for name, digest in row['adapter_files'].items()),
                        f'generated/runtime evidence changed: {case}/{mode}')
    require(source_state() == source and originals() == files and
            sha256(MANIFEST) == manifest_hash and sha256(Path(__file__)) == script_hash and
            sha256(HELPER) == helper_hash and runtime_identity(runtime) == runtime_before and
            runtime_recipe_identity(runtime, nix) == runtime_recipe and
            compiler_recipe_identity(args.compiler_recipe, source, compiler, scheme) == recipe,
            'source/original/runtime/compiler changed during gate')
    receipt = {**source, 'status': 'passed', 'source_clean': True,
               'platform': os.uname().sysname + ' ' + os.uname().machine,
               'upstream_commit': PINNED, 'manifest_sha256': manifest_hash,
               'checker_sha256': script_hash, 'helper_sha256': helper_hash,
               'original_files': files, 'compiler_recipe': recipe,
               'compiler_sha256': sha256(compiler), 'scheme_sha256': sha256(scheme),
               'runtime_recipe': runtime_recipe, 'runtime': runtime_before,
               'case_count': len(CASES), 'positive_count': 1, 'refusal_count': 3,
               'runtime_assertion_count': 2, 'results': results,
               'limitations': ['Darwin arm64 local run; accepted circuit is pure '
                               'and proof-free. Native Linux and original Vitest host remain separate.']}
    (destination / 'receipt.json').write_text(json.dumps(receipt, indent=2,
                                                      sort_keys=True) + '\n')
    print(json.dumps({'status': 'passed', 'head': source['head'],
                      'cases': len(CASES), 'output': str(destination)}))


if __name__ == '__main__':
    main()
