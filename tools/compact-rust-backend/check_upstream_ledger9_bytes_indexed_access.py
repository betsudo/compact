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

"""Replay the pinned static Bytes indexed Fold on generated TypeScript/Rust."""

import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tomllib

from check_compactc_target import check_manifest
from check_unit_mutation_initializers import validate_consumer_lock
from check_upstream_ledger9_bytes_indexed_access_gap import originals, files, refusal
from check_upstream_ledger9_field_casts import (
    compiler_recipe_identity, require, run, runtime_identity,
    runtime_recipe_identity, sha256, source_state,
)


ROOT = Path(__file__).resolve().parents[2]
FAMILY = ROOT / 'test-contracts/primitives/bytes/loops'
CASE = FAMILY / 'indexed_access'
SOURCE = CASE / 'bytes_loop_indexed_access.compact'
CONTROL = ROOT / 'tools/compact-rust-backend/tests/bytes_loops/dynamic_index_refusal.compact'


def generated_files(root: Path) -> dict[str, str]:
    return files(root)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'compiler-recipe', 'ts-runtime', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--cargo-target', type=Path)
    args = parser.parse_args()
    source = source_state()
    signature = subprocess.check_output(['git', 'log', '-1', '--format=%G?%n%B'],
                                        cwd=ROOT, text=True)
    require(signature.startswith('G\n') and 'Signed-off-by:' in signature and
            'Assisted-by: AI' in signature, 'signed DCO+Assisted head required')
    require(not args.output.exists() and not args.output.is_symlink(),
            'fresh output required')
    compiler = args.compiler.resolve(strict=True)
    scheme = compiler.with_name('compactc-scheme').resolve(strict=True)
    runtime = args.ts_runtime.resolve(strict=True)
    package = json.loads((ROOT / 'runtime/package.json').read_text())
    installed = json.loads((runtime / 'package.json').read_text())
    require(all(package[key] == installed[key] for key in
                ('name', 'version', 'dependencies')), 'runtime package differs from source')
    nix = shutil.which('nix')
    require(nix is not None, 'Nix required for runtime binding')
    runtime_recipe = runtime_recipe_identity(runtime, Path(nix).resolve(strict=True))
    runtime_before = runtime_identity(runtime)
    recipe = compiler_recipe_identity(args.compiler_recipe, source, compiler, scheme)
    pinned = originals()
    local_paths = (CASE / 'check.mjs', CASE / 'consumer.rs',
                   ROOT / 'compiler/rust-vector-indices.ss',
                   ROOT / 'compiler/rust-ir-passes.ss',
                   ROOT / 'tools/compact-rust-backend/check_static_bytes_fold_index_domain.py',
                   CONTROL)
    local = {str(path.relative_to(ROOT)): sha256(path) for path in local_paths}
    args.output.mkdir(parents=True)
    link = args.output / 'node_modules/@midnight-ntwrk/compact-runtime'
    link.parent.mkdir(parents=True)
    link.symlink_to(runtime, target_is_directory=True)
    (args.output / 'package.json').write_text('{"type":"module"}\n')
    environment = dict(os.environ, COMPACTC_SCHEME=str(scheme))
    results = {}
    for mode, targets in (('typescript', ('ts',)), ('both', ('ts', 'rust'))):
        destination = args.output / 'generated' / mode
        command = [str(compiler)]
        for target in targets:
            command += ['--target', target]
        command += ['--feature-zkir-v3', '--skip-zk', '--compact-path',
                    str(ROOT / 'compiler'), str(SOURCE), str(destination)]
        run(command, args.output / f'{mode}-compile.log', environment)
        check_manifest(destination, require_zkir=True)
        contract = destination / 'contract'
        observed = run(['node', str(CASE / 'check.mjs'), str(contract / 'index.js')],
                       args.output / f'{mode}-runtime.log', environment)
        require(observed.strip() ==
                'indexed_access: original result, ledger, transcript and gas passed',
                f'generated TypeScript indexed Fold changed: {mode}')
        info = json.loads((destination / 'compiler/contract-info.json').read_text())
        require(info['circuits'] == [{'name': 'bytes_loop_indexed_access',
                                      'pure': False, 'proof': True, 'arguments': [],
                                      'result-type': {'type-name': 'Field'}}],
                f'indexed access capability changed: {mode}')
        results[mode] = {'typescript_sha256': sha256(contract / 'index.js'),
                         'zkir_sha256': sha256(destination / 'zkir/bytes_loop_indexed_access.zkir'),
                         'contract_info_sha256': sha256(destination / 'compiler/contract-info.json'),
                         'runtime_log_sha256': sha256(args.output / f'{mode}-runtime.log')}
        if mode == 'both':
            results[mode].update(rust_sha256=sha256(contract / 'lib.rs'),
                                 rust_ir_sha256=sha256(contract / 'compact-rust-ir.json'))
    for key in ('typescript_sha256', 'zkir_sha256', 'contract_info_sha256'):
        require(results['typescript'][key] == results['both'][key],
                f'paired target changed {key}')
    generated_before = generated_files(args.output / 'generated')
    dynamic = refusal(compiler, CONTROL, args.output / 'generated-dynamic',
                      args.output / 'dynamic-refusal.log', environment, False,
                      'Bytes index did not reduce to a constant nonnegative value at compile time')
    consumer = args.output / 'rust-consumer'
    (consumer / 'tests').mkdir(parents=True)
    shutil.copyfile(CASE / 'consumer.rs', consumer / 'tests/indexed_access.rs')
    shutil.copyfile(ROOT / 'Cargo.lock', consumer / 'Cargo.lock')
    contract = args.output / 'generated/both/contract'
    package_name = tomllib.loads((contract / 'Cargo.toml').read_text())['package']['name']
    manifest = consumer / 'Cargo.toml'
    manifest.write_text('[workspace]\n[package]\nname = "ledger9-bytes-indexed-access"\n'
                        'version = "0.1.0"\nedition = "2024"\n[dependencies]\n'
                        'midnight-onchain-vm = "=4.0.1"\n'
                        'midnight-serialize = "=1.2.1"\n'
                        f'indexed_access = {{ package = {json.dumps(package_name)}, '
                        f'path = {json.dumps(str(contract))} }}\n')
    cargo_target = args.cargo_target.resolve(strict=True) if args.cargo_target else args.output / 'cargo-target'
    require(not args.cargo_target or (cargo_target.is_dir() and
                                     not args.cargo_target.is_symlink()),
            'shared Cargo target must be an ordinary directory')
    cargo_env = dict(environment, CARGO_TARGET_DIR=str(cargo_target), CARGO_INCREMENTAL='0',
                     CARGO_PROFILE_DEV_DEBUG='0')
    metadata = json.loads(run(['cargo', '+1.99.0', 'metadata', '--offline',
                               '--format-version', '1', '--manifest-path', str(manifest)],
                              args.output / 'cargo-metadata.log', cargo_env))
    packages = tomllib.loads((ROOT / 'Cargo.lock').read_text())['package']
    pinned_packages = {(row['name'], row['version'], row['source']): row.get('checksum')
                       for row in packages if 'source' in row}
    allowed = {manifest, contract / 'runtime-rs/Cargo.toml',
               contract / 'runtime-rs-macros/Cargo.toml', contract / 'Cargo.toml'}
    validate_consumer_lock(metadata, consumer / 'Cargo.lock', allowed, pinned_packages)
    run(['cargo', '+1.99.0', 'test', '--locked', '--offline', '--manifest-path', str(manifest),
         '--test', 'indexed_access'], args.output / 'cargo-test.log', cargo_env)
    require('test indexed_access_original_result_recording_and_replay ... ok' in
            (args.output / 'cargo-test.log').read_text(), 'Rust original result/replay changed')
    require(source_state() == source and runtime_identity(runtime) == runtime_before and
            compiler_recipe_identity(args.compiler_recipe, source, compiler, scheme) == recipe and
            originals() == pinned and generated_files(args.output / 'generated') == generated_before and
            all(sha256(ROOT / name) == digest for name, digest in local.items()),
            'source, generated artifact or runtime changed')
    receipt = {**source, 'status': 'passed', 'platform': os.uname().sysname + ' ' + os.uname().machine,
               'upstream_commit': '672bd0f226f47a7d036c5c074ce71fbe1af1925e',
               'original_files': pinned, 'local_sources': local,
               'compiler_recipe': recipe, 'runtime_recipe': runtime_recipe,
               'compiler_sha256': sha256(compiler), 'scheme_sha256': sha256(scheme),
               'runtime': runtime_before, 'generated': results,
               'generated_files_sha256': generated_before, 'dynamic_refusal': dynamic,
               'rust_test_log_sha256': sha256(args.output / 'cargo-test.log'),
               'rust_consumer_lock_sha256': sha256(consumer / 'Cargo.lock'),
               'checker_sha256': sha256(Path(__file__)),
               'proof_execution': False, 'funded_application': False,
               'limitations': ['local generated parity only; fresh proof/funded and native Linux required',
                               'one pinned stateful Bytes indexed Fold, not dynamic Bytes indexing']}
    (args.output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': 'passed', 'head': source['head'], 'output': str(args.output)}))


if __name__ == '__main__':
    main()
