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

"""Stage the byte-exact pinned keyword Vitest host with a local SDK adapter."""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess

from check_upstream_ledger9_field_casts import require, sha256, source_state


ROOT = Path(__file__).resolve().parents[2]
PINNED = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'
# Each tuple is (original files, compile pass, compile fail, runtime pass,
# runtime fail). Count the pinned tree before staging so adding or dropping a
# case cannot silently change the host's coverage claim.
FAMILY_COUNTS = {
    'reserved': (96, 0, 48, 0, 0),
    'const': (57, 11, 12, 11, 0),
    'import': (56, 11, 11, 11, 0),
    'type': (43, 9, 8, 9, 0),
    'for': (23, 3, 7, 3, 0),
    'return': (35, 3, 13, 3, 0),
    'if': (6, 2, 0, 2, 0),
    'include': (26, 3, 4, 3, 0),
    'prefix': (12, 2, 1, 2, 0),
    'assert': (17, 5, 1, 3, 2),
    'module': (20, 4, 4, 4, 0),
    'export': (11, 1, 4, 1, 0),
    'circuit': (36, 8, 6, 8, 0),
    'pure': (9, 1, 3, 1, 0),
    'constructor': (3, 1, 0, 1, 0),
    'ledger': (2, 0, 1, 0, 0),
    'pragma': (4, 0, 2, 0, 0),
    'sealed': (2, 0, 1, 0, 0),
    'struct': (2, 0, 1, 0, 0),
}
FAMILIES = tuple(FAMILY_COUNTS)
HOST = ('.gitignore', '.prettierignore', '.prettierrc.json', '.yarnrc.yml',
        'README.md', 'compact-test-fixture-reporter.ts',
        'compact-test-orchestrator.test.ts', 'compact-test-runner.ts',
        'compact-test.ts', 'package.json', 'test.sh', 'tsconfig.json',
        'types.ts', 'utils.ts', 'vitest.config.ts', 'yarn.lock')
ORIGINAL_COUNT = len(HOST) + sum(row[0] for row in FAMILY_COUNTS.values())
CASE_COUNT = sum(row[1] + row[2] for row in FAMILY_COUNTS.values())
HOST_TEST_COUNT = sum(sum(row[1:]) for row in FAMILY_COUNTS.values())


def pinned_bytes(name: str) -> bytes:
    return subprocess.check_output(['git', 'show', f'{PINNED}:{name}'], cwd=ROOT)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--runtime', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    source = source_state()
    runtime = args.runtime.resolve(strict=True)
    require(runtime.is_dir() and not args.runtime.is_symlink() and
            json.loads((runtime / 'package.json').read_text())['version'] == '0.18.0',
            'released Compact 0.18.0 runtime required')
    entry = (runtime / 'dist/index.js').resolve(strict=True)
    require(entry.is_file(), 'released runtime module required')
    output = args.output.resolve()
    require(not output.exists() and not args.output.is_symlink(),
            'new staging directory required')
    filenames = {f'test-contracts/{name}' for name in HOST}
    for family in FAMILIES:
        names = subprocess.check_output(
            ['git', 'ls-tree', '-r', '--name-only', PINNED,
             f'test-contracts/keywords/{family}'], cwd=ROOT, text=True).splitlines()
        observed = (len(names),) + tuple(
            sum(Path(name).name == kind for name in names)
            for kind in ('compile.pass.test.ts', 'compile.fail.test.ts',
                         'runtime.pass.test.ts', 'runtime.fail.test.ts'))
        require(observed == FAMILY_COUNTS[family],
                f'pinned {family} host case/file roster changed: {observed}')
        filenames.update(names)
    require(len(filenames) == ORIGINAL_COUNT and CASE_COUNT == 191 and
            HOST_TEST_COUNT == 255,
            'pinned nineteen-family host roster changed')
    tests = output / 'test-contracts'
    originals = {}
    for name in sorted(filenames):
        content = pinned_bytes(name)
        local = ROOT / name
        if name.startswith('test-contracts/keywords/'):
            require(local.is_file() and not local.is_symlink() and
                    local.read_bytes() == content,
                    f'pinned keyword original missing or changed: {name}')
        elif local.is_file():
            require(not local.is_symlink() and local.read_bytes() == content,
                    f'pinned host original changed: {name}')
        destination = output / name
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(content)
        originals[name] = hashlib.sha256(content).hexdigest()
    adapter = output / 'runtime'
    adapter.mkdir()
    (adapter / 'package.json').write_text(
        json.dumps({'name': '@midnight-ntwrk/compact-runtime', 'version': '0.18.0',
                    'type': 'module', 'main': './index.js', 'exports': './index.js'}) + '\n')
    (adapter / 'index.js').write_text(
        f"import * as released from {json.dumps(entry.as_uri())};\n"
        f"export * from {json.dumps(entry.as_uri())};\n"
        "export function createCircuitContext(...args) {\n"
        "  if (args.length === 1 && args[0] !== null &&\n"
        "      typeof args[0] === 'object' && 'contractAddress' in args[0]) {\n"
        "    const x = args[0];\n"
        "    return released.createCircuitContext(x.contractAddress,\n"
        "      x.coinPublicKeyOrZswapState, x.contractState, x.privateState);\n"
        "  }\n"
        "  return released.createCircuitContext(...args);\n"
        "}\n")
    (tests / '.compact-runtime').symlink_to(adapter, target_is_directory=True)
    require(source_state() == source and
            all(sha256(output / name) == digest for name, digest in originals.items()),
            'pinned host bytes or source changed during staging')
    report = {**source, 'status': 'staged', 'upstream_commit': PINNED,
              'original_files': originals, 'original_count': len(originals),
              'families': FAMILIES, 'family_counts': FAMILY_COUNTS,
              'case_count': CASE_COUNT, 'host_test_count': HOST_TEST_COUNT,
              'runtime': str(runtime),
              'runtime_package_sha256': sha256(runtime / 'package.json'),
              'runtime_entry_sha256': sha256(entry),
              'adapter_sha256': sha256(adapter / 'index.js'),
              'stager_sha256': sha256(Path(__file__)),
              'limits': ['staging does not execute Vitest',
                         'adapter changes only the released context-call shape']}
    (output / 'staging-receipt.json').write_text(
        json.dumps(report, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': 'staged', 'head': source['head'],
                      'originals': len(originals), 'output': str(output)}))


if __name__ == '__main__':
    main()
