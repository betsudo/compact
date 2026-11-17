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

"""Prove the pinned for/generic_bound export with a funded strict local call."""

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

from check_upstream_ledger9_field_casts import require, sha256, source_state
from did_proof_gate import hashes, unchanged
import proof_runner_input_binding as runner_inputs
import resolve_ledger_test_static as ledger_static


ROOT = Path(__file__).resolve().parents[2]
PROOF_BODY = ROOT / 'tools/compact-rust-proof-smoke/src/for_generic_bound_proof.rs'
PROOF_BODY_SHA256 = 'ca60907993b2b9f9f2f4112eeb76549b703a805d9380761a5fc80136292c4a94'
GENERIC = 'for_generic_bound'
WRONG = 'for_zero_iteration'


def run(command: list[str], log: Path, env: dict[str, str]) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True)
    log.write_text(result.stdout + result.stderr)
    require(result.returncode == 0, f'{command[0]} exited {result.returncode}: {log}')
    return result


def files(root: Path) -> dict[str, str]:
    result = {}
    for path in sorted(root.rglob('*')):
        require(not path.is_symlink(), f'artifact symlink: {path}')
        if path.is_file():
            result[path.relative_to(root).as_posix()] = sha256(path)
    return result


def copy_new(source: Path, destination: Path) -> None:
    require(source.is_file() and not source.is_symlink() and not destination.exists(),
            f'expected fresh physical artifact: {destination}')
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, destination)


def tracked_source_hashes() -> dict[str, str]:
    """Freeze every committed physical source before any Cargo preparation."""
    result = {}
    for raw in subprocess.check_output(['git', 'ls-files', '-z'], cwd=ROOT).split(b'\0'):
        if not raw:
            continue
        name = os.fsdecode(raw)
        path = ROOT / name
        require(path.is_file() and not path.is_symlink(),
                f'tracked source absent or symlink: {name}')
        result[name] = sha256(path)
    return result


def physical_sha256(path: str | Path) -> str:
    return sha256(Path(path))


