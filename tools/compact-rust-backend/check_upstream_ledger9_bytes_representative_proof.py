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

"""Fresh equality/empty-slice proofs and strict funded calls for pinned Bytes originals."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

import check_upstream_ledger9_bytes_remaining_proof as admission
from check_upstream_ledger9_field_casts import require, sha256, source_state
from did_proof_gate import hashes, unchanged
import proof_runner_input_binding as runner_inputs


ROOT = Path(__file__).resolve().parents[2]
SELECTED = {
    'bytes_equality_empty': ('equality/empty', 'bytes-equality-proof', 7,
                             'bytes-equality-empty'),
    'bytes_slice_empty': ('slice/empty', 'bytes-slice-proof', 6,
                          'bytes-slice-empty'),
}


def run(command: list[str], log: Path, env: dict[str, str]) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True)
    log.write_text(result.stdout + result.stderr)
    require(result.returncode == 0, f'{command[0]} exited {result.returncode}: {log}')
    return result


def hash_files(root: Path) -> dict[str, str]:
    result = {}
    for path in sorted(root.rglob('*')):
        require(not path.is_symlink(), f'proof artifact symlink: {path}')
        if path.is_file():
            result[path.relative_to(root).as_posix()] = sha256(path)
    return result


def physical_sha256(path: str | Path) -> str:
    return sha256(Path(path))


def dependency_refusals(tracked: set[str], inventories: dict[str, dict[str, str]],
                        output: Path) -> dict[str, bool]:
    """Exercise unknown-path and altered-hash refusal before any key generation."""
    unknown = output / 'unadmitted-runner-input.rs'
    unknown.write_text('// outside the frozen Cargo input inventories\n')
    try:
        runner_inputs.qualify_dependency_inputs(tracked | {str(unknown)}, inventories,
                                                physical_sha256)
    except ValueError as error:
        unknown_rejected = 'missing or inconsistent' in str(error)
    else:
        unknown_rejected = False
    require(unknown_rejected, 'unknown physical runner dependency escaped refusal')
    selected = sorted(tracked)[0]
    altered = {name: dict(files) for name, files in inventories.items()}
    for files in altered.values():
        if selected in files:
            files[selected] = '0' * 64 if files[selected] != '0' * 64 else 'f' * 64
    try:
        runner_inputs.qualify_dependency_inputs(tracked, altered, physical_sha256)
    except ValueError as error:
        altered_rejected = ('changed after physical prebuild binding' in str(error)
                            or 'missing or inconsistent' in str(error))
    else:
        altered_rejected = False
    require(altered_rejected, 'tampered physical runner dependency escaped refusal')
    return {'unknown_dependency_rejected': unknown_rejected,
            'tampered_dependency_hash_rejected': altered_rejected}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'compiler-recipe', 'ts-runtime', 'zkir', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--cargo-target', type=Path,
                        help='Existing isolated Cargo target; exact depfile is revalidated')
    args = parser.parse_args()
    source = source_state()
    output = args.output.resolve()
    require(not output.exists() and not args.output.is_symlink(), 'new proof output required')
    source_before = admission.tracked_source_hashes()
    output.mkdir(parents=True)
    env = dict(os.environ, PYTHONDONTWRITEBYTECODE='1', RUSTUP_TOOLCHAIN='1.99.0',
               CARGO_INCREMENTAL='0', CARGO_PROFILE_DEV_DEBUG='0')
    env.pop('MIDNIGHT_PROOF_SERVER', None)
    prepare_command = [sys.executable, str(Path(admission.__file__)), '--prepare-only',
                       '--compiler', str(args.compiler), '--compiler-recipe',
                       str(args.compiler_recipe), '--ts-runtime', str(args.ts_runtime),
                       '--zkir', str(args.zkir), '--output', str(output / 'prepared')]
    run(prepare_command, output / 'prepare.log', env)
    prepared = json.loads((output / 'prepared/receipt.json').read_text())
    require(prepared['status'] == 'prepared' and prepared['head'] == source['head'] and
            prepared['tree'] == source['tree'] and prepared['source_count'] == len(source_before) and
            set(prepared['proof_required_exports']) == set(admission.EXPECTED_PROOF),
            'current-head Bytes proof API preparation differs')
    generated = output / 'prepared/protocol/generated'
    proof = output / 'proof'
    (proof / 'zkir').mkdir(parents=True)
    (proof / 'keys').mkdir()
    selected_source = {}
    for name, (case, _, k, _) in SELECTED.items():
        target = generated / case / 'both'
        raw = target / f'zkir/{name}.zkir'
        relative = Path(name) / 'contract/compact-rust-ir.json'
        destination = proof / relative
        destination.parent.mkdir(parents=True)
        shutil.copyfile(target / 'contract/compact-rust-ir.json', destination)
        shutil.copyfile(raw, proof / 'zkir' / raw.name)
        record = prepared['proof_required_exports'][case]
        require(record['name'] == name and record['k'] == k and
                record['ir_sha256'] == sha256(destination) and
                record['zkir_sha256'] == sha256(proof / 'zkir' / raw.name),
                f'selected Bytes source/ZKIR binding changed: {name}')
        selected_source[name] = {'case': case, 'k': k,
                                 'ir_sha256': record['ir_sha256'],
                                 'zkir_sha256': record['zkir_sha256']}
    proof_inputs_before = hash_files(proof)
    env['MIDNIGHT_PP'] = str(Path.home() / '.cache/midnight/zk-params')
    env['MIDNIGHT_LEDGER_TEST_STATIC_DIR'] = prepared['ledger_static']['static_directory']
    require(Path(env['MIDNIGHT_PP']).is_dir(), 'local proof parameter cache absent')
    cargo_target = args.cargo_target.resolve(strict=True) if args.cargo_target else output / 'cargo-target'
    require(cargo_target.is_dir() and not args.cargo_target.is_symlink()
            if args.cargo_target else True,
            'existing Cargo target must be an ordinary directory')
    env['CARGO_TARGET_DIR'] = str(cargo_target)
    runners = {}
    frozen_runner_inputs = []
    for name, (case, feature, _, _) in SELECTED.items():
        env['COMPACT_BYTES_REMAINING_GENERATED'] = str(generated / case / 'both')
        binding_dir = output / 'binding' / name
        binding_dir.mkdir(parents=True)
        binding = {
            'source_hashes': {str((ROOT / path).resolve(strict=True)): digest
                              for path, digest in source_before.items()},
            'generated_input_hashes': hashes(path for path in generated.rglob('*')
                                             if path.is_file()),
        }

        def execute(label: str, command: list[str], log: Path) -> None:
            for field in ('source_hashes', 'generated_input_hashes',
                          'dependency_source_hashes', 'generated_build_inputs_before_compile'):
                if field in binding:
                    unchanged(binding[field])
            run(command, log, env)

        runner_inputs.prepare_generated_inputs(ROOT, binding_dir, binding, execute,
                                               feature, sha256)
        result = run(['cargo', '+1.99.0', 'build', '--locked', '--offline',
                      '--profile', 'proof-checks', '-p', 'compact-rust-proof-smoke',
                      '--features', feature, '--message-format=json'],
                     output / f'{name}-build.log', env)
        rows = [json.loads(line) for line in result.stdout.splitlines() if line.startswith('{')]
        executables = {row['executable'] for row in rows
                       if row.get('reason') == 'compiler-artifact' and
                       row['target']['name'] == 'compact-rust-proof-smoke' and row.get('executable')}
        require(len(executables) == 1, f'ambiguous {name} proof runner')
        built = Path(executables.pop())
        depfile = built.with_suffix('.d')
        tracked = runner_inputs.depfile_prerequisites(depfile.read_text(), ROOT)
        required = [generated / case / 'both/contract/lib.rs',
                    generated / case / 'both/contract/compact-rust-ir.json',
                    generated / case / f'both/zkir/{name}.zkir',
                    ROOT / 'tools/compact-rust-proof-smoke/src/bytes_remaining_proof.rs']
        require(all(str(path.resolve(strict=True)) in tracked for path in required),
                f'{name} proof runner omitted generated or committed source input')
        inventories = {field: binding[field] for field in
                       ('source_hashes', 'generated_input_hashes',
                        'dependency_source_hashes', 'generated_build_inputs_before_compile')}
        physical, origins = runner_inputs.qualify_dependency_inputs(tracked, inventories,
                                                                     physical_sha256)
        refusals = dependency_refusals(tracked, inventories, binding_dir)
        frozen_runner_inputs.extend(inventories.values())
        frozen_runner_inputs.append(physical)
        inventory_path = binding_dir / 'inputs.json'
        inventory_path.write_text(json.dumps({**binding, 'physical_inputs_before_build': physical,
                                              'physical_input_origins': origins},
                                             indent=2, sort_keys=True) + '\n')
        retained_depfile = binding_dir / 'runner.d'
        shutil.copyfile(depfile, retained_depfile)
        runner = output / 'bin' / name
        runner.parent.mkdir(exist_ok=True)
        shutil.copyfile(built, runner)
        runner.chmod(0o755)
        runners[name] = {'path': str(runner), 'sha256': sha256(runner),
                         'depfile_sha256': sha256(depfile),
                         'retained_depfile': str(retained_depfile),
                         'inventory_path': str(inventory_path),
                         'inventory_sha256': sha256(inventory_path),
                         'physical_input_count': len(physical),
                         'physical_input_origins': origins,
                         'dependency_refusals': refusals,
                         'required_inputs': [str(path) for path in required]}
    require(hash_files(proof) == proof_inputs_before,
            'proof source/IR/ZKIR inputs changed during runner build')
    # Refuse altered raw circuit or complete IR before key generation and state.
    bad = output / 'negative-drift'
    shutil.copytree(proof, bad)
    changed = bad / 'zkir/bytes_equality_empty.zkir'
    changed.write_bytes(changed.read_bytes() + b' ')
    negative = subprocess.run([runners['bytes_equality_empty']['path'],
                               '--bytes-remaining-proof', str(bad)],
                              cwd=ROOT, env=env, capture_output=True, text=True)
    (output / 'negative-drift.log').write_text(negative.stdout + negative.stderr)
    require(negative.returncode != 0 and 'raw ZKIR bytes differ' in negative.stderr and
            not list(bad.rglob('*.proof')) and not (bad / 'keys/bytes_equality_empty.prover').exists(),
            'changed raw ZKIR escaped pre-key refusal')
    env['COMPACT_BYTES_REMAINING_GENERATED'] = str(generated / 'equality/empty/both')
    material = run([runners['bytes_equality_empty']['path'], '--prepare-proof-material',
                    '--verify-only', '--parameter-k', '6', '--parameter-k', '7'],
                   output / 'material.json', env)
    verified = json.loads(material.stdout)
    require(verified['mode'] == 'verify-only' and
            verified['verification'] == 'all files checked against locked upstream SHA256 tables' and
            {6, 7} <= {row['k'] for row in verified['parameters']},
            'upstream proving material not verified before keygen')
    material_hashes = {row['name']: sha256(Path(verified['cache_directory']) / row['name'])
                       for row in verified['assets'] + verified['parameters']}
    keygen = {}
    zkir = args.zkir.resolve(strict=True)
    for name, (_, _, k, _) in SELECTED.items():
        command = [str(zkir), 'compile', str(proof / f'zkir/{name}.zkir'),
                   str(proof / f'keys/{name}.prover'),
                   str(proof / f'keys/{name}.verifier')]
        result = run(command, output / f'{name}-keygen.log', env)
        require(f'k={k}' in result.stdout + result.stderr and
                (proof / f'zkir/{name}.bzkir').is_file(), f'{name} actual native key model differs')
        keygen[name] = {'k': k, 'prover_sha256': sha256(proof / f'keys/{name}.prover'),
                        'verifier_sha256': sha256(proof / f'keys/{name}.verifier'),
                        'binary_zkir_sha256': sha256(proof / f'zkir/{name}.bzkir')}
    calls = []
    for name, (_, _, k, stem) in SELECTED.items():
        result = run([runners[name]['path'], '--bytes-remaining-proof', str(proof)],
                     output / f'{name}-proof.log', env)
        summary = json.loads((proof / f'{name}-proof-result.json').read_text())
        require(summary['status'] == 'passed' and
                summary['selected_source_export_proved'] == name and
                summary['constructor_execution_proved'] is False and
                summary['all_exports_proved'] is False and len(summary['calls']) == 1,
                f'{name} native proof summary differs')
        row = summary['calls'][0]
        require(row['circuit'] == name and row['k'] == k and
                all(row[field] is True for field in
                    ('standalone_verified_separately', 'corrupt_proof_rejected',
                     'changed_binding_rejected', 'wrong_verifier_rejected_at_preparation',
                     'wrong_verifier_proof_rejected',
                     'replay_unchanged')) and
                row['strict_application'] == 'default' and
                row['replay_refusal'] == 'IntentAlreadyExists' and
                row['transaction_proof_version'] == 3 and
                row['positive_fee_dust_spends'] == 1,
                f'{name} proof/funded controls differ')
        calls.append(row)
    artifact_hashes = hash_files(proof)
    for inventory in frozen_runner_inputs:
        unchanged(inventory)
    require(source_state() == source and admission.tracked_source_hashes() == source_before and
            admission.generated_hashes(generated) == prepared['generated_hashes'] and
            hash_files(proof) == artifact_hashes and
            all(sha256(Path(row['path'])) == row['sha256'] for row in runners.values()) and
            all(sha256(Path(row['inventory_path'])) == row['inventory_sha256'] and
                sha256(Path(row['retained_depfile'])) == row['depfile_sha256']
                for row in runners.values()) and
            all(sha256(Path(verified['cache_directory']) / name) == digest
                for name, digest in material_hashes.items()) and
            all(artifact_hashes[f'zkir/{name}.zkir'] == record['zkir_sha256'] and
                artifact_hashes[f'{name}/contract/compact-rust-ir.json'] == record['ir_sha256']
                for name, record in selected_source.items()),
            'source, generated, runner or proof artifact drifted')
    report = {**source, 'format': 'compact-ledger9-bytes-representative-proof-gate/v1',
              'status': 'passed', 'platform': os.uname().sysname + ' ' + os.uname().machine,
              'prepare_receipt_sha256': sha256(output / 'prepared/receipt.json'),
              'proof_execution': True, 'funded_application': True,
              'selected_proof_required_exports': list(SELECTED),
              'unproved_proof_required_exports': [
                  admission.semantics.circuit_name(case)
                  for case in admission.EXPECTED_PROOF
                  if admission.semantics.circuit_name(case) not in SELECTED],
              'constructor_execution_proved': False, 'all_exports_proved': False,
              'selected_source': selected_source, 'source_count': len(source_before),
              'source_hashes_sha256': sha256(output / 'prepared/source-hashes.json'),
              'generated_hashes': prepared['generated_hashes'],
              'zkir_sha256': sha256(zkir), 'material': verified,
              'material_hashes': material_hashes,
              'keygen': keygen, 'runner': runners,
              'cargo_target': str(cargo_target),
              'calls': calls, 'artifact_hashes': artifact_hashes,
              'limitations': ['seven proof-required Bytes exports remain unproved',
                              'constructor execution was not proved',
                              'native Linux full matrix and real two-contract transaction remain open']}
    (output / 'receipt.json').write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': 'passed', 'head': source['head'],
                      'selected_proofs': len(calls), 'output': str(output)}))


if __name__ == '__main__':
    main()
