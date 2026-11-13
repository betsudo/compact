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
PROOF_HEAD = '7512d05cb5c10e58adc429fc9fa28bc76eedb951'
PROOF_RECEIPT_SHA = '80117b1c8fb2710ae9fc7636ebb7cc406d08518c0639c033c098d847c7c3a9b3'
PREFLIGHT_RECEIPT_SHA = '55f122e25110b8cee6e247777665d173bc3b2f934bdd308792cca9c929cdc254'
BASE_MANIFEST_SHA = 'c247f5ee7c3bc5ada73417b71c9606b0132e6a9992c7efb84d4dff547b131b33'
BASE_LOCK_SHA = 'd77e63ce87efc463e04d7732ed64cd40d710a4765f6c40ec756211de18a469cd'
MATERIAL_RECEIPT_SHA = '56a004076c7332845fb4cd61ef357892db6dcd7dd05c03f9efebfb43f8df4df7'
GENERATED_MANIFEST_SHA = {'k1': 'f60b95076b029426c978983a2ea74ac35956679360e81c1cc314f0d0ae1a5935',
                          'r1': '95d46740e949bd20dc306fe6d80e0dafa18c5f7972dd8d4183437b3b389ff517'}
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
    require(sha(evidence / 'proof-receipt-7512.json') == PROOF_RECEIPT_SHA and
            sha(preflight / 'receipt.json') == PREFLIGHT_RECEIPT_SHA,
            'pinned proof or preflight receipt changed')
    proof = json.loads((evidence / 'proof-receipt-7512.json').read_text())
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
            'Ledger static directory has unexpected package name')
    material_receipt = material.parent / 'material-receipt.json'
    require(sha(material_receipt) == MATERIAL_RECEIPT_SHA,
            'SDK material receipt changed')
    material_files = json.loads(material_receipt.read_text())['files']
    require(all(sha(material / name) == row['sha256'] and
                (material / name).stat().st_size == row['bytes']
                for name, row in material_files.items()),
            'SDK material files changed')
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
        require(sha(generated / 'Cargo.toml') == GENERATED_MANIFEST_SHA[case],
                f'{case} generated Cargo manifest changed')
        generated_manifest = (generated / 'Cargo.toml').read_text()
        package = tomllib.loads(generated_manifest)['package']['name']
        require(package == f'compact-contract-{case}', 'generated package changed')
        runtime_path = tomllib.loads(generated_manifest)['dependencies']['midnight-compact-runtime']['path']
        require(Path(runtime_path).name == 'runtime-rs' and
                Path(runtime_path).parent ==
                Path('/Users/ysh/.codex/worktrees/ledger9-mixed-original-proofs/compact'),
                f'{case} generated runtime path changed')
        generated_copy = output / 'generated' / case / 'contract'
        generated_copy.parent.mkdir(parents=True)
        shutil.copytree(generated, generated_copy)
        copied_manifest = generated_manifest.replace(
            f'path = "{runtime_path}"', f'path = "{ROOT / "runtime-rs"}"')
        require(copied_manifest != generated_manifest and
                tomllib.loads(copied_manifest)['dependencies']['midnight-compact-runtime']['path'] ==
                str(ROOT / 'runtime-rs'), f'{case} runtime rebinding failed')
        (generated_copy / 'Cargo.toml').write_text(copied_manifest)
        name = f'mixed-original-{case}-funded'
        manifest = manifest_text.replace('mixed-original-k1-funded', name).replace(
            'compact-contract-k1', package).replace(
            '../../exact-head-preflight-v1/k1/contract', str(generated_copy))
        require(manifest != manifest_text or case == 'k1', 'Cargo replacement failed')
        (consumer / 'Cargo.toml').write_text(manifest)
        lock = (base / 'Cargo.lock').read_text().replace(
            'mixed-original-k1-funded', name).replace('compact-contract-k1', package)
        (consumer / 'Cargo.lock').write_text(lock)
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
        require(packages(base / 'Cargo.lock') == packages(consumer / 'Cargo.lock'),
                f'{case} Cargo package set changed before build')
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
                result['sdk_prepare_wrong_verifier_refused'] and
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
               'limitations': ['The initial contract deployment is injected into local test ledger state; only the call is funded and applied as a transaction.',
                               'Local Darwin arm64 qualification; native Linux required.',
                               'Generated EntryPoint and real two-contract qualification remain held.']}
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': 'passed', 'head': head, 'cases': list(results),
                      'receipt': str(output / 'receipt.json')}))


if __name__ == '__main__':
    main()
