#!/usr/bin/env python3

# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#  	http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Bind the two pinned mixed native collection statements to an exact head."""

import argparse
import hashlib
import json
from pathlib import Path
import platform
import re
import subprocess

from collection_gate_binding import compiler_recipe, git, require, sha


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / 'compiler/test.ss'
CAPTURE = ROOT / 'runtime-rs/tests/fixtures/capture-ledger9-mixed-original-proofs.mjs'
UPSTREAM = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'
BASE = 'f22ad3ff822171758ab87ec7c4b9696323f3f036'
CASES = {
    'k1': {
        'source': '1bc9dacd4e33970b8f7561ad76142e2852ae20b4f781231d431d74002d01fe68',
        'callback': 'efbaecf3f822c69963bc91e95de00b0a52ba25a877ed9261e38d6337f1f6bc7e',
        'artifacts': {
            'contract/compact-rust-ir.json': 'c041950021361bc70475cb36bc1e878d42f35182a9ba7bed4b19aefa8ae5b89d',
            'contract/lib.rs': '5b931ebeb052640c06ccbea63433f60aa109085a1131185040a412fafcfe1c33',
            'contract/rust-capabilities.json': '8e545f141a10c33d10f58984eeebb49d396eb5bde2be4310435dec4e899c482c',
            'contract/index.js': '7e6ab18ccc10c86271315aa9fe48d381d6cf6eddf6d5afd91c04e8f0ccfba8da',
            'zkir/test.zkir': 'd6bdfd48ebb44f8dcd2e3372c5e9f16199e1a69a427ff0b8382ed88dbbcfbabd',
        },
        'preimage': '45a19e5339bd938b4cac58ad710acc9a98ca502d260281cc0bcbe99a7d846492',
    },
    'r1': {
        'source': '38a8cdaa58ec14761f253dae5acdd42ef436cf614a374f70fd82e2a44620114d',
        'callback': '9d272e6d5659647fe5b79b1af658292ec27d721a4dac2551a8df0d435872c699',
        'artifacts': {
            'contract/compact-rust-ir.json': 'd1f5acbce16cc7ca43a5672eb6c6005b4a35454d9c8d67a4248fa6539f35285a',
            'contract/lib.rs': '90bb6390059417f6f4567bb0a021a55318dc461a3b4b050103f4fcc9a6c384c7',
            'contract/rust-capabilities.json': '68447f3b1a638085d7eb613a3721625b6c3104aad39427060c2e6f867bc082f9',
            'contract/index.js': '87fdde6ee926d214825db10377b6f5fc13baa1887f616dc885dc3af0b89bc16a',
            'zkir/test.zkir': 'c52e2adaee021294096f075a5ef0f4e7205831574f035d5648505decf2460165',
        },
        'preimage': 'a53380159a09943bf8bd56abc4045c3f21a027cf9b82f04b642ca674894baed2',
    },
}


def run(command: list[str], log: Path) -> str:
    completed = subprocess.run(command, cwd=ROOT, text=True, capture_output=True)
    log.write_text(completed.stdout + completed.stderr)
    require(completed.returncode == 0, f'{command[0]} failed; inspect {log}')
    return completed.stdout + completed.stderr


