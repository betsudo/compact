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

"""Authenticate the source-built equality observer and inventory all intact models.

No keys, proofs or constructor cryptographic/integration acceptance.
"""
from pathlib import Path
import argparse, datetime, hashlib, json, os, shutil, subprocess, time, sys, tomllib
if not __debug__ or sys.flags.optimize or os.environ.get('PYTHONOPTIMIZE'):
    raise RuntimeError('optimized Python invocation is not admitted')
import constructor_proof_dependencies as dependencies
import check_functional_source_sdk as sdk
import check_passport_current_profile as process_owner

def main(*, boundaries=False):
    W = Path(__file__).resolve().parents[2]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--producer-root', type=Path, required=True)
    parser.add_argument('--producer-receipt', type=Path, required=True)
    if boundaries:
        parser.add_argument('--scope-receipt', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    D = args.output.resolve()
    if D.exists() or args.output.is_symlink() or D.is_relative_to(W):
        raise ValueError('fresh output outside source required')
    D.mkdir(parents=True, mode=0o700)
    root = args.producer_root.resolve(strict=True)
    functional = args.producer_receipt.resolve(strict=True)
    sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()
    git = lambda *args: subprocess.check_output(['git', *args], cwd=W, text=True).strip()
    assert not git('status', '--porcelain')
    source = git('rev-parse', 'HEAD')
    accepted = json.loads(functional.read_text())
    assert accepted['status'] == 'passed' and accepted['head']==source and accepted['tree']==git('rev-parse','HEAD^{tree}')
    fixture = 'constructor_equality_boundaries' if boundaries else 'constructor_equality'
    inventory_flag = '--constructor-equality-boundaries-inventory' if boundaries else '--constructor-equality-inventory'
    observation_timeout = 600 if boundaries else 120
    roster_path = W / 'tools/compact-rust-backend/tests' / fixture / 'sources.json'
    if boundaries:
        assert accepted.get('cohort') == 'operand-boundaries', 'boundary producer cohort differs'
    assert sha(roster_path) == accepted['inputs'][str(roster_path)]
    specs = json.loads(roster_path.read_text())
    assert len(specs)==8 and {s['role'] for s in specs}==set(accepted['sources'])
    assert all(s['entries']==['read','advance','advance_other'] for s in specs)
    for spec in specs:
        assert sha(W/spec['path'])==spec['sha256']==accepted['sources'][spec['role']]['source_sha256']
    pins = {str(functional): sha(functional), str(Path(__file__)): sha(__file__), str(Path(dependencies.__file__)):sha(dependencies.__file__)}
    dependency_sources={}
    sdk_roots={}
    pins.update(accepted['inputs'])
    scope_roots = {}
    if boundaries:
        scope_path = args.scope_receipt.resolve(strict=True)
        scope = json.loads(scope_path.read_text())
        assert scope['status'] == 'passed' and scope['head'] == source and scope['tree'] == git('rev-parse', 'HEAD^{tree}')
        assert scope['complete_sources'] == 6 and len(scope['commands']) == 18 and len(scope['HOST_controls']) == 10
        assert not scope['sdk_execution'] and not scope['proof_execution']
        pins[str(scope_path)] = sha(scope_path)
        for path, digest in scope['inputs'].items():
            assert Path(path).is_absolute() and sha(path) == digest, path
            pins[path] = digest
        for relative, digest in scope['artifacts'].items():
            path = (scope_path.parent / relative).resolve(strict=True)
            assert not Path(relative).is_absolute() and path.is_relative_to(scope_path.parent), relative
            assert sha(path) == digest, relative
            pins[str(path)] = digest
        scope_roots = scope['frozen_directories']
        for path, expected_inventory in scope_roots.items():
            sdk.equal_inventory(Path(path), expected_inventory)
    for tool in [Path(sys.executable).resolve(), Path(shutil.which('rustup')).resolve(), Path(dependencies.GIT)]:
        pins[str(tool)]=sha(tool)
    for role, row in accepted['sources'].items():
        complete=dict(row['manifest']['assets'])
        complete['compiler/contract-manifest.json']=row['manifest']['manifest_sha256']
        sdk.equal_inventory(root/role,complete)
        sdk_roots[str(root/role)]=complete
        pins[str(root/role/'compiler/contract-manifest.json')]=complete['compiler/contract-manifest.json']
        for rel, digest in row['manifest']['assets'].items():
            p = root / role / rel
            assert sha(p) == digest, p
            pins[str(p)] = digest
    for rel in git('ls-files', 'tools/compact-rust-proof-smoke', 'tests-rust-backend', 'runtime-rs', 'runtime-rs-macros', 'testkit-rs', 'Cargo.toml', 'Cargo.lock', '.cargo', str(roster_path.relative_to(W))).splitlines():
        p = W / rel
        if p.is_file(): pins[str(p)] = sha(p)
    env = dict(os.environ, RUSTUP_TOOLCHAIN='1.99.0', CARGO_BUILD_JOBS=os.environ.get('CARGO_BUILD_JOBS','2'), CARGO_CACHE_AUTO_CLEAN_FREQUENCY='never', CARGO_INCREMENTAL='0', PYTHONDONTWRITEBYTECODE='1')
    env.setdefault('CARGO_TARGET_DIR', str(D/'cargo-target'))
    assert not Path(env['CARGO_TARGET_DIR']).resolve().is_relative_to(W)
    configuration = sdk.cargo_configuration(env)
    env.pop('LD_LIBRARY_PATH', None)
    r = dict(format='constructor-equality-boundaries-model-observation/v1' if boundaries else 'constructor-equality-model-observation/v1', status='running', producer_source=accepted['head'], qualifier_source=source, qualifier_tree=git('rev-parse', 'HEAD^{tree}'), started_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(), inputs=pins, commands=[], key_generation=False, proof_execution=False, scope='intact model k observation and observer refusal controls only; no proof eligibility or full toolchain admission')
    save = lambda: (D / 'receipt.json').write_text(json.dumps(r, indent=2) + '\n')
    save()
    def run(label, argv, expected=0, diagnostic=None, timeout=600):
        assert git('rev-parse','HEAD')==source and not git('status','--porcelain')
        assert sdk.cargo_configuration(env)==configuration
        start = time.monotonic()
        for name, digest in pins.items(): assert sha(name) == digest, name
        for name, expected_inventory in sdk_roots.items(): sdk.equal_inventory(Path(name),expected_inventory)
        for name, expected_inventory in scope_roots.items(): sdk.equal_inventory(Path(name),expected_inventory)
        dependencies.unchanged(dependency_sources)
        out, err = D / (label + '.stdout'), D / (label + '.stderr')
        if boundaries:
            # Reuse the existing group owner for Cargo/rustc descendants too.
            assert timeout == process_owner.COMMAND_TIMEOUT_SECONDS, 'boundary commands use the maintained finite owner'
            try:
                p = process_owner.bounded_command(list(map(str, argv)), env)
            except subprocess.TimeoutExpired as error:
                for path, value in ((out, error.stdout), (err, error.stderr)):
                    path.write_text(value.decode(errors='replace') if isinstance(value, bytes) else value or '')
                r['commands'].append(dict(label=label, argv=list(map(str, argv)), exit=None,
                    timed_out=True, timeout_seconds=process_owner.COMMAND_TIMEOUT_SECONDS, seconds=time.monotonic()-start, stdout=str(out), stderr=str(err),
                    stdout_sha256=sha(out), stderr_sha256=sha(err)))
                save()
                raise
            out.write_text(p.stdout)
            err.write_text(p.stderr)
        else:
            with out.open('xb') as o, err.open('xb') as e:
                p = subprocess.run(list(map(str, argv)), cwd=W, env=env, stdout=o, stderr=e, timeout=timeout)
        row = dict(label=label, argv=list(map(str, argv)), exit=p.returncode, seconds=time.monotonic()-start, stdout=str(out), stderr=str(err), stdout_sha256=sha(out), stderr_sha256=sha(err))
        if boundaries:
            row['timeout_seconds'] = process_owner.COMMAND_TIMEOUT_SECONDS
        r['commands'].append(row);save()
        assert (p.returncode == 0) if expected == 0 else (p.returncode != 0), row
        if diagnostic: assert diagnostic in err.read_text(), row
        return out
    try:
        tool_paths={}
        for tool in ['cargo', 'rustc']:
            path = subprocess.check_output(['rustup', 'which', tool], cwd=W, env=env, text=True).strip()
            pins[path] = sha(path)
            tool_paths[tool]=path
            run(tool + '-version', [path, '--version', '--verbose'])
        env['PATH']=str(Path(tool_paths['cargo']).parent)+os.pathsep+env['PATH']
        run('metadata', ['cargo', 'metadata', '--locked', '--offline', '--format-version', '1', '--features', 'compact-rust-proof-smoke/constructor-vector-models'])
        metadata=json.loads((D/'metadata.stdout').read_text())
        cohort={(p['name'],p['version'],p.get('source')):p.get('checksum') for p in tomllib.loads((W/'Cargo.lock').read_text())['package']}
        dependencies.authenticate(metadata,cohort,dependency_sources)
        r['dependency_sources']=dependency_sources;r['cargo_configuration']=configuration;save()
        artifacts = run('build', ['cargo', 'build', '--profile', 'proof-checks', '--locked', '--offline', '-p', 'compact-rust-proof-smoke', '--features', 'constructor-vector-models', '--message-format', 'json'])
        candidates = [j['executable'] for line in artifacts.read_text().splitlines() if (j := json.loads(line)).get('reason') == 'compiler-artifact' and j.get('executable') and j['target']['name'] == 'compact-rust-proof-smoke']
        assert len(candidates) == 1, candidates
        built=Path(candidates[0]);(D/'bin').mkdir();binary=D/'bin/compact-rust-proof-smoke'
        shutil.copyfile(built,binary);assert sha(binary)==sha(built);binary.chmod(0o555)
        pins[str(binary)]=sha(binary);r['binary']=dict(path=str(binary),sha256=sha(binary),built_path=str(built),built_sha256=sha(built));save()
        run('clippy', ['cargo', 'clippy', '--profile', 'proof-checks', '--locked', '--offline', '-p', 'compact-rust-proof-smoke', '--bin', 'compact-rust-proof-smoke', '--features', 'constructor-vector-models', '--', '-D', 'warnings'])
        output = run('inventory', [binary, inventory_flag, root], timeout=observation_timeout)
        inventory = json.loads(output.read_text());rows=inventory['circuits']
        assert len(rows) == 24 and {(x['source'], x['circuit']) for x in rows} == {(s['role'], e) for s in specs for e in s['entries']}
        assert not inventory['key_generation'] and not inventory['proof_execution'] and not inventory['selection_made']
        for x in rows:
            p = root / x['source'] / 'zkir' / (x['circuit']+'.zkir')
            assert x['zkir_bytes'] == p.stat().st_size
            assert x['within_tiny_automatic_cap'] == (x['k'] <= 8)
            x['zkir_sha256'] = pins[str(p)]
        r['inventory'] = inventory
        # These are deliberately minimal HOST fixtures for membership refusal tests,
        # not alternate producer SDKs or proof inputs. Original SDKs stay untouched.
        negative = D / 'HOST-membership-controls';negative.mkdir()
        for s in specs:
            target = negative / s['role'];(target/'contract').mkdir(parents=True);(target/'zkir').mkdir()
            shutil.copyfile(root/s['role']/'contract/rust-capabilities.json', target/'contract/rust-capabilities.json')
            for e in s['entries']: shutil.copyfile(root/s['role']/'zkir'/(e+'.zkir'), target/'zkir'/(e+'.zkir'))
            (negative/('.'+s['role']+'.compactc.lock')).touch()
        run('host-positive-control', [binary, inventory_flag, negative], timeout=observation_timeout)
        extra = negative/'unknown';extra.mkdir()
        run('extra-directory', [binary,inventory_flag,negative],1,'complete constructor SDK membership differs');extra.rename(D/'HOST-retained-extra-directory')
        lock = negative/('.'+specs[0]['role']+'.compactc.lock');lock.rename(D/'HOST-retained-lock');lock.mkdir()
        run('lock-is-directory', [binary,inventory_flag,negative],1,'constructor SDK directory or lock sidecar kind differs');lock.rename(D/'HOST-retained-lock-directory');shutil.copyfile(D/'HOST-retained-lock',lock)
        role = negative/specs[0]['role'];role.rename(D/'HOST-retained-role');role.symlink_to(D/'HOST-retained-role',target_is_directory=True)
        run('role-is-symlink',[binary,inventory_flag,negative],1,'constructor SDK directory or lock sidecar kind differs');role.rename(D/'HOST-retained-role-symlink');shutil.copytree(D/'HOST-retained-role',role)
        model = role/'zkir'/(specs[0]['entries'][0]+'.zkir');model.rename(D/'HOST-retained-model')
        run('missing-model',[binary,inventory_flag,negative],1,'complete constructor model membership differs');shutil.copyfile(D/'HOST-retained-model',model)
        extra = role/'zkir'/'unknown.zkir';extra.write_bytes(b'not a model')
        run('extra-model',[binary,inventory_flag,negative],1,'complete constructor model membership differs');extra.rename(D/'HOST-retained-extra-model')
        dependencies.unchanged(dependency_sources)
        for name,digest in pins.items(): assert sha(name)==digest,name
        for name,expected_inventory in sdk_roots.items(): sdk.equal_inventory(Path(name),expected_inventory)
        for name,expected_inventory in scope_roots.items(): sdk.equal_inventory(Path(name),expected_inventory)
        assert git('rev-parse','HEAD')==source and not git('status','--porcelain')
        assert sdk.cargo_configuration(env)==configuration
        r.update(status='passed', input_bytes_unchanged=True, observed_models=len(rows), observed_k=sorted(set(x['k'] for x in rows)), within_numeric_cap=sum(x['k']<=8 for x in rows), missing_or_extra_members_refused=True, frozen_directories=sdk_roots)
        if boundaries:
            r['scope_receipt'] = str(scope_path)
            r['frozen_directories'] = {**sdk_roots, **scope_roots}
    except BaseException as error:
        r.update(status='failed',error=repr(error));raise
    finally:
        save();print(json.dumps({k:r[k] for k in ['status','producer_source','qualifier_source','observed_models','observed_k','key_generation','proof_execution','error'] if k in r},indent=2),flush=True)


if __name__ == "__main__":
    main()
