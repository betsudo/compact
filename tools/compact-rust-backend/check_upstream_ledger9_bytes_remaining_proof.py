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

"""Prepare exact-source Bytes original proof APIs without keygen or proof execution."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tomllib

import check_upstream_ledger9_bytes_remaining_scalar as semantics
from check_upstream_ledger9_field_casts import require, sha256, source_state
from check_release_packages import compatibility_record
import resolve_ledger_test_static as ledger_static


ROOT = Path(__file__).resolve().parents[2]
EXPECTED_PROOF = {
    'default/semantics': (10, 727),
    'equality/empty': (7, 58),
    'indexing/basic': (8, 134),
    'slice/basic': (9, 352),
    'slice/empty': (6, 50),
    'spread/basic': (11, 1257),
    'spread/empty': (6, 50),
    'spread/multiple': (11, 1560),
    'spread/nested': (11, 1257),
}


def tracked_source_hashes() -> dict[str, str]:
    paths = subprocess.check_output(['git', 'ls-files', '-z'], cwd=ROOT).split(b'\0')
    result = {}
    for raw in paths:
        if not raw:
            continue
        name = os.fsdecode(raw)
        path = ROOT / name
        require(path.is_file() and not path.is_symlink(), f'tracked source absent or symlink: {name}')
        result[name] = sha256(path)
    return result


def generated_hashes(root: Path) -> dict[str, str]:
    result = {}
    for path in sorted(root.rglob('*')):
        require(not path.is_symlink(), f'generated artifact symlink: {path}')
        if path.is_file():
            result[path.relative_to(root).as_posix()] = sha256(path)
    return result


def run(command: list[str], log: Path, *, cwd: Path = ROOT) -> str:
    result = subprocess.run(command, cwd=cwd, capture_output=True, text=True)
    log.write_text(result.stdout + result.stderr)
    require(result.returncode == 0, f'{command[0]} exited {result.returncode}: {log}')
    return result.stdout + result.stderr


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'compiler-recipe', 'ts-runtime', 'zkir', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--prepare-only', action='store_true', required=True)
    args = parser.parse_args()
    source = source_state()
    signature = subprocess.check_output(['git', 'log', '-1', '--format=%G?%n%B'],
                                        cwd=ROOT, text=True)
    require(signature.startswith('G\n') and 'Signed-off-by:' in signature and
            'Assisted-by: AI' in signature, 'signed DCO and assisted source required')
    output = args.output.resolve()
    require(not output.exists() and not args.output.is_symlink(), 'output must be absent')
    compiler = args.compiler.resolve(strict=True)
    runtime = args.ts_runtime.resolve(strict=True)
    zkir = args.zkir.resolve(strict=True)
    require(zkir.name == 'zkir' and zkir.parent.name == 'bin', 'unexpected ZKIR executable')
    nix = shutil.which('nix')
    require(nix is not None, 'Nix required for ZKIR recipe binding')
    nix_result = subprocess.run([nix, 'eval', '--offline', '--raw', '--option',
                                 'substitute', 'false', '.#zkir-v3.outPath'],
                                cwd=ROOT, capture_output=True, text=True)
    require(nix_result.returncode == 0 and nix_result.stdout.strip() == str(zkir.parent.parent),
            'ZKIR differs from exact-head Nix recipe')
    version = subprocess.check_output([str(zkir), '--version'], text=True).strip()
    require(version == 'midnight-zkir 3.2.0', 'released ZKIR version changed')
    matrix = compatibility_record()
    require(matrix['ledger_version'] == 'ledger-9.1.0.1' and
            json.loads((ROOT / 'runtime/package.json').read_text())['version'] == '0.18.0',
            'Ledger/Runtime matrix changed')
    source_before = tracked_source_hashes()
    originals = semantics.original_inventory()
    require(set(EXPECTED_PROOF) == {name for name, (_, pure) in semantics.SUCCESS.items() if not pure},
            'Bytes proof-required source export roster changed')
    output.mkdir(parents=True)
    source_file = output / 'source-hashes.json'
    source_file.write_text(json.dumps(source_before, indent=2, sort_keys=True) + '\n')
    gate = [sys.executable, str(Path(semantics.__file__)), '--compiler', str(compiler),
            '--compiler-recipe', str(args.compiler_recipe), '--ts-runtime', str(runtime),
            '--output', str(output / 'protocol')]
    run(gate, output / 'protocol.log')
    protocol = json.loads((output / 'protocol/receipt.json').read_text())
    require(protocol['head'] == source['head'] and protocol['tree'] == source['tree'] and
            protocol['original_files'] == originals and
            protocol['compiler_recipe']['sha256'] == sha256(args.compiler_recipe),
            'Bytes source/API protocol differs from this head')
    generated = output / 'protocol/generated'
    circuits = {}
    for case, (expected_k, expected_rows) in EXPECTED_PROOF.items():
        target = generated / case / 'both'
        info = json.loads((target / 'compiler/contract-info.json').read_text())
        capabilities = json.loads((target / 'contract/rust-capabilities.json').read_text())
        circuit = semantics.circuit_name(case)
        require(info['circuits'] == [{'name': circuit, 'pure': False, 'proof': True,
                                     'arguments': [], 'result-type': semantics.SUCCESS[case][0]}] and
                capabilities['circuits'] == [{
                    'name': circuit,
                    'source': {'file': f'{circuit}.compact', 'line': capabilities['circuits'][0]['source']['line'],
                               'column': capabilities['circuits'][0]['source']['column']},
                    'recorded': True, 'observed_call': True, 'proof_required': True,
                    'recording_status': 'available',
                }], f'Bytes proof/recording API changed: {case}')
        path = target / 'zkir' / f'{circuit}.zkir'
        zkir_record = json.loads(path.read_text())
        require(zkir_record['version'] == {'major': 3, 'minor': 0} and
                zkir_record['do_communications_commitment'] is True,
                f'Bytes ZKIR v3 or communications binding changed: {case}')
        mock = run([str(zkir), 'mock-compile', str(path)],
                   output / f'{case.replace("/", "-")}-mock.log')
        model = re.search(r'k=(\d+), rows=(\d+)', mock)
        require(model is not None and (int(model[1]), int(model[2])) ==
                (expected_k, expected_rows), f'Bytes proof model drift: {case}')
        circuits[case] = {'name': circuit, 'k': expected_k, 'rows': expected_rows,
                          'zkir_sha256': sha256(path),
                          'ir_sha256': sha256(target / 'contract/compact-rust-ir.json'),
                          'rust_sha256': sha256(target / 'contract/lib.rs'),
                          'typescript_sha256': sha256(target / 'contract/index.js'),
                          'recorded': True, 'observed_call': True}
    cargo_metadata = subprocess.run(['cargo', '+1.99.0', 'metadata', '--locked',
                                     '--offline', '--format-version', '1'],
                                    cwd=ROOT, capture_output=True, text=True)
    (output / 'metadata.json').write_text(cargo_metadata.stdout)
    (output / 'metadata.stderr.log').write_text(cargo_metadata.stderr)
    require(cargo_metadata.returncode == 0, 'locked offline Cargo metadata failed')
    ledger = ledger_static.resolve(json.loads(cargo_metadata.stdout),
                                   tomllib.loads((ROOT / 'Cargo.lock').read_text()))
    generated_before = generated_hashes(generated)
    require(source_state() == source and tracked_source_hashes() == source_before and
            generated_hashes(generated) == generated_before and
            semantics.original_inventory() == originals,
            'Bytes proof preparation source or generated artifact drifted')
    report = {**source, 'format': 'compact-ledger9-bytes-remaining-proof-preparation/v1',
              'status': 'prepared', 'platform': os.uname().sysname + ' ' + os.uname().machine,
              'prepare_only': True, 'proof_execution': False,
              'funded_application': False, 'constructor_execution_proved': False,
              'all_exports_proved': False, 'source_count': len(source_before),
              'source_hashes': {'path': str(source_file), 'sha256': sha256(source_file)},
              'protocol_receipt': {'path': str(output / 'protocol/receipt.json'),
                                   'sha256': sha256(output / 'protocol/receipt.json')},
              'original_files': originals, 'proof_required_exports': circuits,
              'generated_hashes': generated_before,
              'compiler_recipe_sha256': sha256(args.compiler_recipe),
              'compiler_sha256': sha256(compiler), 'zkir_sha256': sha256(zkir),
              'zkir_nix_store': str(zkir.parent.parent), 'zkir_version': version,
              'ledger_static': ledger, 'runtime_version': '0.18.0',
              'limitations': ['prepare-only: no keys or fresh proofs generated',
                              'no funded default-strict application or replay tested',
                              'native Linux qualification and constructor execution pending']}
    (output / 'receipt.json').write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': 'prepared', 'head': source['head'],
                      'proof_required_exports': len(circuits), 'output': str(output)}))


if __name__ == '__main__':
    main()