def dependency_refusals(tracked: set[str], inventories: dict[str, dict[str, str]],
                        output: Path) -> dict[str, bool]:
    """Refuse unadmitted and altered physical prerequisites before keygen."""
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
    for name in ('grouped-gate', 'zkir', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--expected-source-sha', required=True)
    parser.add_argument('--cargo-target', required=True, type=Path)
    args = parser.parse_args()
    source = source_state()
    source_before = tracked_source_hashes()
    require(re.fullmatch('[0-9a-f]{40}', args.expected_source_sha) is not None and
            source['head'] == args.expected_source_sha,
            'proof source differs from explicitly selected exact head')
    require(PROOF_BODY.is_file() and not PROOF_BODY.is_symlink() and
            sha256(PROOF_BODY) == PROOF_BODY_SHA256,
            'dedicated signed generic-bound proof body changed')
    output = args.output.resolve()
    require(not output.exists() and not args.output.is_symlink(), 'new proof output required')
    gate_root = args.grouped_gate.resolve(strict=True)
    gate_path = gate_root / 'receipt.json'
    gate = json.loads(gate_path.read_text())
    require(gate['status'] == 'passed' and gate['head'] == source['head'] and
            gate['tree'] == source['tree'] and
            gate.get('case_count') == 10 and gate.get('source_clean') is True,
            'grouped exact-head for-keyword gate is not accepted')
    generated = gate_root / 'generated'
    require(files(generated) == gate['generated_files_sha256'],
            'grouped generated inventory differs from signed source gate')
    both = generated / 'generic_bound/both'
    require(gate['generated']['generic_bound']['typescript']['zkir_sha256'] ==
            gate['generated']['generic_bound']['both']['zkir_sha256'],
            'pinned TypeScript and paired raw ZKIR differ')
    zkir = args.zkir.resolve(strict=True)
    require(zkir.name == 'zkir', 'official zkir executable required')
    output.mkdir(parents=True)
    proof = output / 'proof'
    copy_new(both / 'contract/compact-rust-ir.json',
             proof / f'{GENERIC}/contract/compact-rust-ir.json')
    copy_new(both / f'zkir/{GENERIC}.zkir', proof / f'zkir/{GENERIC}.zkir')
    copy_new(generated / f'zero_iteration/both/zkir/{WRONG}.zkir',
             proof / f'zkir/{WRONG}.zkir')
    (proof / 'keys').mkdir()
    input_hashes = files(proof)
    parameter_cache = Path(os.environ.get('MIDNIGHT_PP',
                           str(Path.home() / '.cache/midnight/zk-params')))
    require(parameter_cache.is_absolute() and parameter_cache.is_dir() and
            not parameter_cache.is_symlink(), 'verified proof parameter cache absent')
    cargo_target = args.cargo_target.resolve(strict=True)
    require(cargo_target.is_dir() and not args.cargo_target.is_symlink(),
            'shared Cargo target must be an ordinary existing directory')
    env = dict(os.environ, PYTHONDONTWRITEBYTECODE='1', RUSTUP_TOOLCHAIN='1.99.0',
               CARGO_INCREMENTAL='0', CARGO_PROFILE_DEV_DEBUG='0',
               COMPACT_FOR_GENERIC_BOUND_GENERATED=str(both),
               CARGO_TARGET_DIR=str(cargo_target),
               MIDNIGHT_PP=str(parameter_cache))
    env.pop('MIDNIGHT_PROOF_SERVER', None)
    metadata = run(['cargo', '+1.99.0', 'metadata', '--locked', '--offline',
                    '--format-version', '1'], output / 'metadata.log', env)
    static = ledger_static.resolve(json.loads(metadata.stdout),
                                   tomllib.loads((ROOT / 'Cargo.lock').read_text()))
    env['MIDNIGHT_LEDGER_TEST_STATIC_DIR'] = static['static_directory']
    binding_dir = output / 'binding'
    binding_dir.mkdir()
    source_hashes_path = binding_dir / 'source-hashes.json'
    source_hashes_path.write_text(json.dumps(source_before, indent=2, sort_keys=True) + '\n')
    source_hashes_sha = sha256(source_hashes_path)
    binding = {
        'source_hashes': {str((ROOT / name).resolve(strict=True)): digest
                          for name, digest in source_before.items()},
        'generated_input_hashes': hashes(path for path in generated.rglob('*')
                                         if path.is_file()),
    }

    def execute(label: str, command: list[str], log: Path) -> subprocess.CompletedProcess[str]:
        for field in ('source_hashes', 'generated_input_hashes',
                      'dependency_source_hashes', 'generated_build_inputs_before_compile'):
            if field in binding:
                unchanged(binding[field])
        return run(command, log, env)

    runner_inputs.prepare_generated_inputs(ROOT, binding_dir, binding, execute,
                                           'for-generic-bound-proof', sha256)
    build = execute('build-proof-runner',
                    ['cargo', '+1.99.0', 'build', '--locked', '--offline', '--profile',
                     'proof-checks', '-p', 'compact-rust-proof-smoke', '--features',
                     'for-generic-bound-proof', '--message-format=json'],
                    output / 'build.log')
    rows = [json.loads(line) for line in build.stdout.splitlines() if line.startswith('{')]
    executables = {row['executable'] for row in rows
                   if row.get('reason') == 'compiler-artifact' and
                   row['target']['name'] == 'compact-rust-proof-smoke' and row.get('executable')}
    require(len(executables) == 1, 'ambiguous generic-bound proof runner')
    built = Path(executables.pop())
    runner = output / 'proof-runner'
    copy_new(built, runner)
    runner.chmod(0o755)
    depfile = built.with_suffix('.d')
    tracked = runner_inputs.depfile_prerequisites(depfile.read_text(), ROOT)
    for expected in (both / 'contract/lib.rs', both / 'contract/compact-rust-ir.json',
                     both / f'zkir/{GENERIC}.zkir',
                     ROOT / 'tools/compact-rust-proof-smoke/src/for_generic_bound_proof.rs'):
        require(str(expected.resolve(strict=True)) in tracked,
                f'runner depfile omitted {expected}')
    inventories = {field: binding[field] for field in
                   ('source_hashes', 'generated_input_hashes',
                    'dependency_source_hashes', 'generated_build_inputs_before_compile')}
    physical, origins = runner_inputs.qualify_dependency_inputs(tracked, inventories,
                                                                 physical_sha256)
    refusals = dependency_refusals(tracked, inventories, binding_dir)
    inventory_path = binding_dir / 'inputs.json'
    inventory_path.write_text(json.dumps({**binding,
                                          'physical_inputs_before_build': physical,
                                          'physical_input_origins': origins},
                                         indent=2, sort_keys=True) + '\n')
    retained_depfile = output / 'proof-runner.d'
    copy_new(depfile, retained_depfile)
    runner_sha = sha256(runner)
    depfile_sha = sha256(retained_depfile)
    inventory_sha = sha256(inventory_path)
    frozen_runner_inputs = [*inventories.values(), physical]
    require(files(proof) == input_hashes and files(generated) == gate['generated_files_sha256'],
            'runner build changed pinned inputs')
    # The embedded IR and raw circuit refuse drift before any key is used.
    for field, fragment in ((f'zkir/{GENERIC}.zkir', 'raw ZKIR bytes differ'),
                            (f'{GENERIC}/contract/compact-rust-ir.json',
                             'complete Rust IR bytes differ')):
        bad = output / ('changed-zkir' if field.startswith('zkir') else 'changed-ir')
        shutil.copytree(proof, bad)
        changed_path = bad / field
        changed_path.write_bytes(changed_path.read_bytes() + b' ')
        result = subprocess.run([str(runner), '--for-generic-bound-proof', str(bad)],
                                cwd=ROOT, env=env, capture_output=True, text=True)
        (output / f'{bad.name}.log').write_text(result.stdout + result.stderr)
        require(result.returncode != 0 and fragment in result.stderr and
                not list(bad.rglob('*.proof')),
                f'{field} drift escaped pre-key refusal')
    material = run([str(runner), '--prepare-proof-material', '--verify-only',
                    '--parameter-k', '6', '--parameter-k', '7'],
                   output / 'material.json', env)
    verified = json.loads(material.stdout)
    require(verified['mode'] == 'verify-only' and
            verified['verification'] == 'all files checked against locked upstream SHA256 tables' and
            {6, 7} <= {row['k'] for row in verified['parameters']},
            'upstream k=6/7 proving material was not verified')
    material_hashes = {row['name']: sha256(Path(verified['cache_directory']) / row['name'])
                       for row in verified['assets'] + verified['parameters']}
    keys = {}
    for name in (GENERIC, WRONG):
        result = run([str(zkir), 'compile', str(proof / f'zkir/{name}.zkir'),
                      str(proof / f'keys/{name}.prover'),
                      str(proof / f'keys/{name}.verifier')],
                     output / f'{name}-keygen.log', env)
        match = re.search(r'k=(\d+), rows=(\d+)', result.stdout + result.stderr)
        require(match is not None and (name != GENERIC or match.groups() == ('7', '97')),
                f'{name} key model changed')
        keys[name] = {'k': int(match[1]), 'rows': int(match[2]),
                      'prover_sha256': sha256(proof / f'keys/{name}.prover'),
                      'verifier_sha256': sha256(proof / f'keys/{name}.verifier'),
                      'binary_zkir_sha256': sha256(proof / f'zkir/{name}.bzkir')}
    run([str(runner), '--for-generic-bound-proof', str(proof)],
        output / 'proof-run.log', env)
    summary = json.loads((proof / f'{GENERIC}-proof-result.json').read_text())
    require(summary['status'] == 'passed' and
            summary['selected_source_export_proved'] == GENERIC and
            summary['result'] == summary['ledger_total'] == 5 and
            summary['public_verify_ops'] == 18 and summary['private_transcript_outputs'] == 0 and
            summary['k'] == 7 and summary['strict_application'] == 'default' and
            summary['replay_refusal'] == 'IntentAlreadyExists' and
            summary['positive_fee_dust_spends'] == 1 and
            all(summary[key] is True for key in
                ('standalone_verified_separately', 'wrong_verifier_rejected_at_preparation',
                 'wrong_verifier_proof_rejected', 'corrupt_proof_rejected',
                 'changed_binding_rejected', 'replay_unchanged')),
            'generic-bound proof, verification or funded application differs')
    for inventory in frozen_runner_inputs:
        unchanged(inventory)
    require(source_state() == source and tracked_source_hashes() == source_before and
            files(generated) == gate['generated_files_sha256'] and
            sha256(runner) == runner_sha and
            sha256(retained_depfile) == depfile_sha and
            sha256(inventory_path) == inventory_sha and
            sha256(source_hashes_path) == source_hashes_sha and
            all(sha256(Path(verified['cache_directory']) / name) == digest
                for name, digest in material_hashes.items()) and
            all(sha256(proof / name) == digest for name, digest in input_hashes.items()),
            'proof source, generated artifact or proving material drifted')
    report = {**source, 'format':'compact-ledger9-for-generic-bound-proof-gate/v1',
              'status':'passed', 'source_clean':True,
              'platform':os.uname().sysname + ' ' + os.uname().machine,
              'grouped_head':source['head'], 'grouped_gate_sha256':sha256(gate_path),
              'generated_input_sha256':gate['generated_files_sha256'],
              'source_count':len(source_before),
              'source_hashes_sha256':source_hashes_sha,
              'runner_sha256':runner_sha, 'runner_depfile_sha256':depfile_sha,
              'runner_inventory_sha256':inventory_sha,
              'runner_physical_input_count':len(physical),
              'runner_physical_input_origins':origins,
              'dependency_refusals':refusals,
              'zkir_sha256':sha256(zkir), 'ledger_static':static,
              'material':verified, 'material_hashes':material_hashes,
              'keygen':keys, 'proof_summary':summary, 'artifact_hashes':files(proof),
              'proof_execution':True, 'funded_application':True,
              'limitations':['constructor execution was not proved']}
    (output / 'receipt.json').write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status':'passed', 'head':source['head'], 'output':str(output)}))


if __name__ == '__main__':
    main()