def original_cases() -> dict[str, tuple[bytes, bytes]]:
    """Extract exact original bodies/assertions from the maintained Scheme tests."""
    scheme = SOURCE.read_text()
    start = scheme.index(';; Pinned mainstream Ledger 9 mixed native collections.')
    marker = "(if (feature-zkir-v3)\n    (test\n    '("
    result = {}
    for name in CASES:
        opening = scheme.index(marker, start) + len(marker)
        stage = scheme.index('    (stage-javascript', opening)
        closing = scheme.index('    (lambda args #t)', stage)
        quoted = r'^\s*("(?:[^"\\]|\\.)*")\s*$'
        source_lines = [json.loads(match.group(1)) for match in
                        re.finditer(quoted, scheme[opening:stage], re.M)]
        stage_lines = [json.loads(match.group(1)) for match in
                       re.finditer(quoted, scheme[stage:closing], re.M)]
        callback_start = next(index for index, line in enumerate(stage_lines)
                              if line.startswith("test('Nested"))
        callback_end = stage_lines.index('});', callback_start)
        require(len(source_lines) == 58 and len(stage_lines) == 32,
                f'{name} original Scheme test shape changed')
        result[name] = ('\n'.join(source_lines).encode() + b'\n',
                        '\n'.join(stage_lines[callback_start:callback_end+1]).encode() + b'\n')
        start = closing + 1
    return result


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler-receipt', type=Path, required=True)
    parser.add_argument('--compiler', type=Path, required=True)
    parser.add_argument('--runtime-package', type=Path, required=True)
    parser.add_argument('--zkir-wasm', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    require(not git('status', '--porcelain'), 'source checkout must be clean')
    head, tree = git('rev-parse', 'HEAD'), git('rev-parse', 'HEAD^{tree}')
    require(git('log', '-1', '--format=%G?') == 'G' and
            'Signed-off-by:' in git('log', '-1', '--format=%B'),
            'source head must be signed and DCO complete')
    require(subprocess.run(['git', 'merge-base', '--is-ancestor', BASE, head],
                           cwd=ROOT).returncode == 0, 'source does not descend from qualified base')
    compiler = args.compiler.resolve(strict=True)
    compiler_recipe(args.compiler_receipt.resolve(strict=True), compiler,
                    compiler.with_name('compactc-scheme'),
                    source_head=head, current_head=head)
    runtime_package = args.runtime_package.resolve(strict=True)
    require(json.loads((runtime_package / 'package.json').read_text())['version'] == '0.18.0',
            'Runtime 0.18.0 package changed')
    wasm = args.zkir_wasm.resolve(strict=True)
    require(json.loads((wasm / 'package.json').read_text())['version'] == '3.2.0',
            'ZKIR wasm 3.2.0 package changed')
    output = args.output.resolve()
    require(not args.output.is_symlink() and not output.exists(), 'output must be absent')
    output.mkdir(parents=True)
    (output / 'package.json').write_text('{"type":"module"}\n')
    module = output / 'node_modules/@midnight-ntwrk/compact-runtime'
    module.parent.mkdir(parents=True)
    module.symlink_to(runtime_package, target_is_directory=True)
    artifacts = {}
    captures = {}
    extracted = original_cases()
    originals = output / 'original'
    originals.mkdir()
    for name, pins in CASES.items():
        body, callback = extracted[name]
        require(hashlib.sha256(body).hexdigest() == pins['source'] and
                hashlib.sha256(callback).hexdigest() == pins['callback'],
                f'{name} pinned source or original assertions changed')
        source = originals / f'{name}.compact'
        source.write_bytes(body)
        (originals / f'{name}.callback.js').write_bytes(callback)
        destination = output / name
        run([str(compiler), '--target', 'ts', '--target', 'rust',
             '--rust-runtime-root', str(ROOT), '--rust-require-recording',
             '--feature-zkir-v3', '--skip-zk',
             str(source), str(destination)],
            output / f'{name}-compile.log')
        for relative, digest in pins['artifacts'].items():
            require(sha(destination / relative) == digest,
                    f'{name} generated {relative} changed')
            artifacts[f'{name}/{relative}'] = digest
        capture = output / f'{name}-capture'
        run(['node', str(CAPTURE), name,
             str(destination / 'contract/index.js'),
             str(destination / 'zkir/test.zkir'), str(wasm), str(capture)],
            output / f'{name}-capture.log')
        result = json.loads((capture / 'capture.json').read_text())
        require(result['honestPreimageSha256'] == pins['preimage'] and
                result['rawZkirSha256'] == pins['artifacts']['zkir/test.zkir'] and
                result['checks'] == ['all original callback assertions and nested Map point',
                                     'honest ZKIR accepted', 'changed nested Map key refused'],
                f'{name} original capture changed')
        captures[name] = result
    receipt = {'format': 'ledger9-mixed-original-preflight/v1',
               'source_head': head, 'source_tree': tree, 'upstream': UPSTREAM,
               'platform': platform.platform(),
               'compiler_receipt_sha256': sha(args.compiler_receipt),
               'compiler_sha256': sha(compiler),
               'runtime_package': str(runtime_package),
               'capture_source_sha256': sha(CAPTURE),
               'gate_source_sha256': sha(Path(__file__)),
               'artifacts': artifacts, 'captures': captures,
               'limitations': ['Original k=17/k=18 fresh proofs and funded application are separate gates.',
                               'Local Darwin arm64 result; native Linux qualification required.']}
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(f'original mixed native preflight passed at {head}; {output}/receipt.json')


if __name__ == '__main__':
    main()
