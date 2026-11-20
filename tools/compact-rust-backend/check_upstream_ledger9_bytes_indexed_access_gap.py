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

"""Bind the pinned static Bytes loop result and current safe Rust refusal."""

import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess

from check_compactc_target import check_manifest
from check_upstream_ledger9_field_casts import (
    compiler_recipe_identity, require, run, runtime_identity,
    runtime_recipe_identity, sha256, source_state,
)


ROOT = Path(__file__).resolve().parents[2]
PINNED = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'
FAMILY = ROOT / 'test-contracts/primitives/bytes/loops'
CASE = FAMILY / 'indexed_access'
MANIFEST = FAMILY / 'indexed-access-original-source-sha256.json'
CONTROL = ROOT / 'tools/compact-rust-backend/tests/bytes_loops/dynamic_index_refusal.compact'


def originals() -> dict[str, str]:
    manifest = json.loads(MANIFEST.read_text())
    expected = {str((CASE / name).relative_to(ROOT)) for name in (
        'bytes_loop_indexed_access.compact', 'compile.pass.test.ts',
        'runtime.pass.test.ts')}
    require(manifest.get('upstream_commit') == PINNED and
            set(manifest.get('files', {})) == expected,
            'pinned indexed-access inventory changed')
    for name, digest in manifest['files'].items():
        path = ROOT / name
        upstream = subprocess.check_output(['git', 'show', f'{PINNED}:{name}'], cwd=ROOT)
        require(path.is_file() and not path.is_symlink() and
                re.fullmatch('[0-9a-f]{64}', digest) is not None and
                path.read_bytes() == upstream and sha256(path) == digest,
                f'pinned indexed-access original changed: {name}')
    require('expect(result).toEqual(32n)' in
            (CASE / 'runtime.pass.test.ts').read_text(),
            'pinned indexed-access result assertion changed')
    return manifest['files']


def files(root: Path) -> dict[str, str]:
    result = {}
    for path in sorted(root.rglob('*')):
        require(not path.is_symlink(), f'generated artifact symlink: {path}')
        if path.is_file():
            result[path.relative_to(root).as_posix()] = sha256(path)
    require(result, 'empty generated artifact inventory')
    return result


