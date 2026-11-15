#!/usr/bin/env python3
# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0

"""Qualify pinned opaque casts, foreign self-subtraction, and Field refusal."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tomllib

from check_compactc_target import check_manifest
from check_unit_mutation_initializers import validate_consumer_lock
from check_upstream_ledger9_field_casts import (
    compiler_recipe_identity, require, run, runtime_identity,
    runtime_recipe_identity, sha256, source_state,
)

ROOT = Path(__file__).resolve().parents[2]
PIN = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'
BASE = ROOT / 'tools/compact-rust-backend/tests/foreign_opaque_field'
ORIGINAL_DIRS = (
    'test-contracts/primitives/opaque/casts/to_opaque',
    'test-contracts/stdlib/foreign_field/self_subtract',
    'test-contracts/primitives/field/literal/out_of_range',
)
SOURCES = {
    'opaque': ROOT / ORIGINAL_DIRS[0] / 'opaque_casts_to_opaque.compact',
    'foreign': ROOT / ORIGINAL_DIRS[1] / 'foreign_field_self_subtract.compact',
    'field': ROOT / ORIGINAL_DIRS[2] / 'field_literal_out_of_range.compact',
}
NAMES = ('self_subtract', 'inlined_self_subtract', 'multiply_self_subtract',
         'secp256r1_self_subtract', 'curve25519_self_subtract')
LOG_NAMES = frozenset(
    [f'{case}-{mode}-{phase}.log'
     for case in ('opaque', 'foreign')
     for mode in ('typescript', 'both')
     for phase in ('compile', 'runtime')]
    + [f'field-{mode}-compile.log' for mode in ('typescript', 'both')]
    + ['cargo-metadata.log', 'cargo-test.log']
)


def inventory():
    names = subprocess.check_output(
        ['git', 'ls-tree', '-r', '--name-only', PIN, *ORIGINAL_DIRS], cwd=ROOT,
        text=True).splitlines()
    require(len(names) == 8 and all(name.startswith(ORIGINAL_DIRS) for name in names),
            'pinned original inventory changed')
    result = {}
    for name in names:
        path = ROOT / name
        original = subprocess.check_output(['git', 'show', f'{PIN}:{name}'], cwd=ROOT)
        require(path.is_file() and not path.is_symlink() and path.read_bytes() == original,
                f'pinned original changed: {name}')
        result[name] = sha256(path)
    opaque = (ROOT / ORIGINAL_DIRS[0] / 'runtime.pass.test.ts').read_text()
    foreign = (ROOT / ORIGINAL_DIRS[1] / 'runtime.pass.test.ts').read_text()
    refusal = (ROOT / ORIGINAL_DIRS[2] / 'compile.fail.test.ts').read_text()
    require('toBeInstanceOf(Uint8Array)' in opaque and
            all(f'c.{name}(ctx,' in foreign for name in NAMES) and
            foreign.count('.result).toBe(0n)') == 5 and
            '/is out of Field range/' in refusal,
            'original assertion or refusal changed')
    return result


def generated_files(path):
    return {str(p.relative_to(path)): sha256(p) for p in sorted(path.rglob('*'))
            if p.is_file() and not p.is_symlink()}


def log_hashes(output):
    """Bind every archived command outcome, including refused compilation."""
    actual = {p.name for p in output.glob('*.log')}
    require(actual == LOG_NAMES, 'archived command log set changed')
    require(all((output / name).is_file() and not (output / name).is_symlink()
                for name in actual), 'archived command log replaced')
    return {name: sha256(output / name) for name in sorted(actual)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('compiler', 'compiler-recipe', 'ts-runtime', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--cargo-target', type=Path)
    args = parser.parse_args()
    source = source_state()
    signature = subprocess.check_output(['git', 'log', '-1', '--format=%G?%n%B'],
                                        cwd=ROOT, text=True)
    require(signature.startswith('G\n') and 'Signed-off-by:' in signature and
            'Assisted-by: AI' in signature, 'signed DCO and assisted head required')
    compiler = args.compiler.resolve(strict=True)
    scheme = compiler.with_name('compactc-scheme').resolve(strict=True)
    runtime = args.ts_runtime.resolve(strict=True)
    output = args.output.resolve()
    require(not args.output.is_symlink() and not output.exists(), 'output must be absent')
    if args.cargo_target:
        require(args.cargo_target.is_absolute() and args.cargo_target.is_dir() and
                not args.cargo_target.is_symlink(), 'shared Cargo target must be an existing directory')
        cargo_target = args.cargo_target.resolve(strict=True)
        require(not cargo_target.is_relative_to(output), 'shared Cargo target inside evidence output')
    else:
        cargo_target = output / 'cargo-target'
    original = inventory()
    adapter, consumer = BASE / 'check.mjs', BASE / 'consumer.rs'
    controls = {str(p.relative_to(ROOT)): sha256(p) for p in (adapter, consumer)}
    require(all(token in adapter.read_text() for token in
                ('Uint8Array', '12345n', '678n', '0n', *NAMES)) and
            all(token in consumer.read_text() for token in
                ('12345', '678', 'zero_k', 'zero_r', 'zero_c', *NAMES)),
            'local assertion controls changed')
    recipe = compiler_recipe_identity(args.compiler_recipe, source, compiler, scheme)
    runtime_before = runtime_identity(runtime)
    nix = shutil.which('nix')
    require(nix is not None, 'Nix required for runtime binding')
    runtime_recipe = runtime_recipe_identity(runtime, Path(nix).resolve(strict=True))
    package = json.loads((ROOT / 'runtime/package.json').read_text())
    installed = json.loads((runtime / 'package.json').read_text())
    require(all(package[k] == installed[k] for k in ('name', 'version', 'dependencies')),
            'TypeScript runtime differs from source')
    output.mkdir(parents=True)
    link = output / 'node_modules/@midnight-ntwrk/compact-runtime'
    link.parent.mkdir(parents=True)
    link.symlink_to(runtime, target_is_directory=True)
    (output / 'package.json').write_text('{"type":"module"}\n')
    env = dict(os.environ, COMPACTC_SCHEME=str(scheme), RUSTUP_TOOLCHAIN='1.99.0')
    generated = {}
    for case in ('opaque', 'foreign'):
        generated[case] = {}
        for mode in ('typescript', 'both'):
            dest = output / 'generated' / case / mode
            cmd = [str(compiler), '--target', 'ts']
            if mode == 'both':
                cmd += ['--target', 'rust', '--rust-runtime-root', str(ROOT),
                        '--rust-require-recording']
            cmd += ['--feature-zkir-v3', '--skip-zk', '--compact-path',
                    str(ROOT / 'compiler'), str(SOURCES[case]), str(dest)]
            run(cmd, output / f'{case}-{mode}-compile.log', env)
            check_manifest(dest, require_zkir=case == 'foreign')
            info = json.loads((dest / 'compiler/contract-info.json').read_text())
            circuits = info['circuits']
            if case == 'opaque':
                require(len(circuits) == 1 and circuits[0]['name'] == 'opaque_casts_to_opaque'
                        and circuits[0]['pure'] and not circuits[0]['proof'] and
                        circuits[0]['result-type'] ==
                        {'type-name': 'Opaque', 'tsType': 'Uint8Array'},
                        'opaque generated API changed')
                require(not (dest / 'zkir').exists(), 'opaque proof=false emitted ZKIR')
            else:
                require(tuple(c['name'] for c in circuits) == NAMES and
                        all(c['proof'] and not c['pure'] for c in circuits) and
                        len(list((dest / 'zkir').glob('*.zkir'))) == 5,
                        'five foreign proof capabilities or ZKIR changed')
            contract = dest / 'contract'
            runtime_log = output / f'{case}-{mode}-runtime.log'
            result = run(['node', str(adapter), str(contract / 'index.js'), case],
                         runtime_log, env)
            require('original' in result and 'passed' in result,
                    f'{case} original TypeScript assertion did not pass')
            generated[case][mode] = generated_files(dest)
        for name in ('contract/index.js', 'compiler/contract-info.json'):
            require(generated[case]['typescript'][name] == generated[case]['both'][name],
                    f'{case} TS-only and paired output differ: {name}')
    for mode in ('typescript', 'both'):
        dest = output / 'generated/field' / mode
        cmd = [str(compiler), '--target', 'ts']
        if mode == 'both':
            cmd += ['--target', 'rust', '--rust-runtime-root', str(ROOT),
                    '--rust-require-recording']
        cmd += ['--feature-zkir-v3', '--skip-zk', '--compact-path',
                str(ROOT / 'compiler'), str(SOURCES['field']), str(dest)]
        failure = subprocess.run(cmd, cwd=ROOT, env=env, capture_output=True, text=True)
        (output / f'field-{mode}-compile.log').write_text(failure.stdout + failure.stderr)
        require(failure.returncode != 0 and 'is out of Field range' in failure.stderr and
                not dest.exists(), f'Field refusal changed: {mode}')
    rust = output / 'rust-consumer'
    (rust / 'tests').mkdir(parents=True)
    shutil.copyfile(consumer, rust / 'tests/foreign_opaque_field.rs')
    shutil.copyfile(ROOT / 'Cargo.lock', rust / 'Cargo.lock')
    dependencies = []
    contracts = []
    for case in ('opaque', 'foreign'):
        contract = output / 'generated' / case / 'both/contract'
        contracts.append(contract)
        package_name = tomllib.loads((contract / 'Cargo.toml').read_text())['package']['name']
        dependencies.append(f'{case} = {{ package = {json.dumps(package_name)}, '
                            f'path = {json.dumps(str(contract))} }}')
    manifest = rust / 'Cargo.toml'
    manifest.write_text('[workspace]\n[package]\nname = "ledger9-foreign-opaque-field-consumer"\n'
                        'version = "0.1.0"\nedition = "2024"\n[dependencies]\n'
                        'midnight-onchain-vm = "=4.0.1"\nmidnight-serialize = "=1.2.1"\n' +
                        '\n'.join(dependencies) + '\n')
    cargo_env = dict(env, CARGO_TARGET_DIR=str(cargo_target),
                     CARGO_INCREMENTAL='0', CARGO_PROFILE_DEV_DEBUG='0')
    metadata = json.loads(run(['cargo', '+1.99.0', 'metadata', '--offline',
                               '--format-version', '1', '--manifest-path', str(manifest)],
                              output / 'cargo-metadata.log', cargo_env))
    root_lock = tomllib.loads((ROOT / 'Cargo.lock').read_text())['package']
    pinned = {(row['name'], row['version'], row['source']): row.get('checksum')
              for row in root_lock if 'source' in row}
    allowed = {manifest, ROOT / 'runtime-rs/Cargo.toml',
               ROOT / 'runtime-rs-macros/Cargo.toml',
               *(p / 'Cargo.toml' for p in contracts)}
    validate_consumer_lock(metadata, rust / 'Cargo.lock', allowed, pinned)
    run(['cargo', '+1.99.0', 'test', '--locked', '--offline', '--manifest-path', str(manifest)],
        output / 'cargo-test.log', cargo_env)
    require('test result: ok. 2 passed; 0 failed;' in
            (output / 'cargo-test.log').read_text(), 'Rust ordinary/recorded tests failed')
    logs = log_hashes(output)
    require(source_state() == source and inventory() == original and
            runtime_identity(runtime) == runtime_before and
            compiler_recipe_identity(args.compiler_recipe, source, compiler, scheme) == recipe and
            all(sha256(ROOT / name) == digest for name, digest in controls.items()),
            'qualified inputs changed')
    require(log_hashes(output) == logs, 'archived command outcome changed')
    for case in generated:
        for mode in generated[case]:
            require(generated_files(output / 'generated' / case / mode) == generated[case][mode],
                    'generated artifacts changed')
    receipt = {**source, 'status': 'passed', 'source_clean': True,
               'platform': os.uname().sysname + ' ' + os.uname().machine,
               'upstream_commit': PIN, 'original_files': original,
               'local_controls': controls, 'generated': generated,
               'compiler_recipe': recipe, 'compiler_sha256': sha256(compiler),
               'scheme_sha256': sha256(scheme), 'runtime': runtime_before,
               'runtime_recipe': runtime_recipe,
               'checker_sha256': sha256(Path(__file__)),
               'rust_consumer_lock_sha256': sha256(rust / 'Cargo.lock'),
               'command_logs_sha256': logs,
               'cargo_target': str(cargo_target),
               'proof_execution': False, 'funded_application': False,
               'limitations': [('native Linux generated semantics qualified' if
                                os.uname().sysname == 'Linux' and os.uname().machine == 'x86_64' else
                                'local Darwin arm64; native Linux qualification required'),
                               'five foreign proof=true exports are unproved: fresh proof, independent verification, funded strict application and replay remain open',
                               'original Vitest harness not run; exact pinned assertions replayed against generated APIs']}
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': 'passed', 'head': source['head'], 'output': str(output)}))


if __name__ == '__main__':
    main()
