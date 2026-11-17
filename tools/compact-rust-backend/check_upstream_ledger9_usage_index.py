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

"""Check the pinned usage-index CLI with current and pinned fixture context."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess

from check_upstream_ledger9_field_casts import require, sha256, source_state


ROOT = Path(__file__).resolve().parents[2]
PINNED = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'
ORIGINALS = ('test-contracts/coverage/types.ts',
             'test-contracts/coverage/usage-index.ts',
             'test-contracts/coverage/usage.md')
CLI = 'test-contracts/coverage/usage-index.ts'
BASELINE = 'test-contracts/coverage/usage.md'
CHECK_ARGS = ('node', CLI, '--check', BASELINE)
REPORT_PATTERN = re.compile(r'usage index: (\d+)/(\d+) names covered by (\d+) fixtures;')


def pinned_names(folder: str, suffixes: tuple[str, ...]) -> list[str]:
    names = subprocess.check_output(
        ['git', 'ls-tree', '-r', '--name-only', PINNED, folder],
        cwd=ROOT, text=True).splitlines()
    return [name for name in names if name.endswith(suffixes)]


def current_names(folder: str, suffixes: tuple[str, ...]) -> list[str]:
    return sorted(str(path.relative_to(ROOT)) for path in (ROOT / folder).rglob('*')
                  if path.is_file() and path.suffix in suffixes)


def pinned_bytes(name: str) -> bytes:
    return subprocess.check_output(['git', 'show', f'{PINNED}:{name}'], cwd=ROOT)


def file_hashes(root: Path, names: list[str]) -> dict[str, str]:
    return {name: sha256(root / name) for name in names}


def invoke(root: Path, args: tuple[str, ...], log: Path) -> dict[str, object]:
    result = subprocess.run(args, cwd=root, text=True, capture_output=True,
                            env=dict(os.environ, NODE_NO_WARNINGS='1'))
    log.write_text(result.stdout + result.stderr)
    return {'command': list(args), 'exit_code': result.returncode,
            'stdout': result.stdout, 'stderr': result.stderr,
            'log_sha256': sha256(log)}


def stage(output: Path, mode: str) -> dict[str, object]:
    root = output / mode
    root.mkdir()
    if mode == 'pinned':
        compiler = pinned_names('compiler', ('.ss', '.compact'))
        fixtures = pinned_names('test-contracts', ('.compact',))
        require(len(fixtures) == 372, 'pinned fixture count changed')
        names = sorted(set(compiler + fixtures + list(ORIGINALS)))
        for name in names:
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(pinned_bytes(name))
    else:
        compiler = current_names('compiler', ('.ss', '.compact'))
        fixtures = current_names('test-contracts', ('.compact',))
        names = sorted(set(compiler + fixtures + list(ORIGINALS)))
        for name in names:
            source = ROOT / name
            require(source.is_file() and not source.is_symlink(),
                    f'current fixture source is not ordinary: {name}')
            target = root / name
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, target)
    before = file_hashes(root, names)
    check = invoke(root, CHECK_ARGS, output / f'{mode}-check.log')
    require(file_hashes(root, names) == before,
            f'original --check mutated {mode} input corpus')
    baseline = root / BASELINE
    baseline_copy = output / f'{mode}-baseline.md'
    shutil.copyfile(baseline, baseline_copy)
    report = invoke(root, ('node', CLI), output / f'{mode}-report.log')
    generated = baseline.read_text()
    match = REPORT_PATTERN.search(report['stdout'])
    require(report['exit_code'] == 0 and match is not None and
            int(match[3]) == len(fixtures),
            f'{mode} usage report did not inventory the required fixture context')
    return {'compiler_file_count': len(compiler), 'fixture_count': len(fixtures),
            'source_file_count': len(names), 'source_file_sha256': before,
            'check': check, 'report': report,
            'report_used': int(match[1]), 'report_offered': int(match[2]),
            'generated_report_sha256': hashlib.sha256(generated.encode()).hexdigest(),
            'baseline_sha256': sha256(baseline_copy),
            'generated_report_fixture_count': int(match[3])}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    source = source_state()
    signed = subprocess.check_output(['git', 'log', '-1', '--format=%G?%n%B'],
                                     cwd=ROOT, text=True)
    require(signed.startswith('G\n') and 'Signed-off-by:' in signed and
            'Assisted-by: AI' in signed, 'signed DCO+Assisted source head required')
    originals = {}
    for name in ORIGINALS:
        local = ROOT / name
        data = pinned_bytes(name)
        digest = hashlib.sha256(data).hexdigest()
        require(local.is_file() and not local.is_symlink() and
                sha256(local) == digest, f'pinned original changed: {name}')
        originals[name] = digest
    output = args.output.resolve()
    require(not args.output.is_symlink() and not output.exists(),
            'fresh output directory required')
    output.mkdir(parents=True)
    direct = invoke(ROOT, CHECK_ARGS, output / 'direct-current-check.log')
    contexts = {mode: stage(output, mode) for mode in ('current', 'pinned')}
    require(contexts['pinned']['generated_report_sha256'] ==
            contexts['pinned']['baseline_sha256'],
            'pinned compiler and fixtures did not regenerate the original report')
    require(direct['exit_code'] == contexts['current']['check']['exit_code'] and
            direct['stdout'] == contexts['current']['check']['stdout'] and
            direct['stderr'] == contexts['current']['check']['stderr'],
            'staged current fixture context changed the original check')
    require(source_state() == source and
            all(sha256(ROOT / name) == digest for name, digest in originals.items()),
            'source or pinned original changed during usage-index qualification')
    require(sha256(ROOT / CLI) == contexts['current']['source_file_sha256'][CLI]
            and sha256(ROOT / BASELINE) == contexts['current']['baseline_sha256'],
            'staged current original differs from exact-head CLI')
    statuses = [direct['exit_code']] + [row['check']['exit_code']
                                        for row in contexts.values()]
    receipt = {**source, 'format': 'ledger9-pinned-usage-index/v1',
               'status': 'passed' if all(code == 0 for code in statuses) else 'refused',
               'upstream_commit': PINNED, 'platform': os.uname().sysname + ' ' + os.uname().machine,
               'node_version': subprocess.check_output(['node', '--version'],
                                                       text=True).strip(),
               'original_files': originals, 'checker_sha256': sha256(Path(__file__)),
               'direct_current_check': direct, 'contexts': contexts,
               'limitations': ['usage mention is not a runtime assertion',
                               'current fixture corpus includes retained local fixtures',
                               'this check does not compile contracts or qualify proofs']}
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2,
                                                    sort_keys=True) + '\n')
    print(json.dumps({'status': receipt['status'], 'head': source['head'],
                      'current_fixtures': contexts['current']['fixture_count'],
                      'pinned_fixtures': contexts['pinned']['fixture_count'],
                      'output': str(output)}))
    if receipt['status'] != 'passed':
        raise SystemExit(1)


if __name__ == '__main__':
    main()
