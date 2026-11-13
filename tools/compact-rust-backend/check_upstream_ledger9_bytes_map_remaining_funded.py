#!/usr/bin/env python3

# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0

"""Fund the three remaining original Bytes map exports under Ledger 9 strictness."""

import argparse
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tomllib

from collection_gate_binding import compiler_recipe, generated_manifest
from check_upstream_ledger9_bytes_map_funded import normalized_lock, sha, require, git, run
from check_upstream_ledger9_bytes_map_remaining_proofs import (
    CASES, CAPTURE, MATERIAL_RECEIPT_SHA256, PARAMS, ROOT,
)
from check_upstream_ledger9_bytes_map_proof import RUNNER_SOURCE_SHA256


PROOF_HEAD = '3c67f1e2630ab3a931f5acaa6599b804c9be11ac'
BASE_QUALIFICATION_RECEIPT_SHA256 = '34bee302b30f1d87a4e04e3904891a452865d36f20ddd61613e39a36a02b4426'
BASE_MANIFEST_SHA256 = 'f87c1f43ac0e78ec220afe8ccda2d06579d3f2dcf21c36f5ae23699943a078cf'
BASE_LOCK_SHA256 = '2f156d51adbeca7dcd87fa0cd28d3ec0f76cfd4700cd2e0f3f56157b9560fadc'
TEMPLATE = ROOT / 'runtime-rs/tests/fixtures/ledger9-bytes-map-remaining-funded.rs.in'
CANONICAL_MANIFEST_SHA256 = {
    'reordering': '5cd370a8177df6bdad5a08b87f7885c3df26edb9ca352738bfb73459852b7223',
    'slice_composition': '15da7ce6b271af0b277c000483a48265ba33b6f7f25d68e87eacddcf2ac5a452',
    'to_bytes': '1cc3af329cd85d643af28ca0b0f460057de86dc9d517ae1371a754574e2f0822',
}
EXPECTED_RUST = {
    'reordering': '[[3u8, 2], [5, 4], [7, 6]]',
    'slice_composition': '[[20u8], [40], [60]]',
    'to_bytes': '[[0u8], [1], [2], [3], [4], [5], [6], [7]]',
}


def clean_cargo_environment() -> dict[str, str]:
    return {key: value for key, value in os.environ.items()
            if not (key.startswith('CARGO_') or key in
                    {'RUSTFLAGS', 'RUSTDOCFLAGS', 'RUSTC', 'RUSTC_WRAPPER',
                     'RUSTC_WORKSPACE_WRAPPER', 'RUSTUP_TOOLCHAIN',
                     'CC', 'CXX', 'LD', 'AR', 'CFLAGS', 'CXXFLAGS', 'LDFLAGS'})}


