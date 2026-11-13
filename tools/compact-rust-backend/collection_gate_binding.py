#!/usr/bin/env python3

# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");

"""Shared source and generated-manifest bindings for collection gates."""

import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tomllib


ROOT = Path(__file__).resolve().parents[2]
COMPILER_INPUTS = (
    'compiler', 'tools/compact', 'tools/compact-rust-backend/src',
    'Cargo.toml', 'Cargo.lock', 'flake.nix', 'flake.lock',
    'tools/compact-rust-backend/Cargo.lock',
    'tools/compact-rust-backend/record_compiler_recipe.py',
)


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def git(*args: str) -> str:
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()


def compiler_recipe(path: Path, compiler: Path, scheme: Path, *,
                    source_head: str, current_head: str) -> dict:
    """Validate an independent clean-source Nix realization receipt and binaries."""
    receipt = json.loads(path.read_text())
    require(receipt.get('format') == 'compact-original-existing-compiler-recipe/v1' and
            receipt.get('status') == 'passed', 'original compiler recipe did not pass')
    require(receipt.get('head') == source_head and
            receipt.get('tree') == git('rev-parse', f'{source_head}^{{tree}}'),
            'compiler recipe source head/tree changed')
    require(subprocess.run(['git', 'merge-base', '--is-ancestor', source_head, current_head],
                           cwd=ROOT).returncode == 0,
            'compiler source is not an ancestor of the current checkout')
    require(subprocess.run(['git', 'diff', '--quiet', source_head, current_head, '--',
                            *COMPILER_INPUTS], cwd=ROOT).returncode == 0,
            'compiler source inputs changed since the qualified build')
    require(git('log', '-1', '--format=%G?', source_head) == 'G' and
            'Signed-off-by:' in git('log', '-1', '--format=%B', source_head),
            'compiler recipe source commit is not signed and DCO complete')
    for key, binary in (('compactc', compiler), ('compactc-scheme', scheme)):
        record = receipt[key]
        require(Path(record['path']).resolve(strict=True) == binary.resolve(strict=True) and
                record['sha256'] == sha(binary), f'{key} differs from compiler recipe')
    require(compiler.parent == scheme.parent and receipt['store'] == str(compiler.parent.parent),
            'compiler and Scheme frontend are not from one realized package')
    pins = receipt['inputs_before_use']
    recipe_roots = [Path(name).parent for name in pins if name.endswith('/flake.nix')]
    require(len(recipe_roots) == 1, 'compiler recipe has no unique source root')
    recipe_root = recipe_roots[0]
    require(subprocess.check_output(['git', '-C', str(recipe_root), 'rev-parse', 'HEAD'],
                                    text=True).strip() == source_head and
            not subprocess.check_output(['git', '-C', str(recipe_root), 'status', '--porcelain'],
                                        text=True).strip(),
            'compiler recipe source checkout changed')
    for relative in ('flake.nix', 'flake.lock', 'Cargo.lock',
                     'tools/compact-rust-backend/Cargo.lock',
                     'compiler/compiler-version.ss',
                     'tools/compact-rust-backend/record_compiler_recipe.py'):
        source = recipe_root / relative
        require(pins.get(str(source)) == sha(source) == sha(ROOT / relative),
                f'compiler recipe input changed: {relative}')
    for name, digest in pins.items():
        require(sha(Path(name)) == digest, f'compiler recipe pinned input changed: {name}')
    for binary in (compiler, scheme):
        require(pins.get(str(binary)) == sha(binary), 'compiler recipe binary input changed')
    command = receipt['command']
    expected_command = [str(Path(shutil.which('nix')).resolve(strict=True)),
                        'build', '--offline', '--no-link', '--print-out-paths',
                        '--max-jobs', '0', '--builders', '', '--option',
                        'substitute', 'false', '.#compactc']
    require(len(receipt['commands']) >= 2 and receipt['commands'][0]['label'] == 'existing-recipe'
            and receipt['commands'][0]['command'] == command == expected_command
            and receipt['commands'][0]['exit'] == 0,
            'compiler recipe realization command changed')
    stdout = Path(receipt['commands'][0]['stdout'])
    require(sha(stdout) == receipt['commands'][0]['stdout_sha256'] and
            stdout.read_text().strip() == receipt['store'],
            'compiler recipe realization output changed')
    return receipt


def generated_manifest(path: Path, *, expected_canonical_sha256: str,
                       source_head: str) -> tuple[str, Path]:
    """Bind all generated Cargo fields while allowing the runtime checkout path to move."""
    data = tomllib.loads(path.read_text())
    dependency = data['dependencies']['midnight-compact-runtime']
    runtime = (path.parent / dependency['path']).resolve(strict=True)
    checkout = Path(subprocess.check_output(
        ['git', '-C', str(runtime), 'rev-parse', '--show-toplevel'], text=True).strip())
    require(runtime == checkout / 'runtime-rs' and
            subprocess.check_output(['git', '-C', str(checkout), 'rev-parse', 'HEAD'],
                                    text=True).strip() == source_head and
            not subprocess.check_output(['git', '-C', str(checkout), 'status', '--porcelain'],
                                        text=True).strip(),
            'generated contract runtime checkout changed')
    dependency['path'] = '@runtime-rs'
    canonical = hashlib.sha256(json.dumps(data, sort_keys=True,
                                          separators=(',', ':')).encode()).hexdigest()
    require(canonical == expected_canonical_sha256,
            'generated contract manifest fields changed')
    return canonical, checkout
