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

"""Qualify paired Rust/TypeScript execution of the pinned slow Bytes reconstruction."""

import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import time
import tomllib

from check_compactc_target import check_manifest
from check_unit_mutation_initializers import validate_consumer_lock
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
RUST_TEST_STACK_BYTES = 4 * 1024 * 1024


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
    parser.add_argument('--cargo-target', type=Path)
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
    local_paths = (MANIFEST, CASE / 'check.mjs', CASE / 'consumer.rs', Path(__file__),
                   ROOT / 'tools/compact-rust-backend/audit_upstream_ledger9_bytes_large_reconstruction.py',
                   ROOT / 'compiler/rust-ir-passes.ss',
                   ROOT / 'compiler/rust-vector-indices.ss',
                   ROOT / 'compiler/test-rust-vector-indices.ss',
                   ROOT / 'tools/compact-rust-backend/check_static_for_index_domain.py',
                   ROOT / 'Cargo.lock')
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
                    str(ts_dest / 'contract/index.js'),
                    str(output / 'typescript-runtime.json'),
                    str(output / 'typescript-transcript.json')],
                   output / 'typescript-runtime.log', environment)
    require(observed.strip() ==
            'large_reconstruction: original result, ledger, and 315 operations passed',
            'slow TypeScript result/state/transcript changed')
    ts_result = json.loads((output / 'typescript-runtime.json').read_text())
    require(ts_result['result'] == ts_result['ledger'] == EXPECTED and
            ts_result['public_operations'] == 315 and
            ts_result['private_operations'] == 0 and
            set(ts_result['gas']) == {'readTime', 'computeTime',
                                      'bytesWritten', 'bytesDeleted'} and
            all(int(value) >= 0 for value in ts_result['gas'].values()),
            'slow TypeScript parity inputs changed')
    ts_transcript = json.loads((output / 'typescript-transcript.json').read_text())
    require(isinstance(ts_transcript, list) and len(ts_transcript) == 315,
            'slow TypeScript ordered transcript changed')
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
    paired_log = output / 'paired-compile.log'
    paired = compile_one(compiler, SOURCE, paired_dest, paired_log, environment, True)
    require(paired['exit'] == 0 and paired['output_published'],
            'untouched slow original did not compile for paired targets')
    check_manifest(paired_dest, require_zkir=True)
    require((paired_dest / 'compiler/contract-info.json').read_bytes() ==
            (ts_dest / 'compiler/contract-info.json').read_bytes() and
            sha256(paired_dest / 'zkir/bytes_manual_update_large_reconstruction.zkir') ==
            sha256(raw_zkir), 'paired capability or raw ZKIR differs from TypeScript')
    paired_files = generated_files(paired_dest)
    require(SOURCE.read_text().count('base_data[i]') == 3,
            'pinned original Vector index occurrences changed')

    consumer = output / 'rust-consumer'
    (consumer / 'tests').mkdir(parents=True)
    shutil.copyfile(CASE / 'consumer.rs', consumer / 'tests/large_reconstruction.rs')
    shutil.copyfile(ROOT / 'Cargo.lock', consumer / 'Cargo.lock')
    package = tomllib.loads((paired_dest / 'contract/Cargo.toml').read_text())['package']['name']
    manifest = consumer / 'Cargo.toml'
    manifest.write_text('[workspace]\n[package]\nname = "ledger9-bytes-large-reconstruction-consumer"\n'
                        'version = "0.1.0"\nedition = "2024"\n[dependencies]\n'
                        'midnight-onchain-vm = "=4.0.1"\nmidnight-serialize = "=1.2.1"\n'
                        'serde_json = "=1.0.149"\n'
                        f'large_reconstruction = {{ package = {json.dumps(package)}, '
                        f'path = {json.dumps(str(paired_dest / "contract"))} }}\n')
    cargo_target = (args.cargo_target.resolve(strict=True) if args.cargo_target else
                    output / 'cargo-target')
    require(not args.cargo_target or
            (cargo_target.is_dir() and not args.cargo_target.is_symlink()),
            'shared Cargo target must be an ordinary directory')
    cargo_env = dict(environment, CARGO_TARGET_DIR=str(cargo_target),
                     CARGO_INCREMENTAL='0', CARGO_PROFILE_DEV_DEBUG='0',
                     # The generated ordinary circuit frame exceeds Darwin's
                     # default 2 MiB Rust test-thread stack. The exact compiled
                     # binary passes at 3 MiB; use 4 MiB with margin here.
                     RUST_MIN_STACK=str(RUST_TEST_STACK_BYTES),
                     TS_GAS_READ_TIME=ts_result['gas']['readTime'],
                     TS_GAS_COMPUTE_TIME=ts_result['gas']['computeTime'],
                     TS_GAS_BYTES_WRITTEN=ts_result['gas']['bytesWritten'],
                     TS_GAS_BYTES_DELETED=ts_result['gas']['bytesDeleted'],
                     RUST_TRANSCRIPT_PATH=str(output / 'rust-transcript.json'))
    metadata = json.loads(run(['cargo', '+1.99.0', 'metadata', '--offline',
                               '--format-version', '1', '--manifest-path', str(manifest)],
                              output / 'cargo-metadata.log', cargo_env))
    root_lock = tomllib.loads((ROOT / 'Cargo.lock').read_text())['package']
    locked = {(row['name'], row['version'], row['source']): row.get('checksum')
              for row in root_lock if 'source' in row}
    allowed = {manifest, ROOT / 'runtime-rs/Cargo.toml',
               ROOT / 'runtime-rs-macros/Cargo.toml', paired_dest / 'contract/Cargo.toml'}
    validate_consumer_lock(metadata, consumer / 'Cargo.lock', allowed, locked)
    run(['cargo', '+1.99.0', 'test', '--locked', '--offline',
         '--manifest-path', str(manifest), '--test', 'large_reconstruction'],
        output / 'cargo-test.log', cargo_env)
    rust_log = (output / 'cargo-test.log').read_text()
    require('test result: ok. 1 passed; 0 failed;' in rust_log and
            'test untouched_original_ordinary_recorded_and_replay ... ok' in rust_log,
            'untouched slow Rust ordinary/recorded parity failed')
    rust_transcript = json.loads((output / 'rust-transcript.json').read_text())
    require(rust_transcript == ts_transcript,
            'ordered Rust and TypeScript public transcript contents differ')

    require(source_state() == source and originals() == pinned and
            runtime_identity(runtime) == runtime_before and
            compiler_recipe_identity(args.compiler_recipe, source, compiler, scheme) == recipe and
            generated_files(ts_dest) == ts_files and
            generated_files(paired_dest) == paired_files and
            all(sha256(ROOT / name) == digest for name, digest in local.items()),
            'slow source, compiler, runtime or generated output changed')
    receipt = {**source, 'format': 'ledger9-bytes-large-reconstruction-paired/v1',
               'status': 'paired_semantics_qualified',
               'platform': os.uname().sysname + ' ' + os.uname().machine,
               'upstream_commit': PINNED, 'original_files': pinned,
               'local_sources': local, 'compiler_recipe': recipe,
               'compiler_sha256': sha256(compiler), 'scheme_sha256': sha256(scheme),
               'runtime': runtime_before, 'runtime_recipe': runtime_recipe,
               'typescript_generated': ts_files, 'typescript_compile': ts,
               'typescript_runtime_log_sha256': sha256(output / 'typescript-runtime.log'),
               'typescript_runtime_sha256': sha256(output / 'typescript-runtime.json'),
               'typescript_observation': ts_result,
               'typescript_transcript_sha256': sha256(output / 'typescript-transcript.json'),
               'rust_transcript_sha256': sha256(output / 'rust-transcript.json'),
               'ordered_transcript_parity': True,
               'result_and_ledger': EXPECTED, 'public_operations': 315,
               'zkir_sha256': sha256(raw_zkir), 'zkir_size': raw_zkir.stat().st_size,
               'zkir_model': {'k': 16, 'rows': 40652,
                              'log_sha256': sha256(output / 'zkir-model.log'),
                              'model_input_sha256': sha256(model_input),
                              'bzkir_sha256': sha256(output / 'model-input.bzkir')},
               'zkir_tool_sha256': sha256(zkir_tool), 'zkir_store': zkir_store,
               'paired_compile': paired, 'paired_generated': paired_files,
               'rust_consumer_manifest_sha256': sha256(manifest),
               'rust_consumer_source_sha256': sha256(consumer / 'tests/large_reconstruction.rs'),
               'rust_consumer_lock_sha256': sha256(consumer / 'Cargo.lock'),
               'cargo_metadata_log_sha256': sha256(output / 'cargo-metadata.log'),
               'cargo_test_log_sha256': sha256(output / 'cargo-test.log'),
               'cargo_target': str(cargo_target),
               'rust_test_stack_bytes': RUST_TEST_STACK_BYTES,
               'proof_execution': False, 'funded_application': False,
               'limitation': 'Darwin default 2 MiB Rust test-thread stack overflows at the generated ordinary circuit entry; this gate uses 4 MiB. Fresh k16 proof, funded default-strict application and native Linux integration remain open'}
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': receipt['status'], 'head': source['head'],
                      'output': str(output)}))


if __name__ == '__main__':
    main()
