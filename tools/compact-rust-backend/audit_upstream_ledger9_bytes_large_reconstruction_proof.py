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

"""Rehash exact-head k16 proof inputs and independently rerun verifier refusals."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

from check_upstream_ledger9_bytes_large_reconstruction_gap import originals


ROOT = Path(__file__).resolve().parents[2]
CIRCUIT = 'bytes_manual_update_large_reconstruction'
WRONG = 'bytes_large_reconstruction_wrong_verifier'
STEM = 'bytes-large-reconstruction'


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def files(root: Path) -> dict[str, str]:
    result = {}
    for path in sorted(root.rglob('*')):
        require(not path.is_symlink(), f'artifact symlink: {path}')
        if path.is_file():
            result[path.relative_to(root).as_posix()] = digest(path)
    return result


def invoke(runner: Path, root: Path, circuit: str, output: Path,
           environment: dict[str, str]) -> subprocess.CompletedProcess[str]:
    command = [str(runner), '--positional-call-verify', str(root), circuit, STEM]
    result = subprocess.run(command, cwd=ROOT, env=environment,
                            capture_output=True, text=True)
    output.write_text(result.stdout + result.stderr)
    return result


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--proof-gate', required=True, type=Path)
    parser.add_argument('--semantic-gate', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    proof_root = args.proof_gate.resolve(strict=True)
    semantic_root = args.semantic_gate.resolve(strict=True)
    output = args.output.absolute()
    require(not output.exists() and not output.is_symlink(), 'fresh audit output required')
    proof_receipt_path = proof_root / 'receipt.json'
    semantic_receipt_path = semantic_root / 'receipt.json'
    receipt = json.loads(proof_receipt_path.read_text())
    semantic = json.loads(semantic_receipt_path.read_text())
    head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT,
                                   text=True).strip()
    tree = subprocess.check_output(['git', 'rev-parse', 'HEAD^{tree}'], cwd=ROOT,
                                   text=True).strip()
    signature = subprocess.check_output(['git', 'log', '-1', '--format=%G?%n%B'],
                                        cwd=ROOT, text=True)
    require(not subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT) and
            signature.startswith('G\n') and 'Signed-off-by:' in signature and
            'Assisted-by: AI' in signature and
            receipt['format'] == 'compact-ledger9-bytes-large-reconstruction-proof-gate/v1' and
            receipt['status'] == 'passed' and receipt['proof_execution'] is True and
            receipt['funded_application'] is True and
            (head, tree) == (receipt['head'], receipt['tree']) ==
            (semantic['head'], semantic['tree']) and
            digest(semantic_receipt_path) == receipt['semantic_gate_sha256'] and
            semantic['original_files'] == originals() and
            semantic['ordered_transcript_parity'] is True and
            semantic['rust_test_stack_bytes'] == 2 * 1024 * 1024 and
            receipt['proof_worker_stack_bytes'] == 64 * 1024 * 1024 and
            semantic['zkir_model']['k'] == 16 and
            semantic['zkir_model']['rows'] == 40652,
            'source, original or exact-head semantic/proof receipt differs')
    generated = semantic_root / 'generated/paired'
    proof = proof_root / 'proof'
    wrong_source = ROOT / 'tools/compact-rust-backend/tests/ledger9_bytes_large_reconstruction_wrong_verifier.compact'
    require(files(generated) == receipt['generated_input_sha256'] ==
            semantic['paired_generated'] and
            files(proof) == receipt['artifact_hashes'] and
            files(proof_root / 'wrong-generated') == receipt['wrong_generated_sha256'] and
            digest(wrong_source) == receipt['wrong_source_sha256'] and
            digest(Path(semantic['zkir_store']) / 'bin/zkir') == receipt['zkir_sha256'] and
            digest(proof / f'zkir/{CIRCUIT}.zkir') ==
            digest(generated / f'zkir/{CIRCUIT}.zkir') == semantic['zkir_sha256'] and
            digest(proof / f'{CIRCUIT}/contract/compact-rust-ir.json') ==
            digest(generated / 'contract/compact-rust-ir.json'),
            'generated source/ZKIR/proof artifact bytes changed')
    compiler_recipe = semantic['compiler_recipe']
    compiler = Path(compiler_recipe['store']) / 'bin/compactc'
    require(digest(Path(compiler_recipe['path'])) == compiler_recipe['sha256'] and
            digest(compiler) == semantic['compiler_sha256'] and
            digest(compiler.with_name('compactc-scheme')) == semantic['scheme_sha256'],
            'source-built compiler recipe or binary changed')
    require(digest(proof_root / 'proof-runner') == receipt['runner_sha256'] and
            digest(proof_root / 'proof-runner.d') == receipt['runner_depfile_sha256'] and
            digest(proof_root / 'binding/inputs.json') ==
            receipt['runner_inventory_sha256'] and
            digest(proof_root / 'binding/source-hashes.json') ==
            receipt['source_hashes_sha256'],
            'physical runner or inventory changed')
    binding = json.loads((proof_root / 'binding/inputs.json').read_text())
    tracked = json.loads((proof_root / 'binding/source-hashes.json').read_text())
    require(len(tracked) == receipt['source_count'] and
            all((ROOT / name).is_file() and digest(ROOT / name) == expected
                for name, expected in tracked.items()) and
            binding['physical_input_origins'] == receipt['runner_physical_input_origins'],
            'complete tracked source or physical provenance changed')
    physical = binding['physical_inputs_before_build']
    require(len(physical) == receipt['runner_physical_input_count'] and
            all(Path(path).is_file() and digest(Path(path)) == expected
                for path, expected in physical.items()),
            'physical runner input inventory changed')
    for name, expected in receipt['command_logs_sha256'].items():
        require(digest(proof_root / name) == expected, f'command log changed: {name}')
    require(digest(proof_root / 'material.json') == receipt['material_log_sha256'] and
            digest(proof_root / 'resource-preflight.json') ==
            receipt['resource_preflight_sha256'] and
            receipt['resource_preflight']['parameter_k'] == 16 and
            receipt['resource_preflight']['model_rows'] == 40652 and
            receipt['resource_preflight']['free_gib'] >= 22,
            'k16 preflight or material log changed')
    material_cache = Path(receipt['material']['cache_directory'])
    require(all(digest(material_cache / name) == expected
                for name, expected in receipt['material_hashes'].items()) and
            digest(material_cache / 'bls_midnight_2p16') ==
            '09c877216d6589b370263e18af40a030a901b41a7a7c37ef58c9901db41f05c6',
            'official proving material changed')
    summary = receipt['proof_summary']
    require(summary == json.loads((proof / 'bytes-large-reconstruction-proof-result.json').read_text())
            and summary['selected_source_export_proved'] == CIRCUIT and
            summary['k'] == 16 and summary['result'] ==
            summary['ledger_result'] == [1, 4, 9, 16, 25, 36, 49, 14, 31, 50] and
            summary['public_verify_ops'] == 315 and
            summary['ordered_transcript_parity'] is True and
            summary['strict_application'] == 'default' and
            summary['replay_refusal'] == 'IntentAlreadyExists' and
            summary['positive_fee_dust_spends'] == 1,
            'proof/funded result changed')
    output.mkdir(parents=True)
    runner = proof_root / 'proof-runner'
    require('.stack_size(64 * 1024 * 1024)' in
            (ROOT / 'tools/compact-rust-proof-smoke/src/main.rs').read_text(),
            'proof worker stack source changed')
    env = dict(os.environ, MIDNIGHT_PP=str(material_cache))
    env.pop('MIDNIGHT_PROOF_SERVER', None)
    env.pop('RUST_MIN_STACK', None)
    good = invoke(runner, proof, CIRCUIT, output / 'independent-verify.log', env)
    wrong = invoke(runner, proof, WRONG, output / 'wrong-verifier.log', env)
    require(good.returncode == 0 and wrong.returncode != 0,
            'independent correct/wrong verifier controls failed')
    tampered = output / 'tampered-proof'
    (tampered / 'zkir').mkdir(parents=True)
    (tampered / 'keys').mkdir()
    for relative in (f'zkir/{CIRCUIT}.zkir', f'zkir/{CIRCUIT}.bzkir',
                     f'keys/{CIRCUIT}.verifier', f'{STEM}.proof', f'{STEM}.inputs'):
        shutil.copyfile(proof / relative, tampered / relative)
    bad_proof = tampered / f'{STEM}.proof'
    data = bytearray(bad_proof.read_bytes())
    require(bool(data), 'empty standalone proof')
    data[0] ^= 1
    bad_proof.write_bytes(data)
    corrupt = invoke(runner, tampered, CIRCUIT, output / 'tampered-proof.log', env)
    require(corrupt.returncode != 0, 'independent verifier accepted altered proof')
    report = {'format':'compact-ledger9-bytes-large-reconstruction-proof-audit/v1',
              'status':'passed', 'head':head, 'tree':tree,
              'proof_gate_sha256':digest(proof_receipt_path),
              'semantic_gate_sha256':digest(semantic_receipt_path),
              'runner_sha256':digest(runner),
              'physical_input_count':len(physical),
              'independent_verify_log_sha256':digest(output / 'independent-verify.log'),
              'wrong_verifier_log_sha256':digest(output / 'wrong-verifier.log'),
              'tampered_proof_log_sha256':digest(output / 'tampered-proof.log'),
              'tampered_inputs_sha256':files(tampered),
              'proof_execution':True, 'funded_application':True,
              'native_linux':False}
    (output / 'receipt.json').write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status':'passed', 'head':head, 'output':str(output)}))


if __name__ == '__main__':
    main()
