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

"""Refuse uncertified Vector loop-index domains before paired output publication.

These are negative capability controls. The intact upstream source and its
ordinary/recorded results are checked by check_upstream_ledger9_for_keywords.py.
"""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess


ROOT = Path(__file__).resolve().parents[2]
ORIGINAL = ROOT / 'test-contracts/keywords/for/generic_bound/for_generic_bound.compact'
PINNED = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def variants(source):
    replace = lambda old, new: source.replace(old, new, 1)
    return {
        'effectful_operand': (replace('v[i]', 'identity(v)[i]').replace(
            'circuit sum_range',
            'circuit identity(v: Vector<4, Uint<8>>): Vector<4, Uint<8>> { return v; }\n\n'
            'circuit sum_range', 1), True, 'no unique constant-valued Vector index'),
        'shadowed_formal': (replace('v[i]', 'v[j]').replace(
            'total = (total +', 'const j = i;\n    total = (total +', 1),
            True, 'no unique constant-valued Vector index'),
        'out_of_range': (replace('sum_range<1, 3>', 'sum_range<1, 5>'),
                         False, 'invalid vector index 4'),
        'cap_eight': (source.replace('Vector<4, Uint<8>>', 'Vector<8, Uint<8>>')
                       .replace('sum_range<1, 3>', 'sum_range<0, 8>')
                       .replace('[1, 2, 3, 4]',
                                '[1, 2, 3, 4, 5, 6, 7, 8]'),
                      True, None),
        'cap_nine': (source.replace('Vector<4, Uint<8>>', 'Vector<10, Uint<8>>')
                     .replace('sum_range<1, 3>', 'sum_range<0, 9>')
                     .replace('[1, 2, 3, 4]',
                              '[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]'),
                     True, 'no unique constant-valued Vector index'),
        'guarded_item': (replace('v[i]', 'v[i - 0]'),
                         True, 'no unique constant-valued Vector index'),
        # The circuit can evaluate the cast, but the Fold source is no longer
        # an exact tuple of unsigned literals certified by the Rust bridge.
        'mixed_literal': (replace('S..E', '[1, 1 as Uint<8>]'),
                          True, 'no unique constant-valued Vector index'),
        'nested_scope': (replace(
            'total = (total + (v[i] as Uint<16>)) as Uint<16>;',
            'for (const j of 0..2) {\n'
            '      total = (total + (v[j] as Uint<16>)) as Uint<16>;\n'
            '    }'), True, 'no unique constant-valued Vector index'),
        'direct_dynamic': (
            'export pure circuit choose(v: Vector<4, Uint<8>>, i: Uint<8>): Uint<8> '
            '{ return v[i]; }\n', False,
            'vector index did not reduce to a constant'),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    require(not args.output.exists(), 'fresh output required')
    compiler = args.compiler.resolve(strict=True)
    original = subprocess.check_output(
        ['git', 'show', f'{PINNED}:{ORIGINAL.relative_to(ROOT)}'], cwd=ROOT)
    require(ORIGINAL.read_bytes() == original, 'pinned original changed')
    source = original.decode()
    args.output.mkdir(parents=True)
    results = {}
    for name, (variant, ts_passes, refusal) in variants(source).items():
        require(variant != source or name == 'direct_dynamic', f'empty mutation: {name}')
        folder = args.output / name
        folder.mkdir()
        contract = folder / f'{name}.compact'
        contract.write_text(variant)
        results[name] = {'source_sha256': sha256(contract)}
        for mode, targets in (('typescript', ('ts',)), ('paired', ('ts', 'rust'))):
            destination = folder / mode
            command = [str(compiler), '--skip-zk']
            for target in targets:
                command += ['--target', target]
            command += [str(contract), str(destination)]
            completed = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
            log = folder / f'{mode}.log'
            log.write_text(completed.stdout + completed.stderr)
            expected_success = (ts_passes and
                                (mode == 'typescript' or refusal is None))
            require((completed.returncode == 0) == expected_success,
                    f'unexpected {name}/{mode} exit {completed.returncode}')
            if not expected_success:
                require(refusal is not None, f'missing refusal contract for {name}')
                require(refusal in log.read_text(), f'missing {name}/{mode} refusal')
                require(not destination.exists(), f'{name}/{mode} published refused output')
            results[name][mode] = {'exit': completed.returncode,
                                   'log_sha256': sha256(log),
                                   'output_published': destination.exists()}
    receipt = {'status': 'passed', 'upstream_commit': PINNED,
               'original_sha256': hashlib.sha256(original).hexdigest(),
               'compiler_sha256': sha256(compiler), 'cases': results,
               'limit': 'literal-domain dispatch admits at most eight items'}
    (args.output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': 'passed', 'cases': len(results)}))


if __name__ == '__main__':
    main()
