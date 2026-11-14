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

"""Require the pinned oversized Vector refusal on TS, Rust and paired compiles."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess

from check_upstream_ledger9_vector_casts import (
    compiler_recipe_identity, sha256, source_state,
)


ROOT = Path(__file__).resolve().parents[2]
FAMILY = ROOT / 'test-contracts/primitives/vector/length'
PINNED = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'
SOURCE = FAMILY / 'exceeds_max/vector_length_exceeds_max.compact'
TEST = FAMILY / 'exceeds_max/compile.fail.test.ts'
MANIFEST = FAMILY / 'original-source-sha256.json'
EXPECTED_PATTERN = r'vector type length[\s\S]*exceeds the maximum supported length 16777216'
EXPECTED_DIAGNOSTIC = (
    'Exception: vector_length_exceeds_max.compact line 19 char 21:\n'
    '  vector type length\n'
    '  16777217\n'
    '  exceeds the maximum supported length 16777216\n'
)
MODES = {'typescript': ('--target', 'ts'),
         'rust': ('--target', 'rust'),
         'paired': ('--target', 'ts', '--target', 'rust')}


def require(ok: bool, message: str) -> None:
    if not ok:
        raise RuntimeError(message)


def pinned_sha256(name: str) -> str:
    blob = subprocess.check_output(['git', 'show', f'{PINNED}:{name}'], cwd=ROOT)
    return hashlib.sha256(blob).hexdigest()


def original_inventory(path: Path) -> dict[str, str]:
    manifest = json.loads(path.read_text())
    expected = {item.relative_to(ROOT).as_posix() for item in (SOURCE, TEST)}
    require(manifest.get('upstream_commit') == PINNED and
            isinstance(manifest.get('files'), dict) and
            set(manifest['files']) == expected,
            'pinned Vector length refusal inventory changed')
    for name, digest in manifest['files'].items():
        source = ROOT / name
        require(re.fullmatch(r'[0-9a-f]{64}', digest) is not None and
                source.is_file() and not source.is_symlink() and
                sha256(source) == digest == pinned_sha256(name),
                f'pinned Vector length original changed: {name}')
    return manifest['files']


def original_pattern(path: Path) -> str:
    matches = re.findall(r'expectedError:\s*/([^/]+)/', path.read_text())
    require(matches == [EXPECTED_PATTERN],
            'original Vector length diagnostic regex changed')
    return matches[0]


def refusal(result: subprocess.CompletedProcess[str], destination: Path,
            pattern: str) -> str:
    message = result.stdout + result.stderr
    require(result.returncode != 0 and result.stdout == '' and
            re.search(pattern, message) is not None and
            message == EXPECTED_DIAGNOSTIC,
            'Vector length refusal diagnostic changed')
    lock = destination.parent / f'.{destination.name}.compactc.lock'
    require(not destination.exists() and
            not list(destination.parent.glob(f'.{destination.name}.compactc-stage-*')) and
            all(child == lock and child.is_file() and not child.is_symlink() and
                child.stat().st_size == 0 for child in destination.parent.iterdir()),
            'refused oversized Vector published output')
    return message


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'compiler-recipe', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    source = source_state()
    require(subprocess.check_output(['git', 'log', '-1', '--format=%G?'], cwd=ROOT,
                                    text=True).strip() == 'G' and
            'Signed-off-by:' in subprocess.check_output(
                ['git', 'log', '-1', '--format=%B'], cwd=ROOT, text=True) and
            'Assisted-by: AI' in subprocess.check_output(
                ['git', 'log', '-1', '--format=%B'], cwd=ROOT, text=True),
            'source head must be signed, DCO complete and assisted')
    compiler = args.compiler.resolve(strict=True)
    scheme = compiler.with_name('compactc-scheme').resolve(strict=True)
    recipe = compiler_recipe_identity(args.compiler_recipe, source, compiler, scheme)
    output = args.output.resolve()
    require(not args.output.is_symlink() and not output.exists(), 'output must be absent')
    originals = original_inventory(MANIFEST)
    pattern = original_pattern(TEST)
    output.mkdir(parents=True)
    env = os.environ.copy()
    env['COMPACTC_SCHEME'] = str(scheme)
    refused = {}
    for mode, targets in MODES.items():
        destination = output / 'modes' / mode / 'product'
        destination.parent.mkdir(parents=True)
        command = [str(compiler), *targets]
        if mode != 'typescript':
            command += ['--rust-runtime-root', str(ROOT)]
        command += ['--feature-zkir-v3', '--skip-zk', '--compact-path',
                    str(ROOT / 'compiler'), str(SOURCE), str(destination)]
        result = subprocess.run(command, cwd=ROOT, env=env,
                                capture_output=True, text=True)
        message = refusal(result, destination, pattern)
        log = output / f'{mode}-diagnostic.log'
        log.write_text(message)
        refused[mode] = {'exit_code': result.returncode, 'command': command,
                         'diagnostic_sha256': sha256(log),
                         'output_absent': True, 'staging_absent': True,
                         'empty_compiler_lock_present':
                         (destination.parent / '.product.compactc.lock').is_file()}
    require(len({row['diagnostic_sha256'] for row in refused.values()}) == 1,
            'Vector length diagnostics differ across target modes')
    require(source_state() == source and
            compiler_recipe_identity(args.compiler_recipe, source, compiler, scheme) == recipe and
            original_inventory(MANIFEST) == originals and original_pattern(TEST) == pattern,
            'source or compiler identity changed during refusal gate')
    receipt = {
        **source, 'source_clean': True,
        'platform': os.uname().sysname + ' ' + os.uname().machine,
        'upstream_commit': PINNED, 'original_files': originals,
        'manifest_sha256': sha256(MANIFEST), 'checker_sha256': sha256(Path(__file__)),
        'compiler_recipe': recipe, 'compiler_sha256': sha256(compiler),
        'scheme_sha256': sha256(scheme), 'original_diagnostic_pattern': pattern,
        'refusals': refused,
        'result': 'pinned oversized Vector refused identically before output on TypeScript, Rust and paired targets',
        'limitations': ['local Darwin arm64; native Linux qualification required',
                        'compile-fail original has no generated runtime or proof/funded circuit'],
    }
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': 'passed', 'head': source['head'], 'output': str(output)}))


if __name__ == '__main__':
    main()
