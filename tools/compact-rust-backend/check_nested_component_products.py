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

"""Complete nested scalar-product native API qualification. Proof qualification is separate.

Requires a clean source and its original current-head compiler recipe receipt.
Always uses an absent output directory and unedited whole generated SDKs.
"""
import argparse
from pathlib import Path
import os, json, subprocess, time, hashlib, shutil, sys, tomllib, datetime
from check_stateful_product_vector_widening import reject_compiler_overrides, source_manifest_snapshot
ROOT = Path(__file__).resolve().parents[2]
FIXTURES = Path(__file__).with_name('tests') / 'nested_component_products'

def require(condition, message):
    if not condition:
        raise ValueError(message)

def main():
    parser = argparse.ArgumentParser(description='Complete source-bound nested scalar-product native API qualification; proof gates are separate.')
    parser.add_argument('--compiler-receipt', type=Path, required=True)
    parser.add_argument('--ts-runtime', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    W = ROOT
    F = FIXTURES
    O = args.output.absolute()
    require(not O.exists() and (not O.is_symlink()), 'output must be absent')
    head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=W, text=True).strip()
    tree = subprocess.check_output(['git', 'rev-parse', 'HEAD^{tree}'], cwd=W, text=True).strip()
    require(not subprocess.check_output(['git', 'status', '--porcelain'], cwd=W, text=True).strip(), 'clean source required')
    B = json.loads(args.compiler_receipt.read_text())
    require(B['status'] == 'passed' and B['head'] == head and (B['tree'] == tree), 'original recipe receipt must bind current clean source')
    C = Path(B['compactc']['path']).resolve(strict=True)
    S = Path(B['compactc-scheme']['path']).resolve(strict=True)
    require(C.parent == S.parent and C.name == 'compactc' and (S.name == 'compactc-scheme'), 'one original compiler package required')
    require(hashlib.sha256(C.read_bytes()).hexdigest() == B['compactc']['sha256'] and hashlib.sha256(S.read_bytes()).hexdigest() == B['compactc-scheme']['sha256'], 'compiler identity changed')
    RT = args.ts_runtime.resolve(strict=True)
    NODE = Path(shutil.which('node')).resolve(strict=True)
    O.mkdir(parents=True)
    sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()
    g = lambda *a: subprocess.check_output(['git', *a], cwd=W, text=True).strip()
    H = g('rev-parse', 'HEAD')
    require(not g('status', '--porcelain'), 'qualification invariant failed')
    env = dict(os.environ)
    config = reject_compiler_overrides(dict(env))
    env.update(RUSTUP_TOOLCHAIN='1.99.0', CARGO_INCREMENTAL='0', CARGO_BUILD_JOBS='2', CARGO_TARGET_DIR='/Users/ysh/.codex/local-targets/functional-ledger8-completion-20261018/target', PYTHONDONTWRITEBYTECODE='1')
    env.pop('LD_LIBRARY_PATH', None)
    rust = {name: Path(subprocess.check_output(['/Users/ysh/.cargo/bin/rustup', 'which', '--toolchain', '1.99.0', name], text=True).strip()).resolve(strict=True) for name in ['cargo', 'rustc', 'clippy-driver', 'cargo-clippy']}
    env['PATH'] = str(rust['cargo'].parent) + ':/nix/var/nix/profiles/default/bin:/opt/homebrew/bin:/usr/local/bin:' + env['PATH']
    env['RUSTC'] = str(rust['rustc'])
    definitions = json.loads((F / 'sources.json').read_text())
    cases = json.loads((F / 'cases.json').read_text())
    require(len(cases) == len({c['id'] for c in cases}) == 23, 'qualification invariant failed')
    pins = {str(p): sha(p) for p in [C, S, NODE, *rust.values(), Path(__file__), W / 'Cargo.lock', W / 'flake.nix', W / 'flake.lock', args.compiler_receipt.resolve(), *[F / n for n in ['sources.json', 'cases.json', 'consumer.rs', 'capture.mjs']]]}
    for folder in ['compiler', 'runtime-rs', 'runtime-rs-macros', 'tools/compact-rust-backend/src']:
        for p in (W / folder).rglob('*'):
            if p.is_file() and p.suffix in ['.rs', '.ss', '.toml', '.json']:
                pins[str(p)] = sha(p)
    for spec in definitions:
        p = W / spec['path']
        pins[str(p)] = sha(p)
    j = dict(status='running', head=H, inputs=pins, tools={str(p): sha(p) for p in [C, S, NODE, *rust.values()]}, commands=[], sources={}, cargo_configuration_sha256=config, scope='Complete two-source /14export native API cohort with strict roots and23freshTS/four-routeRust cases. Source/artifact/configuration identity pins are not complete physical/transitive build attestation; static/ingress/model/proof/review remain separate.')
    report = O / 'receipt.json'
    root_packages = tomllib.loads((W / 'Cargo.lock').read_text())['package']
    expected = {(p['name'], p['version'], p.get('source')): p.get('checksum') for p in root_packages}
    allowed = {('nested-component-products-consumer', '0.1.0')}
    bound = {}

    def unchanged():
        require(g('rev-parse', 'HEAD') == H and (not g('status', '--porcelain')), 'qualification invariant failed')
        require(all((sha(p) == h for p, h in pins.items())), 'qualification invariant failed')
        require(all((sha(p) == h for p, h in bound.items())), 'qualification invariant failed')

    def run(label, cmd):
        unchanged()
        cmd = list(map(str, cmd))
        j['current'] = label
        report.write_text(json.dumps(j, indent=2) + '\n')
        t = time.monotonic()
        with (O / (label + '.stdout')).open('xb') as out, (O / (label + '.stderr')).open('xb') as err:
            r = subprocess.run(cmd, cwd=W, env=env, stdout=out, stderr=err, timeout=300)
        row = dict(label=label, argv=cmd, exit=r.returncode, seconds=time.monotonic() - t)
        for stream in ['stdout', 'stderr']:
            row[stream + '_sha256'] = sha(O / (label + '.' + stream))
        j['commands'].append(row)
        report.write_text(json.dumps(j, indent=2) + '\n')
        require(r.returncode == 0, (label, r.returncode))
        unchanged()
        return (O / (label + '.stdout')).read_text()

    def lock(path):
        for p in tomllib.loads(path.read_text())['package']:
            key = (p['name'], p['version'], p.get('source'))
            require(key in expected or (not p.get('source') and key[:2] in allowed), key)
            if p.get('source'):
                require(p.get('checksum') == expected[key], 'qualification invariant failed')
        bound[str(path)] = sha(path)
    try:
        raw = {}
        deps = []
        for spec in definitions:
            name = spec['source']
            root = O / 'generated' / name
            run('generate-' + name, [C, '--target', 'rust', '--target', 'ts', '--skip-zk', '--rust-runtime-root', W, '--rust-functional-ledger8', '--rust-require-recording', spec['path'], root])
            run('independent-' + name, [S, '--emit-rust-ir', '--skip-ts', '--skip-zk', spec['path'], O / 'independent' / name])
            require((root / 'contract/compact-rust-ir.json').read_bytes() == (O / 'independent' / name / 'contract/compact-rust-ir.json').read_bytes(), 'qualification invariant failed')
            caps = json.loads((root / 'contract/rust-capabilities.json').read_text())['circuits']
            require({x['name'] for x in caps} == set(spec['entries']), 'qualification invariant failed')
            require(all((x['recorded'] and x['observed_call'] and (x['proof_required'] is True) for x in caps)), 'qualification invariant failed')
            require(len(list((root / 'contract/checked-programs').glob('*.json'))) == len(spec['entries']), 'qualification invariant failed')
            require({x['entry'] for x in cases if x['source'] == name} == set(spec['entries']), 'qualification invariant failed')
            raw[name] = {str(p.relative_to(root)): sha(p) for p in root.rglob('*') if p.is_file()}
            bound.update({str(root / n): h for n, h in raw[name].items()})
            j['sources'][name] = dict(source_sha256=sha(W / spec['path']), manifest=source_manifest_snapshot(root, True), raw_files=raw[name])
            M = root / 'contract/Cargo.toml'
            package = tomllib.loads(M.read_text())['package']['name']
            allowed.add((package, '0.1.0'))
            deps.append(name + '={package=' + json.dumps(package) + ',path=' + json.dumps(str(M.parent)) + ',features=["ledger-transaction"]}')
            shutil.copyfile(W / 'Cargo.lock', M.with_name('Cargo.lock'))
            run(name + '-resolve', ['cargo', 'metadata', '--offline', '--manifest-path', M, '--format-version', '1'])
            lock(M.with_name('Cargo.lock'))
            run(name + '-strict', ['cargo', 'clippy', '--locked', '--offline', '--manifest-path', M, '--all-targets', '--all-features', '--', '-D', 'warnings'])
        modules = O / 'node_modules/@midnight-ntwrk'
        modules.mkdir(parents=True)
        (modules / 'compact-runtime').symlink_to(RT, target_is_directory=True)
        (O / 'package.json').write_text('{"type":"module"}\n')
        ts = O / 'ts-results.json'
        run('fresh-ts', [NODE, F / 'capture.mjs', '--generated', O / 'generated', '--runtime', RT, '--version', '0.16.102', '--output', ts])
        r = json.loads(ts.read_text())
        require(r['status'] == 'passed' and len(r['rows']) == 23, 'qualification invariant failed')
        bound[str(ts)] = sha(ts)
        U = O / 'consumer'
        U.mkdir()
        shutil.copyfile(F / 'consumer.rs', U / 'consumer.rs')
        shutil.copyfile(W / 'Cargo.lock', U / 'Cargo.lock')
        M = U / 'Cargo.toml'
        M.write_text('[workspace]\n[package]\nname="nested-component-products-consumer"\nversion="0.1.0"\nedition="2024"\n[lib]\npath="consumer.rs"\n[dependencies]\nserde_json="1"\n' + '\n'.join(deps) + '\n')
        bound.update({str(p): sha(p) for p in [M, U / 'consumer.rs']})
        env['COMPACT_NESTED_COMPONENT_PRODUCTS_TS'] = str(ts)
        run('consumer-resolve', ['cargo', 'metadata', '--offline', '--manifest-path', M, '--format-version', '1'])
        lock(U / 'Cargo.lock')
        out = run('consumer', ['cargo', 'test', '--locked', '--offline', '--manifest-path', M, '--lib'])
        require('test result: ok. 23 passed; 0 failed; 0 ignored;' in out, 'qualification invariant failed')
        run('consumer-strict', ['cargo', 'clippy', '--locked', '--offline', '--manifest-path', M, '--all-targets', '--', '-D', 'warnings'])
        for name, files in raw.items():
            root = O / 'generated' / name
            require(all((sha(root / n) == h for n, h in files.items())), 'qualification invariant failed')
            require({str(p.relative_to(root)) for p in root.rglob('*') if p.is_file()} == set(files) | {'contract/Cargo.lock'}, 'qualification invariant failed')
        unchanged()
        j.update(status='passed', same_head=True, clean=True, bound_build_files=bound)
    except BaseException as exc:
        j.update(status='failed', error=repr(exc))
    finally:
        j['terminal_utc'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
        report.write_text(json.dumps(j, indent=2) + '\n')
        print(json.dumps({k: v for k, v in j.items() if k not in ['inputs', 'tools', 'sources', 'bound_build_files']}, indent=2))
    require(j['status'] == 'passed', 'qualification invariant failed')
if __name__ == '__main__':
    main()
