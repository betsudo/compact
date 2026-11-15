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

"""Prepare seven pinned Bytes proof APIs without creating keys or proofs."""

import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tomllib

from check_release_packages import compatibility_record
from check_upstream_ledger9_bytes_remaining_proof import (generated_hashes,
                                                            tracked_source_hashes)
from check_upstream_ledger9_field_casts import require, sha256, source_state
import resolve_ledger_test_static as ledger_static


ROOT = Path(__file__).resolve().parents[2]
CASES = {
    'manual_update': ('manual_witness', 'bytes_manual_update_semantics', 12, 3628,
                      {'type-name': 'Bytes', 'length': 4}),
    'boolean_reduction': ('fold', 'bytes_fold_boolean_reduction', 7, 58,
                          {'type-name': 'Boolean'}),
    'conditional_accumulation': ('fold', 'bytes_fold_conditional_accumulation', 7, 58,
                                 {'type-name': 'Field'}),
    'summation': ('fold', 'bytes_fold_summation', 7, 58,
                  {'type-name': 'Uint', 'maxval': 255}),
    'empty_iteration': ('loops', 'bytes_loop_empty_iteration', 6, 47,
                        {'type-name': 'Field'}),
    'spread_iteration': ('loops', 'bytes_loop_spread_iteration', 6, 47,
                         {'type-name': 'Field'}),
    'iteration': ('loops', 'bytes_loop_iteration', 8, 223,
                  {'type-name': 'Field'}),
}