def no_cargo_config(directory: Path) -> None:
    names = ('config', 'config.toml') if directory == Path.home() / '.cargo' else (
        '.cargo/config', '.cargo/config.toml')
    require(not any((directory / name).exists() for name in names),
            f'unpinned Cargo configuration at {directory}')


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('proof-gate', 'compiler-receipt', 'runner-source',
                 'base-consumer', 'material', 'output'):
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
        'runtime-rs/tests/fixtures/ledger9-bytes-map-remaining-funded.rs.in',
        'tools/compact-rust-backend/check_upstream_ledger9_bytes_map_remaining_funded.py',
    }, 'funded source changed inputs outside this qualified transaction gate')
    gate = args.proof_gate.resolve(strict=True)
    base = args.base_consumer.resolve(strict=True)
    material = args.material.resolve(strict=True)
    output = args.output.resolve()
    require(not args.output.is_symlink() and not output.exists(), 'output must be absent')
    proof_receipt_path = gate / 'receipt.json'
    proof = json.loads(proof_receipt_path.read_text())
    require(proof['head'] == PROOF_HEAD and
            proof['tree'] == git('rev-parse', f'{PROOF_HEAD}^{{tree}}') and
            proof['upstream'] == '672bd0f226f47a7d036c5c074ce71fbe1af1925e' and
            proof['base_receipt_sha256'] == BASE_QUALIFICATION_RECEIPT_SHA256 and
            proof['original_cases'] == CASES and
            proof['capture_source_sha256'] == sha(CAPTURE) and
            proof['gate_source_sha256'] == sha(
                ROOT / 'tools/compact-rust-backend/check_upstream_ledger9_bytes_map_remaining_proofs.py'),
            'proof receipt source and original-case bindings changed')
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
    material_receipt = material.parent / 'material-receipt.json'
    require(sha(material_receipt) == MATERIAL_RECEIPT_SHA256 ==
            proof['material_receipt_sha256'], 'SDK material receipt changed')
    material_files = json.loads(material_receipt.read_text())['files']
    require(all(sha(material / name) == row['sha256'] and
                (material / name).stat().st_size == row['bytes']
                for name, row in material_files.items()), 'SDK material changed')
    require(all(proof['params_sha256'][str(k)] == digest ==
                material_files[f'bls_midnight_2p{k}']['sha256']
                for k, digest in PARAMS.items()),
            'proof parameters differ from funded SDK material')
    require(sha(base / 'Cargo.toml') == BASE_MANIFEST_SHA256 and
            sha(base / 'Cargo.lock') == BASE_LOCK_SHA256,
            'funded consumer template changed')
    template = TEMPLATE.read_text()
    require(template.count('__CIRCUIT__') == 7 and
            template.count('__EXPECTED__') == 1 and template.count('__K__') == 1,
            'funded Rust template shape changed')
    base_manifest = (base / 'Cargo.toml').read_text()
    old_dependency = [line for line in base_manifest.splitlines()
                      if line.startswith('listcontract = ')]
    require(len(old_dependency) == 1 and
            base_manifest.count('ledger9-generated-list-funded') == 2,
            'funded Cargo template shape changed')

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
    for directory in (runner_source, *runner_source.parents, Path.home() / '.cargo'):
        no_cargo_config(directory)
    for directory in runner_source.parents:
        parent_manifest = directory / 'Cargo.toml'
        require(not (parent_manifest.is_file() and
                     'workspace' in tomllib.loads(parent_manifest.read_text())),
                f'unpinned verifier parent workspace at {directory}')
    clean_env = clean_cargo_environment()
    runner_env = clean_env.copy()
    runner_env['CARGO_TARGET_DIR'] = str(output / 'standalone-verifier-target')
    run(['cargo', '+1.99.0', 'build', '--locked', '--offline', '--bin',
         'secp256k1-unary-proof', '--manifest-path',
         str(runner_source / 'Cargo.toml')],
        output / 'standalone-verifier-build.log', env=runner_env)
    runner = output / 'standalone-verifier-target/debug/secp256k1-unary-proof'
    require(runner.is_file(), 'source-built standalone verifier missing')
    target = output / 'funded-target'
    funded = {}
    for case in EXPECTED_RUST:
        pins = CASES[case]
        circuit = f'bytes_map_{case}'
        qualified = proof['qualified'][case]
        require(qualified['k'] == pins['k'] and
                qualified['wrong_case'] == pins['wrong'] and
                qualified['capture']['rawZkirSha256'] == pins['zkir'] and
                qualified['capture']['generatedTypeScriptSha256'] == pins['typescript'] and
                qualified['capture']['checks'] == ['honest accepted',
                    'changed stored byte refused', 'changed insert count refused'],
                f'{case} proof receipt checks changed')
        for relative, digest in (
            (f'fresh-dual/{case}/contract/index.js', pins['typescript']),
            (f'fresh-dual/{case}/contract/lib.rs', pins['rust']),
            (f'fresh-dual/{case}/contract/compact-rust-ir.json', pins['rust_ir']),
            (f'fresh-dual/{case}/zkir/{circuit}.zkir', pins['zkir']),
            (f'preimage/{case}/honest.preimage',
             qualified['capture']['honestPreimageSha256']),
            (f'proof/{case}/statement.proof', qualified['proof_sha256']),
            (f'proof/{case}/statement.prover', qualified['prover_sha256']),
            (f'proof/{case}/statement.verifier', qualified['verifier_sha256']),
            (f'keys/{pins["wrong"]}.verifier', qualified['wrong_verifier_sha256']),
        ):
            require(sha(gate / relative) == digest,
                    f'{case} proof artifact changed: {relative}')
        verify_log = output / f'{case}-standalone-verification.log'
        run([str(runner), 'verify',
             str(gate / 'fresh-dual' / case / 'zkir' / f'{circuit}.zkir'),
             str(gate / 'preimage' / case / 'honest.preimage'),
             str(gate / 'proof' / case),
             str(gate / 'keys' / f'{pins["wrong"]}.verifier')], verify_log)
        require('verified in separate process; rejected different verifier, changed binding and corrupt proof'
                in verify_log.read_text(), f'{case} standalone verifier controls failed')
        generated_manifest_path = gate / 'fresh-dual' / case / 'contract/Cargo.toml'
        canonical, runtime_checkout = generated_manifest(
            generated_manifest_path,
            expected_canonical_sha256=CANONICAL_MANIFEST_SHA256[case],
            source_head=PROOF_HEAD)
        contract_package = tomllib.loads(generated_manifest_path.read_text())['package']['name']
        package_label = case.replace('_', '-')
        require(contract_package == f'compact-contract-bytes-map-{package_label}',
                f'{case} generated package name changed')
        consumer = output / 'consumer' / case
        consumer.mkdir(parents=True)
        root_name = f'ledger9-bytes-map-{package_label}-funded'
        dependency = (f'bytescontract = {{ package = "{contract_package}", '
                      f'path = "{generated_manifest_path.parent}", '
                      'features = ["ledger-transaction"] }')
        manifest = base_manifest.replace(old_dependency[0], dependency).replace(
            'ledger9-generated-list-funded', root_name)
        (consumer / 'Cargo.toml').write_text(manifest)
        shutil.copyfile(base / 'Cargo.lock', consumer / 'Cargo.lock')
        source = (template.replace('__CIRCUIT__', circuit)
                  .replace('__EXPECTED__', EXPECTED_RUST[case])
                  .replace('__K__', str(pins['k'])))
        require('__CIRCUIT__' not in source and '__EXPECTED__' not in source and
                '__K__' not in source, f'{case} Rust template substitution incomplete')
        (consumer / 'main.rs').write_text(source)
        for directory in (consumer, *consumer.parents,
                          generated_manifest_path.parent, *generated_manifest_path.parent.parents,
                          runtime_checkout / 'runtime-rs', *(runtime_checkout / 'runtime-rs').parents,
                          Path.home() / '.cargo'):
            no_cargo_config(directory)
        build_env = clean_env.copy()
        build_env['CARGO_TARGET_DIR'] = str(target)
        run(['cargo', '+1.99.0', 'build', '--offline',
             '--manifest-path', str(consumer / 'Cargo.toml')],
            output / f'{case}-funded-build.log', env=build_env)
        require(normalized_lock(base / 'Cargo.lock', 'ledger9-generated-list-funded',
                                'compact-contract-list-field') ==
                normalized_lock(consumer / 'Cargo.lock', root_name, contract_package),
                f'{case} funded consumer changed pinned lock packages')
        run(['cargo', '+1.99.0', 'build', '--locked', '--offline',
             '--manifest-path', str(consumer / 'Cargo.toml')],
            output / f'{case}-funded-locked-build.log', env=build_env)
        executable = target / 'debug' / root_name
        require(executable.is_file(), f'{case} source-built consumer missing')
        funded_env = build_env.copy()
        funded_env.update(MIDNIGHT_PP=str(material),
                          MIDNIGHT_LEDGER_TEST_STATIC_DIR=str(material),
                          RAYON_NUM_THREADS='4')
        results = output / 'funded-result' / case
        results.parent.mkdir(exist_ok=True)
        with (output / f'{case}-funded-run.json').open('xb') as stdout, (
                output / f'{case}-funded-run.stderr').open('xb') as stderr:
            subprocess.run([str(executable), str(gate / 'fresh-dual' / case),
                            str(gate / 'preimage' / case), str(gate / 'proof' / case),
                            str(gate / 'keys' / f'{pins["wrong"]}.verifier'), str(results)],
                           cwd=ROOT, env=funded_env, stdout=stdout,
                           stderr=stderr, check=True)
        result = json.loads((output / f'{case}-funded-run.json').read_text())
        require(result['status'] == 'passed' and result['circuit'] == circuit and
                result['k'] == pins['k'] and result['positive_fee_dust_spends'] == 1 and
                result['replay_refusal'] == 'IntentAlreadyExists' and
                result['strict_recheck_refused'] is False,
                f'{case} funded result changed')
        for key in ('default_strict', 'ledger_matches_native',
                    'typescript_preimage_fields_match',
                    'communication_commitment_recomputed', 'wrong_verifier_refused',
                    'replay_unchanged', 'transaction_proof_distinct'):
            require(result[key] is True, f'{case} funded check missing: {key}')
        require((results / 'ledger-after.bin').read_bytes() ==
                (results / 'ledger-replay.bin').read_bytes() and
                sha(results / 'transaction-bound.proof') !=
                sha(results / 'standalone.proof.raw'),
                f'{case} replay changed ledger or reused standalone proof')
        funded[case] = {
            'k': pins['k'], 'result': result,
            'generated_manifest_sha256': sha(generated_manifest_path),
            'generated_manifest_canonical_sha256': canonical,
            'runtime_checkout_head': git('-C', str(runtime_checkout), 'rev-parse', 'HEAD'),
            'consumer_source_sha256': sha(consumer / 'main.rs'),
            'consumer_manifest_sha256': sha(consumer / 'Cargo.toml'),
            'consumer_lock_sha256': sha(consumer / 'Cargo.lock'),
            'consumer_binary_sha256': sha(executable),
            'standalone_verification_log_sha256': sha(verify_log),
            'transaction_sha256': sha(results / 'applied-transaction.bin'),
            'transaction_proof_sha256': sha(results / 'transaction-bound.proof'),
            'ledger_after_sha256': sha(results / 'ledger-after.bin'),
            'ledger_replay_sha256': sha(results / 'ledger-replay.bin'),
        }
    require(not git('status', '--porcelain') and git('rev-parse', 'HEAD') == head,
            'source checkout changed during funded qualification')
    receipt = {
        'head': head, 'tree': tree, 'proof_head': PROOF_HEAD,
        'platform': f'{platform.system()} {platform.machine()}',
        'proof_receipt_sha256': sha(proof_receipt_path),
        'compiler_receipt_sha256': sha(compiler_receipt),
        'material_receipt_sha256': sha(material_receipt),
        'funded_template_sha256': sha(TEMPLATE),
        'funded_gate_source_sha256': sha(Path(__file__)),
        'base_manifest_sha256': sha(base / 'Cargo.toml'),
        'base_lock_sha256': sha(base / 'Cargo.lock'),
        'standalone_verifier_source_sha256': RUNNER_SOURCE_SHA256,
        'standalone_verifier_binary_sha256': sha(runner),
        'standalone_verifier_build_log_sha256': sha(output / 'standalone-verifier-build.log'),
        'funded': funded,
        'checks': ['three source-built generated Rust recorded calls match native state and TypeScript preimages',
                   'three positive-fee Dust funded default-strict Ledger 9 applications',
                   'three IntentAlreadyExists exact replay refusals with byte-identical ledger state',
                   'standalone proofs verified by rebuilt independent verifier'],
        'limitations': ['Local platform only; native Linux remains required.',
                        'Each transaction calls one contract; the real two-contract transaction remains open.',
                        'Each deployment is inserted into local ledger test state.',
                        'Default-strict recheck succeeds; replay refusal occurs at ledger application.'],
    }
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(f'funded remaining Bytes map strict application and replay refusals passed: {output}')


if __name__ == '__main__':
    main()
