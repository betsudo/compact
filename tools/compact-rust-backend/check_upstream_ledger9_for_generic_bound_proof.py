#!/usr/bin/env python3

# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0

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
import resolve_ledger_test_static as ledger_static


ROOT = Path(__file__).resolve().parents[2]
BASE = '32474f67111aea668a42d09006af16af3eb931d0'
RUNNER_FILES = {
    'tools/compact-rust-proof-smoke/Cargo.toml',
    'tools/compact-rust-proof-smoke/build.rs',
    'tools/compact-rust-proof-smoke/src/main.rs',
    'tools/compact-rust-proof-smoke/src/direct_dispatch.rs',
    'tools/compact-rust-proof-smoke/src/for_generic_bound_proof.rs',
    'tools/compact-rust-backend/check_upstream_ledger9_for_generic_bound_proof.py',
}
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


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('grouped-gate', 'zkir', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    source = source_state()
    changed = set(subprocess.check_output(
        ['git', 'diff', '--name-only', BASE, 'HEAD'], cwd=ROOT, text=True).splitlines())
    require(changed == RUNNER_FILES, 'proof head must contain only the signed runner follow-up')
    require(subprocess.run(['git', 'merge-base', '--is-ancestor', BASE, 'HEAD'],
                           cwd=ROOT).returncode == 0, 'grouped base is not an ancestor')
    output = args.output.resolve()
    require(not output.exists() and not args.output.is_symlink(), 'new proof output required')
    gate_root = args.grouped_gate.resolve(strict=True)
    gate_path = gate_root / 'receipt.json'
    gate = json.loads(gate_path.read_text())
    require(gate['status'] == 'passed' and gate['head'] == BASE and
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
    env = dict(os.environ, PYTHONDONTWRITEBYTECODE='1', RUSTUP_TOOLCHAIN='1.99.0',
               CARGO_INCREMENTAL='0', CARGO_PROFILE_DEV_DEBUG='0',
               COMPACT_FOR_GENERIC_BOUND_GENERATED=str(both),
               CARGO_TARGET_DIR=str(output / 'cargo-target'),
               MIDNIGHT_PP=str(Path.home() / '.cache/midnight/zk-params'))
    env.pop('MIDNIGHT_PROOF_SERVER', None)
    require(Path(env['MIDNIGHT_PP']).is_dir(), 'local proof parameter cache absent')
    metadata = run(['cargo', '+1.99.0', 'metadata', '--locked', '--offline',
                    '--format-version', '1'], output / 'metadata.log', env)
    static = ledger_static.resolve(json.loads(metadata.stdout),
                                   tomllib.loads((ROOT / 'Cargo.lock').read_text()))
    env['MIDNIGHT_LEDGER_TEST_STATIC_DIR'] = static['static_directory']
    build = run(['cargo', '+1.99.0', 'build', '--locked', '--offline', '--profile',
                 'proof-checks', '-p', 'compact-rust-proof-smoke', '--features',
                 'for-generic-bound-proof', '--message-format=json'],
                output / 'build.log', env)
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
    copy_new(depfile, output / 'proof-runner.d')
    deptext = depfile.read_text()
    for expected in (both / 'contract/lib.rs', both / 'contract/compact-rust-ir.json',
                     both / f'zkir/{GENERIC}.zkir',
                     ROOT / 'tools/compact-rust-proof-smoke/src/for_generic_bound_proof.rs'):
        require(str(expected) in deptext, f'runner depfile omitted {expected}')
    runner_sha = sha256(runner)
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
    require(source_state() == source and files(generated) == gate['generated_files_sha256'] and
            sha256(runner) == runner_sha and
            all(sha256(Path(verified['cache_directory']) / name) == digest
                for name, digest in material_hashes.items()) and
            all(sha256(proof / name) == digest for name, digest in input_hashes.items()),
            'proof source, generated artifact or proving material drifted')
    report = {**source, 'format':'compact-ledger9-for-generic-bound-proof-gate/v1',
              'status':'passed', 'platform':os.uname().sysname + ' ' + os.uname().machine,
              'grouped_head':BASE, 'grouped_gate_sha256':sha256(gate_path),
              'generated_input_sha256':gate['generated_files_sha256'],
              'runner_sha256':runner_sha, 'runner_depfile_sha256':sha256(output / 'proof-runner.d'),
              'zkir_sha256':sha256(zkir), 'ledger_static':static,
              'material':verified, 'material_hashes':material_hashes,
              'keygen':keys, 'proof_summary':summary, 'artifact_hashes':files(proof),
              'proof_execution':True, 'funded_application':True,
              'limitations':['constructor execution was not proved',
                             'native Linux grouped qualification remains open']}
    (output / 'receipt.json').write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status':'passed', 'head':source['head'], 'output':str(output)}))


if __name__ == '__main__':
    main()
