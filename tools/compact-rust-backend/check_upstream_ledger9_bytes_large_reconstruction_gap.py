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

"""Retain the pinned slow Bytes reconstruction and its safe Rust refusal."""

import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import time

from check_compactc_target import check_manifest
from check_upstream_ledger9_field_casts import (
    compiler_recipe_identity, require, run, runtime_identity,
    runtime_recipe_identity, sha256, source_state,
)


ROOT = Path(__file__).resolve().parents[2]
PINNED = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'
CASE = ROOT / 'test-contracts/primitives/bytes/manual_update/slow/large_reconstruction'
SOURCE = CASE / 'bytes_manual_update_large_reconstruction.compact'
MANIFEST = CASE.parent / 'large-reconstruction-original-source-sha256.json'
EXPECTED = [1, 4, 9, 16, 25, 36, 49, 14, 31, 50]


def originals() -> dict[str, str]:
    manifest = json.loads(MANIFEST.read_text())
    expected = {str((CASE / name).relative_to(ROOT)) for name in (
        SOURCE.name, 'compile.pass.test.ts', 'runtime.pass.test.ts')}
    require(manifest.get('upstream_commit') == PINNED and
            set(manifest.get('files', {})) == expected,
            'pinned large-reconstruction inventory changed')
    for name, digest in manifest['files'].items():
        path = ROOT / name
        upstream = subprocess.check_output(['git', 'show', f'{PINNED}:{name}'], cwd=ROOT)
        require(path.is_file() and not path.is_symlink() and
                re.fullmatch('[0-9a-f]{64}', digest) is not None and
                path.read_bytes() == upstream and sha256(path) == digest,
                f'pinned large-reconstruction original changed: {name}')
    require('Uint8Array.from([1, 4, 9, 16, 25, 36, 49, 14, 31, 50])' in
            (CASE / 'runtime.pass.test.ts').read_text(),
            'pinned slow runtime assertion changed')
    return manifest['files']


def generated_files(root: Path) -> dict[str, str]:
    files = {}
    for path in sorted(root.rglob('*')):
        require(not path.is_symlink(), f'generated artifact symlink: {path}')
        if path.is_file():
            files[path.relative_to(root).as_posix()] = sha256(path)
    require(files, 'empty generated artifact inventory')
    return files


