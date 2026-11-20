#!/usr/bin/env python3

# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
# 	http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Prove fresh generated Runtime 0.18.0 Map.put material and verify independently."""

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

from collection_gate_binding import compiler_recipe


ROOT = Path(__file__).resolve().parents[2]
CAPTURE = ROOT / 'runtime-rs/tests/fixtures/capture-ledger9-map-put-proof-v018.mjs'
SOURCE = ROOT / 'runtime-rs/tests/fixtures/ledger9-map-transition-v018.compact'
ORACLE = ROOT / 'runtime-rs/tests/fixtures/ledger9-map-transition.json'
RUNNER_SHA256 = 'b068a30c9e6b3ea42f5d7e8c9da472b2ba126604d413d7b7c785b488254121e6'
RUNNER_MAIN_SHA256 = '2846dd67da2916bcf3a15db93e91a1007cfb5d6b97a1d0070e08c6fe312194e9'
RUNNER_MANIFEST_SHA256 = '1e7a8f3a18ff81ba8eacfb982f20501f0a320668d2e7947612deb4c6986367e0'
RUNNER_LOCK_SHA256 = 'c66ac3ad4ad9e2291ce67a90d764af5f7573fbffacdc48953619df7b92ec2ce5'


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def run(command: list[str], *, env: dict[str, str] | None = None,
        log: Path | None = None, cwd: Path = ROOT) -> str:
    result = subprocess.run(command, cwd=cwd, env=env, capture_output=True, text=True)
    if log:
        log.write_text(result.stdout + result.stderr)
    if result.returncode:
        raise RuntimeError(f'{command[0]} exited {result.returncode}:\n{result.stdout[-4000:]}\n{result.stderr[-4000:]}')
    return result.stdout + result.stderr


