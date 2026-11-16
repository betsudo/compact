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

"""Qualify every pinned reserved-keyword diagnostic in TS and paired Rust modes."""

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
PINNED = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'
FAMILY = Path('test-contracts/keywords/reserved')
MANIFEST = ROOT / FAMILY / 'original-source-sha256.json'
PATTERN = re.compile(r'expectedError:\s*/((?:\\.|[^/])*)/')
MODES = {'typescript': ('--target', 'ts'),
         'paired': ('--target', 'ts', '--target', 'rust')}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def pinned_files() -> list[str]:
    result = subprocess.check_output(
        ['git', 'ls-tree', '-r', '--name-only', PINNED, str(FAMILY)],
        cwd=ROOT, text=True)
    return result.splitlines()


def original_inventory() -> dict[str, str]:
    manifest = json.loads(MANIFEST.read_text())
    files = manifest.get('files')
    expected = pinned_files()
    require(manifest.get('upstream_commit') == PINNED and
            isinstance(files, dict) and len(files) == 96 and
            sorted(files) == expected,
            'reserved-keyword original roster differs from pinned upstream')
    tests = [name for name in files if name.endswith('/compile.fail.test.ts')]
    contracts = [name for name in files if name.endswith('.compact')]
    require(len(tests) == len(contracts) == 48 and
            len({str(Path(name).parent) for name in tests}) == 48 and
            all(len(list((ROOT / name).parent.glob('*.compact'))) == 1 for name in tests),
            'reserved-keyword case roster is incomplete')
    for name in expected:
        path = ROOT / name
        pinned = subprocess.check_output(['git', 'show', f'{PINNED}:{name}'], cwd=ROOT)
        require(re.fullmatch(r'[0-9a-f]{64}', files[name]) is not None and
                path.is_file() and not path.is_symlink() and
                sha256(path) == files[name] == hashlib.sha256(pinned).hexdigest(),
                f'reserved-keyword original changed: {name}')
    return files


def original_pattern(path: Path) -> str:
    matches = PATTERN.findall(path.read_text())
    require(len(matches) == 1 and 'parse error:' in matches[0],
            f'one original parse-error regex required: {path}')
    return matches[0]


def diagnostic_matches(pattern: str, diagnostic: str) -> bool:
    return re.search(pattern, diagnostic) is not None


def refusal(result: subprocess.CompletedProcess[str], destination: Path,
            pattern: str) -> str:
    diagnostic = result.stdout + result.stderr
    require(result.returncode != 0 and result.stdout == '' and
            diagnostic_matches(pattern, diagnostic) and
            not diagnostic_matches(pattern, diagnostic.replace('parse error:',
                                                               'changed diagnostic:')),
            f'original diagnostic or negative control failed: {destination}')
    lock = destination.parent / f'.{destination.name}.compactc.lock'
    require(not destination.exists() and
            not list(destination.parent.glob(f'.{destination.name}.compactc-stage-*')) and
            all(child == lock and child.is_file() and not child.is_symlink() and
                child.stat().st_size == 0 for child in destination.parent.iterdir()),
            f'refused keyword published output: {destination}')
    return diagnostic


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
    originals = original_inventory()
    tests = sorted(name for name in originals if name.endswith('/compile.fail.test.ts'))
    patterns = {name: original_pattern(ROOT / name) for name in tests}
    script_hash = sha256(Path(__file__))
    manifest_hash = sha256(MANIFEST)
    output.mkdir(parents=True)
    env = dict(os.environ, COMPACTC_SCHEME=str(scheme))
    refusals = {}
    for name in tests:
        case = str(Path(name).parent.relative_to(FAMILY))
        contract = next((ROOT / name).parent.glob('*.compact'))
        refusals[case] = {}
        for mode, targets in MODES.items():
            destination = output / 'modes' / mode / case / 'product'
            destination.parent.mkdir(parents=True)
            command = [str(compiler), *targets]
            if mode == 'paired':
                command += ['--rust-runtime-root', str(ROOT)]
            command += ['--feature-zkir-v3', '--skip-zk', '--compact-path',
                        str(ROOT / 'compiler'), str(contract), str(destination)]
            result = subprocess.run(command, cwd=ROOT, env=env,
                                    capture_output=True, text=True)
            diagnostic = refusal(result, destination, patterns[name])
            log = output / 'diagnostics' / mode / (case + '.log')
            log.parent.mkdir(parents=True, exist_ok=True)
            log.write_text(diagnostic)
            refusals[case][mode] = {
                'command': command, 'exit_code': result.returncode,
                'original_test': name, 'original_regex': patterns[name],
                'diagnostic_sha256': sha256(log), 'output_absent': True,
                'negative_control_rejected': True,
            }
        require(refusals[case]['typescript']['diagnostic_sha256'] ==
                refusals[case]['paired']['diagnostic_sha256'],
                f'target diagnostics differ: {case}')
    require(source_state() == source and original_inventory() == originals and
            all(original_pattern(ROOT / name) == patterns[name] for name in tests) and
            sha256(Path(__file__)) == script_hash and sha256(MANIFEST) == manifest_hash and
            compiler_recipe_identity(args.compiler_recipe, source, compiler, scheme) == recipe,
            'source, upstream originals, checker, or compiler changed during gate')
    receipt = {
        **source, 'source_clean': True, 'platform': os.uname().sysname + ' ' + os.uname().machine,
        'upstream_commit': PINNED, 'original_files': originals,
        'manifest_sha256': manifest_hash, 'checker_sha256': script_hash,
        'compiler_recipe': recipe, 'compiler_sha256': sha256(compiler),
        'scheme_sha256': sha256(scheme), 'case_count': 48, 'mode_count': 2,
        'refusals': refusals,
        'result': '48 original reserved-keyword diagnostics refused identically in TypeScript and paired Rust',
        'limitations': ['local Darwin arm64; native Linux qualification required',
                        'compile-fail originals have no generated runtime, proof, or funded circuit'],
    }
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': 'passed', 'head': source['head'], 'cases': 48,
                      'output': str(output)}))


if __name__ == '__main__':
    main()
