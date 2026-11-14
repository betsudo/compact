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

"""Prove both pinned mixed native collection statements at one signed source head."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import tomllib

from collection_gate_binding import (
    GENERATED_CONTRACT_FILES, compiler_recipe, generated_contract_file_set,
)
from check_upstream_ledger9_bytes_map_proof import (
    RUNNER_SOURCE_SHA256, WASM_TGZ_SHA256, ZKIR_SHA256, require_grouped_lineage,
)
from check_upstream_ledger9_mixed_original_preflight import (
    CASES as ORIGINAL_CASES, UPSTREAM, original_cases, validate_runtime_baseline,
)


ROOT = Path(__file__).resolve().parents[2]
RUNNER_RECEIPT_SHA256 = '69d945f0a2131eaa3e55d10056f52421761638dae534a0883f9d258dddb4f444'
MATERIAL_RECEIPT_SHA256 = '56a004076c7332845fb4cd61ef357892db6dcd7dd05c03f9efebfb43f8df4df7'
WRONG_RECEIPT_SHA256 = '62e8ed0fc4dc4fb3bae6f213b4ab5a8b519adab4b866d6a70cbde82205a65770'
CAPTURE = ROOT / 'runtime-rs/tests/fixtures/capture-ledger9-mixed-original-proofs.mjs'
PREFLIGHT = ROOT / 'tools/compact-rust-backend/check_upstream_ledger9_mixed_original_preflight.py'
CASES = {
    'k1': {'k': 17, 'wrong': 'two',
           'params': '4a9ef6c7c0619aab74eede44b13e753e3ba54508a02dd3b7106a949aabb73b74',
           'prover': '80835fc849b6080e2950f908f94bc9f3d773ab61c2e630032ef2aaab551e3ac4',
           'verifier': '96e1246e0d5441f6353a875bbbdfeaf53176fb2619bb1a78f226b4ca851b9df4',
           'wrong_verifier': 'd1064554362d3e900784ce248e07c8dd2f4ef40391894a9e413cee30d77592a3'},
    'r1': {'k': 18, 'wrong': 'three',
           'params': 'e8436dc5d8b598f169c127c745135d889744007e6d384ff126df8d1332522f86',
           'prover': '0090379e690b45365d2ff62331516723940435e39b0bbe385e8c61600fedd59b',
           'verifier': 'ee933d521cb3522ae0eb97341f450d5feb834f430ad0ca499464421e1cec7425',
           'wrong_verifier': '088760a04adad570b55c228fa86eb1372952efdee7a59142b00718213b745c03'},
}
VERIFY_RESULT = ('verified in separate process; rejected different verifier, '
                 'changed binding and corrupt proof')


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def git(*args: str) -> str:
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()


def run(command: list[str], log: Path, *, cwd: Path = ROOT,
        env: dict[str, str] | None = None) -> str:
    with log.open('xb') as stream:
        completed = subprocess.run(command, cwd=cwd, env=env, stdout=stream,
                                   stderr=subprocess.STDOUT, check=True)
    require(completed.returncode == 0, f'{command[0]} failed; inspect {log}')
    return log.read_text()


def require_pin(path: Path, expected: str, label: str) -> None:
    require(re.fullmatch(r'[0-9a-f]{64}', expected) is not None and
            path.is_file() and not path.is_symlink() and sha(path) == expected,
            f'{label} SHA-256 pin changed')


def validate_preflight(path: Path, pin: str, *, head: str, tree: str,
                       compiler_receipt: Path, compiler: Path,
                       runtime_baseline: Path, baseline_pin: str,
                       extracted: dict[str, tuple[bytes, bytes]]) -> dict:
    """Bind the receipt, original fixtures, captures, runtime and compiler."""
    require(path.name == 'receipt.json', 'mixed preflight must be a gate receipt')
    require_pin(path, pin, 'mixed preflight receipt')
    prior = json.loads(path.read_text())
    require(prior.get('format') == 'ledger9-mixed-original-preflight/v1' and
            prior.get('source_head') == head and prior.get('source_tree') == tree and
            prior.get('upstream') == UPSTREAM,
            'mixed preflight source head/tree or upstream changed')
    require(prior.get('compiler_receipt_sha256') == sha(compiler_receipt) and
            prior.get('compiler_sha256') == sha(compiler) and
            prior.get('gate_source_sha256') == sha(PREFLIGHT) and
            prior.get('capture_source_sha256') == sha(CAPTURE),
            'mixed preflight compiler or producer source changed')
    require(prior.get('runtime_baseline_sha256') == baseline_pin and
            prior.get('runtime_recipe_log_sha256') ==
                sha(path.parent / 'runtime-recipe.log'),
            'mixed preflight runtime baseline or recipe changed')
    baseline = validate_runtime_baseline(
        runtime_baseline, baseline_pin, head=head, tree=tree,
        compiler_receipt=compiler_receipt, compiler_sha=sha(compiler),
        runtime_store=Path(prior['runtime_store']), root=ROOT)
    runtime = Path(prior['runtime_store']) / 'lib/node_modules/@midnight-ntwrk/compact-runtime'
    require(prior.get('runtime_manifest_sha256') == sha(runtime / 'package.json') ==
            baseline['typescript_runtime_package_sha256'] and
            prior.get('runtime_dist_files') == baseline['typescript_runtime_dist_files'] and
            prior.get('zkir_wasm_tgz_sha256') == WASM_TGZ_SHA256,
            'mixed preflight runtime or ZKIR wasm changed')
    require(set(prior.get('captures', {})) == set(CASES) and
            set(prior.get('artifacts', {})) == {
                f'{case}/{relative}' for case in CASES
                for relative in ORIGINAL_CASES[case]['artifacts']},
            'mixed preflight case or artifact inventory changed')
    for case in CASES:
        body, callback = extracted[case]
        pins = ORIGINAL_CASES[case]
        require(hashlib.sha256(body).hexdigest() == pins['source'] ==
                sha(path.parent / 'original' / f'{case}.compact') and
                hashlib.sha256(callback).hexdigest() == pins['callback'] ==
                sha(path.parent / 'original' / f'{case}.callback.js'),
                f'{case} original source or callback changed')
        generated_contract_file_set(path.parent / case / 'contract')
        for relative, digest in pins['artifacts'].items():
            require(prior['artifacts'][f'{case}/{relative}'] == digest ==
                    sha(path.parent / case / relative),
                    f'{case} generated artifact changed: {relative}')
        capture = prior['captures'][case]
        require(capture == json.loads((path.parent / f'{case}-capture/capture.json').read_text()) and
                capture.get('honestPreimageSha256') == pins['preimage'] ==
                    sha(path.parent / f'{case}-capture/honest.preimage') and
                capture.get('changedKeyPreimageSha256') == pins['changed_key_preimage'] ==
                    sha(path.parent / f'{case}-capture/changed-key.preimage') and
                capture.get('callbackSha256') == pins['callback'] and
                capture.get('rawZkirSha256') == pins['artifacts']['zkir/test.zkir'] and
                capture.get('checks') == [
                    'all original callback assertions and nested Map point',
                    'honest ZKIR accepted', 'changed nested Map key refused'],
                f'{case} original callback or refusal capture changed')
    return prior


def validate_runner_source(source: Path, receipt: Path) -> None:
    require_pin(receipt, RUNNER_RECEIPT_SHA256, 'proof runner source receipt')
    archived = json.loads(receipt.read_text())
    require(archived.get('runner_source_sha256') == RUNNER_SOURCE_SHA256 and
            source.is_dir() and not source.is_symlink() and
            {file.relative_to(source).as_posix() for file in source.rglob('*')
             if file.is_file() or file.is_symlink()} == set(RUNNER_SOURCE_SHA256) and
            all((source / name).is_file() and not (source / name).is_symlink() and
                sha(source / name) == digest
                for name, digest in RUNNER_SOURCE_SHA256.items()),
            'proof runner source or receipt changed')
    manifest = tomllib.loads((source / 'Cargo.toml').read_text())
    require(manifest['package']['name'] == 'secp256k1-unary-proof' and
            not any(isinstance(dep, dict) and 'path' in dep
                    for dep in manifest['dependencies'].values()),
            'proof runner manifest changed')
    for directory in (source, *source.parents, Path.home() / '.cargo'):
        names = ('config', 'config.toml') if directory == Path.home() / '.cargo' else (
            '.cargo/config', '.cargo/config.toml')
        require(not any((directory / name).exists() for name in names),
                f'unpinned proof runner Cargo configuration: {directory}')
    for directory in source.parents:
        parent_manifest = directory / 'Cargo.toml'
        require(not (parent_manifest.is_file() and
                     'workspace' in tomllib.loads(parent_manifest.read_text())),
                f'unpinned proof runner parent workspace: {directory}')


def validate_material_and_wrong(material: Path, wrong_root: Path) -> None:
    receipt = material.parent / 'material-receipt.json'
    require_pin(receipt, MATERIAL_RECEIPT_SHA256, 'Ledger 9 material receipt')
    files = json.loads(receipt.read_text())['files']
    wrong_receipt = wrong_root / 'receipt.json'
    require_pin(wrong_receipt, WRONG_RECEIPT_SHA256, 'same-k wrong verifier receipt')
    wrong = json.loads(wrong_receipt.read_text())
    for case, pins in CASES.items():
        params = material / f'bls_midnight_2p{pins["k"]}'
        row = files[params.name]
        verifier = wrong_root / pins['wrong'] / 'statement.verifier'
        require(sha(params) == pins['params'] == row['sha256'] and
                params.stat().st_size == row['bytes'] and
                wrong[pins['wrong']]['k'] == pins['k'] and
                wrong[pins['wrong']]['pins']['statement.verifier'] ==
                    pins['wrong_verifier'] == sha(verifier),
                f'{case} parameters or same-k wrong verifier changed')


def validate_verification_output(output: str, *, case: str) -> None:
    require(VERIFY_RESULT in output,
            f'{case} independent verification or refusal controls changed')


def bytes_baseline_artifacts(path: Path) -> list[Path]:
    """Admit the Bytes qualification files named by the runtime baseline."""
    baseline = json.loads(path.read_text())
    families = ('field_to_uint', 'large_index', 'reordering',
                'transformation', 'slice_composition', 'to_bytes')
    expected = {f'{family}/{mode}/typescript' for family in families
                for mode in ('default', 'paired')}
    expected.update(f'{family}/paired/{kind}' for family in families
                    for kind in ('rust', 'rust_ir'))
    artifacts = baseline.get('artifacts', {})
    require(set(artifacts) == expected, 'Bytes runtime baseline artifact inventory changed')
    names = {'typescript': 'index.js', 'rust': 'lib.rs',
             'rust_ir': 'compact-rust-ir.json'}
    paths = []
    for key, digest in artifacts.items():
        family, mode, kind = key.split('/')
        file = path.parent / mode / family / 'contract' / names[kind]
        require(sha(file) == digest, f'Bytes runtime baseline artifact changed: {key}')
        paths.append(file)
    for file, key in ((path.parent / 'cargo-test.log', 'cargo_test_log_sha256'),
                      (path.parent / 'rust-consumer/Cargo.lock',
                       'rust_consumer_lock_sha256')):
        require(sha(file) == baseline.get(key),
                f'Bytes runtime baseline evidence changed: {key}')
        paths.append(file)
    return paths


def external_input_paths(*, preflight_path: Path, baseline_path: Path,
                         compiler_path: Path, compiler: Path, scheme: Path,
                         runner_source: Path, runner_receipt: Path, material: Path,
                         wrong_root: Path, zkir: Path, prior: dict) -> list[Path]:
    """List the admitted external bytes that proof and receipt creation may read."""
    paths = [preflight_path, baseline_path, compiler_path, compiler, scheme,
             runner_receipt, zkir, material.parent / 'material-receipt.json',
             wrong_root / 'receipt.json',
             preflight_path.parent / 'runtime-recipe.log',
             baseline_path.parent / 'typescript-runtime-recipe.log']
    paths.extend(runner_source / name for name in RUNNER_SOURCE_SHA256)
    paths.extend(bytes_baseline_artifacts(baseline_path))
    runtime = (Path(prior['runtime_store']) /
               'lib/node_modules/@midnight-ntwrk/compact-runtime')
    paths.append(runtime / 'package.json')
    paths.extend(runtime / 'dist' / name for name in prior['runtime_dist_files'])
    for case, pins in CASES.items():
        paths.extend((preflight_path.parent / 'original' / f'{case}.{suffix}')
                     for suffix in ('compact', 'callback.js'))
        contract = preflight_path.parent / case / 'contract'
        paths.extend(contract / name for name in GENERATED_CONTRACT_FILES)
        paths.extend((preflight_path.parent / case / 'zkir/test.zkir',
                      preflight_path.parent / f'{case}-capture/capture.json',
                      preflight_path.parent / f'{case}-capture/honest.preimage',
                      preflight_path.parent / f'{case}-capture/changed-key.preimage',
                      material / f'bls_midnight_2p{pins["k"]}',
                      wrong_root / pins['wrong'] / 'statement.verifier'))
    return paths


def fingerprint_inputs(paths: list[Path]) -> dict[str, str]:
    return {str(path): sha(path) for path in paths}


def require_inputs_unchanged(paths: list[Path], initial: dict[str, str]) -> None:
    require(fingerprint_inputs(paths) == initial,
            'externally mutable mixed proof input changed during qualification')


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('preflight-receipt', 'compiler-receipt', 'runtime-baseline-receipt',
                 'runner-source', 'runner-source-receipt', 'zkir', 'material',
                 'wrong-verifiers', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
    parser.add_argument('--preflight-receipt-sha256', required=True)
    parser.add_argument('--runtime-baseline-receipt-sha256', required=True)
    args = parser.parse_args()
    require(not git('status', '--porcelain'), 'source checkout must be clean')
    head, tree = git('rev-parse', 'HEAD'), git('rev-parse', 'HEAD^{tree}')
    require(git('log', '-1', '--format=%G?') == 'G' and
            'Signed-off-by:' in git('log', '-1', '--format=%B'),
            'source head must be signed and DCO complete')
    require_grouped_lineage(head)
    output = args.output.resolve()
    require(not args.output.is_symlink() and not output.exists(), 'output must be absent')
    preflight_path = args.preflight_receipt.resolve(strict=True)
    compiler_path = args.compiler_receipt.resolve(strict=True)
    baseline_path = args.runtime_baseline_receipt.resolve(strict=True)
    runner_source = args.runner_source.resolve(strict=True)
    runner_receipt = args.runner_source_receipt.resolve(strict=True)
    material = args.material.resolve(strict=True)
    wrong_root = args.wrong_verifiers.resolve(strict=True)
    zkir = args.zkir.resolve(strict=True)
    require(sha(zkir) == ZKIR_SHA256 and
            subprocess.check_output([str(zkir), '--version'], text=True).strip() ==
                'midnight-zkir 3.2.0', 'native ZKIR 3.2.0 changed')
    recipe = json.loads(compiler_path.read_text())
    compiler = Path(recipe['compactc']['path']).resolve(strict=True)
    scheme = Path(recipe['compactc-scheme']['path']).resolve(strict=True)
    compiler_recipe(compiler_path, compiler, scheme,
                    source_head=head, current_head=head)
    prior = validate_preflight(
        preflight_path, args.preflight_receipt_sha256, head=head, tree=tree,
        compiler_receipt=compiler_path, compiler=compiler,
        runtime_baseline=baseline_path,
        baseline_pin=args.runtime_baseline_receipt_sha256,
        extracted=original_cases())
    validate_runner_source(runner_source, runner_receipt)
    validate_material_and_wrong(material, wrong_root)
    input_paths = external_input_paths(
        preflight_path=preflight_path, baseline_path=baseline_path,
        compiler_path=compiler_path, compiler=compiler, scheme=scheme,
        runner_source=runner_source, runner_receipt=runner_receipt,
        material=material, wrong_root=wrong_root, zkir=zkir, prior=prior)
    admitted_inputs = fingerprint_inputs(input_paths)
    rustc = subprocess.check_output(['rustc', '+1.99.0', '-vV'], text=True)
    require('release: 1.99.0' in rustc.splitlines() and
            'commit-hash: b940084d7eb6a299eb4bfeb8e34901bc051e7ac4'
            in rustc.splitlines(), 'proof runner toolchain changed')
    output.mkdir(parents=True)
    runtime_recipe = run([
        str(Path(shutil.which('nix')).resolve(strict=True)), 'build', '--offline',
        '--no-link', '--print-out-paths', '--max-jobs', '0', '--builders', '',
        '--option', 'substitute', 'false', '.#runtime.package',
    ], output / 'runtime-recipe.log')
    require(str(Path(runtime_recipe.splitlines()[-1]).resolve(strict=True)) ==
            prior['runtime_store'], 'mixed proof runtime differs from current-head Nix output')
    clean_env = {key: value for key, value in os.environ.items()
                 if not (key.startswith('CARGO_') or key in
                         {'RUSTFLAGS', 'RUSTDOCFLAGS', 'RUSTC', 'RUSTC_WRAPPER',
                          'RUSTC_WORKSPACE_WRAPPER', 'RUSTUP_TOOLCHAIN',
                          'CC', 'CXX', 'LD', 'AR', 'CFLAGS', 'CXXFLAGS', 'LDFLAGS'})}
    clean_env['CARGO_TARGET_DIR'] = str(output / 'runner-target')
    run(['cargo', '+1.99.0', 'build', '--locked', '--offline', '--bin',
         'secp256k1-unary-proof', '--manifest-path', str(runner_source / 'Cargo.toml')],
        output / 'runner-build.log', cwd=runner_source, env=clean_env)
    runner = output / 'runner-target/debug/secp256k1-unary-proof'
    require(runner.is_file() and not runner.is_symlink(), 'source-built proof runner missing')
    runner_sha = sha(runner)
    cases = {}
    for case, pins in CASES.items():
        raw = preflight_path.parent / case / 'zkir/test.zkir'
        preimage = preflight_path.parent / f'{case}-capture/honest.preimage'
        params = material / f'bls_midnight_2p{pins["k"]}'
        wrong = wrong_root / pins['wrong'] / 'statement.verifier'
        proof = output / 'proof' / case
        proof.mkdir(parents=True)
        proved = run([str(runner), 'prove', str(raw), str(preimage),
                      str(proof), str(params)], output / f'{case}-prove.log')
        require(proved.startswith(f'proved k={pins["k"]},'),
                f'{case} fresh k={pins["k"]} proof missing')
        verified = run([str(runner), 'verify', str(raw), str(preimage),
                        str(proof), str(wrong)], output / f'{case}-verify.log')
        validate_verification_output(verified, case=case)
        require(sha(proof / 'statement.prover') == pins['prover'] and
                sha(proof / 'statement.verifier') == pins['verifier'] and
                sha(proof / 'statement.verifier') != sha(wrong) and
                (proof / 'statement.proof').stat().st_size > 0,
                f'{case} proof keys or statement changed')
        cases[case] = {
            'k': pins['k'], 'raw_zkir_sha256': sha(raw),
            'preimage_sha256': sha(preimage), 'params_sha256': sha(params),
            'prover_sha256': sha(proof / 'statement.prover'),
            'verifier_sha256': sha(proof / 'statement.verifier'),
            'wrong_verifier_sha256': sha(wrong),
            'proof_sha256': sha(proof / 'statement.proof'),
            'prove_log_sha256': sha(output / f'{case}-prove.log'),
            'verification_log_sha256': sha(output / f'{case}-verify.log'),
            'independent_verification': True,
            'wrong_verifier_changed_binding_corrupt_proof_refused': True,
        }
    require_inputs_unchanged(input_paths, admitted_inputs)
    require(not git('status', '--porcelain') and git('rev-parse', 'HEAD') == head and
            git('rev-parse', 'HEAD^{tree}') == tree,
            'source checkout changed during mixed proof gate')
    compiler_recipe(compiler_path, compiler, scheme,
                    source_head=head, current_head=head)
    require(validate_preflight(
                preflight_path, args.preflight_receipt_sha256, head=head, tree=tree,
                compiler_receipt=compiler_path, compiler=compiler,
                runtime_baseline=baseline_path,
                baseline_pin=args.runtime_baseline_receipt_sha256,
                extracted=original_cases()) == prior,
            'mixed preflight changed during proof gate')
    validate_runner_source(runner_source, runner_receipt)
    validate_material_and_wrong(material, wrong_root)
    require(sha(zkir) == ZKIR_SHA256 and sha(runner) == runner_sha,
            'ZKIR or source-built proof runner changed during qualification')
    for case in CASES:
        proof_dir = output / 'proof' / case
        qualified = cases[case]
        for filename, digest in (
            ('statement.proof', qualified['proof_sha256']),
            ('statement.prover', qualified['prover_sha256']),
            ('statement.verifier', qualified['verifier_sha256']),
        ):
            require(sha(proof_dir / filename) == digest,
                    f'{case} proof output changed after independent verification')
        require(sha(output / f'{case}-prove.log') == qualified['prove_log_sha256'] and
                sha(output / f'{case}-verify.log') == qualified['verification_log_sha256'],
                f'{case} proof log changed after independent verification')
    receipt = {
        'format': 'ledger9-mixed-original-proofs/v1',
        'source_head': head, 'source_tree': tree,
        'preflight_receipt_sha256': sha(preflight_path),
        'compiler_receipt_sha256': sha(compiler_path),
        'runner_binary_sha256': runner_sha,
        'runner_source_receipt_sha256': sha(runner_receipt),
        'verifier_source_sha256': sha(Path(__file__)),
        'zkir_binary_sha256': sha(zkir),
        'runtime_baseline_receipt_sha256': sha(baseline_path),
        'runtime_recipe_log_sha256': sha(output / 'runtime-recipe.log'),
        'runner_build_log_sha256': sha(output / 'runner-build.log'),
        'cases': cases, 'platform': f'{platform.system()} {platform.machine()}',
        'limitations': ['Local platform only; native Linux qualification required.',
                        'Proofs are standalone; funded application is a separate gate.',
                        'Real two-contract transaction remains open.'],
    }
    (output / 'proof-receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(f'original mixed fresh proofs verified: {output}/proof-receipt.json')


if __name__ == '__main__':
    main()
