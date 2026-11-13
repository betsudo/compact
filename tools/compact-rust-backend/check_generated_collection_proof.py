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

"""Prove fresh generated Runtime 0.18.0 Set.add material and verify independently."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess


ROOT = Path(__file__).resolve().parents[2]
CAPTURE = ROOT / 'runtime-rs/tests/fixtures/capture-ledger9-set-add-proof-v018.mjs'
SOURCE = ROOT / 'runtime-rs/tests/fixtures/ledger9-set-transition-v018.compact'
ORACLE = ROOT / 'runtime-rs/tests/fixtures/ledger9-set-transition.json'


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def run(command: list[str], *, env: dict[str, str] | None = None, log: Path | None = None) -> str:
    result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True)
    if log:
        log.write_text(result.stdout + result.stderr)
    if result.returncode:
        raise RuntimeError(f'{command[0]} exited {result.returncode}:\n{result.stdout[-4000:]}\n{result.stderr[-4000:]}')
    return result.stdout + result.stderr


def git(*args: str) -> str:
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('generated-root', 'compiler', 'scheme', 'runtime-ts', 'zkir-wasm',
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
    for name in ('generated_root', 'compiler', 'scheme', 'runtime_ts', 'zkir_wasm',
                 'zkir', 'proof_runner', 'proof_runner_receipt', 'material'):
        paths[name] = getattr(args, name).resolve(strict=True)
    output = args.output.resolve()
    if args.output.is_symlink() or output.exists():
        parser.error('output must be absent')
    generated = paths['generated_root']
    prior = json.loads((generated / 'qualification.json').read_text())
    source = prior['sources_and_artifacts']['set']
    contract = generated / 'set/contract'
    raw = generated / 'set/zkir/add.zkir'
    wrong_raw = generated / 'set/zkir/remove.zkir'
    require(source['source'] == SOURCE.relative_to(ROOT).as_posix(), 'generated Set source path changed')
    require(source['source_sha256'] == sha(SOURCE), 'generated Set source hash changed')
    require(source['oracle_sha256'] == sha(ORACLE), 'generated Set oracle hash changed')
    require(source['typescript_sha256'] == sha(contract / 'index.js'), 'generated TypeScript hash changed')
    require(source['rust_sha256'] == sha(contract / 'lib.rs'), 'generated Rust hash changed')
    require(source['ir_sha256'] == sha(contract / 'compact-rust-ir.json'), 'generated Rust IR hash changed')
    require(prior['compiler_sha256'] == sha(paths['compiler']), 'compiler binary changed')
    require(prior['scheme_frontend_sha256'] == sha(paths['scheme']), 'Scheme compiler changed')
    require(prior['runtime_compatibility_sha256'] == sha(ROOT / 'runtime-rs/compatibility.json'),
            'runtime compatibility matrix changed')
    runner_receipt = json.loads(paths['proof_runner_receipt'].read_text())
    require(runner_receipt['proof_runner']['sha256'] == sha(paths['proof_runner']),
            'proof runner changed from source-qualified binary')
    material_receipt = json.loads((paths['material'].parent / 'material-receipt.json').read_text())
    params = paths['material'] / 'bls_midnight_2p6'
    require(sha(params) == material_receipt['files'][params.name]['sha256'], 'k=6 parameters changed')
    require(json.loads((paths['runtime_ts'] / 'package.json').read_text())['version'] == '0.18.0',
            'TypeScript runtime version changed')
    require(json.loads((paths['zkir_wasm'] / 'package.json').read_text())['version'] == '3.2.0',
            'ZKIR wasm version changed')
    require(run([str(paths['zkir']), '--version']).strip() == 'midnight-zkir 3.2.0',
            'native ZKIR version changed')
    add_model = run([str(paths['zkir']), 'mock-compile', str(raw)])
    remove_model = run([str(paths['zkir']), 'mock-compile', str(wrong_raw)])
    require('k=6' in add_model, 'Set.add is not k=6')
    require('k=6' in remove_model, 'Set.remove wrong-key circuit is not k=6')

    output.mkdir(parents=True)
    fresh = output / 'fresh-dual'
    compiler_env = os.environ.copy()
    compiler_env['COMPACTC_SCHEME'] = str(paths['scheme'])
    run([str(paths['compiler']), '--target', 'ts', '--target', 'rust',
         '--rust-runtime-root', str(ROOT), '--rust-require-recording', '--skip-zk',
         '--compact-path', str(ROOT / 'compiler'), str(SOURCE), str(fresh)],
        env=compiler_env, log=output / 'fresh-dual.log')
    require(sha(fresh / 'zkir/add.zkir') == sha(raw), 'Set.add raw ZKIR differs from fresh source compile')
    require(sha(fresh / 'zkir/remove.zkir') == sha(wrong_raw), 'Set.remove raw ZKIR differs from fresh source compile')
    require(sha(fresh / 'contract/index.js') == source['typescript_sha256'],
            'fresh generated TypeScript differs from qualified artifact')
    require(sha(fresh / 'contract/lib.rs') == source['rust_sha256'],
            'fresh generated Rust differs from qualified artifact')
    staged = output / 'set-contract'
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
    require(capture['honest_check'] == 'passed' and capture['changed_insert_check'] == 'refused',
            'ZKIR wasm preimage controls did not pass')
    require(capture['generated_index_sha256'] == source['typescript_sha256'], 'preimage TS artifact changed')
    require(capture['raw_zkir_sha256'] == sha(raw), 'preimage raw ZKIR changed')
    require(capture['preimage_sha256'] == sha(capture_dir / 'honest.preimage'), 'honest preimage hash changed')
    require(capture['changed_insert_sha256'] == sha(capture_dir / 'changed-insert.preimage'),
            'changed insert preimage hash changed')

    env = os.environ.copy()
    env['MIDNIGHT_LEDGER_TEST_STATIC_DIR'] = str(paths['material'])
    wrong = output / 'wrong-key'
    wrong.mkdir()
    run([str(paths['zkir']), 'compile', str(wrong_raw),
         str(wrong / 'remove.prover'), str(wrong / 'remove.verifier')],
        env=env, log=output / 'wrong-key.log')
    proof = output / 'proof'
    proof.mkdir()
    prove = run([str(paths['proof_runner']), 'prove', str(raw),
                 str(capture_dir / 'honest.preimage'), str(proof), str(params)],
                log=output / 'prove.log')
    require(prove.startswith('proved k=6,'), 'proof runner did not report k=6 proof')
    verify = run([str(paths['proof_runner']), 'verify', str(raw),
                  str(capture_dir / 'honest.preimage'), str(proof),
                  str(wrong / 'remove.verifier')], log=output / 'verify.log')
    require('verified in separate process; rejected different verifier, changed binding and corrupt proof' in verify,
            'independent verification or refusal controls did not pass')
    if git('status', '--porcelain') or git('rev-parse', 'HEAD') != head:
        raise RuntimeError('source checkout changed during qualification')
    receipt = {
        'platform': f'{platform.system()} {platform.machine()}',
        'head': head, 'tree': tree,
        'generated_receipt_sha256': sha(generated / 'qualification.json'),
        'source_sha256': sha(SOURCE), 'oracle_sha256': sha(ORACLE),
        'capture_source_sha256': sha(CAPTURE),
        'fresh_add_zkir_sha256': sha(fresh / 'zkir/add.zkir'),
        'fresh_remove_zkir_sha256': sha(fresh / 'zkir/remove.zkir'),
        'fresh_typescript_sha256': sha(fresh / 'contract/index.js'),
        'fresh_rust_sha256': sha(fresh / 'contract/lib.rs'),
        'compiler_sha256': sha(paths['compiler']), 'scheme_sha256': sha(paths['scheme']),
        'runtime_package_sha256': sha(paths['runtime_ts'] / 'package.json'),
        'zkir_wasm_package_sha256': sha(paths['zkir_wasm'] / 'package.json'),
        'zkir_sha256': sha(paths['zkir']), 'proof_runner_sha256': sha(paths['proof_runner']),
        'params_k6_sha256': sha(params), 'raw_add_sha256': sha(raw),
        'raw_remove_sha256': sha(wrong_raw),
        'typescript_sha256': source['typescript_sha256'], 'rust_sha256': source['rust_sha256'],
        'preimage_sha256': sha(capture_dir / 'honest.preimage'),
        'changed_insert_preimage_sha256': sha(capture_dir / 'changed-insert.preimage'),
        'proof_sha256': sha(proof / 'statement.proof'),
        'prover_sha256': sha(proof / 'statement.prover'),
        'verifier_sha256': sha(proof / 'statement.verifier'),
        'wrong_same_k_verifier_sha256': sha(wrong / 'remove.verifier'),
        'checks': ['ZKIR wasm honest preimage accepted', 'changed insert refused',
                   'k=6 fresh proof', 'independent process verification',
                   'same-k different verifier refused', 'changed binding refused',
                   'corrupt proof refused'],
        'limitations': ['Local platform only; native Linux remains required.',
                        'No funded default-strict transaction or replay refusal in this slice.',
                        'Proof runner source provenance is bound by the supplied prior qualification receipt.'],
    }
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(f'generated Ledger 9 Set.add proof and independent verification passed: {output}')


if __name__ == '__main__':
    main()
