#!/usr/bin/env python3

# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0

"""Prove the three remaining pinned Ledger 9 Bytes map exports."""

import argparse
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
from check_upstream_ledger9_bytes_map_proof import (
    BASE, BASE_RECEIPT_SHA256, FAMILY, PINNED, ROOT, RUNNER_SOURCE_SHA256,
    WASM_TGZ_SHA256, ZKIR_SHA256, dist_hashes, git, require, run, sha,
)


CAPTURE = ROOT / 'runtime-rs/tests/fixtures/capture-ledger9-bytes-map-remaining-proofs.mjs'
MATERIAL_RECEIPT_SHA256 = '9f8a8464782ab57edfd58e317d28d16e1f9ff620e5fdbaf662a4f6685ab4bae2'
PARAMS = {9: 'b9009f1098bcefffec3c461ab3a5e3a17f7e5599f0f08c70fcdc55a89227bcbd',
          10: '46b2290933cbed4c378889e4ba971f1a92888331ffb09466acd4ff61a1e2cb42'}
CASES = {
    'reordering': {
        'k': 10, 'wrong': 'to_bytes',
        'source': '9ebbdec60c7cb08515f93c17eaac62e4dff141b12614bd087166bba4737dff00',
        'zkir': 'deb4ce5571ec75aca6516e9e1a7935e9c11ca096316c5262a9bc5736785a5ae8',
        'typescript': '0edb34981c7c8a5b9e327193de69c744abc02cfc378f52b6ca2fa6267bb9e04e',
        'rust': 'd37d1ce9a4a222388f1c39722d8e57348de881603430942e5541bd6edc63b54a',
        'rust_ir': '981c6a431867f4236d5b1d9d29a1cf371812ce3ac16c730a19b2e797861df73e',
    },
    'slice_composition': {
        'k': 9, 'wrong': 'transformation',
        'source': '49e433b537a1a31d75c07597d69c723387ff32b1e65a8b336140200ecbf40b88',
        'zkir': 'b76e900ac3a8069c34fdaf21a67d332a61106eae53d716554007cfe5ec368f20',
        'typescript': 'e297af729ada7df416e88a246357d9469236f7cb4c162a83266597dc4f43472e',
        'rust': '2894b921160d9d351041f3d18399c2be618d16188c1a6d1f7f59040342a2a77d',
        'rust_ir': '64d6cb0588a452d1ec0319a484a69ade1a7dee3a46d10a80b15466c309a3b000',
    },
    'to_bytes': {
        'k': 10, 'wrong': 'reordering',
        'source': '4f1cc4bddee8087b4863d9536a530e9b55e5e5aa704ecb01928e6fe5687b4296',
        'zkir': '9da2c1422b1495f549b3b34d015a15108464861aefa1ffacbe6be45c149b5ef0',
        'typescript': '35e0dbbb7382d7c63f0013c9370324ff1ab1d9055dd9dc0905600283cfa5b3a8',
        'rust': '17db583932696258303bcd709d33791019f52d44f0ae3888357094418fb8524b',
        'rust_ir': 'a889a959ee3887c38c3a77ffcd6b6f01e4f36cdaaf63ea694e857bede2914f58',
    },
    'transformation': {
        'k': 9, 'wrong': None,
        'source': '38f12526a78734b8c93f4be6cffa0ef9efaae59db552d56b3cf36ecbbd924680',
        'zkir': 'd35a45c657469547c91d998476943ad004c5e47962ce345d350045ccf691dd34',
        'typescript': '602c78bf81af248ffa055ac35190322220ea3e679b4d96b1d04a0ad36d217b61',
        'rust': '05c7ca2ae0e8ec3c6610174c11e426a9e25abdc32d12e0e55fe6b1da2f94afea',
        'rust_ir': 'a01b9cd019cac2d9e777c395eb3a2f88cf78fad3f610926b49a198ad66e48f6e',
    },
}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('base-output', 'compiler-receipt', 'zkir', 'zkir-wasm-tgz',
                 'runner-source', 'material', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
    args = parser.parse_args()
    require(not git('status', '--porcelain'), 'source checkout must be clean')
    head, tree = git('rev-parse', 'HEAD'), git('rev-parse', 'HEAD^{tree}')
    require(git('log', '-1', '--format=%G?') == 'G' and
            'Signed-off-by:' in git('log', '-1', '--format=%B'),
            'source head must be signed and DCO complete')
    require(subprocess.run(['git', 'merge-base', '--is-ancestor', BASE, head],
                           cwd=ROOT).returncode == 0, 'source is not stacked on Bytes map base')
    output = args.output.resolve()
    require(not args.output.is_symlink() and not output.exists(), 'output must be absent')
    base = args.base_output.resolve(strict=True)
    base_receipt = base / 'receipt.json'
    require(sha(base_receipt) == BASE_RECEIPT_SHA256, 'base qualification receipt changed')
    prior = json.loads(base_receipt.read_text())
    require(prior['head'] == BASE and prior['upstream'] == PINNED and
            prior['tree'] == git('rev-parse', f'{BASE}^{{tree}}') and
            sha(FAMILY / 'check.mjs') == prior['adapter_sha256'],
            'base source or adapter binding changed')
    recipe_path = args.compiler_receipt.resolve(strict=True)
    recipe = json.loads(recipe_path.read_text())
    compiler = Path(recipe['compactc']['path']).resolve(strict=True)
    scheme = Path(recipe['compactc-scheme']['path']).resolve(strict=True)
    compiler_recipe(recipe_path, compiler, scheme, source_head=head, current_head=head)
    zkir = args.zkir.resolve(strict=True)
    wasm_tgz = args.zkir_wasm_tgz.resolve(strict=True)
    material = args.material.resolve(strict=True)
    material_receipt = material.parent / 'material-receipt.json'
    require(sha(zkir) == ZKIR_SHA256 and sha(wasm_tgz) == WASM_TGZ_SHA256 and
            sha(material_receipt) == MATERIAL_RECEIPT_SHA256,
            'ZKIR or SDK material changed')
    manifest = json.loads(material_receipt.read_text())['files']
    require(all(sha(material / f'bls_midnight_2p{k}') == digest ==
                manifest[f'bls_midnight_2p{k}']['sha256']
                for k, digest in PARAMS.items()), 'k=9 or k=10 parameters changed')
    require(subprocess.check_output([str(zkir), '--version'], text=True).strip()
            == 'midnight-zkir 3.2.0', 'native ZKIR version changed')
    runner_source = args.runner_source.resolve(strict=True)
    require({file.relative_to(runner_source).as_posix() for file in runner_source.rglob('*')
             if file.is_file()} == set(RUNNER_SOURCE_SHA256) and
            all(sha(runner_source / name) == digest for name, digest in
                RUNNER_SOURCE_SHA256.items()), 'source-built runner inputs changed')
    runner_manifest = tomllib.loads((runner_source / 'Cargo.toml').read_text())
    require(runner_manifest['package']['name'] == 'secp256k1-unary-proof' and
            not any(isinstance(dep, dict) and 'path' in dep
                    for dep in runner_manifest['dependencies'].values()),
            'source-built runner manifest changed')
    rustc = subprocess.check_output(['rustc', '+1.99.0', '-vV'], text=True)
    require('release: 1.99.0' in rustc.splitlines() and
            'commit-hash: b940084d7eb6a299eb4bfeb8e34901bc051e7ac4'
            in rustc.splitlines(), 'runner Rust toolchain changed')
    for directory in (runner_source, *runner_source.parents, Path.home() / '.cargo'):
        names = ('config', 'config.toml') if directory == Path.home() / '.cargo' else (
            '.cargo/config', '.cargo/config.toml')
        require(not any((directory / name).exists() for name in names),
                f'unpinned Cargo configuration at {directory}')
    for directory in runner_source.parents:
        parent_manifest = directory / 'Cargo.toml'
        require(not (parent_manifest.is_file() and
                     'workspace' in tomllib.loads(parent_manifest.read_text())),
                f'unpinned runner parent workspace at {directory}')
    clean_env = {key: value for key, value in os.environ.items()
                 if not (key.startswith('CARGO_') or key in
                         {'RUSTFLAGS', 'RUSTDOCFLAGS', 'RUSTC', 'RUSTC_WRAPPER',
                          'RUSTC_WORKSPACE_WRAPPER', 'RUSTUP_TOOLCHAIN',
                          'CC', 'CXX', 'LD', 'AR', 'CFLAGS', 'CXXFLAGS', 'LDFLAGS'})}

    output.mkdir(parents=True)
    runtime_recipe = run([
        str(Path(shutil.which('nix')).resolve(strict=True)), 'build', '--offline',
        '--no-link', '--print-out-paths', '--max-jobs', '0', '--builders', '',
        '--option', 'substitute', 'false', '.#runtime.package',
    ], output / 'runtime-recipe.log')
    runtime_store = Path(runtime_recipe.splitlines()[-1]).resolve(strict=True)
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
    compiler_env = clean_env.copy()
    compiler_env['COMPACTC_SCHEME'] = str(scheme)
    fresh = output / 'fresh-dual'
    artifacts = {}
    for case, pins in CASES.items():
        circuit = f'bytes_map_{case}'
        source = FAMILY / case / f'{circuit}.compact'
        relative = source.relative_to(ROOT).as_posix()
        require(sha(source) == prior['original_files'][relative] == pins['source'],
                f'{case} original source changed')
        destination = fresh / case
        run([str(compiler), '--target', 'ts', '--target', 'rust',
             '--rust-runtime-root', str(ROOT), '--rust-require-recording', '--skip-zk',
             '--compact-path', str(ROOT / 'compiler'), str(source), str(destination)],
            output / f'{case}-compile.log', env=compiler_env)
        for key, filename in (('typescript', 'index.js'), ('rust', 'lib.rs'),
                              ('rust_ir', 'compact-rust-ir.json')):
            digest = sha(destination / 'contract' / filename)
            require(digest == pins[key] == prior['artifacts'][f'{case}/paired/{key}'],
                    f'{case} generated {key} changed')
            artifacts[f'{case}/{key}'] = digest
        raw = destination / 'zkir' / f'{circuit}.zkir'
        base_raw = base / 'paired' / case / 'zkir' / raw.name
        require(sha(raw) == sha(base_raw) == pins['zkir'],
                f'{case} ZKIR differs from original qualification')
        artifacts[f'{case}/zkir'] = sha(raw)
        run(['node', str(FAMILY / 'check.mjs'),
             str(destination / 'contract/index.js'), circuit],
            output / f'{case}-typescript.log')
    keys = output / 'keys'
    keys.mkdir()
    for case, pins in CASES.items():
        raw = fresh / case / 'zkir' / f'bytes_map_{case}.zkir'
        key_log = run([str(zkir), 'compile', str(raw),
                       str(keys / f'{case}.prover'), str(keys / f'{case}.verifier')],
                      output / f'{case}-key.log')
        require(re.search(rf'\bk={pins["k"]}\b', key_log) is not None,
                f'{case} native ZKIR k changed')
    runner_env = clean_env.copy()
    runner_env['CARGO_TARGET_DIR'] = str(output / 'runner-target')
    run(['cargo', '+1.99.0', 'build', '--locked', '--offline', '--bin',
         'secp256k1-unary-proof', '--manifest-path',
         str(runner_source / 'Cargo.toml')], output / 'runner-build.log',
        cwd=runner_source, env=runner_env)
    runner = output / 'runner-target/debug/secp256k1-unary-proof'
    require(runner.is_file(), 'source-built proof runner missing')
    qualified = {}
    for case, pins in CASES.items():
        if pins['wrong'] is None:
            continue
        destination = fresh / case
        raw = destination / 'zkir' / f'bytes_map_{case}.zkir'
        preimage = output / 'preimage' / case
        preimage.parent.mkdir(exist_ok=True)
        run(['node', str(CAPTURE), case,
             str(destination / 'contract/index.js'), str(raw),
             str(wasm_package), str(preimage)], output / f'{case}-capture.log')
        captured = json.loads((preimage / 'capture.json').read_text())
        require(captured['rawZkirSha256'] == pins['zkir'] and
                captured['generatedTypeScriptSha256'] == pins['typescript'] and
                captured['honestPreimageSha256'] == sha(preimage / 'honest.preimage') and
                captured['checks'] == ['honest accepted', 'changed stored byte refused',
                                       'changed insert count refused'],
                f'{case} preimage capture changed')
        wrong_key = keys / f'{pins["wrong"]}.verifier'
        require(sha(wrong_key) != sha(keys / f'{case}.verifier'),
                f'{case} wrong verifier equals own verifier')
        proof = output / 'proof' / case
        proof.mkdir(parents=True)
        proved = run([str(runner), 'prove', str(raw),
                      str(preimage / 'honest.preimage'), str(proof),
                      str(material / f'bls_midnight_2p{pins["k"]}')],
                     output / f'{case}-prove.log')
        require(proved.startswith(f'proved k={pins["k"]},'),
                f'{case} fresh proof missing')
        verified = run([str(runner), 'verify', str(raw),
                        str(preimage / 'honest.preimage'), str(proof),
                        str(wrong_key)], output / f'{case}-verify.log')
        require('verified in separate process; rejected different verifier, changed binding and corrupt proof'
                in verified, f'{case} independent verifier controls failed')
        qualified[case] = {
            'k': pins['k'], 'wrong_case': pins['wrong'], 'capture': captured,
            'proof_sha256': sha(proof / 'statement.proof'),
            'prover_sha256': sha(proof / 'statement.prover'),
            'verifier_sha256': sha(proof / 'statement.verifier'),
            'wrong_verifier_sha256': sha(wrong_key),
            'prove_log_sha256': sha(output / f'{case}-prove.log'),
            'verify_log_sha256': sha(output / f'{case}-verify.log'),
        }
    require(not git('status', '--porcelain') and git('rev-parse', 'HEAD') == head,
            'source checkout changed during qualification')
    receipt = {
        'head': head, 'tree': tree, 'upstream': PINNED,
        'platform': f'{platform.system()} {platform.machine()}',
        'base_head': BASE, 'base_receipt_sha256': sha(base_receipt),
        'compiler_receipt_sha256': sha(recipe_path),
        'compiler_sha256': sha(compiler), 'scheme_sha256': sha(scheme),
        'capture_source_sha256': sha(CAPTURE), 'gate_source_sha256': sha(Path(__file__)),
        'runtime_nix_store': str(runtime_store),
        'wasm_tgz_sha256': sha(wasm_tgz), 'zkir_binary_sha256': sha(zkir),
        'material_receipt_sha256': sha(material_receipt),
        'params_sha256': PARAMS, 'runner_source_sha256': RUNNER_SOURCE_SHA256,
        'runner_binary_sha256': sha(runner),
        'original_cases': CASES, 'generated_artifacts': artifacts,
        'qualified': qualified,
        'checks': ['three original TypeScript runtime results and internal Compact assertions',
                   'exact-head generated Rust/TypeScript/IR/ZKIR match pinned qualified base',
                   'three honest preimages accepted; changed stored bytes and insert counts refused',
                   'three fresh proofs independently verified with same-k wrong verifiers, changed binding and corrupt proof refusals'],
        'limitations': ['Local platform only; native Linux remains required.',
                        'Funded default-strict transaction and replay refusal remain pending for these three exports.'],
    }
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
    print(f'upstream remaining Bytes map proof gate passed: {output}')


if __name__ == '__main__':
    main()