def refusal(compiler: Path, source: Path, destination: Path,
            log: Path, environment: dict[str, str], paired: bool,
            expected: str) -> dict:
    command = [str(compiler), '--target', 'ts']
    if paired:
        command += ['--target', 'rust', '--rust-runtime-root', str(ROOT),
                    '--rust-require-recording']
    command += ['--feature-zkir-v3', '--skip-zk', '--compact-path',
                str(ROOT / 'compiler'), str(source), str(destination)]
    outcome = subprocess.run(command, cwd=ROOT, env=environment,
                             capture_output=True, text=True)
    log.write_text(outcome.stdout + outcome.stderr)
    require(outcome.returncode != 0 and expected in outcome.stderr and
            not destination.exists(), f'indexed-access refusal changed: {source}')
    return {'exit': outcome.returncode, 'log_sha256': sha256(log),
            'output_published': destination.exists()}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'compiler-recipe', 'ts-runtime', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
    args = parser.parse_args()
    source = source_state()
    signature = subprocess.check_output(['git', 'log', '-1', '--format=%G?%n%B'],
                                        cwd=ROOT, text=True)
    require(signature.startswith('G\n') and 'Signed-off-by:' in signature and
            'Assisted-by: AI' in signature, 'head must be signed, DCO and assisted')
    output = args.output.resolve()
    require(not args.output.is_symlink() and not output.exists(),
            'output must be absent')
    compiler = args.compiler.resolve(strict=True)
    scheme = compiler.with_name('compactc-scheme').resolve(strict=True)
    runtime = args.ts_runtime.resolve(strict=True)
    installed = json.loads((runtime / 'package.json').read_text())
    package = json.loads((ROOT / 'runtime/package.json').read_text())
    require(all(installed[key] == package[key] for key in
                ('name', 'version', 'dependencies')),
            'TypeScript runtime package differs from source')
    nix = shutil.which('nix')
    require(nix is not None, 'Nix required for runtime binding')
    runtime_recipe = runtime_recipe_identity(runtime, Path(nix).resolve(strict=True))
    runtime_before = runtime_identity(runtime)
    recipe = compiler_recipe_identity(args.compiler_recipe, source, compiler, scheme)
    pinned = originals()
    local_paths = (MANIFEST, FAMILY / 'check.mjs', CONTROL,
                   ROOT / 'compiler/rust-ir-passes.ss',
                   ROOT / 'compiler/rust-vector-indices.ss')
    local = {str(path.relative_to(ROOT)): sha256(path) for path in local_paths}
    output.mkdir(parents=True)
    link = output / 'node_modules/@midnight-ntwrk/compact-runtime'
    link.parent.mkdir(parents=True)
    link.symlink_to(runtime, target_is_directory=True)
    (output / 'package.json').write_text('{"type":"module"}\n')
    environment = dict(os.environ, COMPACTC_SCHEME=str(scheme))
    destination = output / 'generated/typescript'
    contract = destination / 'contract'
    command = [str(compiler), '--target', 'ts', '--feature-zkir-v3', '--skip-zk',
               '--compact-path', str(ROOT / 'compiler'),
               str(CASE / 'bytes_loop_indexed_access.compact'), str(destination)]
    run(command, output / 'typescript-compile.log', environment)
    check_manifest(destination, require_zkir=True)
    info = json.loads((destination / 'compiler/contract-info.json').read_text())
    require(info['circuits'] == [{'name': 'bytes_loop_indexed_access',
                                  'pure': False, 'proof': True, 'arguments': [],
                                  'result-type': {'type-name': 'Field'}}],
            'pinned indexed-access capability changed')
    observed = run(['node', str(FAMILY / 'check.mjs'), str(contract / 'index.js'),
                    'indexed_access'], output / 'typescript-runtime.log', environment)
    require(observed.strip() ==
            'indexed_access: original Bytes loop result and ledger passed',
            'pinned indexed-access TypeScript result/state changed')
    generated = files(destination)
    paired = refusal(compiler, CASE / 'bytes_loop_indexed_access.compact',
                     output / 'generated/paired', output / 'paired-refusal.log',
                     environment, True, 'Rust backend requires a constant Bytes index')
    dynamic = refusal(compiler, CONTROL, output / 'generated/dynamic',
                      output / 'dynamic-refusal.log', environment, False,
                      'Bytes index did not reduce to a constant nonnegative value at compile time')
    require(source_state() == source and originals() == pinned and
            runtime_identity(runtime) == runtime_before and
            compiler_recipe_identity(args.compiler_recipe, source, compiler, scheme) == recipe and
            files(destination) == generated and
            all(sha256(ROOT / name) == digest for name, digest in local.items()),
            'indexed-access source, compiler, runtime or generated output changed')
    receipt = {**source, 'format': 'ledger9-bytes-indexed-access-gap/v1',
               'status': 'safe_refusal_qualified', 'platform': os.uname().sysname + ' ' + os.uname().machine,
               'upstream_commit': PINNED, 'original_files': pinned,
               'local_sources': local, 'compiler_recipe': recipe,
               'compiler_sha256': sha256(compiler), 'scheme_sha256': sha256(scheme),
               'runtime': runtime_before, 'runtime_recipe': runtime_recipe,
               'typescript_generated': generated, 'paired_rust_refusal': paired,
               'dynamic_frontend_refusal': dynamic,
               'typescript_runtime_log_sha256': sha256(output / 'typescript-runtime.log'),
               'checker_sha256': sha256(Path(__file__)),
               'proof_execution': False, 'funded_application': False,
               'limitation': 'static loop Bytes index lacks circuit-certified per-iteration Rust projection; native Linux and Rust acceptance remain open'}
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': receipt['status'], 'head': source['head'],
                      'output': str(output)}))


if __name__ == '__main__':
    main()
