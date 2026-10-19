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
"""Existing optional Forge PR path predicate, using immutable base/merge SHAs."""
import argparse
import fnmatch
from pathlib import Path
import re
import subprocess

PATTERNS = ('compiler/**', 'third_party/compiler/**', 'srcMaps/**', 'runtime/**', 'test-center/**',
            'runtime-rs/**', 'runtime-rs-macros/**', 'testkit-rs/**', 'tools/compact-rust-backend/**',
            'tools/compact-rust-proof-smoke/**', 'tests-rust-backend/**', 'examples/**', 'Cargo.toml',
            'Cargo.lock', 'flake.nix', 'flake.lock', '_sources/**', 'nix/**', '.github/actions/**',
            '.github/workflows/forge-local-ci.yml', '.github/scripts/**',
            '.github/workflows/compiler-producer.yml', '.github/workflows/build-compiler.yml')


def selected(paths):
    return any(fnmatch.fnmatchcase(path, pattern) for path in paths for pattern in PATTERNS)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--base', required=True)
    parser.add_argument('--merge', required=True)
    parser.add_argument('--github-output', type=Path, required=True)
    args = parser.parse_args()
    if not all(re.fullmatch('[0-9a-f]{40}', value) for value in (args.base, args.merge)):
        raise ValueError('expected immutable git SHAs')
    # No rename detection: a move out of a covered tree still includes deletion.
    names = subprocess.run(['git', 'diff', '--no-renames', '--name-only', '-z', args.base, args.merge, '--'],
                           check=True, capture_output=True).stdout.split(b'\0')
    result = selected(name.decode('utf-8', errors='surrogateescape') for name in names if name)
    with args.github_output.open('a') as stream:
        stream.write('forge=' + str(result).lower() + '\n')


if __name__ == '__main__':
    main()
