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

"""Independently rehash the exact-head slow Bytes paired semantic receipt."""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess


ROOT = Path(__file__).resolve().parents[2]
PINNED = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'
EXPECTED = [1, 4, 9, 16, 25, 36, 49, 14, 31, 50]


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


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--gate', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    gate = args.gate.resolve(strict=True)
    require(not args.output.exists() and not args.output.is_symlink(),
            'independent audit output must be fresh')
    receipt_path = gate / 'receipt.json'
    receipt = json.loads(receipt_path.read_text())
    require(receipt['format'] == 'ledger9-bytes-large-reconstruction-paired/v1' and
            receipt['status'] == 'paired_semantics_qualified' and
            receipt['upstream_commit'] == PINNED and
            receipt['ordered_transcript_parity'] is True and
            receipt['rust_test_stack_bytes'] == 2 * 1024 * 1024 and
            receipt['rust_test_stack_modes'] == ['default', 'explicit-2m'] and
            receipt['proof_execution'] is False and
            receipt['funded_application'] is False,
            'paired semantic receipt identity/status changed')
    head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    tree = subprocess.check_output(['git', 'rev-parse', 'HEAD^{tree}'],
                                   cwd=ROOT, text=True).strip()
    require((head, tree) == (receipt['head'], receipt['tree']) and
            not subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT),
            'source head/tree or cleanliness changed')
    for name, expected in receipt['original_files'].items():
        path = ROOT / name
        upstream = subprocess.check_output(['git', 'show', f'{PINNED}:{name}'], cwd=ROOT)
        require(path.is_file() and not path.is_symlink() and
                path.read_bytes() == upstream and digest(path) == expected,
                f'pinned original changed: {name}')
    require(len(receipt['original_files']) == 3, 'original inventory differs')
    for name, expected in receipt['local_sources'].items():
        require(digest(ROOT / name) == expected, f'local source changed: {name}')
    for kind, directory in (('typescript_generated', 'typescript'),
                            ('paired_generated', 'paired')):
        require(files(gate / 'generated' / directory) == receipt[kind],
                f'{kind} artifact inventory changed')
    raw = gate / 'generated/typescript/zkir/bytes_manual_update_large_reconstruction.zkir'
    paired = gate / 'generated/paired/zkir/bytes_manual_update_large_reconstruction.zkir'
    require(digest(raw) == digest(paired) == digest(gate / 'model-input.zkir') ==
            receipt['zkir_sha256'] and raw.stat().st_size == receipt['zkir_size'] and
            (receipt['zkir_model']['k'], receipt['zkir_model']['rows']) == (16, 40652),
            'paired raw ZKIR/model binding changed')
    require(digest(gate / 'model-input.zkir') ==
            receipt['zkir_model']['model_input_sha256'] and
            digest(gate / 'model-input.bzkir') ==
            receipt['zkir_model']['bzkir_sha256'] and
            digest(gate / 'zkir-model.log') ==
            receipt['zkir_model']['log_sha256'] and
            digest(Path(receipt['zkir_store']) / 'bin/zkir') ==
            receipt['zkir_tool_sha256'],
            'released ZKIR model input/tool/log changed')
    require(digest(gate / 'typescript-runtime.json') ==
            receipt['typescript_runtime_sha256'] and
            json.loads((gate / 'typescript-runtime.json').read_text()) ==
            receipt['typescript_observation'] and
            receipt['typescript_observation']['result'] ==
            receipt['typescript_observation']['ledger'] == EXPECTED and
            receipt['typescript_observation']['public_operations'] == 315,
            'TypeScript result/state/transcript changed')
    ts_transcript_path = gate / 'typescript-transcript.json'
    rust_transcript_path = gate / 'rust-transcript.json'
    explicit_transcript_path = gate / 'rust-transcript-2m.json'
    ts_transcript = json.loads(ts_transcript_path.read_text())
    rust_transcript = json.loads(rust_transcript_path.read_text())
    explicit_transcript = json.loads(explicit_transcript_path.read_text())
    require(digest(ts_transcript_path) == receipt['typescript_transcript_sha256'] and
            digest(rust_transcript_path) == receipt['rust_transcript_sha256'] and
            digest(explicit_transcript_path) == receipt['rust_transcript_explicit_2m_sha256'] and
            isinstance(ts_transcript, list) and len(ts_transcript) == 315 and
            ts_transcript == rust_transcript == explicit_transcript,
            'full ordered Rust/TypeScript transcript contents changed')
    for name, path in (
        ('typescript_runtime_log_sha256', gate / 'typescript-runtime.log'),
        ('rust_consumer_manifest_sha256', gate / 'rust-consumer/Cargo.toml'),
        ('rust_consumer_source_sha256', gate / 'rust-consumer/tests/large_reconstruction.rs'),
        ('rust_consumer_lock_sha256', gate / 'rust-consumer/Cargo.lock'),
        ('cargo_metadata_log_sha256', gate / 'cargo-metadata.log'),
        ('cargo_build_log_sha256', gate / 'cargo-build.log'),
        ('cargo_test_log_sha256', gate / 'cargo-test.log'),
        ('cargo_test_explicit_2m_log_sha256', gate / 'cargo-test-2m.log'),
    ):
        require(digest(path) == receipt[name], f'consumer input/log changed: {name}')
    require(digest(gate / 'typescript-compile.log') ==
            receipt['typescript_compile']['log_sha256'] and
            digest(gate / 'paired-compile.log') ==
            receipt['paired_compile']['log_sha256'],
            'compiler logs changed')
    require('test result: ok. 1 passed; 0 failed;' in
            (gate / 'cargo-test.log').read_text() and
            'test result: ok. 1 passed; 0 failed;' in
            (gate / 'cargo-test-2m.log').read_text(),
            'default or explicit-2-MiB Rust consumer result changed')
    require(digest(gate / 'rust-test-binary') == receipt['rust_test_binary_sha256'],
            'retained exact Rust test binary changed')
    require(digest(Path(receipt['compiler_recipe']['path'])) ==
            receipt['compiler_recipe']['sha256'], 'compiler recipe changed')
    compiler = Path(receipt['compiler_recipe']['store']) / 'bin/compactc'
    require(digest(compiler) == receipt['compiler_sha256'] and
            digest(compiler.with_name('compactc-scheme')) == receipt['scheme_sha256'],
            'compiler binary changed')
    runtime = Path(receipt['runtime']['path'])
    require(digest(runtime / 'package.json') ==
            receipt['runtime']['package_sha256'] and
            files(runtime / 'dist') == receipt['runtime']['dist_files'],
            'TypeScript runtime package or dist contents changed')
    result = {'format': 'ledger9-bytes-large-reconstruction-independent-audit/v1',
              'status': 'passed', 'head': head, 'tree': tree,
              'gate_receipt_sha256': digest(receipt_path),
              'auditor_sha256': digest(Path(__file__)),
              'raw_zkir_sha256': digest(raw),
              'proof_execution': False, 'funded_application': False}
    args.output.write_text(json.dumps(result, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': 'passed', 'head': head}))


if __name__ == '__main__':
    main()
