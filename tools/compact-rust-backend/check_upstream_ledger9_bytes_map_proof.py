#!/usr/bin/env python3

# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0

"""Prove the pinned Bytes map transformation with exact-head Rust/TS artifacts."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import tarfile
import tomllib

from collection_gate_binding import compiler_recipe


ROOT = Path(__file__).resolve().parents[2]
FAMILY = ROOT / 'test-contracts/primitives/bytes/map'
CAPTURE = ROOT / 'runtime-rs/tests/fixtures/capture-ledger9-bytes-map-transformation-proof.mjs'
PINNED = '672bd0f226f47a7d036c5c074ce71fbe1af1925e'
BASE = '7425aa72eeec8102e6180808042a52c3bf55f44a'
BASE_RECEIPT_SHA256 = '34bee302b30f1d87a4e04e3904891a452865d36f20ddd61613e39a36a02b4426'
SOURCE_SHA256 = '38f12526a78734b8c93f4be6cffa0ef9efaae59db552d56b3cf36ecbbd924680'
WASM_TGZ_SHA256 = 'bc3e69e37e452463c1b2f4c98fce1138eb04f8d3776fce704b9081989a699f19'
ZKIR_SHA256 = '731d8273bd1d212a30b09182898bf7f1f96f49618dcb337e40689de40c96ae0a'
ZKIR_RAW_SHA256 = {
    'transformation': 'd35a45c657469547c91d998476943ad004c5e47962ce345d350045ccf691dd34',
    'slice_composition': 'b76e900ac3a8069c34fdaf21a67d332a61106eae53d716554007cfe5ec368f20',
}
PARAMS_K9_SHA256 = 'b9009f1098bcefffec3c461ab3a5e3a17f7e5599f0f08c70fcdc55a89227bcbd'
RUNNER_SOURCE_SHA256 = {
    'src/main.rs': '2846dd67da2916bcf3a15db93e91a1007cfb5d6b97a1d0070e08c6fe312194e9',
    'Cargo.toml': '1e7a8f3a18ff81ba8eacfb982f20501f0a320668d2e7947612deb4c6986367e0',
    'Cargo.lock': 'c66ac3ad4ad9e2291ce67a90d764af5f7573fbffacdc48953619df7b92ec2ce5',
}


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require(ok: bool, message: str) -> None:
    if not ok:
        raise RuntimeError(message)


def git(*args: str) -> str:
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()


def run(command: list[str], log: Path, *, cwd: Path = ROOT,
        env: dict[str, str] | None = None) -> str:
    with log.open('xb') as stream:
        completed = subprocess.run(command, cwd=cwd, env=env, stdout=stream,
                                   stderr=subprocess.STDOUT, check=True)
    require(completed.returncode == 0, f'{command[0]} failed')
    return log.read_text()


def dist_hashes(root: Path) -> dict[str, str]:
    return {file.relative_to(root).as_posix(): sha(file)
            for file in root.rglob('*') if file.is_file()}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('base-output', 'compiler-receipt', 'zkir', 'zkir-wasm-tgz',
                 'runner-source', 'params-k9', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
    args = parser.parse_args()
    require(not git('status', '--porcelain'), 'source checkout must be clean')
    head, tree = git('rev-parse', 'HEAD'), git('rev-parse', 'HEAD^{tree}')
    require(git('log', '-1', '--format=%G?') == 'G' and
            'Signed-off-by:' in git('log', '-1', '--format=%B'),
            'source head must be signed and DCO complete')
    require(subprocess.run(['git', 'merge-base', '--is-ancestor', BASE, head],
                           cwd=ROOT).returncode == 0, 'source does not extend qualified Bytes map')
    output = args.output.resolve()
    require(not args.output.is_symlink() and not output.exists(), 'output must be absent')
    base = args.base_output.resolve(strict=True)
    base_receipt = base / 'receipt.json'
    require(sha(base_receipt) == BASE_RECEIPT_SHA256, 'base qualification receipt changed')
    prior = json.loads(base_receipt.read_text())
    require(prior['head'] == BASE and prior['upstream'] == PINNED and
            prior['tree'] == git('rev-parse', f'{BASE}^{{tree}}') and
            prior['original_files'][
                'test-contracts/primitives/bytes/map/transformation/bytes_map_transformation.compact'
            ] == SOURCE_SHA256, 'base qualification is not the pinned original')
    source = FAMILY / 'transformation/bytes_map_transformation.compact'
    wrong_source = FAMILY / 'slice_composition/bytes_map_slice_composition.compact'
    require(sha(source) == SOURCE_SHA256 and
            sha(wrong_source) == prior['original_files'][wrong_source.relative_to(ROOT).as_posix()],
            'pinned Bytes map source changed')
    recipe_path = args.compiler_receipt.resolve(strict=True)
    recipe = json.loads(recipe_path.read_text())
    compiler = Path(recipe['compactc']['path']).resolve(strict=True)
    scheme = Path(recipe['compactc-scheme']['path']).resolve(strict=True)
    compiler_recipe(recipe_path, compiler, scheme, source_head=head, current_head=head)
    zkir = args.zkir.resolve(strict=True)
    wasm_tgz = args.zkir_wasm_tgz.resolve(strict=True)
    params = args.params_k9.resolve(strict=True)
    require(sha(zkir) == ZKIR_SHA256 and sha(wasm_tgz) == WASM_TGZ_SHA256 and
            sha(params) == PARAMS_K9_SHA256, 'ZKIR, wasm or k=9 parameters changed')
    require(subprocess.check_output([str(zkir), '--version'], text=True).strip()
            == 'midnight-zkir 3.2.0', 'native ZKIR version changed')
    runner_source = args.runner_source.resolve(strict=True)
    require({file.relative_to(runner_source).as_posix() for file in runner_source.rglob('*')
             if file.is_file()} == set(RUNNER_SOURCE_SHA256), 'runner source has unpinned files')
    require(all(sha(runner_source / name) == digest for name, digest in
                RUNNER_SOURCE_SHA256.items()), 'runner source changed')
    manifest = tomllib.loads((runner_source / 'Cargo.toml').read_text())
    require(manifest['package']['name'] == 'secp256k1-unary-proof' and
            not any(isinstance(dep, dict) and 'path' in dep
                    for dep in manifest['dependencies'].values()),
            'runner manifest changed')
    rustc = subprocess.check_output(['rustc', '+1.99.0', '-vV'], text=True)
    require('release: 1.99.0' in rustc.splitlines() and
            'commit-hash: b940084d7eb6a299eb4bfeb8e34901bc051e7ac4'
            in rustc.splitlines(), 'proof runner toolchain changed')

    output.mkdir(parents=True)
    runtime_recipe_output = run([
        str(Path(shutil.which('nix')).resolve(strict=True)), 'build', '--offline',
        '--no-link', '--print-out-paths', '--max-jobs', '0', '--builders', '',
        '--option', 'substitute', 'false', '.#runtime.package',
    ], output / 'runtime-recipe.log')
    runtime_store = Path(runtime_recipe_output.splitlines()[-1]).resolve(strict=True)
    runtime = runtime_store / 'lib/node_modules/@midnight-ntwrk/compact-runtime'
    require(json.loads((runtime / 'package.json').read_text())['version'] == '0.18.0'
            and dist_hashes(runtime / 'dist') == prior['typescript_runtime_dist_files'],
            'exact-head runtime differs from base qualified runtime')
    package = output / 'node_modules/@midnight-ntwrk/compact-runtime'
    package.parent.mkdir(parents=True)
    package.symlink_to(runtime, target_is_directory=True)
    (output / 'package.json').write_text('{"type":"module"}\n')
    wasm = output / 'zkir-wasm'
    wasm.mkdir()
    with tarfile.open(wasm_tgz, 'r:gz') as archive:
        members = archive.getmembers()
        require(all((member.name == 'package' or member.name.startswith('package/')) and
                    '..' not in Path(member.name).parts and
                    (member.isfile() or member.isdir()) and not member.issym()
                    for member in members), 'ZKIR wasm archive has unsafe members')
        archive.extractall(wasm, filter='data')
    wasm_package = wasm / 'package'
    require(json.loads((wasm_package / 'package.json').read_text())['version'] == '3.2.0',
            'ZKIR wasm package version changed')
    compiler_env = os.environ.copy()
    compiler_env.pop('COMPACT_RUST_RUNTIME_DIR', None)
    compiler_env['COMPACTC_SCHEME'] = str(scheme)
    fresh = output / 'fresh-dual'
    run([str(compiler), '--target', 'ts', '--target', 'rust',
         '--rust-runtime-root', str(ROOT), '--rust-require-recording', '--skip-zk',
         '--compact-path', str(ROOT / 'compiler'), str(source), str(fresh)],
        output / 'fresh-dual.log', env=compiler_env)
    wrong = output / 'wrong-dual'
    run([str(compiler), '--target', 'ts', '--target', 'rust',
         '--rust-runtime-root', str(ROOT), '--rust-require-recording', '--skip-zk',
         '--compact-path', str(ROOT / 'compiler'), str(wrong_source), str(wrong)],
        output / 'wrong-dual.log', env=compiler_env)
    artifact_names = {'typescript': 'index.js', 'rust': 'lib.rs',
                      'rust_ir': 'compact-rust-ir.json'}
    artifacts = {}
    for case, folder in [('transformation', fresh), ('slice_composition', wrong)]:
        for key, filename in artifact_names.items():
            actual = sha(folder / 'contract' / filename)
            require(actual == prior['artifacts'][f'{case}/paired/{key}'],
                    f'{case} generated {key} differs from qualified base')
            artifacts[f'{case}/{key}'] = actual
        raw = folder / 'zkir' / f'bytes_map_{case}.zkir'
        base_raw = base / 'paired' / case / 'zkir' / raw.name
        require(sha(raw) == sha(base_raw) == ZKIR_RAW_SHA256[case],
                f'{case} ZKIR differs from pinned original qualification')
        artifacts[f'{case}/zkir'] = sha(raw)
    raw = fresh / 'zkir/bytes_map_transformation.zkir'
    wrong_raw = wrong / 'zkir/bytes_map_slice_composition.zkir'
    preimage = output / 'preimage'
    run(['node', str(CAPTURE), str(fresh / 'contract/index.js'), str(raw),
         str(wasm_package), str(preimage)], output / 'capture.log')
    capture = json.loads((preimage / 'capture.json').read_text())
    require(capture['honestPreimageSha256'] == sha(preimage / 'honest.preimage') and
            capture['rawZkirSha256'] == sha(raw) and
            capture['generatedTypeScriptSha256'] == artifacts['transformation/typescript'] and
            capture['checks'] == ['honest accepted', 'changed stored byte refused',
                                  'changed insert count refused'], 'preimage capture changed')
    wrong_key = output / 'wrong-key'
    wrong_key.mkdir()
    wrong_log = run([str(zkir), 'compile', str(wrong_raw),
                     str(wrong_key / 'slice_composition.prover'),
                     str(wrong_key / 'slice_composition.verifier')], output / 'wrong-key.log')
    require(re.search(r'\bk=9\b', wrong_log) is not None,
            'wrong circuit does not share k=9')
    clean_env = {key: value for key, value in os.environ.items()
                 if not (key.startswith('CARGO_') or key in
                         {'RUSTFLAGS', 'RUSTDOCFLAGS', 'RUSTC', 'RUSTC_WRAPPER',
                          'RUSTC_WORKSPACE_WRAPPER', 'RUSTUP_TOOLCHAIN'})}
    for directory in (runner_source, *runner_source.parents, Path.home() / '.cargo'):
        names = ('config', 'config.toml') if directory == Path.home() / '.cargo' else (
            '.cargo/config', '.cargo/config.toml')
        require(not any((directory / name).exists() for name in names),
                f'unpinned Cargo configuration at {directory}')
    for directory in runner_source.parents:
        parent_manifest = directory / 'Cargo.toml'
        require(not (parent_manifest.is_file() and
                     'workspace' in tomllib.loads(parent_manifest.read_text())),
                f'unpinned parent Cargo workspace at {directory}')
    clean_env['CARGO_TARGET_DIR'] = str(output / 'runner-target')
    run(['cargo', '+1.99.0', 'build', '--locked', '--offline', '--bin',
         'secp256k1-unary-proof', '--manifest-path',
         str(runner_source / 'Cargo.toml')], output / 'runner-build.log',
        cwd=runner_source, env=clean_env)
    runner = output / 'runner-target/debug/secp256k1-unary-proof'
    require(runner.is_file(), 'source-built runner missing')
    proof = output / 'proof'
    proof.mkdir()
    proved = run([str(runner), 'prove', str(raw),
                  str(preimage / 'honest.preimage'), str(proof), str(params)],
                 output / 'prove.log')
    require(proved.startswith('proved k=9,'), 'fresh k=9 proof missing')
    verified = run([str(runner), 'verify', str(raw),
                    str(preimage / 'honest.preimage'), str(proof),
                    str(wrong_key / 'slice_composition.verifier')],
                   output / 'verify.log')
    require('verified in separate process; rejected different verifier, changed binding and corrupt proof'
            in verified, 'independent verifier controls failed')
    require(sha(proof / 'statement.verifier') !=
            sha(wrong_key / 'slice_composition.verifier'), 'wrong verifier equals correct verifier')
    require(not git('status', '--porcelain') and git('rev-parse', 'HEAD') == head,
            'source checkout changed during proof gate')
    receipt = {
        'head': head, 'tree': tree, 'platform': f'{platform.system()} {platform.machine()}',
        'upstream': PINNED, 'base_head': BASE,
        'base_qualification_receipt_sha256': sha(base_receipt),
        'compiler_receipt_sha256': sha(recipe_path), 'compiler_sha256': sha(compiler),
        'scheme_sha256': sha(scheme), 'capture_source_sha256': sha(CAPTURE),
        'gate_source_sha256': sha(Path(__file__)), 'source_sha256': sha(source),
        'runtime_nix_store': str(runtime_store), 'wasm_tgz_sha256': sha(wasm_tgz),
        'zkir_binary_sha256': sha(zkir), 'params_k9_sha256': sha(params),
        'runner_source_sha256': RUNNER_SOURCE_SHA256, 'runner_binary_sha256': sha(runner),
        'artifacts': artifacts, 'capture': capture,
        'proof_sha256': sha(proof / 'statement.proof'),
        'prover_sha256': sha(proof / 'statement.prover'),
        'verifier_sha256': sha(proof / 'statement.verifier'),
        'wrong_verifier_sha256': sha(wrong_key / 'slice_composition.verifier'),
        'proof_log_sha256': sha(output / 'prove.log'),
        'verify_log_sha256': sha(output / 'verify.log'),
        'checks': ['exact-head generated Rust/TypeScript/IR/ZKIR bound to qualified base',
                   'honest ZKIR wasm preimage accepted; changed value and insert refused',
                   'fresh k=9 proof', 'independent verifier and refusal controls'],
        'limitations': ['Local platform only; native Linux remains required.',
                        'Funded default-strict transaction and replay refusal remain pending.'],
    }
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(f'upstream Bytes map proof gate passed: {output}')


if __name__ == '__main__':
    main()
