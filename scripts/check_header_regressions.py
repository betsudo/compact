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

"""Report inherited full-header debt and reject new violations on pull requests."""
import argparse
import hashlib
import io
import os
import json
import logging
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from add_headers import HeaderManager


# This sole copy is a byte-authenticated historical compiler regression, not
# newly authored code. Preserve its original source/proof binding. All three
# byte identities (base authority, current authority, current copy) must match.
RETAINED_SOURCE_COPIES = {
    'examples/rust_backend/vector_component_products/original_regression.compact': (
        'tools/compact-rust-backend/tests/stateful_vector_product_elements/unsupported/product_below_component.compact',
        '6fb741f1171b87312d8e9e0815c148e7d2f010cfcecfefd135f819a5d26df07c'),
}


def base_contents(root, base, names):
    # Resolve paths once, then read unique blobs in one process. Filenames never
    # become line-delimited batch commands, so spaces/newlines remain unambiguous.
    tree = subprocess.run(['git', 'ls-tree', '-rz', '--full-tree', base], cwd=root,
                          capture_output=True, check=True).stdout
    wanted = set(names)
    ids = {}
    for entry in tree.split(b'\0'):
        if not entry:
            continue
        metadata, name = entry.split(b'\t', 1)
        name = os.fsdecode(name)
        if name in wanted:
            mode, kind, oid = metadata.split()
            if kind != b'blob':
                raise ValueError('base path is not a blob: ' + name)
            ids[name] = oid
    unique = list(dict.fromkeys(ids.values()))
    blobs = {}
    if unique:
        raw = subprocess.run(['git', 'cat-file', '--batch'], cwd=root,
                             input=b'\n'.join(unique) + b'\n', capture_output=True, check=True).stdout
        stream = io.BytesIO(raw)
        for expected in unique:
            header = stream.readline().split()
            if len(header) != 3 or header[:2] != [expected, b'blob']:
                raise ValueError('cannot read base blob')
            size = int(header[2])
            content = stream.read(size)
            if len(content) != size or stream.read(1) != b'\n':
                raise ValueError('incomplete base blob')
            blobs[expected] = content
        if stream.read():
            raise ValueError('unexpected base blob output')
    return {name: blobs[ids[name]] if name in ids else None for name in wanted}


def base_bytes(root, base, name):
    return base_contents(root, base, [name])[name]


def rust_header_start(content):
    """Skip a Rust shebang, but not an inner attribute separated by trivia."""
    if not content.startswith('#!'):
        return 0
    index = 2
    while index < len(content):
        if content[index].isspace():
            index += 1
        elif content.startswith('//', index):
            end = content.find('\n', index)
            index = len(content) if end == -1 else end + 1
        elif content.startswith('/*', index):
            index += 2
            depth = 1
            while index < len(content) and depth:
                if content.startswith('/*', index):
                    depth += 1
                    index += 2
                elif content.startswith('*/', index):
                    depth -= 1
                    index += 2
                else:
                    index += 1
            if depth:
                return 0
        else:
            break
    if content.startswith('[', index):
        return 0
    newline = content.find('\n')
    return len(content) if newline == -1 else newline + 1


def rust_header_placement_violations(root, base, manager):
    """Check only new/modified tracked Rust files; leave shared writer rules intact."""
    names = subprocess.run(
        ['git', 'diff', '--name-only', '--no-renames', '--diff-filter=ACMRT', '-z',
         base, '--', '*.rs'], cwd=root, capture_output=True, check=True).stdout
    invalid = []
    pattern = manager._get_header_pattern('//')
    for name in names.split(b'\0'):
        if not name:
            continue
        path = Path(root) / os.fsdecode(name)
        if path.is_symlink():
            raise ValueError('changed Rust source must be a regular file: ' + str(path))
        try:
            # Match the existing validator's universal-newline text view without
            # rewriting source bytes; Rust also permits a leading UTF-8 BOM.
            content = path.read_text(encoding='utf-8-sig')
        except UnicodeDecodeError:
            invalid.append(path)
            continue
        start = rust_header_start(content)
        while start < len(content) and content[start].isspace():
            start += 1
        if not pattern.match(content, start):
            invalid.append(path)
    return invalid


def classify(root, base, missing):
    names = [Path(path).relative_to(root).as_posix() for path in missing]
    authorities = [RETAINED_SOURCE_COPIES[name][0] for name in names if name in RETAINED_SOURCE_COPIES]
    originals = base_contents(root, base, names + authorities)
    inherited, retained, violations = [], [], []
    for path in sorted(missing):
        path = Path(path)
        name = path.relative_to(root).as_posix()
        current = path.read_bytes()
        original = originals[name]
        row = dict(path=name, sha256=hashlib.sha256(current).hexdigest())
        if original is not None and original == current:
            inherited.append(row)
            continue
        authority = RETAINED_SOURCE_COPIES.get(name)
        if authority is not None:
            source, expected = authority
            current_authority = root / source
            if (not path.is_symlink() and current_authority.is_file() and not current_authority.is_symlink()
                    and row['sha256'] == expected and originals[source] == current
                    and current_authority.read_bytes() == current):
                retained.append(dict(row, authority=source, reason='pinned exact historical source copy'))
                continue
        violations.append(row)
    return dict(scope='PR header regressions; not full-tree license compliance', base_commit=base,
                missing_full_headers=len(missing), inherited_unchanged=inherited, retained_source_copies=retained,
                new_or_modified_violations=violations, passed=not violations)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--base', required=True, help='frozen full PR base commit SHA')
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args(argv)
    if not re.fullmatch(r'[0-9a-f]{40}|[0-9a-f]{64}', args.base):
        parser.error('full base commit SHA required')
    subprocess.run(['git', 'cat-file', '-e', args.base + '^{commit}'], cwd=ROOT, check=True)
    # Preserve the original full-template validator and writer. Add placement
    # enforcement only for changed Rust sources, including headers inside strings.
    # Keep per-file debt
    # in the receipt instead of flooding the log with inherited warnings.
    logging.getLogger().setLevel(logging.ERROR)
    manager = HeaderManager(str(ROOT / 'header_config.json'), str(ROOT))
    misplaced = rust_header_placement_violations(ROOT, args.base, manager)
    missing = list(dict.fromkeys([*manager.validate_headers(), *misplaced]))
    report = classify(ROOT, args.base, missing)
    report['rust_placement_violations'] = [p.relative_to(ROOT).as_posix() for p in misplaced]
    with args.report.open('x') as output:
        json.dump(report, output, indent=2)
        output.write('\n')
    print(json.dumps(dict(scope=report['scope'], missing_full_headers=report['missing_full_headers'],
                         inherited_count=len(report['inherited_unchanged']), retained_source_copies=report['retained_source_copies'],
                         violations=report['new_or_modified_violations'], passed=report['passed']), indent=2))
    return 0 if report['passed'] else 1


if __name__ == '__main__':
    try:
        sys.exit(main())
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        raise SystemExit(str(error)) from None
