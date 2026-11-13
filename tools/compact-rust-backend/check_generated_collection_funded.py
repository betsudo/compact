#!/usr/bin/env python3

# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");

"""Apply a generated Set.add call with a funded default-strict Ledger 9 transaction."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tomllib


ROOT = Path(__file__).resolve().parents[2]
FUNDED = ROOT / 'runtime-rs/tests/fixtures/ledger9-set-funded.rs'
SOURCE = ROOT / 'runtime-rs/tests/fixtures/ledger9-set-transition-v018.compact'
PROOF_RECEIPT_SHA256 = 'e9a7396152ba7a4303b0b9f73ef8f52bdd1ef2808657d6b5ecf80f7da8f5f555'
BASE_MANIFEST_SHA256 = '7d69e964783ad25221daded85b38ef4ca3d603f166b3c0659207fd52a1f3deec'
BASE_LOCK_SHA256 = '73ebbcf500f92f0105aa36eaff0879ef759df6da10e9f6c3f5f0f1adbe756362'
GENERATED_MANIFEST_SHA256 = 'fdc667692e01cccebcd895f6fc65c257a173e120db348a1edc6a7d5585dd0978'


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def git(*args: str) -> str:
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()


def run(command: list[str], *, env: dict[str, str] | None = None, log: Path) -> None:
    with log.open('xb') as output:
        subprocess.run(command, cwd=ROOT, env=env, stdout=output,
                       stderr=subprocess.STDOUT, check=True)


def normalized_lock(path: Path) -> list[str]:
    packages = tomllib.loads(path.read_text())['package']
    old = ('ledger9-generated-set-funded', 'compact-contract-ledger9-set-transition-v018')
    new = ('ledger9-mixed-foreign-casts-funded', 'compact-contract-constant')
    rows = []
    for package in packages:
        row = dict(package)
        for from_name, to_name in zip(old, new):
            if row['name'] == from_name:
                row['name'] = to_name
            row['dependencies'] = [dependency.replace(from_name, to_name, 1)
                                   if dependency.split(' ')[0] == from_name else dependency
                                   for dependency in row.get('dependencies', [])]
        rows.append(json.dumps(row, sort_keys=True))
    return sorted(rows)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('proof-gate', 'base-consumer', 'material', 'cargo-target', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    require(not git('status', '--porcelain'), 'source checkout must be clean')
    head = git('rev-parse', 'HEAD')
    tree = git('rev-parse', 'HEAD^{tree}')
    require(git('log', '-1', '--format=%G?') == 'G' and
            'Signed-off-by:' in git('log', '-1', '--format=%B'),
            'source commit must be signed and DCO complete')
    gate = args.proof_gate.resolve(strict=True)
    base = args.base_consumer.resolve(strict=True)
    material = args.material.resolve(strict=True)
    cargo_target = args.cargo_target.resolve(strict=True)
    output = args.output.resolve()
    require(not args.output.is_symlink() and not output.exists(), 'output must be absent')
    gate_receipt = gate / 'receipt.json'
    require(sha(gate_receipt) == PROOF_RECEIPT_SHA256, 'proof gate receipt changed')
    proof = json.loads(gate_receipt.read_text())
    require(subprocess.run(['git', 'merge-base', '--is-ancestor', proof['head'], head],
                           cwd=ROOT).returncode == 0, 'funded source is not stacked on proof source')
    require(subprocess.run(['git', 'diff', '--quiet', proof['head'], head, '--',
                            'runtime-rs/src', 'runtime-rs/Cargo.toml',
                            'runtime-rs-macros/src', 'runtime-rs-macros/Cargo.toml'],
                           cwd=ROOT).returncode == 0,
            'funded branch changed the runtime used by the generated contract')
    require(proof['source_sha256'] == sha(SOURCE), 'generated Set source changed')
    for relative, key in (
        ('fresh-dual/contract/index.js', 'fresh_typescript_sha256'),
        ('fresh-dual/contract/lib.rs', 'fresh_rust_sha256'),
        ('fresh-dual/contract/compact-rust-ir.json', 'fresh_rust_ir_sha256'),
        ('fresh-dual/zkir/add.zkir', 'fresh_add_zkir_sha256'),
        ('fresh-dual/zkir/remove.zkir', 'fresh_remove_zkir_sha256'),
        ('preimage/honest.preimage', 'preimage_sha256'),
        ('proof/statement.proof', 'proof_sha256'),
        ('proof/statement.prover', 'prover_sha256'),
        ('proof/statement.verifier', 'verifier_sha256'),
        ('wrong-key/remove.verifier', 'wrong_same_k_verifier_sha256'),
    ):
        require(sha(gate / relative) == proof[key], f'proof gate artifact changed: {relative}')
    generated_manifest = gate / 'fresh-dual/contract/Cargo.toml'
    require(sha(generated_manifest) == GENERATED_MANIFEST_SHA256,
            'generated contract manifest changed')
    runtime_path = (generated_manifest.parent / Path(
        tomllib.loads(generated_manifest.read_text())['dependencies']
        ['midnight-compact-runtime']['path'])).resolve(strict=True)
    runtime_checkout = Path(subprocess.check_output(
        ['git', '-C', str(runtime_path), 'rev-parse', '--show-toplevel'], text=True).strip())
    require(runtime_path == runtime_checkout / 'runtime-rs' and
            subprocess.check_output(['git', '-C', str(runtime_checkout), 'rev-parse', 'HEAD'],
                                    text=True).strip() == proof['head'] and
            not subprocess.check_output(['git', '-C', str(runtime_checkout), 'status', '--porcelain'],
                                        text=True).strip(),
            'generated contract runtime checkout changed')
    require(sha(base / 'Cargo.toml') == BASE_MANIFEST_SHA256 and
            sha(base / 'Cargo.lock') == BASE_LOCK_SHA256,
            'archived funded consumer changed')
    params = material / 'bls_midnight_2p6'
    require(sha(params) == proof['params_k6_sha256'], 'funded k=6 parameters changed')
    output.mkdir(parents=True)
    consumer = output / 'consumer'
    consumer.mkdir()
    manifest = (base / 'Cargo.toml').read_text()
    package_name = 'ledger9-mixed-foreign-casts-funded'
    original_dependency = ('constant = { package = "compact-contract-constant", path = '
                           f'"{base.parent / "both/constant/contract"}", '
                           'features = ["ledger-transaction"] }')
    require(manifest.count(package_name) == 2 and manifest.count(original_dependency) == 1,
            'archived funded manifest shape changed')
    manifest = manifest.replace(package_name, 'ledger9-generated-set-funded')
    manifest = manifest.replace(original_dependency,
        f'setcontract = {{ package = "compact-contract-ledger9-set-transition-v018", '
        f'path = "{gate / "fresh-dual/contract"}", features = ["ledger-transaction"] }}')
    (consumer / 'Cargo.toml').write_text(manifest)
    shutil.copyfile(base / 'Cargo.lock', consumer / 'Cargo.lock')
    shutil.copyfile(FUNDED, consumer / 'main.rs')
    require(sha(consumer / 'main.rs') == sha(FUNDED), 'copied funded source changed')
    env = os.environ.copy()
    env['CARGO_TARGET_DIR'] = str(cargo_target)
    run(['cargo', '+1.99.0', 'build', '--offline', '--manifest-path',
         str(consumer / 'Cargo.toml')], env=env, log=output / 'funded-build.log')
    require(normalized_lock(base / 'Cargo.lock') == normalized_lock(consumer / 'Cargo.lock'),
            'funded consumer changed pinned lock packages')
    run(['cargo', '+1.99.0', 'build', '--locked', '--offline', '--manifest-path',
         str(consumer / 'Cargo.toml')], env=env, log=output / 'funded-locked-build.log')
    require(sha(consumer / 'main.rs') == sha(FUNDED), 'funded source changed during build')
    executable = cargo_target / 'debug/ledger9-generated-set-funded'
    require(executable.is_file(), 'funded consumer binary missing')
    funded_env = env.copy()
    funded_env.update(MIDNIGHT_PP=str(material), MIDNIGHT_LEDGER_TEST_STATIC_DIR=str(material),
                      RAYON_NUM_THREADS='4')
    results = output / 'funded-result'
    with (output / 'funded-run.json').open('xb') as stdout, (
            output / 'funded-run.stderr').open('xb') as stderr:
        subprocess.run([str(executable), str(gate / 'fresh-dual'), str(gate / 'preimage'),
                        str(gate / 'proof'), str(gate / 'wrong-key/remove.verifier'), str(results)],
                       cwd=ROOT, env=funded_env, stdout=stdout, stderr=stderr, check=True)
    row = json.loads((output / 'funded-run.json').read_text())
    require(row['status'] == 'passed' and row['circuit'] == 'add' and row['k'] == 6,
            'funded Set result changed')
    for key in ('default_strict', 'ledger_matches_native', 'typescript_preimage_fields_match',
                'communication_commitment_recomputed', 'wrong_verifier_refused',
                'replay_unchanged', 'transaction_proof_distinct'):
        require(row[key] is True, f'funded Set check missing: {key}')
    require(row['positive_fee_dust_spends'] == 1 and row['replay_refusal'] == 'IntentAlreadyExists',
            'funded fee or replay result changed')
    require(row['strict_recheck_refused'] is False,
            'default-strict replay recheck behavior changed')
    require((results / 'ledger-after.bin').read_bytes() ==
            (results / 'ledger-replay.bin').read_bytes(), 'replay changed ledger bytes')
    require(sha(results / 'transaction-bound.proof') != sha(results / 'standalone.proof.raw'),
            'transaction proof equals standalone proof')
    require(not git('status', '--porcelain') and git('rev-parse', 'HEAD') == head,
            'source checkout changed during qualification')
    receipt = {
        'platform': f'{platform.system()} {platform.machine()}', 'head': head, 'tree': tree,
        'proof_gate_receipt_sha256': sha(gate_receipt), 'funded_source_sha256': sha(FUNDED),
        'generated_manifest_sha256': sha(generated_manifest),
        'generated_runtime_checkout_head': proof['head'],
        'base_manifest_sha256': sha(base / 'Cargo.toml'),
        'base_lock_sha256': sha(base / 'Cargo.lock'),
        'funded_lock_sha256': sha(consumer / 'Cargo.lock'),
        'funded_binary_sha256': sha(executable),
        'funded_build_log_sha256': sha(output / 'funded-build.log'),
        'funded_locked_build_log_sha256': sha(output / 'funded-locked-build.log'),
        'funded_results': row,
        'applied_transaction_sha256': sha(results / 'applied-transaction.bin'),
        'transaction_bound_proof_sha256': sha(results / 'transaction-bound.proof'),
        'standalone_proof_raw_sha256': sha(results / 'standalone.proof.raw'),
        'ledger_after_sha256': sha(results / 'ledger-after.bin'),
        'ledger_replay_sha256': sha(results / 'ledger-replay.bin'),
        'limitations': ['Local platform only; native Linux remains required.',
                        'This is one Set.add contract call, not a two-contract transaction.',
                        'Contract deployment is inserted into the local ledger test state, not applied as a deployment transaction.',
                        'Wrong-verifier refusal is generated-client prepare refusal.',
                        'A default-strict recheck of the already applied transaction succeeds; IntentAlreadyExists is refused at ledger application of the previously verified transaction.'],
    }
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(f'funded generated Set.add strict application and unchanged replay refusal passed: {output}')


if __name__ == '__main__':
    main()
