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

"""Execute intact pinned keyword runtime callbacks on exact-head paired products."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

from check_upstream_ledger9_field_casts import require, sha256, source_state


ROOT = Path(__file__).resolve().parents[2]
PINNED = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'
FAMILIES = {'const': 11, 'for': 3, 'return': 3, 'if': 2}
VITEST_SHIM = (
    "import {deepStrictEqual} from 'node:assert';\n"
    "export function expect(actual) { return {"
    "toEqual(expected) {deepStrictEqual(actual, expected)},"
    "toBe(expected) {if (!Object.is(actual, expected)) "
    "throw new Error(`expected ${String(expected)}, got ${String(actual)}`)}"
    "}; }\n"
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
    "await definition.run(Contract); console.log('intact original runtime passed');\n"
)


def inventory(root: Path) -> dict[str, str]:
    result = {}
    for path in sorted(root.rglob('*')):
        require(not path.is_symlink(), f'generated artifact symlink: {path}')
        if path.is_file():
            result[path.relative_to(root).as_posix()] = sha256(path)
    return result


def stage(test: Path, destination: Path, product: Path, runtime: Path) -> None:
    destination.mkdir(parents=True)
    copied = destination / 'runtime.pass.test.ts'
    shutil.copyfile(test, copied)
    require(sha256(copied) == sha256(test), 'intact original test copy changed')
    (destination / 'package.json').write_text('{"type":"module"}\n')
    (destination / 'run.mjs').write_text(RUNNER)
    (destination / 'product').symlink_to(product, target_is_directory=True)
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


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    for family in FAMILIES:
        parser.add_argument('--' + family + '-gate', required=True, type=Path)
    parser.add_argument('--ts-runtime', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    source = source_state()
    runtime = args.ts_runtime.resolve(strict=True)
    require(runtime.is_dir() and not args.ts_runtime.is_symlink() and
            json.loads((runtime / 'package.json').read_text())['version'] == '0.18.0',
            'released synchronous Compact runtime required')
    output = args.output.resolve()
    require(not output.exists() and not args.output.is_symlink(),
            'new callback output directory required')
    gates = {}
    originals = {}
    for family, count in FAMILIES.items():
        root = getattr(args, family + '_gate').resolve(strict=True)
        receipt_path = root / 'receipt.json'
        gate = json.loads(receipt_path.read_text())
        require(gate['head'] == source['head'] and gate['tree'] == source['tree'] and
                gate['upstream_commit'] == PINNED and gate['source_clean'] is True and
                gate['case_count'] >= count and
                gate.get('status', 'passed') == 'passed',
                f'exact-head generated gate required: {family}')
        generated = root / 'generated'
        require(inventory(generated) == gate['generated_files_sha256'],
                f'generated inventory changed: {family}')
        cases = sorted({Path(name).parent.name for name in gate['original_files']
                        if name.startswith(f'test-contracts/keywords/{family}/') and
                        name.endswith('/runtime.pass.test.ts')})
        require(len(cases) == count, f'pinned runtime roster changed: {family}')
        for case in cases:
            require(case in gate['generated'],
                    f'pinned runtime generated product missing: {family}/{case}')
            name = f'test-contracts/keywords/{family}/{case}/runtime.pass.test.ts'
            expected = gate['original_files'].get(name)
            test = ROOT / name
            pinned = subprocess.check_output(['git', 'show', f'{PINNED}:{name}'],
                                             cwd=ROOT)
            require(expected and test.is_file() and not test.is_symlink() and
                    sha256(test) == expected == hashlib.sha256(pinned).hexdigest(),
                    f'intact pinned runtime test required: {name}')
            originals[name] = expected
        gates[family] = {'root': root, 'receipt_sha256': sha256(receipt_path),
                         'generated_sha256': gate['generated_files_sha256'],
                         'cases': cases}
    output.mkdir(parents=True)
    results = {}
    for family, gate in gates.items():
        results[family] = {}
        for case in gate['cases']:
            results[family][case] = {}
            test = ROOT / f'test-contracts/keywords/{family}/{case}/runtime.pass.test.ts'
            for mode in ('typescript', 'both'):
                product = gate['root'] / 'generated' / case / mode
                require((product / 'contract/index.js').is_file() and
                        (product / 'contract/lib.rs').is_file() == (mode == 'both'),
                        f'paired generated product required: {family}/{case}/{mode}')
                destination = output / family / case / mode
                stage(test, destination, product, runtime)
                result = subprocess.run(['node', str(destination / 'run.mjs')],
                                        cwd=ROOT, env=dict(os.environ,
                                        PYTHONDONTWRITEBYTECODE='1'),
                                        capture_output=True, text=True)
                log = destination / 'run.log'
                log.write_text(result.stdout + result.stderr)
                require(result.returncode == 0 and
                        result.stdout.strip() == 'intact original runtime passed',
                        f'original callback failed: {family}/{case}/{mode}: {log}')
                results[family][case][mode] = sha256(log)
            require(results[family][case]['typescript'] ==
                    results[family][case]['both'],
                    f'paired callback result changed: {family}/{case}')
    require(source_state() == source and
            all(inventory(gate['root'] / 'generated') == gate['generated_sha256']
                for gate in gates.values()) and
            all(sha256(ROOT / name) == digest for name, digest in originals.items()),
            'source, generated or pinned original changed during callback run')
    report = {**source, 'status': 'passed', 'platform': os.uname().sysname +
              ' ' + os.uname().machine, 'upstream_commit': PINNED,
              'runtime': str(runtime), 'runtime_sha256': sha256(runtime / 'package.json'),
              'node_version': subprocess.check_output(['node', '--version'], text=True).strip(),
              'callback_count': len(originals), 'execution_count':
              sum(len(modes) for cases in results.values() for modes in cases.values()),
              'original_files': originals,
              'gates': {family: {'receipt_sha256': gate['receipt_sha256'],
                                 'generated_files_sha256': gate['generated_sha256']}
                        for family, gate in gates.items()},
              'callback_logs_sha256': results,
              'runner_source_sha256': sha256(Path(__file__)),
              'limitations': ['original Vitest orchestrator and host package not run',
                              'generated Rust consumer parity is qualified by input gates']}
    (output / 'receipt.json').write_text(json.dumps(report, indent=2,
                                                   sort_keys=True) + '\n')
    print(json.dumps({'status': 'passed', 'head': source['head'],
                      'callbacks': len(originals), 'executions':
                      report['execution_count']}))


if __name__ == '__main__':
    main()
