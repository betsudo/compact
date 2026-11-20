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

"""Run twenty pinned keyword families through the original Vitest host."""

import argparse
import json
import os
from pathlib import Path
import re
import subprocess

from check_upstream_ledger9_field_casts import (
    compiler_recipe_identity, require, sha256, source_state,
)
from stage_upstream_keyword_vitest_host import (
    CASE_COUNT, FAMILIES, FAMILY_COUNTS, HOST_TEST_COUNT,
    ORIGINAL_COUNT, PINNED, ROOT,
)

ANSI_SGR = re.compile(r'\x1b\[[0-9;]*m')


def original_host_counts_match(stdout: str) -> bool:
    # Vitest colors its summary on Linux even when stdout is captured. Strip
    # only SGR styling; keep the original per-case markers and exact counts.
    summary = ANSI_SGR.sub('', stdout)
    return (re.search(rf'Tests\s+{HOST_TEST_COUNT} passed \({HOST_TEST_COUNT}\)', summary) is not None
            and stdout.count('✓') == HOST_TEST_COUNT
            and re.search(r'Test Files\s+1 passed \(1\)', summary) is not None)


def run(command: list[str], cwd: Path, log: Path,
        env: dict[str, str]) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(command, cwd=cwd, env=env, capture_output=True,
                            text=True)
    log.write_text(result.stdout + result.stderr)
    require(result.returncode == 0, f'host command failed: {log}')
    return result


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--stage', type=Path, required=True)
    parser.add_argument('--compiler', type=Path, required=True)
    parser.add_argument('--compiler-recipe', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    source = source_state()
    stage = args.stage.resolve(strict=True)
    require(stage.is_dir() and not args.stage.is_symlink(),
            'ordinary pinned stage required')
    staging_path = stage / 'staging-receipt.json'
    staging = json.loads(staging_path.read_text())
    require(staging['status'] == 'staged' and staging['head'] == source['head'] and
            staging['tree'] == source['tree'] and staging['upstream_commit'] == PINNED and
            tuple(staging['families']) == FAMILIES and
            {family: tuple(row) for family, row in
             staging['family_counts'].items()} == FAMILY_COUNTS and
            staging['original_count'] == ORIGINAL_COUNT and
            staging['case_count'] == CASE_COUNT and
            staging['host_test_count'] == HOST_TEST_COUNT and
            staging['stager_sha256'] == sha256(ROOT /
                'tools/compact-rust-backend/stage_upstream_keyword_vitest_host.py'),
            'exact-head complete host staging receipt required')
    originals = staging['original_files']
    require(len(originals) == ORIGINAL_COUNT and
            all(sha256(stage / name) == digest for name, digest in originals.items()),
            'byte-exact pinned original host files required')
    runtime = Path(staging['runtime']).resolve(strict=True)
    require(sha256(runtime / 'package.json') == staging['runtime_package_sha256']
            and sha256(runtime / 'dist/index.js') ==
            staging['runtime_entry_sha256'] and
            sha256(stage / 'runtime/index.js') == staging['adapter_sha256'],
            'released runtime or narrow adapter changed')
    compiler = args.compiler.resolve(strict=True)
    scheme = compiler.with_name('compactc-scheme').resolve(strict=True)
    recipe = compiler_recipe_identity(args.compiler_recipe, source, compiler,
                                      scheme)
    output = args.output.resolve()
    require(not output.exists() and not args.output.is_symlink(),
            'fresh host output directory required')
    output.mkdir(parents=True)
    tests = stage / 'test-contracts'
    env = dict(os.environ, COMPACT_BINARY=str(compiler),
               PYTHONDONTWRITEBYTECODE='1')
    yarn = run(['corepack', 'yarn', '--version'], tests,
               output / 'yarn-version.log', env)
    require(yarn.stdout.strip() == '4.16.0', 'pinned Yarn version changed')
    run(['corepack', 'yarn', 'install', '--immutable'], tests,
        output / 'yarn-install.log', env)
    package = tests / 'node_modules/vitest/package.json'
    require(json.loads(package.read_text())['version'] == '4.1.8',
            'pinned Vitest version changed')
    selected = [f'keywords/{family}' for family in FAMILIES]
    result = run(['node', 'compact-test-runner.ts', *selected,
                  '--maxWorkers=1'], tests, output / 'original-vitest.log', env)
    require(original_host_counts_match(result.stdout),
            'original keyword host count changed')
    require(source_state() == source and
            all(sha256(stage / name) == digest for name, digest in originals.items()) and
            sha256(stage / 'runtime/index.js') == staging['adapter_sha256'] and
            compiler_recipe_identity(args.compiler_recipe, source, compiler,
                                     scheme) == recipe,
            'source, original host, adapter or compiler changed during Vitest')
    counts = {'cases': CASE_COUNT,
              'compile_pass': sum(row[1] for row in FAMILY_COUNTS.values()),
              'compile_fail': sum(row[2] for row in FAMILY_COUNTS.values()),
              'runtime_pass': sum(row[3] for row in FAMILY_COUNTS.values()),
              'runtime_fail': sum(row[4] for row in FAMILY_COUNTS.values()),
              'host_tests': HOST_TEST_COUNT}
    report = {**source, 'status': 'passed', 'upstream_commit': PINNED,
              'platform': os.uname().sysname + ' ' + os.uname().machine,
              'counts': counts, 'families': FAMILIES,
              'staging_receipt_sha256': sha256(staging_path),
              'original_files_sha256': originals,
              'compiler_recipe_sha256': sha256(args.compiler_recipe),
              'compiler_sha256': sha256(compiler),
              'runtime_package_sha256': staging['runtime_package_sha256'],
              'adapter_sha256': staging['adapter_sha256'],
              'yarn_install_log_sha256': sha256(output / 'yarn-install.log'),
              'vitest_log_sha256': sha256(output / 'original-vitest.log'),
              'runner_sha256': sha256(Path(__file__)),
              'node_version': subprocess.check_output(['node', '--version'],
                                                      text=True).strip(),
              'limits': ['host uses released 0.18.0 context-call adapter',
                         'native Linux and proof/funded transaction gates are separate']}
    (output / 'receipt.json').write_text(json.dumps(report, indent=2,
                                                   sort_keys=True) + '\n')
    print(json.dumps({'status': 'passed', 'head': source['head'],
                      'cases': CASE_COUNT, 'host_tests': HOST_TEST_COUNT}))


if __name__ == '__main__':
    main()
