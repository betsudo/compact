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

from collection_gate_binding import compiler_recipe, generated_manifest
from check_upstream_ledger9_bytes_map_funded import (
    RUNNER_SOURCE_SHA256, require_receipt_pin, source_bound_to_receipt,
)
from check_upstream_ledger9_mixed_original_preflight import (
    CASES as ORIGINAL_CASES, HISTORICAL_RUNTIME_BASE_SHA256, UPSTREAM,
    original_cases, validate_runtime_baseline,
)


ROOT = Path(__file__).resolve().parents[2]
PROOF_HEAD = '7512d05cb5c10e58adc429fc9fa28bc76eedb951'
PROOF_RECEIPT_SHA = '80117b1c8fb2710ae9fc7636ebb7cc406d08518c0639c033c098d847c7c3a9b3'
PREFLIGHT_RECEIPT_SHA = '55f122e25110b8cee6e247777665d173bc3b2f934bdd308792cca9c929cdc254'
BASE_MANIFEST_SHA = 'c247f5ee7c3bc5ada73417b71c9606b0132e6a9992c7efb84d4dff547b131b33'
BASE_LOCK_SHA = 'd77e63ce87efc463e04d7732ed64cd40d710a4765f6c40ec756211de18a469cd'
MATERIAL_RECEIPT_SHA = '56a004076c7332845fb4cd61ef357892db6dcd7dd05c03f9efebfb43f8df4df7'
GENERATED_MANIFEST_SHA = {'k1': 'f60b95076b029426c978983a2ea74ac35956679360e81c1cc314f0d0ae1a5935',
                          'r1': '95d46740e949bd20dc306fe6d80e0dafa18c5f7972dd8d4183437b3b389ff517'}
GENERATED_MANIFEST_CANONICAL_SHA = {
    'k1': '83636a1cba4d5e897b5656c020f1b4e8fa3b3db0dc15d741cb4967325674cf30',
    'r1': 'eda6623072a80e2663b994a2d7b931a07c1ff196c702362de9df6e4def7ca7e0',
}
AGGREGATE_CONTRACT_EXTRA_SHA = {
    'k1': {
        'index.d.ts': '06587ff29585bc9e7c84e8a1ea6f0bcd60cada251fe3016e00276dd44981be62',
        'index.js.map': '904d3e7284bd6ed94c3fc2064acd204561745461eb719727ef1c485633c5b8f8',
    },
    'r1': {
        'index.d.ts': '250a541fc9281f0b50e0d466d7dd4bbb1fcfc51192fa1907d473022e6972d1c5',
        'index.js.map': '312f0de4f246b2872d71f98bf480a4844eabd4e556e0557aa982c3de7080b4d7',
    },
}
AGGREGATE_ORIGINS_CANONICAL_SHA = {
    'k1': '9ec4ca9685aa74adf06e31ca51da37b937b85bb8358c3c3f1765104e8013ad58',
    'r1': 'ae3314a75fa953b1abc3da99ecc2288d6026cbc12c6260c6875c84391c816401',
}
AGGREGATE_TEMPLATE_SHA = '0fc67ef4f13a5b836634aa6f0f7d7aad53d75281824155b839ddc39472948f87'
PREFLIGHT_CAPTURE_SHA = 'f6ad89f624ee0ddb307e226ff4c8efd52d53def41083701366322ca8e5f22870'
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


def snapshot_inputs(files: list[Path], directories: list[Path]) -> dict[str, str]:
    """Fingerprint admitted external inputs, including directory membership."""
    paths = set(files)
    membership = {}
    links = {}
    for directory in directories:
        require(directory.is_dir() and not directory.is_symlink(),
                f'mixed aggregate input directory changed: {directory}')
        members = []
        for path in directory.rglob('*'):
            if path.is_symlink():
                target = os.readlink(path)
                links[str(path) + '@link'] = hashlib.sha256(target.encode()).hexdigest()
                members.append(path.relative_to(directory).as_posix() + '@link')
            elif path.is_file():
                paths.add(path)
                members.append(path.relative_to(directory).as_posix())
        membership[str(directory) + '/'] = hashlib.sha256(
            '\n'.join(sorted(members)).encode()).hexdigest()
    require(all(path.is_file() and not path.is_symlink() for path in paths),
            'mixed aggregate input missing or replaced')
    return {**membership, **links,
            **{str(path): sha(path) for path in sorted(paths)}}


