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

"""Shared source and generated-manifest bindings for collection gates."""

import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tomllib


ROOT = Path(__file__).resolve().parents[2]
RUNNER_MAIN_SHA256 = '2846dd67da2916bcf3a15db93e91a1007cfb5d6b97a1d0070e08c6fe312194e9'
RUNNER_MANIFEST_SHA256 = '1e7a8f3a18ff81ba8eacfb982f20501f0a320668d2e7947612deb4c6986367e0'
RUNNER_LOCK_SHA256 = 'c66ac3ad4ad9e2291ce67a90d764af5f7573fbffacdc48953619df7b92ec2ce5'
GENERATED_CONTRACT_FILES = frozenset({
    'Cargo.toml', 'compact-rust-ir.json', 'compact-rust-origins.raw.json',
    'index.d.ts', 'index.js', 'index.js.map', 'lib.rs', 'rust-capabilities.json',
})


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def git(*args: str) -> str:
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()


def generated_contract_file_set(contract: Path) -> None:
    """Reject unqualified Cargo inputs in a fresh dual-target contract."""
    require(contract.is_dir() and not contract.is_symlink(),
            'generated contract directory missing or symlinked')
    entries = list(contract.iterdir())
    require({entry.name for entry in entries} == GENERATED_CONTRACT_FILES and
            all(not entry.is_symlink() and entry.is_file() for entry in entries),
            'generated contract contains an unexpected file, directory or symlink')


def compiler_recipe(path: Path, compiler: Path, scheme: Path, *,
                    source_head: str, current_head: str) -> dict:
    """Validate an independent clean-source Nix realization receipt and binaries."""
    receipt = json.loads(path.read_text())
    require(receipt.get('format') == 'compact-original-existing-compiler-recipe/v1' and
            receipt.get('status') == 'passed', 'original compiler recipe did not pass')
    require(receipt.get('head') == source_head and
            receipt.get('tree') == git('rev-parse', f'{source_head}^{{tree}}'),
            'compiler recipe source head/tree changed')
    require(source_head == current_head,
            'fresh collection qualification requires an exact-head compiler recipe')
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


def verify_standalone_proof(proof: dict, gate: Path, *, raw_name: str,
                            wrong_name: str, output: Path) -> dict:
    """Rebuild the pinned verifier source and independently verify fresh proof bytes."""
    source_root = Path(proof['proof_runner_build_cwd']).resolve(strict=True)
    manifest = source_root / 'Cargo.toml'
    sources = {'main': source_root / 'src/main.rs', 'manifest': manifest,
               'lock': source_root / 'Cargo.lock'}
    expected = {'main': RUNNER_MAIN_SHA256, 'manifest': RUNNER_MANIFEST_SHA256,
                'lock': RUNNER_LOCK_SHA256}
    require(proof['proof_runner_source_sha256'] == expected and
            all(sha(sources[name]) == digest for name, digest in expected.items()),
            'standalone verifier source changed')
    require({path.relative_to(source_root).as_posix() for path in source_root.rglob('*')
             if path.is_file()} == {'Cargo.toml', 'Cargo.lock', 'src/main.rs'},
            'standalone verifier has unpinned local inputs')
    manifest_data = tomllib.loads(manifest.read_text())
    require(manifest_data['package']['name'] == 'secp256k1-unary-proof' and
            not any(isinstance(dependency, dict) and 'path' in dependency
                    for dependency in manifest_data['dependencies'].values()),
            'standalone verifier manifest changed')
    rustc = subprocess.check_output(['rustc', '+1.99.0', '-vV'], text=True)
    require('release: 1.99.0' in rustc.splitlines() and
            'commit-hash: b940084d7eb6a299eb4bfeb8e34901bc051e7ac4' in rustc.splitlines(),
            'standalone verifier Rust toolchain changed')
    cargo_home = Path.home() / '.cargo'
    for directory in (source_root, *source_root.parents, cargo_home):
        names = ('config', 'config.toml') if directory == cargo_home else (
            '.cargo/config', '.cargo/config.toml')
        for name in names:
            require(not (directory / name).exists(),
                    f'unpinned standalone verifier Cargo config: {directory / name}')
    for directory in source_root.parents:
        parent_manifest = directory / 'Cargo.toml'
        if parent_manifest.is_file():
            require('workspace' not in tomllib.loads(parent_manifest.read_text()),
                    f'unpinned standalone verifier parent workspace: {parent_manifest}')
    environment = {key: value for key, value in os.environ.items()
                   if not (key.startswith('CARGO_') or key in
                           {'RUSTFLAGS', 'RUSTDOCFLAGS', 'RUSTC', 'RUSTC_WRAPPER',
                            'RUSTC_WORKSPACE_WRAPPER', 'RUSTUP_TOOLCHAIN'})}
    target = output / 'standalone-verifier-target'
    environment['CARGO_TARGET_DIR'] = str(target)
    command = ['cargo', '+1.99.0', 'build', '--locked', '--offline', '--bin',
               'secp256k1-unary-proof', '--manifest-path', str(manifest)]
    with (output / 'standalone-verifier-build.log').open('xb') as log:
        subprocess.run(command, cwd=source_root, env=environment,
                       stdout=log, stderr=subprocess.STDOUT, check=True)
    runner = target / 'debug/secp256k1-unary-proof'
    require(runner.is_file(), 'rebuilt standalone verifier missing')
    verify = [str(runner), 'verify', str(gate / f'fresh-dual/zkir/{raw_name}.zkir'),
              str(gate / 'preimage/honest.preimage'), str(gate / 'proof'),
              str(gate / f'wrong-key/{wrong_name}.verifier')]
    with (output / 'standalone-verifier-run.log').open('xb') as log:
        subprocess.run(verify, cwd=ROOT, stdout=log,
                       stderr=subprocess.STDOUT, check=True)
    require('verified in separate process; rejected different verifier, changed binding and corrupt proof'
            in (output / 'standalone-verifier-run.log').read_text(),
            'standalone proof verification or refusal controls changed')
    return {'runner_sha256': sha(runner),
            'source_sha256': expected,
            'build_log_sha256': sha(output / 'standalone-verifier-build.log'),
            'verification_log_sha256': sha(output / 'standalone-verifier-run.log')}
