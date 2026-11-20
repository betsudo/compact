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

"""Refuse uncertified Bytes Fold domains before paired output publication."""

import argparse
import json
from pathlib import Path
import subprocess

from check_upstream_ledger9_bytes_indexed_access_gap import originals
from check_upstream_ledger9_field_casts import require, sha256


ROOT = Path(__file__).resolve().parents[2]
ORIGINAL = ROOT / 'test-contracts/primitives/bytes/loops/indexed_access/bytes_loop_indexed_access.compact'
DYNAMIC = ROOT / 'tools/compact-rust-backend/tests/bytes_loops/dynamic_index_refusal.compact'


def variants(source: str) -> dict[str, tuple[str, bool, str | None]]:
    identity = 'circuit identity(b: Bytes<9>): Bytes<9> { return b; }\n\n'
    return {
        'original': (source, True, None),
        'shadowed_formal':
            (source.replace('loop_result = loop_result + bytes2[b];',
                            'const j = b;\n        loop_result = loop_result + bytes2[j];'),
             True, 'Rust backend requires a constant Bytes index'),
        'effectful_operand':
            (source.replace('export circuit bytes_loop_indexed_access()',
                            identity + 'export circuit bytes_loop_indexed_access()')
                   .replace('bytes2[b]', 'identity(bytes2)[b]'),
             True, 'Rust backend requires a constant Bytes index'),
        'nonliteral_binding':
            (source.replace('export circuit bytes_loop_indexed_access()',
                            'circuit identity(b: Bytes<4>): Bytes<4> { return b; }\n\n'
                            'export circuit bytes_loop_indexed_access()')
                   .replace('const b1 = Bytes[1, 2, 3, 4];',
                            'const b1 = identity(Bytes[1, 2, 3, 4]);'),
             True, 'Rust backend requires a constant Bytes index'),
        'out_of_bounds':
            (source.replace('Bytes[5, 6, 7, 8]', 'Bytes[5, 6, 7, 9]'),
             False, 'invalid Bytes index 9 for a Bytes value of length 9'),
        'nine_item_cap':
            (source.replace('Bytes[5, 6, 7, 8]', 'Bytes[5, 6, 7, 8, 9]')
                   .replace('loop_result == 32 as Field', 'loop_result == 41 as Field')
                   .replace('Indexed accumulation should equal 32',
                            'Indexed accumulation should equal 41'),
             True, 'Rust backend requires a constant Bytes index'),
        'dynamic_frontend':
            (DYNAMIC.read_text(), False,
             'Bytes index did not reduce to a constant nonnegative value at compile time'),
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    require(not args.output.exists() and not args.output.is_symlink(), 'fresh output required')
    pinned = originals()
    source = ORIGINAL.read_text()
    compiler = args.compiler.resolve(strict=True)
    args.output.mkdir(parents=True)
    results = {}
    for name, (variant, ts_passes, rust_refusal) in variants(source).items():
        directory = args.output / name
        directory.mkdir()
        path = directory / f'{name}.compact'
        path.write_text(variant)
        result = {'source_sha256': sha256(path)}
        for mode, targets in (('typescript', ('ts',)), ('paired', ('ts', 'rust'))):
            destination = directory / mode
            command = [str(compiler)]
            for target in targets:
                command += ['--target', target]
            command += ['--feature-zkir-v3', '--skip-zk', '--compact-path',
                        str(ROOT / 'compiler'), str(path), str(destination)]
            completed = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
            log = directory / f'{mode}.log'
            log.write_text(completed.stdout + completed.stderr)
            success = ts_passes and (mode == 'typescript' or rust_refusal is None)
            require((completed.returncode == 0) == success,
                    f'unexpected {name}/{mode} exit: {log}')
            if not success:
                require(rust_refusal in completed.stderr and not destination.exists(),
                        f'{name}/{mode} refusal or no-product contract changed')
            result[mode] = {'exit': completed.returncode, 'log_sha256': sha256(log),
                            'output_published': destination.exists()}
        results[name] = result
    receipt = {'status': 'passed', 'pinned_originals': pinned,
               'compiler_sha256': sha256(compiler), 'cases': results,
               'limit': 'only a pure immutable Bytes binding chain with one to eight literal bytes is certified'}
    (args.output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': 'passed', 'cases': len(results)}))


if __name__ == '__main__':
    main()
