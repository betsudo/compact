#!/usr/bin/env python3

# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0

"""Fund both original mixed native collection circuits from pinned proof artifacts."""

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
PROOF_HEAD = 'c071118163bac61512e878ce37b73d42f6e36f54'
PROOF_RECEIPT_SHA = 'e083a54d7a68c280c97586ba4d0cbd426297bb1216e5053026c1093df731ad58'
PREFLIGHT_RECEIPT_SHA = '39a554dfafd2217e96d26f39f14657d228bcfe714d0e5841b644bc0e27128d20'
BASE_MANIFEST_SHA = 'c247f5ee7c3bc5ada73417b71c9606b0132e6a9992c7efb84d4dff547b131b33'
BASE_LOCK_SHA = 'd77e63ce87efc463e04d7732ed64cd40d710a4765f6c40ec756211de18a469cd'
TEMPLATE = ROOT / 'runtime-rs/tests/fixtures/ledger9-mixed-original-funded.rs.in'
CASES = {'k1': (17, 'Secp256k1Scalar', 'two'),
         'r1': (18, 'Secp256r1Scalar', 'three')}


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()


def run(command, log, *, env=None):
    result = subprocess.run(command, cwd=ROOT, env=env, text=True,
                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    log.write_text(result.stdout)
    require(result.returncode == 0, f'{command[0]} failed; inspect {log}')
    return result.stdout


def packages(lock):
    data = tomllib.loads(Path(lock).read_text())
    return sorted((row['name'], row['version'], row.get('checksum'))
                  for row in data['package'] if row.get('source'))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for key in ('evidence', 'preflight', 'base-consumer', 'material',
                'wrong-verifiers', 'ledger-static', 'output'):
        parser.add_argument('--' + key, required=True, type=Path)
    args = parser.parse_args()
    require(not git('status', '--porcelain'), 'funded source must be clean')
    head, tree = git('rev-parse', 'HEAD'), git('rev-parse', 'HEAD^{tree}')
    require(git('log', '-1', '--format=%G?') == 'G' and
            'Signed-off-by:' in git('log', '-1', '--format=%B'),
            'funded source must be signed and DCO complete')
    require(subprocess.run(['git', 'merge-base', '--is-ancestor', PROOF_HEAD, head],
                           cwd=ROOT).returncode == 0, 'proof head is not an ancestor')
    require(set(git('diff', '--name-only', PROOF_HEAD, head).splitlines()) ==
            {TEMPLATE.relative_to(ROOT).as_posix(),
             Path(__file__).relative_to(ROOT).as_posix()},
            'funded slice changed unexpected source')
    evidence = args.evidence.resolve(strict=True)
    preflight = args.preflight.resolve(strict=True)
    material = args.material.resolve(strict=True)
    wrong_root = args.wrong_verifiers.resolve(strict=True)
    base = args.base_consumer.resolve(strict=True)
    static = args.ledger_static.resolve(strict=True)
    output = args.output.resolve()
    require(not args.output.is_symlink() and not output.exists(), 'output must be absent')
    require(sha(evidence / 'proof-receipt-c071.json') == PROOF_RECEIPT_SHA and
            sha(preflight / 'receipt.json') == PREFLIGHT_RECEIPT_SHA,
            'pinned proof or preflight receipt changed')
    proof = json.loads((evidence / 'proof-receipt-c071.json').read_text())
    prior = json.loads((preflight / 'receipt.json').read_text())
    require(proof['source_head'] == PROOF_HEAD and
            proof['source_tree'] == git('rev-parse', f'{PROOF_HEAD}^{{tree}}') and
            proof['preflight_receipt_sha256'] == PREFLIGHT_RECEIPT_SHA and
            prior['source_head'] == PROOF_HEAD and
            prior['source_tree'] == proof['source_tree'] and
            proof['platform'] == 'Darwin arm64',
            'proof source or platform binding changed')
    require(sha(base / 'Cargo.toml') == BASE_MANIFEST_SHA and
            sha(base / 'Cargo.lock') == BASE_LOCK_SHA,
            'pinned funded Cargo baseline changed')
    manifest_text = (base / 'Cargo.toml').read_text()
    require('compact-contract-k1' in manifest_text and
            'mixed-original-k1-funded' in manifest_text,
            'funded Cargo baseline shape changed')
    template = TEMPLATE.read_text()
    require(all(token in template for token in ('__SCALAR__', '__WRONG_ROOT__',
            '__PREFLIGHT__', '__K__', '__WRONG__', '__CASE__')),
            'funded Rust template shape changed')
    require(static.name == 'static' and
            static.parent.name == 'midnight-ledger-v9-1.0.1',
            'Ledger static directory is not the pinned SDK package')
    require((material / 'bls_midnight_2p17').is_file() and
            (material / 'bls_midnight_2p18').is_file(),
            'Ledger proof parameters missing')
    output.mkdir(parents=True)
    env = {key: value for key, value in os.environ.items()
           if not key.startswith('CARGO_') and key not in
           {'RUSTFLAGS', 'RUSTC_WRAPPER', 'RUSTUP_TOOLCHAIN'}}
    env['CARGO_TARGET_DIR'] = str(output / 'target')
    env['MIDNIGHT_PP'] = str(material)
    env['MIDNIGHT_LEDGER_TEST_STATIC_DIR'] = str(static)
    rustc = run(['rustc', '+1.99.0', '-vV'], output / 'rustc.log')
    require('release: 1.99.0' in rustc.splitlines(), 'Rust toolchain changed')
    results = {}
    for case, (k, scalar, wrong) in CASES.items():
        qualified = proof['cases'][case]
        require(qualified['k'] == k and qualified['independent_verification'] and
                qualified['wrong_verifier_changed_binding_corrupt_proof_refused'],
                f'{case} independent proof receipt changed')
        for path, digest in (
            (evidence / 'proof' / case / 'statement.proof', qualified['proof_sha256']),
            (evidence / 'proof' / case / 'statement.prover', qualified['prover_sha256']),
            (evidence / 'proof' / case / 'statement.verifier', qualified['verifier_sha256']),
            (preflight / case / 'zkir/test.zkir', qualified['raw_zkir_sha256']),
            (preflight / f'{case}-capture/honest.preimage', qualified['preimage_sha256']),
            (preflight / case / 'contract/lib.rs', prior['artifacts'][f'{case}/contract/lib.rs']),
        ):
            require(sha(path) == digest, f'{case} pinned artifact changed: {path}')
        require(sha(material / f'bls_midnight_2p{k}') == qualified['params_sha256'],
                f'{case} proof parameters changed')
        require(sha(wrong_root / wrong / 'statement.verifier') ==
                qualified['wrong_verifier_sha256'], f'{case} wrong verifier changed')
        consumer = output / 'consumer' / case
        consumer.mkdir(parents=True)
        generated = preflight / case / 'contract'
        package = tomllib.loads((generated / 'Cargo.toml').read_text())['package']['name']
        require(package == f'compact-contract-{case}', 'generated package changed')
        name = f'mixed-original-{case}-funded'
        manifest = manifest_text.replace('mixed-original-k1-funded', name).replace(
            'compact-contract-k1', package).replace(
            '../../exact-head-preflight-v1/k1/contract', str(generated))
        require(manifest != manifest_text or case == 'k1', 'Cargo replacement failed')
        (consumer / 'Cargo.toml').write_text(manifest)
        shutil.copyfile(base / 'Cargo.lock', consumer / 'Cargo.lock')
        replacements = {'__SCALAR__': scalar,
                        '__WRONG_ROOT__': json.dumps(str(wrong_root)),
                        '__PREFLIGHT__': json.dumps(str(preflight)),
                        '__K__': str(k), '__WRONG__': wrong, '__CASE__': case}
        source = template
        for token, replacement in replacements.items():
            source = source.replace(token, replacement)
        require(all(token not in source for token in replacements),
                'funded Rust substitution incomplete')
        (consumer / 'main.rs').write_text(source)
        run(['cargo', '+1.99.0', 'build', '--offline', '--manifest-path',
             str(consumer / 'Cargo.toml')], output / f'{case}-build.log', env=env)
        require(packages(base / 'Cargo.lock') == packages(consumer / 'Cargo.lock'),
                f'{case} Cargo package set changed')
        run(['cargo', '+1.99.0', 'build', '--locked', '--offline', '--manifest-path',
             str(consumer / 'Cargo.toml')], output / f'{case}-locked-build.log', env=env)
        destination = output / f'{case}-funded'
        stdout = run([str(output / 'target/debug' / name), str(evidence),
                      str(destination)], output / f'{case}-funded.log', env=env)
        lines = [line for line in stdout.splitlines() if line.startswith('{')]
        require(len(lines) == 1, f'{case} funded result missing')
        result = json.loads(lines[0])
        require(result['status'] == 'passed' and result['k'] == k and
                result['ledger_matches_native'] and result['default_strict'] and
                result['positive_fee_dust_spends'] == 1 and
                result['typescript_preimage_fields_match'] and
                result['communication_commitment_recomputed'] and
                result['wrong_verifier_refused'] and
                result['replay_refusal'] == 'IntentAlreadyExists' and
                result['replay_unchanged'] and
                result['transaction_proof_bytes'] > 0,
                f'{case} funded assertions failed')
        require((destination / 'ledger-after.bin').read_bytes() ==
                (destination / 'ledger-replay.bin').read_bytes(),
                f'{case} replay changed serialized ledger')
        results[case] = {'result': result,
                         'transaction_sha256': sha(destination / 'applied-transaction.bin'),
                         'transaction_proof_sha256': sha(destination / 'transaction-bound.proof'),
                         'ledger_sha256': sha(destination / 'ledger-after.bin'),
                         'consumer_source_sha256': sha(consumer / 'main.rs')}
    receipt = {'format': 'ledger9-mixed-original-funded/v1',
               'head': head, 'tree': tree, 'proof_head': PROOF_HEAD,
               'proof_receipt_sha256': PROOF_RECEIPT_SHA,
               'preflight_receipt_sha256': PREFLIGHT_RECEIPT_SHA,
               'template_sha256': sha(TEMPLATE), 'gate_sha256': sha(__file__),
               'platform': platform.platform(), 'cases': results,
               'limitations': ['Local Darwin arm64 qualification; native Linux required.',
                               'Generated EntryPoint and real two-contract qualification remain held.']}
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': 'passed', 'head': head, 'cases': list(results),
                      'receipt': str(output / 'receipt.json')}))


if __name__ == '__main__':
    main()