def run(command: list[str], log: Path, env: dict[str, str]) -> str:
    result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True)
    log.write_text(result.stdout + result.stderr)
    require(result.returncode == 0, f'{command[0]} exited {result.returncode}: {log}')
    return result.stdout + result.stderr


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'compiler-recipe', 'ts-runtime', 'zkir', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--cargo-target', type=Path)
    parser.add_argument('--prepare-only', action='store_true', required=True)
    args = parser.parse_args()
    source = source_state()
    signature = subprocess.check_output(['git', 'log', '-1', '--format=%G?%n%B'],
                                        cwd=ROOT, text=True)
    require(signature.startswith('G\n') and 'Signed-off-by:' in signature and
            'Assisted-by: AI' in signature, 'signed DCO source required')
    output = args.output.resolve()
    require(not output.exists() and not args.output.is_symlink(), 'new output required')
    compiler = args.compiler.resolve(strict=True)
    runtime = args.ts_runtime.resolve(strict=True)
    zkir = args.zkir.resolve(strict=True)
    require(zkir.name == 'zkir' and zkir.parent.name == 'bin', 'unexpected ZKIR executable')
    nix = shutil.which('nix')
    require(nix is not None, 'Nix required for exact ZKIR recipe')
    recipe = subprocess.run([nix, 'eval', '--offline', '--raw', '--option',
                             'substitute', 'false', '.#zkir-v3.outPath'],
                            cwd=ROOT, capture_output=True, text=True)
    require(recipe.returncode == 0 and recipe.stdout.strip() == str(zkir.parent.parent),
            'ZKIR differs from exact source recipe')
    require(subprocess.check_output([str(zkir), '--version'], text=True).strip() ==
            'midnight-zkir 3.2.0', 'ZKIR version changed')
    matrix = compatibility_record()
    require(matrix['ledger_version'] == 'ledger-9.1.0.1' and
            json.loads((ROOT / 'runtime/package.json').read_text())['version'] == '0.18.0',
            'Ledger/Runtime baseline changed')
    source_before = tracked_source_hashes()
    output.mkdir(parents=True)
    (output / 'source-hashes.json').write_text(json.dumps(source_before, indent=2,
                                                        sort_keys=True) + '\n')
    target = args.cargo_target.resolve(strict=True) if args.cargo_target else output / 'cargo-target'
    if args.cargo_target:
        require(target.is_dir() and not args.cargo_target.is_symlink(),
                'existing ordinary Cargo target required')
    else:
        target.mkdir()
    env = dict(os.environ, PYTHONDONTWRITEBYTECODE='1', RUSTUP_TOOLCHAIN='1.99.0',
               CARGO_INCREMENTAL='0', CARGO_PROFILE_DEV_DEBUG='0',
               CARGO_TARGET_DIR=str(target))
    protocol = {}
    for family in ('manual_witness', 'fold', 'loops'):
        gate = ROOT / f'tools/compact-rust-backend/check_upstream_ledger9_bytes_{family}.py'
        command = [sys.executable, str(gate), '--compiler', str(compiler),
                   '--compiler-recipe', str(args.compiler_recipe), '--ts-runtime',
                   str(runtime), '--output', str(output / 'protocol' / family)]
        if family != 'manual_witness':
            command += ['--cargo-target', str(target)]
        (output / 'protocol').mkdir(exist_ok=True)
        run(command, output / f'{family}-gate.log', env)
        receipt = output / 'protocol' / family / 'receipt.json'
        row = json.loads(receipt.read_text())
        require(row['head'] == source['head'] and row['tree'] == source['tree'] and
                row['compiler_recipe']['sha256'] == sha256(args.compiler_recipe),
                f'{family} generated protocol differs from exact source')
        protocol[family] = {'receipt': str(receipt), 'sha256': sha256(receipt),
                            'original_files': row['original_files']}
    generated_root = output / 'protocol'
    exports = {}
    for case, (family, name, expected_k, expected_rows, result_type) in CASES.items():
        target = generated_root / family / 'generated' / case / 'both'
        info = json.loads((target / 'compiler/contract-info.json').read_text())
        require(info['circuits'] == [{'name': name, 'pure': False, 'proof': True,
                                     'arguments': [], 'result-type': result_type}],
                f'proof API changed: {name}')
        caps = json.loads((target / 'contract/rust-capabilities.json').read_text())
        require(len(caps['circuits']) == 1 and caps['circuits'][0]['name'] == name and
                all(caps['circuits'][0][key] is True for key in
                    ('recorded', 'observed_call', 'proof_required')) and
                caps['circuits'][0]['recording_status'] == 'available',
                f'Rust proof capability changed: {name}')
        raw = target / 'zkir' / f'{name}.zkir'
        circuit = json.loads(raw.read_text())
        require(circuit['version'] == {'major': 3, 'minor': 0} and
                circuit['do_communications_commitment'] is True,
                f'ZKIR v3 communications binding changed: {name}')
        mock = run([str(zkir), 'mock-compile', str(raw)],
                   output / f'{name}-mock.log', env)
        model = re.search(r'k=(\d+), rows=(\d+)', mock)
        require(model is not None and (int(model[1]), int(model[2])) ==
                (expected_k, expected_rows), f'ZKIR model changed: {name}')
        exports[case] = {'family': family, 'name': name, 'k': expected_k,
                         'rows': expected_rows, 'zkir_sha256': sha256(raw),
                         'ir_sha256': sha256(target / 'contract/compact-rust-ir.json'),
                         'rust_sha256': sha256(target / 'contract/lib.rs'),
                         'typescript_sha256': sha256(target / 'contract/index.js'),
                         'recorded': True, 'observed_call': True}
    witness = generated_root / 'manual_witness/generated/witness_return/both'
    witness_info = json.loads((witness / 'compiler/contract-info.json').read_text())
    require(witness_info['circuits'][0]['proof'] is False and not (witness / 'zkir').exists(),
            'proof=false witness return changed')
    metadata = subprocess.run(['cargo', '+1.99.0', 'metadata', '--locked', '--offline',
                               '--format-version', '1'], cwd=ROOT, env=env,
                              capture_output=True, text=True)
    (output / 'metadata.json').write_text(metadata.stdout)
    (output / 'metadata.stderr.log').write_text(metadata.stderr)
    require(metadata.returncode == 0, 'locked offline Cargo metadata failed')
    ledger = ledger_static.resolve(json.loads(metadata.stdout),
                                   tomllib.loads((ROOT / 'Cargo.lock').read_text()))
    generated_before = {family: generated_hashes(generated_root / family / 'generated')
                        for family in protocol}
    require(source_state() == source and tracked_source_hashes() == source_before and
            all(generated_hashes(generated_root / family / 'generated') == hashes
                for family, hashes in generated_before.items()),
            'source or generated proof inputs drifted')
    report = {**source, 'format': 'compact-ledger9-bytes-next-proof-preparation/v1',
              'status': 'prepared', 'prepare_only': True, 'proof_execution': False,
              'funded_application': False, 'constructor_execution_proved': False,
              'all_exports_proved': False,
              'platform': os.uname().sysname + ' ' + os.uname().machine,
              'source_count': len(source_before),
              'source_hashes_sha256': sha256(output / 'source-hashes.json'),
              'protocol': protocol, 'proof_required_exports': exports,
              'proof_false_exports': ['witness_return'],
              'generated_hashes': generated_before,
              'ledger_static': ledger, 'compiler_sha256': sha256(compiler),
              'compiler_recipe_sha256': sha256(args.compiler_recipe),
              'zkir_sha256': sha256(zkir), 'zkir_nix_store': str(zkir.parent.parent),
              'runtime_version': '0.18.0',
              'limitations': ['prepare-only: no keys, proofs or funded calls',
                              'native Linux and constructor execution remain open']}
    (output / 'receipt.json').write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': 'prepared', 'head': source['head'],
                      'proof_required_exports': len(exports), 'output': str(output)}))


if __name__ == '__main__':
    main()