def git(*args: str) -> str:
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('generated-root', 'compiler', 'scheme', 'compiler-receipt',
                 'runtime-ts', 'zkir-wasm',
                 'zkir', 'proof-runner', 'proof-runner-receipt', 'material', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    if git('status', '--porcelain'):
        parser.error('source checkout must be clean')
    head = git('rev-parse', 'HEAD')
    tree = git('rev-parse', 'HEAD^{tree}')
    if git('log', '-1', '--format=%G?') != 'G' or 'Signed-off-by:' not in git('log', '-1', '--format=%B'):
        parser.error('source commit must be signed and DCO complete')
    paths = {}
    for name in ('generated_root', 'compiler', 'scheme', 'compiler_receipt',
                 'runtime_ts', 'zkir_wasm',
                 'zkir', 'proof_runner', 'proof_runner_receipt', 'material'):
        paths[name] = getattr(args, name).resolve(strict=True)
    output = args.output.resolve()
    if args.output.is_symlink() or output.exists():
        parser.error('output must be absent')
    generated = paths['generated_root']
    prior = json.loads((generated / 'qualification.json').read_text())
    require(prior['compiler_receipt_sha256'] == sha(paths['compiler_receipt']),
            'generated Map compiler recipe receipt changed')
    require(prior['tree'] == git('rev-parse', f"{prior['head']}^{{tree}}"),
            'generated Map source tree changed')
    compiler_recipe(paths['compiler_receipt'], paths['compiler'], paths['scheme'],
                    source_head=prior['head'], current_head=head)
    source = prior['sources_and_artifacts']['map']
    qualified_contract = generated / 'map/contract'
    qualified_raw = generated / 'map/zkir/put.zkir'
    qualified_wrong_raw = generated / 'map/zkir/has.zkir'
    require(source['source'] == SOURCE.relative_to(ROOT).as_posix(), 'generated Map source path changed')
    require(source['source_sha256'] == sha(SOURCE), 'generated Map source hash changed')
    require(source['oracle_sha256'] == sha(ORACLE), 'generated Map oracle hash changed')
    require(source['typescript_sha256'] == sha(qualified_contract / 'index.js'), 'generated TypeScript hash changed')
    require(source['rust_sha256'] == sha(qualified_contract / 'lib.rs'), 'generated Rust hash changed')
    require(source['ir_sha256'] == sha(qualified_contract / 'compact-rust-ir.json'), 'generated Rust IR hash changed')
    require(prior['compiler_sha256'] == sha(paths['compiler']), 'compiler binary changed')
    require(prior['scheme_frontend_sha256'] == sha(paths['scheme']), 'Scheme compiler changed')
    require(prior['runtime_compatibility_sha256'] == sha(ROOT / 'runtime-rs/compatibility.json'),
            'runtime compatibility matrix changed')
    runner_receipt = json.loads(paths['proof_runner_receipt'].read_text())
    require(runner_receipt['proof_runner']['sha256'] == RUNNER_SHA256 == sha(paths['proof_runner']),
            'proof runner changed from source-qualified binary')
    require(runner_receipt['proof_runner']['source_main_sha256'] == RUNNER_MAIN_SHA256 and
            runner_receipt['proof_runner']['source_manifest_sha256'] == RUNNER_MANIFEST_SHA256 and
            runner_receipt['proof_runner']['source_lock_sha256'] == RUNNER_LOCK_SHA256,
            'proof runner source hashes changed')
    toolchain_record = runner_receipt['artifacts']['base_toolchain']
    toolchain_path = Path(toolchain_record['path']).resolve(strict=True)
    require(sha(toolchain_path) == toolchain_record['sha256'], 'proof runner toolchain record changed')
    toolchain = json.loads(toolchain_path.read_text())
    runner_source = toolchain['proof_runner_source']
    for part, expected in (('main', RUNNER_MAIN_SHA256), ('manifest', RUNNER_MANIFEST_SHA256),
                           ('lock', RUNNER_LOCK_SHA256)):
        require(sha(Path(runner_source[part]['path'])) == expected, f'proof runner {part} source changed')
    manifest = Path(runner_source['manifest']['path']).resolve(strict=True)
    source_root = manifest.parent
    require(Path(runner_source['main']['path']).resolve(strict=True) == source_root / 'src/main.rs',
            'pinned proof runner main is not Cargo default binary source')
    require(Path(runner_source['lock']['path']).resolve(strict=True) == source_root / 'Cargo.lock',
            'pinned proof runner lock is not the Cargo lock')
    require({path.relative_to(source_root).as_posix() for path in source_root.rglob('*') if path.is_file()}
            == {'Cargo.toml', 'Cargo.lock', 'src/main.rs'}, 'proof runner has unpinned local inputs')
    manifest_data = tomllib.loads(manifest.read_text())
    require(manifest_data['package']['name'] == 'secp256k1-unary-proof' and
            not any(isinstance(dependency, dict) and 'path' in dependency
                    for dependency in manifest_data['dependencies'].values()),
            'proof runner manifest has a changed binary name or path dependency')
    rustc_version = run(['rustc', '+1.99.0', '-vV']).strip()
    require('release: 1.99.0' in rustc_version.splitlines() and
            'commit-hash: b940084d7eb6a299eb4bfeb8e34901bc051e7ac4' in rustc_version.splitlines(),
            'proof runner Rust toolchain changed')
    output.mkdir(parents=True)
    runner_target = output / 'runner-target'
    runner_build_env = {key: value for key, value in os.environ.items()
                        if not (key.startswith('CARGO_') or key in
                                {'RUSTFLAGS', 'RUSTDOCFLAGS', 'RUSTC', 'RUSTC_WRAPPER',
                                 'RUSTC_WORKSPACE_WRAPPER', 'RUSTUP_TOOLCHAIN'})}
    require('CARGO_HOME' not in runner_build_env, 'external Cargo home was not cleared')
    runner_build_env['CARGO_TARGET_DIR'] = str(runner_target)
    cargo_home = Path(runner_build_env.get('HOME', str(Path.home()))) / '.cargo'
    config_roots = (source_root, *source_root.parents, cargo_home)
    for directory in config_roots:
        for name in ('.cargo/config', '.cargo/config.toml') if directory != cargo_home else ('config', 'config.toml'):
            require(not (directory / name).exists(), f'unpinned Cargo config at {directory / name}')
    for directory in source_root.parents:
        candidate = directory / 'Cargo.toml'
        if candidate.is_file():
            require('workspace' not in tomllib.loads(candidate.read_text()),
                    f'unpinned parent Cargo workspace at {candidate}')
    runner_command = ['cargo', '+1.99.0', 'build', '--locked', '--offline', '--bin',
                      'secp256k1-unary-proof', '--manifest-path', str(manifest)]
    run(runner_command, env=runner_build_env, log=output / 'runner-build.log', cwd=source_root)
    rebuilt_runner = runner_target / 'debug/secp256k1-unary-proof'
    require(rebuilt_runner.is_file(), 'fresh proof runner build produced no binary')
    material_receipt = json.loads((paths['material'].parent / 'material-receipt.json').read_text())
    params = paths['material'] / 'bls_midnight_2p7'
    require(sha(params) == material_receipt['files'][params.name]['sha256'], 'k=7 parameters changed')
    require(json.loads((paths['runtime_ts'] / 'package.json').read_text())['version'] == '0.18.0',
            'TypeScript runtime version changed')
    require(json.loads((paths['zkir_wasm'] / 'package.json').read_text())['version'] == '3.2.0',
            'ZKIR wasm version changed')
    require(run([str(paths['zkir']), '--version']).strip() == 'midnight-zkir 3.2.0',
            'native ZKIR version changed')
    fresh = output / 'fresh-dual'
    compiler_env = os.environ.copy()
    compiler_env.pop('COMPACT_RUST_RUNTIME_DIR', None)
    compiler_env['COMPACTC_SCHEME'] = str(paths['scheme'])
    compile_command = [str(paths['compiler']), '--target', 'ts', '--target', 'rust',
                       '--rust-runtime-root', str(ROOT), '--rust-require-recording', '--skip-zk',
                       '--compact-path', str(ROOT / 'compiler'), str(SOURCE), str(fresh)]
    run(compile_command, env=compiler_env, log=output / 'fresh-dual.log')
    require(sha(fresh / 'zkir/put.zkir') == sha(qualified_raw), 'Map.put raw ZKIR differs from fresh source compile')
    require(sha(fresh / 'zkir/has.zkir') == sha(qualified_wrong_raw),
            'Map.has raw ZKIR differs from fresh source compile')
    require(sha(fresh / 'contract/index.js') == source['typescript_sha256'],
            'fresh generated TypeScript differs from qualified artifact')
    require(sha(fresh / 'contract/lib.rs') == source['rust_sha256'],
            'fresh generated Rust differs from qualified artifact')
    require(sha(fresh / 'contract/compact-rust-ir.json') == source['ir_sha256'],
            'fresh generated Rust IR differs from qualified artifact')
    contract = fresh / 'contract'
    raw = fresh / 'zkir/put.zkir'
    wrong_raw = fresh / 'zkir/has.zkir'
    put_model = run([str(paths['zkir']), 'mock-compile', str(raw)])
    has_model = run([str(paths['zkir']), 'mock-compile', str(wrong_raw)])
    require(re.search(r'\bk=7\b', put_model) is not None, 'Map.put is not k=7')
    require(re.search(r'\bk=7\b', has_model) is not None, 'Map.has wrong-key circuit is not k=7')
    staged = output / 'map-contract'
    staged.mkdir()
    shutil.copyfile(contract / 'index.js', staged / 'index.js')
    (staged / 'package.json').write_text('{"type":"module"}\n')
    runtime_link = staged / 'node_modules/@midnight-ntwrk/compact-runtime'
    runtime_link.parent.mkdir(parents=True)
    runtime_link.symlink_to(paths['runtime_ts'], target_is_directory=True)
    capture_dir = output / 'preimage'
    run(['node', str(CAPTURE), str(staged / 'index.js'), str(raw),
         str(paths['zkir_wasm']), str(capture_dir)], log=output / 'capture.log')
    capture = json.loads((capture_dir / 'capture.json').read_text())
    require(capture['honest_check'] == 'passed' and capture['changed_insert_check'] == 'refused'
            and capture['changed_value_check'] == 'refused',
            'ZKIR wasm preimage controls did not pass')
    require(capture['circuit'] == 'put' and capture['arguments'] == [True, '42'],
            'Map.put proof arguments changed')
    require(capture['generated_index_sha256'] == source['typescript_sha256'], 'preimage TS artifact changed')
    require(capture['raw_zkir_sha256'] == sha(raw), 'preimage raw ZKIR changed')
    require(capture['preimage_sha256'] == sha(capture_dir / 'honest.preimage'), 'honest preimage hash changed')
    require(capture['changed_insert_sha256'] == sha(capture_dir / 'changed-insert.preimage'),
            'changed insert preimage hash changed')
    require(capture['changed_value_sha256'] == sha(capture_dir / 'changed-value.preimage'),
            'changed value preimage hash changed')

    env = os.environ.copy()
    env['MIDNIGHT_LEDGER_TEST_STATIC_DIR'] = str(paths['material'])
    wrong = output / 'wrong-key'
    wrong.mkdir()
    run([str(paths['zkir']), 'compile', str(wrong_raw),
         str(wrong / 'has.prover'), str(wrong / 'has.verifier')],
        env=env, log=output / 'wrong-key.log')
    require(re.search(r'\bk=7\b', (output / 'wrong-key.log').read_text()) is not None,
            'wrong-key compiler did not report k=7')
    proof = output / 'proof'
    proof.mkdir()
    prove = run([str(rebuilt_runner), 'prove', str(raw),
                 str(capture_dir / 'honest.preimage'), str(proof), str(params)],
                log=output / 'prove.log')
    require(prove.startswith('proved k=7,'), 'proof runner did not report k=7 proof')
    verify = run([str(rebuilt_runner), 'verify', str(raw),
                  str(capture_dir / 'honest.preimage'), str(proof),
                  str(wrong / 'has.verifier')], log=output / 'verify.log')
    require('verified in separate process; rejected different verifier, changed binding and corrupt proof' in verify,
            'independent verification or refusal controls did not pass')
    require(sha(wrong / 'has.verifier') != sha(proof / 'statement.verifier'),
            'different-circuit verifier equals the Map.put verifier')
    if git('status', '--porcelain') or git('rev-parse', 'HEAD') != head:
        raise RuntimeError('source checkout changed during qualification')
    receipt = {
        'platform': f'{platform.system()} {platform.machine()}',
        'head': head, 'tree': tree,
        'generated_receipt_sha256': sha(generated / 'qualification.json'),
        'generated_receipt_path': str(generated / 'qualification.json'),
        'compiler_receipt_sha256': sha(paths['compiler_receipt']),
        'compiler_source_head': prior['head'],
        'compiler_source_tree': prior['tree'],
        'proof_gate_script_sha256': sha(Path(__file__)),
        'source_sha256': sha(SOURCE), 'oracle_sha256': sha(ORACLE),
        'capture_source_sha256': sha(CAPTURE),
        'fresh_put_zkir_sha256': sha(fresh / 'zkir/put.zkir'),
        'fresh_has_zkir_sha256': sha(fresh / 'zkir/has.zkir'),
        'fresh_typescript_sha256': sha(fresh / 'contract/index.js'),
        'fresh_rust_sha256': sha(fresh / 'contract/lib.rs'),
        'fresh_rust_ir_sha256': sha(fresh / 'contract/compact-rust-ir.json'),
        'fresh_compile_command': compile_command,
        'fresh_compile_log_sha256': sha(output / 'fresh-dual.log'),
        'proof_runner_source_sha256': {
            part: sha(Path(runner_source[part]['path'])) for part in ('main', 'manifest', 'lock')
        },
        'proof_runner_build_command': runner_command,
        'proof_runner_build_cwd': str(source_root),
        'proof_runner_cargo_home': str(cargo_home),
        'proof_runner_build_log_sha256': sha(output / 'runner-build.log'),
        'proof_runner_rustc_version': rustc_version,
        'proof_runner_path': str(rebuilt_runner),
        'compiler_sha256': sha(paths['compiler']), 'scheme_sha256': sha(paths['scheme']),
        'runtime_package_sha256': sha(paths['runtime_ts'] / 'package.json'),
        'zkir_wasm_package_sha256': sha(paths['zkir_wasm'] / 'package.json'),
        'zkir_sha256': sha(paths['zkir']), 'proof_runner_sha256': sha(rebuilt_runner),
        'archived_proof_runner_sha256': sha(paths['proof_runner']),
        'params_k7_sha256': sha(params), 'raw_put_sha256': sha(raw),
        'raw_has_sha256': sha(wrong_raw),
        'typescript_sha256': source['typescript_sha256'], 'rust_sha256': source['rust_sha256'],
        'preimage_sha256': sha(capture_dir / 'honest.preimage'),
        'changed_insert_preimage_sha256': sha(capture_dir / 'changed-insert.preimage'),
        'changed_value_preimage_sha256': sha(capture_dir / 'changed-value.preimage'),
        'proof_sha256': sha(proof / 'statement.proof'),
        'prover_sha256': sha(proof / 'statement.prover'),
        'verifier_sha256': sha(proof / 'statement.verifier'),
        'wrong_same_k_verifier_sha256': sha(wrong / 'has.verifier'),
        'checks': ['Fresh dual-target TypeScript, Rust, Rust IR and ZKIR match qualified artifacts',
                   'ZKIR wasm honest preimage accepted', 'changed insert and value refused',
                   'k=7 fresh proof', 'independent process verification',
                   'same-k different verifier refused', 'changed binding refused',
                   'corrupt proof refused'],
        'limitations': ['Local platform only; native Linux remains required.',
                        'No funded default-strict transaction or replay refusal in this slice.',
                        'Compiler binary is pinned by the prior generated parity receipt and comes from that worktree.'],
    }
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(f'generated Ledger 9 Map.put proof and independent verification passed: {output}')


if __name__ == '__main__':
    main()