def compile_one(compiler: Path, source: Path, dest: Path, log: Path,
                environment: dict[str, str], paired: bool) -> dict:
    command = [str(compiler), '--target', 'ts']
    if paired:
        command += ['--target', 'rust', '--rust-runtime-root', str(ROOT),
                    '--rust-require-recording']
    command += ['--feature-zkir-v3', '--skip-zk', '--compact-path',
                str(ROOT / 'compiler'), str(source), str(dest)]
    started = time.monotonic()
    outcome = subprocess.run(command, cwd=ROOT, env=environment,
                             capture_output=True, text=True)
    log.write_text(outcome.stdout + outcome.stderr)
    return {'exit': outcome.returncode, 'seconds': round(time.monotonic() - started, 3),
            'log_sha256': sha256(log), 'output_published': dest.exists()}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'compiler-recipe', 'ts-runtime', 'zkir', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
    args = parser.parse_args()
    source = source_state()
    signature = subprocess.check_output(['git', 'log', '-1', '--format=%G?%n%B'],
                                        cwd=ROOT, text=True)
    require(signature.startswith('G\n') and 'Signed-off-by:' in signature and
            'Assisted-by: AI' in signature, 'signed DCO+Assisted head required')
    output = args.output.resolve()
    require(not output.exists(), 'fresh output required')
    compiler = args.compiler.resolve(strict=True)
    scheme = (compiler.parent / 'compactc-scheme').resolve(strict=True)
    runtime = args.ts_runtime.resolve(strict=True)
    zkir_tool = args.zkir.resolve(strict=True)
    nix = shutil.which('nix')
    require(nix is not None, 'Nix required for runtime binding')
    zkir_store = subprocess.check_output(
        [nix, 'eval', '--offline', '--raw', '--option', 'substitute',
         'false', '.#zkir-v3.outPath'], cwd=ROOT, text=True).strip()
    require(zkir_tool == Path(zkir_store) / 'bin/zkir' and
            subprocess.check_output([str(zkir_tool), '--version'], text=True).strip() ==
            'midnight-zkir 3.2.0', 'released ZKIR v3 tool changed')
    runtime_recipe = runtime_recipe_identity(runtime, Path(nix).resolve(strict=True))
    runtime_before = runtime_identity(runtime)
    recipe = compiler_recipe_identity(args.compiler_recipe, source, compiler, scheme)
    pinned = originals()
    local_paths = (MANIFEST, CASE / 'check.mjs', Path(__file__),
                   ROOT / 'compiler/rust-ir-passes.ss',
                   ROOT / 'compiler/rust-vector-indices.ss')
    local = {str(path.relative_to(ROOT)): sha256(path) for path in local_paths}
    output.mkdir(parents=True)
    link = output / 'node_modules/@midnight-ntwrk/compact-runtime'
    link.parent.mkdir(parents=True)
    link.symlink_to(runtime, target_is_directory=True)
    (output / 'package.json').write_text('{"type":"module"}\n')
    environment = dict(os.environ, COMPACTC_SCHEME=str(scheme))

    ts_dest = output / 'generated/typescript'
    ts = compile_one(compiler, SOURCE, ts_dest, output / 'typescript-compile.log',
                     environment, False)
    require(ts['exit'] == 0 and ts['output_published'], 'slow TypeScript compile failed')
    check_manifest(ts_dest, require_zkir=True)
    info = json.loads((ts_dest / 'compiler/contract-info.json').read_text())
    require(info['circuits'] == [{
        'name': 'bytes_manual_update_large_reconstruction', 'pure': False,
        'proof': True, 'arguments': [],
        'result-type': {'type-name': 'Bytes', 'length': 10},
    }], 'slow circuit capability changed')
    observed = run(['node', str(CASE / 'check.mjs'),
                    str(ts_dest / 'contract/index.js')],
                   output / 'typescript-runtime.log', environment)
    require(observed.strip() ==
            'large_reconstruction: original result, ledger, and 315 operations passed',
            'slow TypeScript result/state/transcript changed')
    ts_files = generated_files(ts_dest)
    raw_zkir = ts_dest / 'zkir/bytes_manual_update_large_reconstruction.zkir'
    model_input = output / 'model-input.zkir'
    shutil.copyfile(raw_zkir, model_input)
    require(sha256(model_input) == sha256(raw_zkir), 'slow model input changed')
    run([str(zkir_tool), 'mock-compile', str(model_input)],
        output / 'zkir-model.log', environment)
    model = re.search(r'k=(\d+), rows=(\d+)',
                      (output / 'zkir-model.log').read_text())
    require(model is not None and (int(model[1]), int(model[2])) == (16, 40652),
            'slow ZKIR complexity model changed')

    paired_dest = output / 'generated/paired'
    paired_log = output / 'paired-refusal.log'
    paired = compile_one(compiler, SOURCE, paired_dest, paired_log, environment, True)
    require(paired['exit'] != 0 and not paired['output_published'] and
            'line 27 char 14' in paired_log.read_text() and
            'Rust backend has no unique constant-valued Vector index for this occurrence'
            in paired_log.read_text(), 'slow Rust refusal changed')

    mutant = output / 'constant-vector-index.compact'
    original_text = SOURCE.read_text()
    require(original_text.count('base_data[i]') == 3,
            'expected three pinned Vector index occurrences')
    mutant.write_text(original_text.replace('base_data[i]', 'base_data[0]'))
    mutant_dest = output / 'generated/constant-vector-index'
    mutant_gate = compile_one(compiler, mutant, mutant_dest,
                              output / 'constant-index-compile.log', environment, True)
    require(mutant_gate['exit'] == 0 and mutant_gate['output_published'],
            'constant-index diagnostic no longer compiles')
    mutant_gate['source_sha256'] = sha256(mutant)
    mutant_gate['generated_sha256'] = generated_files(mutant_dest)

    require(source_state() == source and originals() == pinned and
            runtime_identity(runtime) == runtime_before and
            compiler_recipe_identity(args.compiler_recipe, source, compiler, scheme) == recipe and
            generated_files(ts_dest) == ts_files and
            all(sha256(ROOT / name) == digest for name, digest in local.items()),
            'slow source, compiler, runtime or generated output changed')
    receipt = {**source, 'format': 'ledger9-bytes-large-reconstruction-gap/v1',
               'status': 'safe_refusal_qualified',
               'platform': os.uname().sysname + ' ' + os.uname().machine,
               'upstream_commit': PINNED, 'original_files': pinned,
               'local_sources': local, 'compiler_recipe': recipe,
               'compiler_sha256': sha256(compiler), 'scheme_sha256': sha256(scheme),
               'runtime': runtime_before, 'runtime_recipe': runtime_recipe,
               'typescript_generated': ts_files, 'typescript_compile': ts,
               'typescript_runtime_log_sha256': sha256(output / 'typescript-runtime.log'),
               'result_and_ledger': EXPECTED, 'public_operations': 315,
               'zkir_sha256': sha256(raw_zkir), 'zkir_size': raw_zkir.stat().st_size,
               'zkir_model': {'k': 16, 'rows': 40652,
                              'log_sha256': sha256(output / 'zkir-model.log'),
                              'model_input_sha256': sha256(model_input),
                              'bzkir_sha256': sha256(output / 'model-input.bzkir')},
               'zkir_tool_sha256': sha256(zkir_tool), 'zkir_store': zkir_store,
               'paired_rust_refusal': paired,
               'constant_index_diagnostic': mutant_gate,
               'proof_execution': False, 'funded_application': False,
               'limitation': 'ten-iteration Vector index domain exceeds the current eight-item certified dispatch on #750; native Linux and Rust acceptance remain open'}
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': receipt['status'], 'head': source['head'],
                      'output': str(output)}))


if __name__ == '__main__':
    main()
