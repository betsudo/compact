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

"""Qualify the intact pinned Vector indexing results and three refusals."""

import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tomllib

from check_upstream_ledger9_vector_casts import (
    compiler_recipe_identity, require_unchanged, run, runtime_identity,
    runtime_recipe_identity, sha256, source_state,
)


ROOT = Path(__file__).resolve().parents[2]
FAMILY = ROOT / 'test-contracts/primitives/vector/indexing'
PINNED = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'
POSITIVES = ('basic', 'nested')
REFUSALS = {
    'non_constant_index': 'vector index did not reduce to a constant nonnegative value at compile time',
    'out_of_bounds': 'index 10 is out-of-bounds for a vector of length 4',
    'unknown_index_type': 'expected index to have an unsigned type, received Unknown',
}


def require(ok: bool, message: str) -> None:
    if not ok:
        raise RuntimeError(message)


def original_inventory(manifest_path: Path) -> dict[str, str]:
    manifest = json.loads(manifest_path.read_text())
    require(manifest.get('upstream_commit') == PINNED and
            isinstance(manifest.get('files'), dict) and len(manifest['files']) == 12,
            'pinned indexing inventory changed')
    expected = {f'test-contracts/primitives/vector/indexing/{name}/{filename}'
                for name in POSITIVES + tuple(REFUSALS)
                for filename in ((f'vector_indexing_{name}.compact',
                                  'compile.pass.test.ts', 'runtime.pass.test.ts')
                                 if name in POSITIVES else
                                 (f'vector_indexing_{name}.compact', 'compile.fail.test.ts'))}
    require(set(manifest['files']) == expected, 'pinned indexing file set changed')
    for name, digest in manifest['files'].items():
        file = ROOT / name
        require(re.fullmatch(r'[0-9a-f]{64}', digest) is not None and
                file.is_file() and not file.is_symlink() and sha256(file) == digest,
                f'pinned original indexing fixture changed: {name}')
    return manifest['files']