def require_final_binding(head: str, tree: str, admitted: dict[str, str],
                          files: list[Path], directories: list[Path],
                          copies: dict[str, str], copy_directories: list[Path]) -> None:
    require(not git('status', '--porcelain') and git('rev-parse', 'HEAD') == head and
            git('rev-parse', 'HEAD^{tree}') == tree,
            'mixed aggregate funded source changed during qualification')
    require(snapshot_inputs(files, directories) == admitted,
            'mixed aggregate funded input changed during qualification')
    require(snapshot_inputs([], copy_directories) == copies,
            'mixed aggregate funded generated copy changed during qualification')
    require(not git('status', '--porcelain') and git('rev-parse', 'HEAD') == head and
            git('rev-parse', 'HEAD^{tree}') == tree,
            'mixed aggregate funded source changed during final input check')


def run(command, log, *, env=None, cwd=ROOT):
    result = subprocess.run(command, cwd=cwd, env=env, text=True,
                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    log.write_text(result.stdout)
    require(result.returncode == 0, f'{command[0]} failed; inspect {log}')
    return result.stdout


def packages(lock):
    data = tomllib.loads(Path(lock).read_text())
    return sorted((row['name'], row['version'], row.get('checksum'))
                  for row in data['package'] if row.get('source'))


def validate_aggregate_contract_files(contract: Path, case: str) -> None:
    expected = {'Cargo.toml', 'compact-rust-ir.json', 'compact-rust-origins.raw.json',
                'index.d.ts', 'index.js', 'index.js.map', 'lib.rs',
                'rust-capabilities.json'}
    actual = {path.relative_to(contract).as_posix() for path in contract.rglob('*')
              if path.is_file() or path.is_symlink()}
    require(actual == expected and all((contract / name).is_file() and
                                       not (contract / name).is_symlink()
                                       for name in expected),
            f'{case} generated contract has unpinned or missing files')
    require(all(sha(contract / name) == digest for name, digest in
                AGGREGATE_CONTRACT_EXTRA_SHA[case].items()),
            f'{case} generated contract auxiliary artifact changed')
    origins = json.loads((contract / 'compact-rust-origins.raw.json').read_text())
    files = origins.get('files')
    original = (contract.parents[1] / 'original' / f'{case}.compact').resolve(strict=True)
    require(isinstance(files, list) and len(files) == 2 and
            files[0] == {'id': 0, 'path': str(original)} and
            files[1] == {'id': 1, 'path': 'compiler/standard-library.compact'},
            f'{case} generated source origin does not name the qualified original')
    origins['files'][0]['path'] = '<original source>'
    canonical = json.dumps(origins, sort_keys=True, separators=(',', ':')).encode()
    require(hashlib.sha256(canonical).hexdigest() ==
            AGGREGATE_ORIGINS_CANONICAL_SHA[case],
            f'{case} generated source origins changed beyond the qualified path')


def validate_proof_checker_source(proof: dict, supplied: Path | None = None) -> None:
    checker = ROOT / 'tools/compact-rust-backend/check_upstream_ledger9_mixed_original_proof.py'
    require(source_bound_to_receipt(proof.get('verifier_source_sha256'), checker),
            'mixed aggregate proof checker differs from current repository source')
    if supplied is not None:
        require(supplied.resolve(strict=True) == checker.resolve(strict=True),
                'mixed aggregate proof checker path differs from current repository source')


def validate_aggregate_receipts(*, proof_path: Path, preflight_path: Path,
                                compiler_path: Path, runtime_baseline_path: Path,
                                runtime_baseline_sha: str, proof_sha: str,
                                preflight_sha: str, compiler_sha: str,
                                head: str, tree: str) -> tuple[dict, dict]:
    """Admit only independent receipt pins for this clean signed source tree."""
    for path, digest, label in ((proof_path, proof_sha, 'mixed proof'),
                                (preflight_path, preflight_sha, 'mixed preflight'),
                                (compiler_path, compiler_sha, 'mixed compiler')):
        require_receipt_pin(path, digest, label)
    proof = json.loads(proof_path.read_text())
    prior = json.loads(preflight_path.read_text())
    require(proof.get('format') == 'ledger9-mixed-original-proofs/v1' and
            prior.get('format') == 'ledger9-mixed-original-preflight/v1' and
            proof.get('source_head') == prior.get('source_head') == head and
            proof.get('source_tree') == prior.get('source_tree') == tree and
            proof.get('preflight_receipt_sha256') == preflight_sha and
            proof.get('compiler_receipt_sha256') ==
            prior.get('compiler_receipt_sha256') == compiler_sha and
            prior.get('upstream') == UPSTREAM,
            'mixed aggregate receipts must bind the current head, tree, and each other')
    validate_proof_checker_source(proof)
    require(source_bound_to_receipt(
                prior.get('gate_source_sha256'),
                ROOT / 'tools/compact-rust-backend/check_upstream_ledger9_mixed_original_preflight.py') and
            prior.get('capture_source_sha256') == PREFLIGHT_CAPTURE_SHA == sha(
                ROOT / 'runtime-rs/tests/fixtures/capture-ledger9-mixed-original-proofs.mjs'),
            'mixed aggregate preflight producer or capture source changed')
    baseline = validate_runtime_baseline(
        runtime_baseline_path, runtime_baseline_sha, head=head, tree=tree,
        compiler_receipt=compiler_path, compiler_sha=prior.get('compiler_sha256'),
        runtime_store=Path(prior['runtime_store']), root=ROOT)
    require(prior.get('runtime_baseline_sha256') == runtime_baseline_sha and
            prior.get('runtime_manifest_sha256') ==
                baseline['typescript_runtime_package_sha256'] and
            prior.get('runtime_dist_files') ==
                baseline['typescript_runtime_dist_files'],
            'mixed aggregate preflight runtime differs from current-head baseline')
    extracted = original_cases()
    for case, pins in ORIGINAL_CASES.items():
        validate_aggregate_contract_files(preflight_path.parent / case / 'contract', case)
        body, callback = extracted[case]
        require(hashlib.sha256(body).hexdigest() == pins['source'] ==
                sha(preflight_path.parent / 'original' / f'{case}.compact') and
                hashlib.sha256(callback).hexdigest() == pins['callback'] ==
                sha(preflight_path.parent / 'original' / f'{case}.callback.js'),
                f'{case} original mixed source or callback changed')
        for relative, digest in pins['artifacts'].items():
            require(prior['artifacts'].get(f'{case}/{relative}') == digest ==
                    sha(preflight_path.parent / case / relative),
                    f'{case} generated mixed artifact changed: {relative}')
        capture = prior['captures'][case]
        require(capture == json.loads((preflight_path.parent /
                                      f'{case}-capture/capture.json').read_text()) and
                capture['checks'] == ['all original callback assertions and nested Map point',
                                      'honest ZKIR accepted', 'changed nested Map key refused'] and
                capture['honestPreimageSha256'] == pins['preimage'] ==
                sha(preflight_path.parent / f'{case}-capture/honest.preimage') and
                capture['changedKeyPreimageSha256'] == pins['changed_key_preimage'] ==
                sha(preflight_path.parent / f'{case}-capture/changed-key.preimage'),
                f'{case} original callback or refusal capture changed')
    return proof, prior


def rebound_runtime_manifest(manifest: str, runtime_path: str, *, aggregate: bool) -> str:
    rebound = manifest.replace(
        f'path = "{runtime_path}"', f'path = "{ROOT / "runtime-rs"}"')
    require((aggregate or rebound != manifest) and
            tomllib.loads(rebound)['dependencies']['midnight-compact-runtime']['path'] ==
            str(ROOT / 'runtime-rs'), 'generated runtime rebinding failed')
    return rebound


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for key in ('evidence', 'preflight', 'base-consumer', 'material',
                'wrong-verifiers', 'ledger-static', 'output'):
        parser.add_argument('--' + key, required=True, type=Path)
    parser.add_argument('--aggregate-head', action='store_true')
    for key in ('proof-receipt', 'compiler-receipt', 'proof-runner', 'proof-runner-source',
                'proof-runner-receipt', 'proof-verifier-source',
                'runtime-baseline-receipt'):
        parser.add_argument('--' + key, type=Path)
    for key in ('proof', 'preflight', 'compiler'):
        parser.add_argument('--aggregate-' + key + '-receipt-sha256')
    parser.add_argument('--aggregate-runtime-baseline-receipt-sha256')
    args = parser.parse_args()
    require(not git('status', '--porcelain'), 'funded source must be clean')
    head, tree = git('rev-parse', 'HEAD'), git('rev-parse', 'HEAD^{tree}')
    require(git('log', '-1', '--format=%G?') == 'G' and
            'Signed-off-by:' in git('log', '-1', '--format=%B'),
            'funded source must be signed and DCO complete')
    aggregate_inputs = (args.proof_receipt, args.compiler_receipt, args.proof_runner,
                        args.proof_runner_source,
                        args.proof_runner_receipt, args.proof_verifier_source,
                        args.runtime_baseline_receipt,
                        args.aggregate_proof_receipt_sha256,
                        args.aggregate_preflight_receipt_sha256,
                        args.aggregate_compiler_receipt_sha256,
                        args.aggregate_runtime_baseline_receipt_sha256)
    require(all(value is not None for value in aggregate_inputs) if args.aggregate_head
            else all(value is None for value in aggregate_inputs),
            'aggregate mode requires proof/compiler/runner/runtime baseline inputs and four receipt SHA-256 pins')
    proof_head = head if args.aggregate_head else PROOF_HEAD
    if args.aggregate_head:
        require(sha(TEMPLATE) == AGGREGATE_TEMPLATE_SHA,
                'aggregate funded Rust template changed')
    else:
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
    if args.aggregate_head:
        proof_path = args.proof_receipt.resolve(strict=True)
        preflight_path = preflight / 'receipt.json'
        compiler_path = args.compiler_receipt.resolve(strict=True)
        proof, prior = validate_aggregate_receipts(
            proof_path=proof_path, preflight_path=preflight_path,
            compiler_path=compiler_path,
            runtime_baseline_path=args.runtime_baseline_receipt.resolve(strict=True),
            runtime_baseline_sha=args.aggregate_runtime_baseline_receipt_sha256,
            proof_sha=args.aggregate_proof_receipt_sha256,
            preflight_sha=args.aggregate_preflight_receipt_sha256,
            compiler_sha=args.aggregate_compiler_receipt_sha256,
            head=head, tree=tree)
        recipe = json.loads(compiler_path.read_text())
        compiler = Path(recipe['compactc']['path']).resolve(strict=True)
        scheme = Path(recipe['compactc-scheme']['path']).resolve(strict=True)
        compiler_recipe(compiler_path, compiler, scheme,
                        source_head=head, current_head=head)
        require(prior['compiler_sha256'] == sha(compiler),
                'mixed aggregate preflight compiler binary changed')
        validate_proof_checker_source(proof, args.proof_verifier_source)
        generated_runner = args.proof_runner.resolve(strict=True)
        runner_source = args.proof_runner_source.resolve(strict=True)
        runner_receipt_path = args.proof_runner_receipt.resolve(strict=True)
        runner_receipt = json.loads(runner_receipt_path.read_text())
        require(proof['runner_source_receipt_sha256'] == sha(runner_receipt_path) and
                proof['runner_binary_sha256'] == sha(generated_runner) and
                runner_receipt['runner_source_sha256'] == RUNNER_SOURCE_SHA256 and
                {path.relative_to(runner_source).as_posix()
                 for path in runner_source.rglob('*') if path.is_file()} ==
                set(RUNNER_SOURCE_SHA256) and
                all(sha(runner_source / name) == digest
                    for name, digest in RUNNER_SOURCE_SHA256.items()),
                'mixed aggregate proof runner source or binary changed')
        runner_manifest = tomllib.loads((runner_source / 'Cargo.toml').read_text())
        require(runner_manifest['package']['name'] == 'secp256k1-unary-proof' and
                not any(isinstance(dependency, dict) and 'path' in dependency
                        for dependency in runner_manifest['dependencies'].values()),
                'mixed aggregate proof runner manifest changed')
        for directory in (runner_source, *runner_source.parents, Path.home() / '.cargo'):
            names = ('config', 'config.toml') if directory == Path.home() / '.cargo' else (
                '.cargo/config', '.cargo/config.toml')
            require(not any((directory / name).exists() for name in names),
                    f'unpinned mixed verifier Cargo configuration: {directory}')
        for directory in runner_source.parents:
            parent_manifest = directory / 'Cargo.toml'
            require(not (parent_manifest.is_file() and
                         'workspace' in tomllib.loads(parent_manifest.read_text())),
                    f'unpinned mixed verifier parent workspace: {directory}')
    else:
        proof_path = evidence / 'proof-receipt-7512.json'
        preflight_path = preflight / 'receipt.json'
        require(sha(proof_path) == PROOF_RECEIPT_SHA and
                sha(preflight_path) == PREFLIGHT_RECEIPT_SHA,
                'pinned proof or preflight receipt changed')
        proof = json.loads(proof_path.read_text())
        prior = json.loads(preflight_path.read_text())
        require(proof['source_head'] == PROOF_HEAD and
                proof['source_tree'] == git('rev-parse', f'{PROOF_HEAD}^{{tree}}') and
                proof['preflight_receipt_sha256'] == PREFLIGHT_RECEIPT_SHA and
                prior['source_head'] == PROOF_HEAD and
                prior['source_tree'] == proof['source_tree'] and
                prior['runtime_baseline_sha256'] == HISTORICAL_RUNTIME_BASE_SHA256 and
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
    admitted_files: list[Path] = []
    admitted_directories: list[Path] = []
    admitted_hashes: dict[str, str] = {}
    copied_hashes: dict[str, str] = {}
    copied_directories: list[Path] = []
    if args.aggregate_head:
        runtime_package = (Path(prior['runtime_store']) /
                           'lib/node_modules/@midnight-ntwrk/compact-runtime')
        admitted_files = [proof_path, compiler_path, runner_receipt_path,
                          args.runtime_baseline_receipt.resolve(strict=True),
                          compiler, scheme, generated_runner,
                          base / 'Cargo.toml', base / 'Cargo.lock',
                          material_receipt, TEMPLATE,
                          runtime_package / 'package.json',
                          args.runtime_baseline_receipt.resolve(strict=True).parent /
                          'typescript-runtime-recipe.log',
                          *(material / name for name in material_files)]
        admitted_directories = [preflight, evidence, wrong_root, runner_source,
                                runtime_package / 'dist']
        admitted_hashes = snapshot_inputs(admitted_files, admitted_directories)
    output.mkdir(parents=True)
    env = {key: value for key, value in os.environ.items()
           if not key.startswith('CARGO_') and key not in
           {'RUSTFLAGS', 'RUSTC_WRAPPER', 'RUSTUP_TOOLCHAIN'}}
    if args.aggregate_head:
        for key in ('RUSTC', 'RUSTC_WORKSPACE_WRAPPER', 'RUSTDOCFLAGS',
                    'CC', 'CXX', 'LD', 'AR', 'CFLAGS', 'CXXFLAGS', 'LDFLAGS'):
            env.pop(key, None)
        for directory in (ROOT, *ROOT.parents, output, *output.parents,
                          Path.home() / '.cargo'):
            names = ('config', 'config.toml') if directory == Path.home() / '.cargo' else (
                '.cargo/config', '.cargo/config.toml')
            require(not any((directory / name).exists() for name in names),
                    f'unpinned mixed funded Cargo configuration: {directory}')
    env['CARGO_TARGET_DIR'] = str(output / 'target')
    env['MIDNIGHT_PP'] = str(material)
    env['MIDNIGHT_LEDGER_TEST_STATIC_DIR'] = str(static)
    rustc = run(['rustc', '+1.99.0', '-vV'], output / 'rustc.log')
    require('release: 1.99.0' in rustc.splitlines(), 'Rust toolchain changed')
    if args.aggregate_head:
        require('commit-hash: b940084d7eb6a299eb4bfeb8e34901bc051e7ac4' in
                rustc.splitlines(), 'mixed independent verifier toolchain changed')
        verifier_env = {key: value for key, value in os.environ.items()
                        if not (key.startswith('CARGO_') or key in
                                {'RUSTFLAGS', 'RUSTDOCFLAGS', 'RUSTC', 'RUSTC_WRAPPER',
                                 'RUSTC_WORKSPACE_WRAPPER', 'RUSTUP_TOOLCHAIN',
                                 'CC', 'CXX', 'LD', 'AR', 'CFLAGS', 'CXXFLAGS', 'LDFLAGS'})}
        verifier_env['CARGO_TARGET_DIR'] = str(output / 'independent-runner-target')
        run(['cargo', '+1.99.0', 'build', '--locked', '--offline', '--bin',
             'secp256k1-unary-proof', '--manifest-path',
             str(runner_source / 'Cargo.toml')],
            output / 'independent-runner-build.log', env=verifier_env,
            cwd=runner_source)
        runner = output / 'independent-runner-target/debug/secp256k1-unary-proof'
        require(runner.is_file(), 'mixed independent verifier build missing')
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
        if args.aggregate_head:
            verification = run([
                str(runner), 'verify', str(preflight / case / 'zkir/test.zkir'),
                str(preflight / f'{case}-capture/honest.preimage'),
                str(evidence / 'proof' / case),
                str(wrong_root / wrong / 'statement.verifier'),
            ], output / f'{case}-independent-verification.log')
            require('verified in separate process; rejected different verifier, changed binding and corrupt proof'
                    in verification,
                    f'{case} independent verification or proof refusals changed')
        consumer = output / 'consumer' / case
        consumer.mkdir(parents=True)
        generated = preflight / case / 'contract'
        if args.aggregate_head:
            generated_manifest(
                generated / 'Cargo.toml',
                expected_canonical_sha256=GENERATED_MANIFEST_CANONICAL_SHA[case],
                source_head=head)
        else:
            require(sha(generated / 'Cargo.toml') == GENERATED_MANIFEST_SHA[case],
                    f'{case} generated Cargo manifest changed')
        generated_manifest_text = (generated / 'Cargo.toml').read_text()
        package = tomllib.loads(generated_manifest_text)['package']['name']
        require(package == f'compact-contract-{case}', 'generated package changed')
        runtime_path = tomllib.loads(generated_manifest_text)['dependencies']['midnight-compact-runtime']['path']
        if not args.aggregate_head:
            require(Path(runtime_path).name == 'runtime-rs' and
                    Path(runtime_path).parent ==
                    Path('/Users/ysh/.codex/worktrees/ledger9-mixed-original-proofs/compact'),
                    f'{case} generated runtime path changed')
        generated_copy = output / 'generated' / case / 'contract'
        generated_copy.parent.mkdir(parents=True)
        shutil.copytree(generated, generated_copy)
        copied_manifest = rebound_runtime_manifest(
            generated_manifest_text, runtime_path, aggregate=args.aggregate_head)
        (generated_copy / 'Cargo.toml').write_text(copied_manifest)
        if args.aggregate_head:
            source_files = {path.relative_to(generated) for path in generated.rglob('*')
                            if path.is_file()}
            copied_files = {path.relative_to(generated_copy)
                            for path in generated_copy.rglob('*') if path.is_file()}
            require(source_files == copied_files and all(
                sha(generated / name) == sha(generated_copy / name)
                for name in source_files if name != Path('Cargo.toml')),
                f'{case} generated contract copy differs from admitted preflight')
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
        if args.aggregate_head:
            copied_directories.extend((generated_copy, consumer))
            copied_hashes.update(snapshot_inputs([], [generated_copy, consumer]))
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
    if args.aggregate_head:
        require_final_binding(head, tree, admitted_hashes, admitted_files,
                              admitted_directories, copied_hashes,
                              copied_directories)
    receipt = {'format': 'ledger9-mixed-original-funded/v1',
               'head': head, 'tree': tree, 'proof_head': proof_head,
               'proof_receipt_sha256': sha(proof_path),
               'preflight_receipt_sha256': sha(preflight_path),
               'template_sha256': sha(TEMPLATE), 'gate_sha256': sha(__file__),
               'platform': platform.platform(), 'cases': results,
               'limitations': ['The initial contract deployment is injected into local test ledger state; only the call is funded and applied as a transaction.',
                               'Local Darwin arm64 qualification; native Linux required.',
                               'Generated EntryPoint and real two-contract qualification remain held.']}
    if args.aggregate_head:
        receipt['mode'] = 'aggregate-exact-head'
        receipt['compiler_receipt_sha256'] = sha(compiler_path)
        receipt['proof_runner_sha256'] = sha(generated_runner)
        receipt['independent_runner_sha256'] = sha(runner)
        receipt['independent_runner_build_log_sha256'] = sha(
            output / 'independent-runner-build.log')
        receipt['independent_verification_logs_sha256'] = {
            case: sha(output / f'{case}-independent-verification.log') for case in CASES
        }
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': 'passed', 'head': head, 'cases': list(results),
                      'receipt': str(output / 'receipt.json')}))


if __name__ == '__main__':
    main()
