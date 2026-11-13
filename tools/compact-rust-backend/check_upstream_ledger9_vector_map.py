#!/usr/bin/env python3

# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#  http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Qualify the two intact upstream Vector map families on generated Rust/TS."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tomllib

from check_unit_mutation_initializers import validate_consumer_lock
from collection_gate_binding import compiler_recipe


ROOT = Path(__file__).resolve().parents[2]
FAMILY = ROOT / 'test-contracts/primitives/vector/map'
PINNED = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'
CASES = {
    'semantics': 'vector_map_semantics',
    'empty_tuple': 'vector_map_empty_tuple',
}


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(*args: str) -> str:
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()


def run(command: list[str], log: Path, *, env: dict[str, str] | None = None) -> str:
    result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True)
    text = result.stdout + result.stderr
    log.write_text(text)
    if result.returncode:
        raise RuntimeError(f'{command[0]} exited {result.returncode}: {text[-4000:]}')
    return result.stdout


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler-receipt', required=True, type=Path)
    parser.add_argument('--ts-runtime', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    if git('status', '--porcelain'):
        parser.error('source checkout must be clean')
    head = git('rev-parse', 'HEAD')
    tree = git('rev-parse', 'HEAD^{tree}')
    if git('log', '-1', '--format=%G?') != 'G' or 'Signed-off-by:' not in git('log', '-1', '--format=%B'):
        parser.error('source commit must be signed and DCO complete')
    receipt_path = args.compiler_receipt.resolve(strict=True)
    recipe = json.loads(receipt_path.read_text())
    compiler = Path(recipe['compactc']['path']).resolve(strict=True)
    scheme = Path(recipe['compactc-scheme']['path']).resolve(strict=True)
    compiler_recipe(receipt_path, compiler, scheme, source_head=head, current_head=head)
    runtime = args.ts_runtime.resolve(strict=True)
    built_package = json.loads((runtime / 'package.json').read_text())
    source_package = json.loads((ROOT / 'runtime/package.json').read_text())
    if any(built_package[key] != source_package[key] for key in ('name', 'version', 'dependencies')):
        parser.error('TypeScript runtime package does not match source checkout')
    output = args.output.resolve()
    if args.output.is_symlink() or output.exists():
        parser.error('output must be absent')

    original = {}
    for family, circuit in CASES.items():
        for filename in (f'{circuit}.compact', 'compile.pass.test.ts', 'runtime.pass.test.ts'):
            path = FAMILY / family / filename
            relative = path.relative_to(ROOT).as_posix()
            upstream = subprocess.check_output(['git', 'show', f'{PINNED}:{relative}'], cwd=ROOT)
            if path.read_bytes() != upstream:
                raise RuntimeError(f'pinned upstream fixture changed: {relative}')
            original[relative] = sha(path)

    runtime_link = output / 'node_modules/@midnight-ntwrk/compact-runtime'
    runtime_link.parent.mkdir(parents=True)
    runtime_link.symlink_to(runtime, target_is_directory=True)
    (output / 'package.json').write_text('{"type":"module"}\n')
    contracts = {}
    artifacts = {}
    for family, circuit in CASES.items():
        source = FAMILY / family / f'{circuit}.compact'
        default = output / 'default' / family
        paired = output / 'paired' / family
        run([str(compiler), str(source), str(default)], output / f'{family}-default.log')
        run([str(compiler), '--target', 'ts', '--target', 'rust',
             '--rust-runtime-root', str(ROOT), '--rust-require-recording', '--skip-zk',
             str(source), str(paired)], output / f'{family}-paired.log')
        for label, generated in (('default', default), ('paired', paired)):
            info = json.loads((generated / 'compiler/contract-info.json').read_text())
            circuits = info['circuits']
            if (len(circuits) != 1 or circuits[0]['name'] != circuit or
                    not circuits[0]['pure'] or circuits[0]['proof'] or circuits[0]['arguments']):
                raise RuntimeError(f'{family} is not a parameterless pure circuit without a proof')
            run(['node', str(FAMILY / 'check.mjs'), str(generated / 'contract/index.js'), circuit],
                output / f'{family}-{label}-typescript.log')
            artifacts[f'{family}/{label}/typescript'] = sha(generated / 'contract/index.js')
        contracts[family] = paired / 'contract'
        artifacts[f'{family}/paired/rust'] = sha(paired / 'contract/lib.rs')
        artifacts[f'{family}/paired/rust_ir'] = sha(paired / 'contract/compact-rust-ir.json')

    consumer = output / 'rust-consumer'
    (consumer / 'tests').mkdir(parents=True)
    shutil.copyfile(FAMILY / 'consumer.rs', consumer / 'tests/vector_map.rs')
    lock = consumer / 'Cargo.lock'
    shutil.copyfile(ROOT / 'Cargo.lock', lock)
    manifest = consumer / 'Cargo.toml'
    lines = ['[workspace]', '[package]', 'name = "upstream-ledger9-vector-map-consumer"',
             'version = "0.0.0"', 'edition = "2024"', '[dependencies]']
    for family, contract in contracts.items():
        package = tomllib.loads((contract / 'Cargo.toml').read_text())['package']['name']
        lines.append(f'{family} = {{ package = {json.dumps(package)}, path = {json.dumps(str(contract))} }}')
    manifest.write_text('\n'.join(lines) + '\n')
    env = os.environ.copy()
    env['CARGO_TARGET_DIR'] = str(output / 'rust-target')
    metadata = json.loads(run(['cargo', '+1.99.0', 'metadata', '--offline', '--format-version', '1',
                               '--manifest-path', str(manifest)], output / 'cargo-metadata.log', env=env))
    workspace_lock = tomllib.loads((ROOT / 'Cargo.lock').read_text())['package']
    pinned = {(row['name'], row['version'], row['source']): row.get('checksum')
              for row in workspace_lock if 'source' in row}
    local = {manifest, ROOT / 'runtime-rs/Cargo.toml', ROOT / 'runtime-rs-macros/Cargo.toml'}
    local.update(contract / 'Cargo.toml' for contract in contracts.values())
    validate_consumer_lock(metadata, lock, local, pinned)
    run(['cargo', '+1.99.0', 'test', '--locked', '--offline', '--manifest-path', str(manifest)],
        output / 'cargo-test.log', env=env)
    if git('status', '--porcelain') or git('rev-parse', 'HEAD') != head:
        raise RuntimeError('source checkout changed during qualification')
    receipt = {
        'head': head, 'tree': tree, 'platform': f'{platform.system()} {platform.machine()}',
        'upstream': PINNED, 'original_files': original,
        'compiler_receipt_sha256': sha(receipt_path), 'compiler_sha256': sha(compiler),
        'scheme_sha256': sha(scheme), 'typescript_runtime_version': built_package['version'],
        'typescript_runtime_package_sha256': sha(runtime / 'package.json'),
        'adapter_sha256': sha(FAMILY / 'check.mjs'), 'rust_consumer_sha256': sha(FAMILY / 'consumer.rs'),
        'runner_sha256': sha(Path(__file__)), 'artifacts': artifacts,
        'rust_consumer_lock_sha256': sha(lock), 'cargo_test_log_sha256': sha(output / 'cargo-test.log'),
        'result': 'Both original compile-pass/runtime-pass Vector map cases passed generated TypeScript and Rust',
        'limitations': ['Local platform only; native Linux remains required.',
                        'These are pure circuits with no ZKIR proof or funded transaction.'],
    }
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(f'upstream Vector map generated Rust/TypeScript qualification passed: {output}')


if __name__ == '__main__':
    main()