def refusal(result: subprocess.CompletedProcess[str], destination: Path,
            diagnostic: str) -> None:
    message = result.stdout + result.stderr
    require(result.returncode != 0 and diagnostic in message,
            f'original indexing refusal changed: {message}')
    require(not destination.exists() and
            not list(destination.parent.glob(f'.{destination.name}.compactc-stage-*')),
            'refused indexing case published output')


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'compiler-recipe', 'ts-runtime', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
    parser.add_argument('--cargo-target', type=Path)
    args = parser.parse_args()
    source = source_state()
    compiler = args.compiler.resolve(strict=True)
    scheme = compiler.with_name('compactc-scheme').resolve(strict=True)
    runtime = args.ts_runtime.resolve(strict=True)
    output = args.output.resolve()
    require(not args.output.is_symlink() and not output.exists(), 'output must be absent')
    source_package = json.loads((ROOT / 'runtime/package.json').read_text())
    runtime_package = json.loads((runtime / 'package.json').read_text())
    require(all(source_package[key] == runtime_package[key]
                for key in ('name', 'version', 'dependencies')),
            'TypeScript runtime package differs from source')
    nix = shutil.which('nix')
    require(nix is not None, 'Nix required for exact-head runtime binding')
    runtime_recipe = runtime_recipe_identity(runtime, Path(nix).resolve(strict=True))
    runtime_before = runtime_identity(runtime)
    compiler_recipe = compiler_recipe_identity(args.compiler_recipe, source, compiler, scheme)
    manifest_path = FAMILY / 'original-source-sha256.json'
    originals = original_inventory(manifest_path)
    output.mkdir(parents=True)
    link = output / 'node_modules/@midnight-ntwrk/compact-runtime'
    link.parent.mkdir(parents=True)
    link.symlink_to(runtime, target_is_directory=True)
    env = os.environ.copy()
    env['COMPACTC_SCHEME'] = str(scheme)
    generated = {}
    for name in POSITIVES:
        fixture = FAMILY / name / f'vector_indexing_{name}.compact'
        cases = {}
        for mode in ('typescript', 'both'):
            destination = output / 'generated' / mode / name
            command = [str(compiler), '--target', 'ts']
            if mode == 'both':
                command += ['--target', 'rust', '--rust-runtime-root', str(ROOT)]
            command += ['--feature-zkir-v3', '--skip-zk', '--compact-path',
                        str(ROOT / 'compiler'), str(fixture), str(destination)]
            run(command, output / f'{name}-{mode}.log', env=env)
            info = json.loads((destination / 'compiler/contract-info.json').read_text())
            circuits = info['circuits']
            require(len(circuits) == 1 and circuits[0]['name'] == f'vector_indexing_{name}'
                    and circuits[0]['pure'] and not circuits[0]['proof'] and
                    not circuits[0]['arguments'],
                    f'{name} indexing circuit capability changed')
            contract = destination / 'contract'
            result = run(['node', str(FAMILY / 'check.mjs'),
                          str(contract / 'index.js'), str(runtime), name],
                         output / f'{name}-{mode}-runtime.log', env=env)
            require(result.strip() == f'{name}: original indexing assertion passed',
                    f'{name} original TypeScript result changed')
            cases[mode] = {'typescript_sha256': sha256(contract / 'index.js'),
                           'contract_info_sha256': sha256(destination / 'compiler/contract-info.json'),
                           'runtime_result': result.strip()}
            if mode == 'both':
                cases[mode]['rust_sha256'] = sha256(contract / 'lib.rs')
                cases[mode]['rust_ir_sha256'] = sha256(contract / 'compact-rust-ir.json')
        require(cases['typescript']['typescript_sha256'] ==
                cases['both']['typescript_sha256'],
                f'{name} TypeScript output differs in paired compile')
        generated[name] = cases
    refused = {}
    for name, diagnostic in REFUSALS.items():
        refused[name] = {}
        for target in ('typescript', 'rust'):
            destination = output / 'refused' / target / name
            command = [str(compiler), '--target', 'ts' if target == 'typescript' else 'rust',
                       '--skip-zk', '--compact-path', str(ROOT / 'compiler'),
                       str(FAMILY / name / f'vector_indexing_{name}.compact'), str(destination)]
            result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True)
            log = output / f'{name}-{target}-refused.log'
            log.write_text(result.stdout + result.stderr)
            refusal(result, destination, diagnostic)
            refused[name][target] = {'diagnostic': diagnostic, 'log_sha256': sha256(log)}
    consumer = output / 'rust-consumer'
    (consumer / 'src').mkdir(parents=True)
    shutil.copyfile(FAMILY / 'consumer.rs', consumer / 'src/main.rs')
    cargo = ['[package]', 'name = "ledger9-vector-indexing-consumer"',
             'version = "0.1.0"', 'edition = "2024"', '[workspace]', '[dependencies]']
    for name in POSITIVES:
        contract = output / 'generated/both' / name / 'contract'
        package = tomllib.loads((contract / 'Cargo.toml').read_text())['package']['name']
        cargo.append(f'{name} = {{ package = {json.dumps(package)}, '
                     f'path = {json.dumps(str(contract))} }}')
    (consumer / 'Cargo.toml').write_text('\n'.join(cargo) + '\n')
    cargo_target = args.cargo_target.resolve() if args.cargo_target else output / 'rust-target'
    cargo_env = dict(env, CARGO_TARGET_DIR=str(cargo_target))
    run(['cargo', '+1.99.0', 'generate-lockfile', '--offline', '--manifest-path',
         str(consumer / 'Cargo.toml')], output / 'rust-lock.log', env=cargo_env)
    rust = run(['cargo', '+1.99.0', 'run', '--locked', '--offline', '--manifest-path',
                str(consumer / 'Cargo.toml')], output / 'rust-run.log', env=cargo_env)
    require(rust.strip() == 'two original Vector indexing assertions passed',
            'generated Rust original assertions changed')
    require_unchanged(source, runtime_before, compiler_recipe, runtime,
                      args.compiler_recipe, compiler, scheme)
    require(original_inventory(manifest_path) == originals,
            'original indexing files changed during qualification')
    receipt = {
        **source, 'source_clean': True,
        'platform': os.uname().sysname + ' ' + os.uname().machine,
        'upstream_commit': PINNED, 'original_files': originals,
        'manifest_sha256': sha256(manifest_path),
        'checker_sha256': sha256(Path(__file__)),
        'adapter_sha256': sha256(FAMILY / 'check.mjs'),
        'consumer_source_sha256': sha256(FAMILY / 'consumer.rs'),
        'identity_library_sha256': sha256(ROOT / 'tools/compact-rust-backend/check_upstream_ledger9_vector_casts.py'),
        'compiler_recipe': compiler_recipe, 'compiler_sha256': sha256(compiler),
        'scheme_sha256': sha256(scheme), 'runtime': runtime_before,
        'runtime_recipe': runtime_recipe, 'generated': generated,
        'refusals': refused, 'cargo_target': str(cargo_target),
        'rust_consumer_sha256': sha256(consumer / 'src/main.rs'),
        'rust_lock_sha256': sha256(consumer / 'Cargo.lock'),
        'rust_result': rust.strip(),
        'limitations': ['local Darwin arm64; native Linux qualification required',
                        'both pinned circuits are pure with proof=false and no recorded API',
                        'nonconstant Vector indices retain the original compile refusal'],
    }
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': 'passed', 'head': source['head'], 'output': str(output)}))


if __name__ == '__main__':
    main()
