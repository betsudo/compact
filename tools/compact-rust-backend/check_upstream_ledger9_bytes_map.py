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

"""Qualify four intact upstream Bytes map families on generated Rust/TS."""

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
FAMILY = ROOT / 'test-contracts/primitives/bytes/map'
PINNED = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'
CASES = {
    'reordering': 'bytes_map_reordering',
    'slice_composition': 'bytes_map_slice_composition',
    'to_bytes': 'bytes_map_to_bytes',
    'transformation': 'bytes_map_transformation',
}
# SHA-256 of the original files at PINNED. These were checked against the
# upstream Git object when copied, and let the gate run in a fresh fork clone
# that has not fetched LFDT-Minokawa's history.
ORIGINAL_SHA256 = {
    'reordering': {
        'bytes_map_reordering.compact': '9ebbdec60c7cb08515f93c17eaac62e4dff141b12614bd087166bba4737dff00',
        'compile.pass.test.ts': '16bd80193bfe6493846325f8b1a59f9929e4059076d5a038b3fc1770b78b23a0',
        'runtime.pass.test.ts': '23f3a7cbfcca00de798961a051a0e4c0905a70e25a3515812d3effbdcf2ef3ef',
    },
    'slice_composition': {
        'bytes_map_slice_composition.compact': '49e433b537a1a31d75c07597d69c723387ff32b1e65a8b336140200ecbf40b88',
        'compile.pass.test.ts': '16bd80193bfe6493846325f8b1a59f9929e4059076d5a038b3fc1770b78b23a0',
        'runtime.pass.test.ts': '521d28925a3eb5be933dadadbe05ac1651018b93f66c26f9c277e45217b8069f',
    },
    'to_bytes': {
        'bytes_map_to_bytes.compact': '4f1cc4bddee8087b4863d9536a530e9b55e5e5aa704ecb01928e6fe5687b4296',
        'compile.pass.test.ts': '16bd80193bfe6493846325f8b1a59f9929e4059076d5a038b3fc1770b78b23a0',
        'runtime.pass.test.ts': '12277752edb537cf968facbb92bca245cfa6ca3e1e78ad0f9eb5584056412c10',
    },
    'transformation': {
        'bytes_map_transformation.compact': '38f12526a78734b8c93f4be6cffa0ef9efaae59db552d56b3cf36ecbbd924680',
        'compile.pass.test.ts': '16bd80193bfe6493846325f8b1a59f9929e4059076d5a038b3fc1770b78b23a0',
        'runtime.pass.test.ts': '2ddaf0ed116788d20793450aacabc87cc2405fcaf63028f9a323decad72491f8',
    },
}


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def file_hashes(root: Path) -> dict[str, str]:
    return {path.relative_to(root).as_posix(): sha(path)
            for path in root.rglob('*') if path.is_file()}


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
            if sha(path) != ORIGINAL_SHA256[family][filename]:
                raise RuntimeError(f'pinned upstream fixture changed: {relative}')
            original[relative] = sha(path)

    output.mkdir(parents=True)
    nix = Path(shutil.which('nix')).resolve(strict=True)
    runtime_store = Path(run([
        str(nix), 'build', '--offline', '--no-link', '--print-out-paths',
        '--max-jobs', '0', '--builders', '', '--option', 'substitute', 'false',
        '.#runtime.package',
    ], output / 'typescript-runtime-recipe.log').strip()).resolve(strict=True)
    source_runtime = runtime_store / 'lib/node_modules/@midnight-ntwrk/compact-runtime'
    nix_package = json.loads((source_runtime / 'package.json').read_text())
    if any(nix_package[key] != built_package[key]
           for key in ('name', 'version', 'dependencies')):
        raise RuntimeError('supplied TypeScript runtime differs from exact-head Nix package metadata')
    supplied_dist = file_hashes(runtime / 'dist')
    source_dist = file_hashes(source_runtime / 'dist')
    if not source_dist or supplied_dist != source_dist:
        raise RuntimeError('supplied TypeScript runtime dist differs from exact-head Nix build')
    runtime_link = output / 'node_modules/@midnight-ntwrk/compact-runtime'
    runtime_link.parent.mkdir(parents=True)
    runtime_link.symlink_to(source_runtime, target_is_directory=True)
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
                    circuits[0]['pure'] or not circuits[0]['proof'] or circuits[0]['arguments']):
                raise RuntimeError(f'{family} is not a parameterless provable stateful circuit')
            run(['node', str(FAMILY / 'check.mjs'), str(generated / 'contract/index.js'), circuit],
                output / f'{family}-{label}-typescript.log')
            artifacts[f'{family}/{label}/typescript'] = sha(generated / 'contract/index.js')
        contracts[family] = paired / 'contract'
        artifacts[f'{family}/paired/rust'] = sha(paired / 'contract/lib.rs')
        artifacts[f'{family}/paired/rust_ir'] = sha(paired / 'contract/compact-rust-ir.json')
        if sha(default / 'contract/index.js') != sha(paired / 'contract/index.js'):
            raise RuntimeError(f'{family} default and paired TypeScript differ')

    field_source = FAMILY / 'field_to_uint_checked.compact'
    default = output / 'default' / 'field_to_uint'
    paired = output / 'paired' / 'field_to_uint'
    run([str(compiler), str(field_source), str(default)], output / 'field-to-uint-default.log')
    run([str(compiler), '--target', 'ts', '--target', 'rust',
         '--rust-runtime-root', str(ROOT), '--rust-require-recording', '--skip-zk',
         str(field_source), str(paired)], output / 'field-to-uint-paired.log')
    for label, generated in (('default', default), ('paired', paired)):
        info = json.loads((generated / 'compiler/contract-info.json').read_text())
        circuits = info['circuits']
        if (len(circuits) != 1 or circuits[0]['name'] != 'field_to_byte' or
                not circuits[0]['pure'] or circuits[0]['proof'] or len(circuits[0]['arguments']) != 1):
            raise RuntimeError('Field-to-Uint regression must be a unary pure circuit')
        run(['node', str(FAMILY / 'check_field_to_uint.mjs'),
             str(generated / 'contract/index.js')], output / f'field-to-uint-{label}-typescript.log')
        artifacts[f'field_to_uint/{label}/typescript'] = sha(generated / 'contract/index.js')
    if artifacts['field_to_uint/default/typescript'] != artifacts['field_to_uint/paired/typescript']:
        raise RuntimeError('Field-to-Uint default and paired TypeScript differ')
    contracts['field_to_uint'] = paired / 'contract'
    artifacts['field_to_uint/paired/rust'] = sha(paired / 'contract/lib.rs')
    artifacts['field_to_uint/paired/rust_ir'] = sha(paired / 'contract/compact-rust-ir.json')

    large_source = FAMILY / 'bytes_index_large.compact'
    default = output / 'default' / 'large_index'
    paired = output / 'paired' / 'large_index'
    run([str(compiler), str(large_source), str(default)], output / 'large-index-default.log')
    run([str(compiler), '--target', 'ts', '--target', 'rust',
         '--rust-runtime-root', str(ROOT), '--rust-require-recording', '--skip-zk',
         str(large_source), str(paired)], output / 'large-index-paired.log')
    for label, generated in (('default', default), ('paired', paired)):
        info = json.loads((generated / 'compiler/contract-info.json').read_text())
        circuits = info['circuits']
        if (len(circuits) != 1 or circuits[0]['name'] != 'read_last' or
                not circuits[0]['pure'] or circuits[0]['proof'] or len(circuits[0]['arguments']) != 1):
            raise RuntimeError('large index regression must be a unary pure circuit')
        run(['node', str(FAMILY / 'check_large_index.mjs'),
             str(generated / 'contract/index.js')], output / f'large-index-{label}-typescript.log')
        artifacts[f'large_index/{label}/typescript'] = sha(generated / 'contract/index.js')
    if artifacts['large_index/default/typescript'] != artifacts['large_index/paired/typescript']:
        raise RuntimeError('large index default and paired TypeScript differ')
    large_rust = (paired / 'contract/lib.rs').read_text()
    if ('FixedBytes::<1>::new' not in large_rust or '.0[4095' not in large_rust or
            'FixedVector<runtime::BoundedUint<255>, 4096>' in large_rust or
            '.into_array().map(|byte|' in large_rust):
        raise RuntimeError('large Bytes indexing did not use direct one-byte projection')
    contracts['large_index'] = paired / 'contract'
    artifacts['large_index/paired/rust'] = sha(paired / 'contract/lib.rs')
    artifacts['large_index/paired/rust_ir'] = sha(paired / 'contract/compact-rust-ir.json')

    consumer = output / 'rust-consumer'
    (consumer / 'tests').mkdir(parents=True)
    shutil.copyfile(FAMILY / 'consumer.rs', consumer / 'tests/bytes_map.rs')
    lock = consumer / 'Cargo.lock'
    shutil.copyfile(ROOT / 'Cargo.lock', lock)
    manifest = consumer / 'Cargo.toml'
    lines = ['[workspace]', '[package]', 'name = "upstream-ledger9-bytes-map-consumer"',
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
        'typescript_runtime_nix_store': str(runtime_store),
        'typescript_runtime_dist_files': source_dist,
        'typescript_runtime_recipe_log_sha256': sha(output / 'typescript-runtime-recipe.log'),
        'adapter_sha256': sha(FAMILY / 'check.mjs'),
        'field_to_uint_source_sha256': sha(field_source),
        'regression_adapter_sha256': sha(FAMILY / 'check_field_to_uint.mjs'),
        'large_index_source_sha256': sha(large_source),
        'large_index_adapter_sha256': sha(FAMILY / 'check_large_index.mjs'),
        'rust_consumer_sha256': sha(FAMILY / 'consumer.rs'),
        'runner_sha256': sha(Path(__file__)), 'artifacts': artifacts,
        'rust_consumer_lock_sha256': sha(lock), 'cargo_test_log_sha256': sha(output / 'cargo-test.log'),
        'result': 'Four original Bytes map cases passed generated TypeScript and ordinary/recorded Rust; checked Field-to-Uint and large-index regressions passed',
        'limitations': ['Local platform only; native Linux remains required.',
                        'The gate uses skip-zk; these provable circuits have no fresh proof or funded transaction in this slice.',
                        'Dynamic Bytes indexes remain outside this constant-index lowering.'],
    }
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(f'upstream Bytes map generated Rust/TypeScript qualification passed: {output}')


if __name__ == '__main__':
    main()
