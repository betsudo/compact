#!/usr/bin/env python3

# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0

"""Apply a proved original Bytes map call with funded Ledger 9 strictness."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tomllib

from collection_gate_binding import compiler_recipe, generated_manifest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / 'runtime-rs/tests/fixtures/ledger9-bytes-map-transformation-funded.rs'
PROOF_HEAD = 'e75cca456604659d89efd98fcadeea45322252f2'
PROOF_GATE_SOURCE_SHA256 = 'bf5013a02845bac478042c6c93b0bea48a122170ea5914706a881741e2b54c9c'
CAPTURE_SOURCE_SHA256 = 'db6df1c1aa1944249fdef392025e3ff2555af760f85692c82bf5b35a05307d4b'
BASE_QUALIFICATION_RECEIPT_SHA256 = '34bee302b30f1d87a4e04e3904891a452865d36f20ddd61613e39a36a02b4426'
ORIGINAL_SOURCE_SHA256 = '38f12526a78734b8c93f4be6cffa0ef9efaae59db552d56b3cf36ecbbd924680'
ORIGINAL_ZKIR_SHA256 = 'd35a45c657469547c91d998476943ad004c5e47962ce345d350045ccf691dd34'
GENERATED_SHA256 = {
    'transformation/typescript': '602c78bf81af248ffa055ac35190322220ea3e679b4d96b1d04a0ad36d217b61',
    'transformation/rust': '05c7ca2ae0e8ec3c6610174c11e426a9e25abdc32d12e0e55fe6b1da2f94afea',
    'transformation/rust_ir': 'a01b9cd019cac2d9e777c395eb3a2f88cf78fad3f610926b49a198ad66e48f6e',
}
BASE_MANIFEST_SHA256 = 'f87c1f43ac0e78ec220afe8ccda2d06579d3f2dcf21c36f5ae23699943a078cf'
BASE_LOCK_SHA256 = '2f156d51adbeca7dcd87fa0cd28d3ec0f76cfd4700cd2e0f3f56157b9560fadc'
MATERIAL_RECEIPT_SHA256 = '9f8a8464782ab57edfd58e317d28d16e1f9ff620e5fdbaf662a4f6685ab4bae2'
GENERATED_MANIFEST_CANONICAL_SHA256 = '5aa69f074cd531d9cc35681a492cd883ec41a4e6837910f44e4b9ef9484380bb'
RUNNER_SOURCE_SHA256 = {
    'src/main.rs': '2846dd67da2916bcf3a15db93e91a1007cfb5d6b97a1d0070e08c6fe312194e9',
    'Cargo.toml': '1e7a8f3a18ff81ba8eacfb982f20501f0a320668d2e7947612deb4c6986367e0',
    'Cargo.lock': 'c66ac3ad4ad9e2291ce67a90d764af5f7573fbffacdc48953619df7b92ec2ce5',
}


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require(ok: bool, message: str) -> None:
    if not ok:
        raise RuntimeError(message)


def git(*args: str) -> str:
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()


def run(command: list[str], log: Path, *, env: dict[str, str] | None = None) -> None:
    with log.open('xb') as stream:
        subprocess.run(command, cwd=ROOT, env=env, stdout=stream,
                       stderr=subprocess.STDOUT, check=True)


def normalized_lock(path: Path, root_name: str, contract_name: str) -> list[str]:
    rows = []
    for package in tomllib.loads(path.read_text())['package']:
        row = dict(package)
        for old, new in ((root_name, '@funded-root'), (contract_name, '@generated-contract')):
            if row['name'] == old:
                row['name'] = new
            row['dependencies'] = [
                value.replace(old, new, 1) if value.split(' ')[0] == old else value
                for value in row.get('dependencies', [])
            ]
        rows.append(json.dumps(row, sort_keys=True))
    return sorted(rows)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('proof-gate', 'compiler-receipt', 'runner-source', 'base-consumer', 'material',
                 'cargo-target', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
    args = parser.parse_args()
    require(not git('status', '--porcelain'), 'source checkout must be clean')
    head, tree = git('rev-parse', 'HEAD'), git('rev-parse', 'HEAD^{tree}')
    require(git('log', '-1', '--format=%G?') == 'G' and
            'Signed-off-by:' in git('log', '-1', '--format=%B'),
            'funded source head must be signed and DCO complete')
    require(subprocess.run(['git', 'merge-base', '--is-ancestor', PROOF_HEAD, head],
                           cwd=ROOT).returncode == 0, 'funded source not stacked on proof')
    changed = set(git('diff', '--name-only', PROOF_HEAD, head).splitlines())
    require(changed == {
        'runtime-rs/tests/fixtures/ledger9-bytes-map-transformation-funded.rs',
        'tools/compact-rust-backend/check_upstream_ledger9_bytes_map_funded.py',
    }, 'funded source changed inputs outside this qualified transaction gate')
    gate = args.proof_gate.resolve(strict=True)
    base = args.base_consumer.resolve(strict=True)
    material = args.material.resolve(strict=True)
    target = args.cargo_target.resolve()
    output = args.output.resolve()
    require(not args.output.is_symlink() and not output.exists(), 'output must be absent')
    receipt = gate / 'receipt.json'
    proof = json.loads(receipt.read_text())
    require(proof['head'] == PROOF_HEAD and
            proof['tree'] == git('rev-parse', f'{PROOF_HEAD}^{{tree}}') and
            proof['upstream'] == '672bd0f226f47a7d036c5c074ce71fbe1af1925e' and
            proof['base_qualification_receipt_sha256'] == BASE_QUALIFICATION_RECEIPT_SHA256 and
            proof['source_sha256'] == ORIGINAL_SOURCE_SHA256 and
            proof['artifacts']['transformation/zkir'] == ORIGINAL_ZKIR_SHA256 and
            proof['gate_source_sha256'] == PROOF_GATE_SOURCE_SHA256 == sha(
                ROOT / 'tools/compact-rust-backend/check_upstream_ledger9_bytes_map_proof.py') and
            proof['capture_source_sha256'] == CAPTURE_SOURCE_SHA256 == sha(
                ROOT / 'runtime-rs/tests/fixtures/capture-ledger9-bytes-map-transformation-proof.mjs') and
            all(proof['artifacts'].get(key) == digest
                for key, digest in GENERATED_SHA256.items()) and
            proof['capture']['checks'] == [
                'honest accepted', 'changed stored byte refused',
                'changed insert count refused'],
            'proof receipt source binding changed')
    compiler_receipt = args.compiler_receipt.resolve(strict=True)
    require(sha(compiler_receipt) == proof['compiler_receipt_sha256'],
            'proof compiler recipe receipt changed')
    recipe = json.loads(compiler_receipt.read_text())
    compiler = Path(recipe['compactc']['path']).resolve(strict=True)
    scheme = Path(recipe['compactc-scheme']['path']).resolve(strict=True)
    compiler_recipe(compiler_receipt, compiler, scheme,
                    source_head=PROOF_HEAD, current_head=PROOF_HEAD)
    require(sha(compiler) == proof['compiler_sha256'] and
            sha(scheme) == proof['scheme_sha256'],
            'proof compiler binaries changed')
    for relative, expected in (
        ('fresh-dual/contract/index.js', proof['artifacts']['transformation/typescript']),
        ('fresh-dual/contract/lib.rs', proof['artifacts']['transformation/rust']),
        ('fresh-dual/contract/compact-rust-ir.json',
         proof['artifacts']['transformation/rust_ir']),
        ('fresh-dual/zkir/bytes_map_transformation.zkir',
         proof['artifacts']['transformation/zkir']),
        ('preimage/honest.preimage', proof['capture']['honestPreimageSha256']),
        ('proof/statement.proof', proof['proof_sha256']),
        ('proof/statement.prover', proof['prover_sha256']),
        ('proof/statement.verifier', proof['verifier_sha256']),
        ('wrong-key/slice_composition.verifier', proof['wrong_verifier_sha256']),
    ):
        require(sha(gate / relative) == expected, f'proof artifact changed: {relative}')
    output.mkdir(parents=True)
    runner_source = args.runner_source.resolve(strict=True)
    require({file.relative_to(runner_source).as_posix() for file in runner_source.rglob('*')
             if file.is_file()} == set(RUNNER_SOURCE_SHA256) and
            all(sha(runner_source / name) == digest for name, digest in
                RUNNER_SOURCE_SHA256.items()) and
            proof['runner_source_sha256'] == RUNNER_SOURCE_SHA256,
            'standalone verifier source changed')
    runner_manifest = tomllib.loads((runner_source / 'Cargo.toml').read_text())
    require(runner_manifest['package']['name'] == 'secp256k1-unary-proof' and
            not any(isinstance(dep, dict) and 'path' in dep
                    for dep in runner_manifest['dependencies'].values()),
            'standalone verifier manifest changed')
    rustc = subprocess.check_output(['rustc', '+1.99.0', '-vV'], text=True)
    require('release: 1.99.0' in rustc.splitlines() and
            'commit-hash: b940084d7eb6a299eb4bfeb8e34901bc051e7ac4'
            in rustc.splitlines(), 'standalone verifier toolchain changed')
    def no_cargo_config(directory: Path) -> None:
        names = ('config', 'config.toml') if directory == Path.home() / '.cargo' else (
            '.cargo/config', '.cargo/config.toml')
        require(not any((directory / name).exists() for name in names),
                f'unpinned Cargo configuration at {directory}')

    for directory in (runner_source, *runner_source.parents, Path.home() / '.cargo'):
        no_cargo_config(directory)
    for directory in runner_source.parents:
        parent_manifest = directory / 'Cargo.toml'
        require(not (parent_manifest.is_file() and
                     'workspace' in tomllib.loads(parent_manifest.read_text())),
                f'unpinned verifier parent workspace at {directory}')
    clean_env = {key: value for key, value in os.environ.items()
                 if not (key.startswith('CARGO_') or key in
                         {'RUSTFLAGS', 'RUSTDOCFLAGS', 'RUSTC', 'RUSTC_WRAPPER',
                          'RUSTC_WORKSPACE_WRAPPER', 'RUSTUP_TOOLCHAIN',
                          'CC', 'CXX', 'LD', 'AR', 'CFLAGS', 'CXXFLAGS', 'LDFLAGS'})}
    runner_env = clean_env.copy()
    runner_env['CARGO_TARGET_DIR'] = str(output / 'standalone-verifier-target')
    run(['cargo', '+1.99.0', 'build', '--locked', '--offline', '--bin',
         'secp256k1-unary-proof', '--manifest-path',
         str(runner_source / 'Cargo.toml')],
        output / 'standalone-verifier-build.log', env=runner_env)
    runner = output / 'standalone-verifier-target/debug/secp256k1-unary-proof'
    require(runner.is_file(), 'source-built standalone verifier missing')
    verifier_log = output / 'standalone-verification.log'
    run([str(runner), 'verify',
         str(gate / 'fresh-dual/zkir/bytes_map_transformation.zkir'),
         str(gate / 'preimage/honest.preimage'), str(gate / 'proof'),
         str(gate / 'wrong-key/slice_composition.verifier')], verifier_log)
    require('verified in separate process; rejected different verifier, changed binding and corrupt proof'
            in verifier_log.read_text(), 'standalone proof or refusals did not verify')
    canonical, runtime_checkout = generated_manifest(
        gate / 'fresh-dual/contract/Cargo.toml',
        expected_canonical_sha256=GENERATED_MANIFEST_CANONICAL_SHA256,
        source_head=PROOF_HEAD)
    require(sha(base / 'Cargo.toml') == BASE_MANIFEST_SHA256 and
            sha(base / 'Cargo.lock') == BASE_LOCK_SHA256,
            'funded consumer template changed')
    material_receipt = material.parent / 'material-receipt.json'
    require(sha(material_receipt) == MATERIAL_RECEIPT_SHA256,
            'SDK material receipt changed')
    material_files = json.loads(material_receipt.read_text())['files']
    require(all(sha(material / name) == row['sha256'] and
                (material / name).stat().st_size == row['bytes']
                for name, row in material_files.items()), 'SDK material changed')
    require(proof['params_k9_sha256'] == material_files['bls_midnight_2p9']['sha256'],
            'proof parameters differ from funded SDK material')
    consumer = output / 'consumer'
    consumer.mkdir()
    template = (base / 'Cargo.toml').read_text()
    old = [line for line in template.splitlines() if line.startswith('listcontract = ')]
    require(len(old) == 1 and
            template.count('ledger9-generated-list-funded') == 2,
            'funded consumer template shape changed')
    new = ('bytescontract = { package = "compact-contract-bytes-map-transformation", '
           f'path = "{gate / "fresh-dual/contract"}", '
           'features = ["ledger-transaction"] }')
    manifest = template.replace(old[0], new).replace(
        'ledger9-generated-list-funded', 'ledger9-bytes-map-transformation-funded')
    (consumer / 'Cargo.toml').write_text(manifest)
    shutil.copyfile(base / 'Cargo.lock', consumer / 'Cargo.lock')
    shutil.copyfile(SOURCE, consumer / 'main.rs')
    generated_contract = gate / 'fresh-dual/contract'
    runtime_crate = runtime_checkout / 'runtime-rs'
    for directory in (consumer, *consumer.parents,
                      generated_contract, *generated_contract.parents,
                      runtime_crate, *runtime_crate.parents,
                      Path.home() / '.cargo'):
        no_cargo_config(directory)
    build_env = clean_env.copy()
    build_env['CARGO_TARGET_DIR'] = str(target)
    run(['cargo', '+1.99.0', 'build', '--offline',
         '--manifest-path', str(consumer / 'Cargo.toml')],
        output / 'funded-build.log', env=build_env)
    require(normalized_lock(base / 'Cargo.lock', 'ledger9-generated-list-funded',
                            'compact-contract-list-field') ==
            normalized_lock(consumer / 'Cargo.lock',
                            'ledger9-bytes-map-transformation-funded',
                            'compact-contract-bytes-map-transformation'),
            'funded consumer changed pinned lock packages')
    run(['cargo', '+1.99.0', 'build', '--locked', '--offline',
         '--manifest-path', str(consumer / 'Cargo.toml')],
        output / 'funded-locked-build.log', env=build_env)
    executable = target / 'debug/ledger9-bytes-map-transformation-funded'
    require(executable.is_file() and sha(consumer / 'main.rs') == sha(SOURCE),
            'source-built funded consumer missing or changed')
    funded_env = build_env.copy()
    funded_env.update(MIDNIGHT_PP=str(material), MIDNIGHT_LEDGER_TEST_STATIC_DIR=str(material),
                      RAYON_NUM_THREADS='4')
    results = output / 'funded-result'
    with (output / 'funded-run.json').open('xb') as stdout, (
            output / 'funded-run.stderr').open('xb') as stderr:
        subprocess.run([str(executable), str(gate / 'fresh-dual'),
                        str(gate / 'preimage'), str(gate / 'proof'),
                        str(gate / 'wrong-key/slice_composition.verifier'), str(results)],
                       cwd=ROOT, env=funded_env, stdout=stdout, stderr=stderr, check=True)
    result = json.loads((output / 'funded-run.json').read_text())
    require(result['status'] == 'passed' and
            result['circuit'] == 'bytes_map_transformation' and result['k'] == 9 and
            result['positive_fee_dust_spends'] == 1 and
            result['replay_refusal'] == 'IntentAlreadyExists',
            'funded Bytes map result changed')
    for key in ('default_strict', 'ledger_matches_native',
                'typescript_preimage_fields_match',
                'communication_commitment_recomputed', 'wrong_verifier_refused',
                'replay_unchanged', 'transaction_proof_distinct'):
        require(result[key] is True, f'funded gate check missing: {key}')
    require(result['strict_recheck_refused'] is False and
            (results / 'ledger-after.bin').read_bytes() ==
            (results / 'ledger-replay.bin').read_bytes() and
            sha(results / 'transaction-bound.proof') !=
            sha(results / 'standalone.proof.raw'),
            'default-strict replay or proof distinction changed')
    require(not git('status', '--porcelain') and git('rev-parse', 'HEAD') == head,
            'source checkout changed during funded gate')
    sealed = {
        'head': head, 'tree': tree, 'platform': f'{platform.system()} {platform.machine()}',
        'proof_head': PROOF_HEAD, 'proof_receipt_sha256': sha(receipt),
        'funded_source_sha256': sha(SOURCE), 'gate_source_sha256': sha(Path(__file__)),
        'generated_manifest_canonical_sha256': canonical,
        'generated_runtime_checkout_head': git('-C', str(runtime_checkout), 'rev-parse', 'HEAD'),
        'material_receipt_sha256': sha(material_receipt),
        'funded_manifest_sha256': sha(consumer / 'Cargo.toml'),
        'funded_lock_sha256': sha(consumer / 'Cargo.lock'),
        'funded_binary_sha256': sha(executable),
        'standalone_verification_log_sha256': sha(verifier_log),
        'standalone_verifier_build_log_sha256': sha(output / 'standalone-verifier-build.log'),
        'standalone_verifier_binary_sha256': sha(runner),
        'standalone_verifier_source_sha256': RUNNER_SOURCE_SHA256,
        'funded_result': result,
        'applied_transaction_sha256': sha(results / 'applied-transaction.bin'),
        'transaction_bound_proof_sha256': sha(results / 'transaction-bound.proof'),
        'standalone_proof_raw_sha256': sha(results / 'standalone.proof.raw'),
        'ledger_after_sha256': sha(results / 'ledger-after.bin'),
        'ledger_replay_sha256': sha(results / 'ledger-replay.bin'),
        'limitations': ['Local Darwin arm64 qualification; native Linux remains required.',
                        'One generated contract call; real two-contract transaction remains open.',
                        'Deployment inserted into the local test ledger state.',
                        'Exact replay is refused by ledger application after a default-strict recheck succeeds.'],
    }
    (output / 'receipt.json').write_text(json.dumps(sealed, indent=2, sort_keys=True) + '\n')
    print(f'funded original Bytes map strict application and replay refusal passed: {output}')


if __name__ == '__main__':
    main()
