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

"""Local Forge CI orchestration. The official helper owns cache identity/metrics."""
import argparse
import hashlib
import re
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time

import forge_circuits as adapter

ROOT = Path(__file__).resolve().parents[2]
QUALIFICATION = Path(__file__).with_name('forge_ci_qualification.json')
DEFAULT_BUDGET = 1024 * 1024 * 1024


def require(ok, message):
    if not ok:
        raise ValueError(message)


def load(path):
    return adapter.read_json(path.read_bytes())


def write(path, value):
    with path.open('x') as stream:
        json.dump(value, stream, indent=2)
        stream.write('\n')


def file_hash(path):
    require(path.is_file() and not path.is_symlink(), 'regular file required: ' + str(path))
    return adapter.digest(path)


def execute(argv, directory, phase, *, env=None, failure=False):
    start = time.monotonic()
    result = subprocess.run(list(map(str, argv)), cwd=ROOT, env=env, capture_output=True, text=True)
    write(directory / (phase + '.command.json'), dict(argv=list(map(str, argv)), exit_code=result.returncode,
          wall_seconds=time.monotonic() - start, stdout=result.stdout, stderr=result.stderr))
    if result.returncode != 0 and not failure:
        # Hosted runners disappear after the job. Surface bounded diagnostics;
        # the complete command receipt remains available for artifact upload.
        for label, content in (('stdout', result.stdout), ('stderr', result.stderr)):
            if content:
                print(phase + ' ' + label + ' (last 8192 characters):\n' + content[-8192:],
                      file=sys.stderr)
    require(result.returncode != 0 if failure else result.returncode == 0,
            phase + (' accepted corrupt verifier' if failure else ' failed; inspect retained log'))
    return result.stdout


def parameters(directory, q, *, allow_missing=False):
    missing = []
    for name, expected in q['parameters'].items():
        path = directory / name
        if not os.path.lexists(path):
            missing.append(name)
        else:
            require(file_hash(path) == expected, 'parameter digest mismatch: ' + name)
    require(allow_missing or not missing, 'missing verified parameters: ' + ', '.join(missing))
    return missing


def profile(forge, q, output):
    adapter.validate_consumer()
    value = adapter.read_json(execute([forge, '--profile', q['selector'], 'profile'], output, 'profile'))
    require(value.get('profile') == adapter.PROFILE
            and value.get('ledger_revision') == adapter.REVISION
            and value.get('build_fingerprint') == q['build_fingerprint'],
            'unqualified Forge profile/build/revision')


def compiler_command(compiler, q, output, *, skip=False):
    return [compiler, '--target', 'rust', *(['--skip-zk'] if skip else []),
            '--rust-runtime-root', ROOT, ROOT / q['compact_source'], output]


def bootstrap(args):
    q = load(QUALIFICATION)
    args.output.mkdir(parents=True, exist_ok=False)
    missing = parameters(args.params_dir, q, allow_missing=True)
    require(not missing or args.fetch_params, 'missing parameters; acquisition requires --fetch-params')
    profile(args.forge, q, args.output)
    if missing:
        args.params_dir.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix='bootstrap-', dir=args.output) as temporary:
            private = Path(temporary)
            sources = {}
            for name in missing:
                row = q['bootstrap_circuits'][name]
                sources.setdefault(row['compact_source'], []).append((name, row['circuit']))
            for index, (source, circuits) in enumerate(sources.items()):
                generated = private / ('generated-' + str(index))
                execute(compiler_command(args.compiler, dict(q, compact_source=source), generated, skip=True),
                        args.output, 'bootstrap-compile-' + str(index))
                for name, circuit in circuits:
                    execute([args.forge, '--profile', q['selector'], '--cache-dir', private / 'store',
                             '--params-dir', args.params_dir, '--fetch-params', 'prepare',
                             generated / 'zkir' / (circuit + '.zkir')], args.output, 'bootstrap-' + name)
        parameters(args.params_dir, q)
    value = dict(parameters_verified=True, acquired=missing, proof_evidence=False)
    write(args.output / 'bootstrap.json', value)
    return value


def tool_bindings(compiler, proof_runner, helper, native=None):
    compiler = compiler.resolve(strict=True)
    paths = dict(compiler=compiler, scheme=compiler.with_name('compactc-scheme'),
                 proof_runner=proof_runner, helper=helper)
    if native:
        paths.update(native_helper=Path(native["helper"]), native_policy=Path(native["policy"]),
                     actual_zkir=Path(native["zkir"]), native_script=Path(native["script"]),
                     forge=Path(native["forge"]))
    return {name: dict(path=str(path.resolve(strict=True)), sha256=file_hash(path.resolve(strict=True)))
            for name, path in paths.items()}


def check_tools(invocation):
    for name, row in invocation['tools'].items():
        require(file_hash(Path(row['path'])) == row['sha256'], name + ' changed after planning')
    if invocation.get('native'):
        native = invocation['native']
        for name, expected in native['expected_tools'].items():
            require(invocation['tools'][name]['sha256'] == expected, 'independent native pin differs: ' + name)
        require(native_workload_inputs(Path(native['script'])) == sorted(native['inputs']),
                'native workload membership changed after planning')
        for name, expected in native['inputs'].items():
            require(file_hash(ROOT / name) == expected, 'native workload changed: ' + name)
        parameters(Path(invocation['params_dir']), dict(parameters=native['parameters']))


def validate_invocation(invocation, key):
    """Reconcile execution aliases with the keyed workload and pinned policy."""
    if not invocation.get('native'):
        return
    native, tools = invocation['native'], invocation['tools']
    workload = adapter.read_json(key['workload'])
    require(workload['tools'] == {name: row['sha256'] for name, row in tools.items()},
            'native tool map differs from keyed workload')
    require(workload['selector'] == 'native-script' and key['selector'] == 'ledger8-v2'
            and Path(key['workspace']).resolve(strict=True) == ROOT.resolve(), 'native workload identity mismatch')
    script = Path(workload['script'])
    require(not script.is_absolute() and '..' not in script.parts, 'unsafe keyed script path')
    expected_script = (ROOT / script).resolve(strict=True)
    require(Path(native['script']).resolve(strict=True) == expected_script
            == Path(tools['native_script']['path']).resolve(strict=True), 'executed native script differs from keyed script')
    for name, tool in (('helper', 'native_helper'), ('policy', 'native_policy'), ('zkir', 'actual_zkir'), ('forge', 'forge')):
        require(Path(native[name]).resolve(strict=True) == Path(tools[tool]['path']).resolve(strict=True),
                'native execution path differs from bound tool: ' + name)
    require(Path(invocation['forge']).resolve(strict=True) == Path(tools['forge']['path']).resolve(strict=True),
            'invocation Forge differs from bound executable')
    policy_path = Path(tools['native_policy']['path'])
    require(file_hash(policy_path) == tools['native_policy']['sha256'], 'native policy changed')
    policy = load(policy_path)
    require(workload['policy'] == tools['native_policy']['sha256'], 'policy differs from keyed workload')
    require(Path(invocation['params_dir']).resolve(strict=True) == Path(key['params_dir']).resolve(strict=True)
            == Path(policy['parameter_directory']).resolve(strict=True), 'native parameter directory differs from key/policy')
    for name, tool in (('forge', 'forge'), ('zkir', 'actual_zkir')):
        require(Path(policy[name]['path']).resolve(strict=True) == Path(tools[tool]['path']).resolve(strict=True)
                and policy[name]['sha256'] == tools[tool]['sha256'], 'native tool differs from pinned policy: ' + name)
    expected_tools = dict(native_policy=tools['native_policy']['sha256'], forge=policy['forge']['sha256'], actual_zkir=policy['zkir']['sha256'])
    require(native['expected_tools'] == expected_tools, 'duplicated native tool pins differ from policy')
    expected_params = {'bls_midnight_2p' + k: value for k, value in policy['parameters'].items()}
    key_params = {row['name']: row['sha256'] for row in key['parameters']}
    require(len(key_params) == len(key['parameters']) and native['parameters'] == key_params == expected_params
            and all((Path(invocation['params_dir']) / row['name']).stat().st_size == row['bytes'] for row in key['parameters']),
            'native parameter declarations differ from key/policy')
    require(type(native['tiny_max_k']) is int and native['selection'] == workload['selection'] == policy.get('selection', 'auto')
            and native['tiny_max_k'] == workload['tiny_max_k'] == policy.get('tiny_max_k', 8)
            and native['build_fingerprint'] == policy['build_fingerprint'], 'native selection/build differs from key/policy')
    producer = key['tool']
    require(producer['binary_sha256'] == tools['forge']['sha256']
            and all(producer['root'][name] == policy[name] for name in ('profile', 'ledger_revision', 'build_fingerprint')),
            'keyed producer differs from native policy')
    key_inputs = {row['path']: row['sha256'] for row in key['inputs']}
    require(len(key_inputs) == len(key['inputs']) and key_inputs == native['inputs']
            and sorted(key_inputs) == native_workload_inputs(expected_script)
            and all((ROOT / row['path']).stat().st_size == row['bytes'] for row in key['inputs']),
            'native workload membership changed: source declarations differ from keyed inventory')


def helper_command(invocation, *args):
    helper = Path(invocation['tools']['helper']['path'])
    return [*([sys.executable, '-B'] if helper.suffix == '.py' else []), helper,
            '--binary', invocation['forge'], *args]


def helper(invocation, directory, phase, *args):
    if invocation.get('native') and phase != 'key':
        validate_invocation(invocation, load(directory / 'key.json'))
    check_tools(invocation)
    value = adapter.read_json(execute(helper_command(invocation, *args), directory, phase))
    check_tools(invocation)
    return value


def workload_inputs(q):
    names = {*q['inputs'], q['compact_source'],
             *(row['compact_source'] for row in q['bootstrap_circuits'].values())}
    for tree in q['input_trees']:
        names.update(p.relative_to(ROOT).as_posix() for p in (ROOT / tree).rglob('*')
                     if p.is_file() and p.suffix in q['input_suffixes'] and '__pycache__' not in p.parts)
    return sorted(names)


def native_workload_inputs(script):
    """Complete source inventory: no suffix filter can hide a relevant new file."""
    roots = ['compiler', 'srcMaps', 'examples', 'test-center/test-contracts', 'testkit-rs',
             '_sources', 'nix', 'third_party',
             'tools', 'tests-rust-backend', '.github/workflows', '.github/actions', '.github/scripts']
    roots += [path.name for path in ROOT.glob('runtime*') if path.is_dir()]
    excluded = {'target', 'node_modules', '__pycache__', '.git', '.venv', 'coverage',
                'artifacts', 'out', 'gen', 'dist', 'reports', 'logs', '.direnv', '.parcel-cache'}
    compiler_outputs = {'testdir', 'javascript-code', 'javascript-tests.ts', 'testfile.compact', 'compactc-version.ss'}
    names = {'Cargo.toml', 'Cargo.lock', 'flake.nix', 'flake.lock'}
    def ignored(path):
        relative = path.relative_to(ROOT)
        return (bool(excluded.intersection(relative.parts)) or path.name == '.DS_Store'
                or (relative.parts[0] == 'compiler' and len(relative.parts) > 1
                    and relative.parts[1] in compiler_outputs))

    def walk_error(error):
        raise error

    for tree in roots:
        root = ROOT / tree
        require(not root.is_symlink(), 'native workload symlink refused: ' + str(root))
        if not root.exists():
            continue
        for directory, directories, files in os.walk(root, topdown=True, followlinks=False,
                                                     onerror=walk_error):
            directory = Path(directory)
            retained = []
            for name in directories:
                path = directory / name
                if ignored(path):
                    continue
                require(not path.is_symlink(), 'native workload symlink refused: ' + str(path))
                retained.append(name)
            directories[:] = retained
            for name in files:
                path = directory / name
                if ignored(path):
                    continue
                require(not path.is_symlink(), 'native workload symlink refused: ' + str(path))
                if path.is_file():
                    names.add(path.relative_to(ROOT).as_posix())
    script = script.resolve(strict=True)
    require(script.is_relative_to(ROOT.resolve()), 'native script must be repository-owned')
    names.add(script.relative_to(ROOT.resolve()).as_posix())
    for name in names:
        file_hash(ROOT / name)
    return sorted(names)


def native_configuration(args):
    selected = [getattr(args, 'native_' + name, None) for name in ('script', 'policy', 'helper')]
    require(not any(selected) or all(selected), 'native-script, native-policy and native-helper require all three')
    if not any(selected):
        return None
    script, policy_path, native_helper = [path.resolve(strict=True) for path in selected]
    require(script.suffix == '.sh', 'native script must be a repository-owned shell file')
    with policy_path.open('rb') as stream:
        policy_bytes = stream.read(16777217)
    require(len(policy_bytes) <= 16777216, 'native policy exceeds JSON bound')
    policy = adapter.read_json(policy_bytes)
    required = {'schema', 'forge', 'zkir', 'profile', 'ledger_revision', 'build_fingerprint',
                'serialization', 'keygen_recipe', 'parameter_directory', 'parameters', 'unavailable'}
    require(isinstance(policy, dict) and required <= set(policy) <= required | {'selection', 'tiny_max_k'},
            'unsupported native policy fields')
    require(type(policy['schema']) is int and policy['schema'] == 1
            and policy['profile'] == adapter.PROFILE and policy['ledger_revision'] == adapter.REVISION
            and policy['serialization'] == adapter.SERIALIZATION
            and policy['keygen_recipe'] == 'library-keygen/default-options/no-rng', 'native policy profile/recipe mismatch')
    require(policy.get('selection', 'auto') in ('auto', 'force')
            and type(policy.get('tiny_max_k', 8)) is int and 0 <= policy.get('tiny_max_k', 8) <= 255
            and policy['unavailable'] in ('refuse', 'keygen'), 'invalid native acquisition policy')
    require(re.fullmatch('[0-9a-f]{64}', policy['build_fingerprint']) is not None, 'invalid independent native build pin')
    for name in ('forge', 'zkir'):
        row = policy[name]
        require(isinstance(row, dict) and set(row) == {'path', 'sha256'}
                and Path(row['path']).is_absolute() and file_hash(Path(row['path'])) == row['sha256'],
                'actual ' + name + ' differs from native policy')
    require(args.forge.resolve(strict=True) == Path(policy['forge']['path']).resolve(strict=True), 'planned Forge differs from native policy')
    require(Path(policy['parameter_directory']).is_absolute()
            and args.params_dir.resolve(strict=True) == Path(policy['parameter_directory']).resolve(strict=True), 'native parameter directory mismatch')
    require(isinstance(policy['parameters'], dict) and policy['parameters'], 'native parameters must be declared')
    declared = {}
    for k, expected in policy['parameters'].items():
        require(re.fullmatch('0|[1-9][0-9]{0,2}', k) is not None and int(k) <= 255
                and re.fullmatch('[0-9a-f]{64}', expected) is not None, 'invalid native parameter pin')
        declared['bls_midnight_2p' + k] = expected
    names = native_workload_inputs(script)
    return dict(script=str(script), policy=str(policy_path), helper=str(native_helper),
                zkir=str(Path(policy['zkir']['path']).resolve(strict=True)), forge=str(Path(policy['forge']['path']).resolve(strict=True)),
                expected_tools=dict(native_policy=hashlib.sha256(policy_bytes).hexdigest(),
                                    forge=policy['forge']['sha256'], actual_zkir=policy['zkir']['sha256']), parameters=declared,
                build_fingerprint=policy['build_fingerprint'], selection=policy.get('selection', 'auto'),
                tiny_max_k=policy.get('tiny_max_k', 8), inputs={name: file_hash(ROOT / name) for name in names})


def plan(args):
    native = native_configuration(args)
    q = load(QUALIFICATION)
    if native:
        q = dict(q, parameters=native['parameters'], build_fingerprint=native['build_fingerprint'])
    parameters(args.params_dir, q)
    args.output.mkdir(parents=True, exist_ok=False)
    profile(args.forge, q, args.output)
    tools = tool_bindings(args.compiler, args.proof_runner, args.helper, native)
    invocation = dict(forge=str(args.forge), params_dir=str(args.params_dir), tools=tools)
    if native:
        invocation['native'] = native
        check_tools(invocation)
        version = execute([native['zkir'], '--version'], args.output, 'native-zkir-version').strip()
        require(version in ('zkir 2.1.1', 'midnight-zkir 2.1.1'), 'native actual ZKIR version mismatch')
        check_tools(invocation)
    # Executable bytes join declared source bytes in the helper's workload identity.
    workload = json.dumps(dict(selector=q['proof_selector'], target='rust',
                         rust_runtime_root='workspace', proofs=3, corrupt_verifier=True,
                         tools={name: row['sha256'] for name, row in tools.items()}), sort_keys=True)
    if native:
        workload = json.dumps(dict(selector='native-script', script=str(Path(native['script']).relative_to(ROOT.resolve())),
                                   policy=tools['native_policy']['sha256'], selection=native['selection'],
                                   tiny_max_k=native['tiny_max_k'], scope='fresh script status, no proof-count inference',
                                   tools={name: row['sha256'] for name, row in tools.items()}), sort_keys=True)
    argv = ['key', '--workspace', ROOT, '--params-dir', args.params_dir,
            '--profile', q['selector'], '--workload', workload, '--generation', args.generation,
            '--output', args.output / 'key.json']
    for name in q['parameters']:
        argv += ['--parameter', name]
    for name in sorted(native['inputs']) if native else workload_inputs(q):
        argv += ['--input', name]
    if args.github_output:
        argv += ['--github-output', args.github_output]
    value = helper(invocation, args.output, 'key', *argv)
    write(args.output / 'invocation.json', invocation)
    return value


def artifact_hashes(output):
    paths = [p for tree in ('zkir', 'keys') for p in (output / tree).rglob('*') if p.is_file()]
    require(paths, 'compiler produced no circuit artifacts')
    return {p.relative_to(output).as_posix(): file_hash(p) for p in sorted(paths)}


def exercise(invocation, directory, q):
    env = dict(os.environ, MIDNIGHT_PP=invocation['params_dir'], COMPACT_ZKIR_MODE='forge')
    with tempfile.TemporaryDirectory(prefix='fixtures-', dir=directory) as temporary:
        output = Path(temporary) / 'generated'
        compiler = invocation['tools']['compiler']['path']
        proof_runner = invocation['tools']['proof_runner']['path']
        check_tools(invocation)
        execute([sys.executable, '-B', Path(__file__).with_name('forge_circuits.py'),
                 '--forge', invocation['forge'], '--cache-dir', directory / 'cache',
                 '--params-dir', invocation['params_dir'], '--receipt-dir', directory / 'events',
                 '--', *compiler_command(compiler, q, output)], directory, 'compile', env=env)
        check_tools(invocation)
        hashes = artifact_hashes(output)
        stdout = execute([proof_runner, q['proof_selector'], output], directory, 'proof', env=env)
        check_tools(invocation)
        require(all(stdout.splitlines().count(marker) == 1 for marker in q['success_markers']),
                'missing or repeated proof/ledger success marker')
        negative = Path(temporary) / 'negative'
        shutil.copytree(output, negative, symlinks=False)
        (negative / 'keys' / (q['corrupt_verifier'] + '.verifier')).write_bytes(b'invalid verifier')
        execute([proof_runner, q['proof_selector'], negative], directory, 'corrupt-verifier', env=env, failure=True)
        check_tools(invocation)
        require(artifact_hashes(output) == hashes, 'consumer changed circuit artifacts')
        write(directory / 'artifacts.json', hashes)


def exercise_native(invocation, directory):
    native, tools = invocation['native'], invocation['tools']
    env = dict(os.environ, MIDNIGHT_PP=invocation['params_dir'], COMPACTC=tools['compiler']['path'],
               COMPACT_RUST_PROOF_RUNNER=tools['proof_runner']['path'],
               COMPACT_ZKIR_MODE='forge', COMPACT_ZKIR_REAL=tools['actual_zkir']['path'])
    # Explicit four pins; native helper alone owns model/acquisition checks.
    argv = [sys.executable, '-B', Path(__file__).with_name('forge_circuits.py'),
            '--forge', invocation['forge'], '--cache-dir', directory / 'cache',
            '--params-dir', invocation['params_dir'], '--receipt-dir', directory / 'events',
            '--native-helper', tools['native_helper']['path'], '--native-helper-sha256', tools['native_helper']['sha256'],
            '--native-policy', tools['native_policy']['path'], '--native-policy-sha256', tools['native_policy']['sha256'],
            '--', 'bash', native['script']]
    check_tools(invocation)
    validate_invocation(invocation, load(directory / 'key.json'))
    execute(argv, directory, 'native-script', env=env)
    validate_invocation(invocation, load(directory / 'key.json'))
    check_tools(invocation)


def snapshot_expectation(args, key):
    expected_id = getattr(args, 'snapshot_id', None)
    expected_partition = getattr(args, 'snapshot_partition', None)
    require((expected_id is None) == (expected_partition is None), 'snapshot ID and partition require both trusted expectations')
    if expected_id is None:
        return []
    require(all(isinstance(value, str) and re.fullmatch('[0-9a-f]{64}', value)
                for value in (expected_id, expected_partition)), 'invalid expected snapshot identity')
    require(args.snapshot.is_dir() and not args.snapshot.is_symlink(), 'supplied snapshot must exist; cold fallback is forbidden')
    require(key['partition'] == expected_partition, 'supplied snapshot partition differs from current planned tools/parameters')
    return ['--expected-id', expected_id]


def run(args):
    directory = args.plan.resolve(strict=True)
    invocation = load(directory / 'invocation.json')
    q = load(QUALIFICATION)
    if invocation.get('native'):
        q = dict(q, parameters=invocation['native']['parameters'], build_fingerprint=invocation['native']['build_fingerprint'])
    if args.max_bytes is None:
        args.max_bytes = 268435456 if invocation.get('native') else DEFAULT_BUDGET
    require(args.max_bytes > 0, 'byte budget must be positive')
    if args.snapshot_out:
        require(not args.snapshot_out.resolve().is_relative_to((directory / 'cache').resolve()),
                'snapshot output must be outside cache')
    expected_snapshot = snapshot_expectation(args, load(directory / 'key.json'))
    # Exclusive marker refuses reruns and stale test verdict reuse, including failed attempts.
    write(directory / 'run-started.json', dict(started_ns=time.time_ns()))
    planned_tools = json.loads(load(directory / 'key.json')['workload'])['tools']
    require(planned_tools == {name: row['sha256'] for name, row in invocation['tools'].items()},
            'invocation tool hashes differ from the helper workload')
    validate_invocation(invocation, load(directory / 'key.json'))
    check_tools(invocation)
    parameters(Path(invocation['params_dir']), q)
    common = ['--key', directory / 'key.json', '--cache-dir', directory / 'cache']
    restored = helper(invocation, directory, 'restore', 'restore', *common,
           '--snapshot', args.snapshot, '--max-bytes', args.max_bytes, *expected_snapshot)
    if expected_snapshot:
        require(restored.get('restored') is True, 'supplied snapshot was not restored; cold fallback is forbidden')
    helper(invocation, directory, 'baseline', 'baseline', *common, '--output', directory / 'baseline.json')
    try:
        if invocation.get('native'):
            exercise_native(invocation, directory)
        else:
            exercise(invocation, directory, q)
        if args.uncached_zkir:
            validate_invocation(invocation, load(directory / 'key.json'))
            execute([sys.executable, '-B', Path(__file__).with_name('check_forge_circuit_cache.py'),
                     '--compiler', invocation['tools']['compiler']['path'],
                     '--proof-runner', invocation['tools']['proof_runner']['path'],
                     '--forge', invocation['forge'], '--params-dir', invocation['params_dir'],
                     '--uncached-zkir', args.uncached_zkir, '--report', directory / 'uncached-report.json'],
                    directory, 'uncached')
            check_tools(invocation)
    except BaseException:
        # Keep the original failure if the post-failure identity/metrics check also refuses.
        try:
            report = helper(invocation, directory, 'report', 'report', *common, '--baseline', directory / 'baseline.json')
            write(directory / 'cache-report.json', report)
        except (ValueError, OSError):
            pass
        raise
    report = helper(invocation, directory, 'report', 'report', *common, '--baseline', directory / 'baseline.json')
    write(directory / 'cache-report.json', report)
    if args.snapshot_out:
        key = load(directory / 'key.json')
        scope = [item for row in key['parameters'] for item in ['--parameter-digest', row['sha256']]]
        validate_invocation(invocation, key)
        execute([invocation['forge'], '--profile', q['selector'], '--cache-dir', directory / 'cache',
                 'snapshot', '--output', args.snapshot_out, '--partition', key['partition'],
                 '--max-bytes', args.max_bytes, *scope], directory, 'snapshot')
        check_tools(invocation)
    validate_invocation(invocation, load(directory / 'key.json'))
    value = (dict(passed=True, mode='native-script', proof_scenarios=None,
                  scope='supplied script exited successfully; no inferred specification/proof coverage')
             if invocation.get('native') else dict(passed=True, proof_scenarios=3, corrupt_verifier_rejected=True))
    write(directory / 'run.json', value)
    return report


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    subs = parser.add_subparsers(dest='command', required=True)
    for name in ('bootstrap', 'plan'):
        p = subs.add_parser(name)
        for flag in ('compiler', 'forge', 'params-dir', 'output'):
            p.add_argument('--' + flag, type=Path, required=True)
        if name == 'bootstrap':
            p.add_argument('--fetch-params', action='store_true')
        else:
            p.add_argument('--proof-runner', type=Path, required=True)
            p.add_argument('--helper', type=Path, required=True)
            p.add_argument('--generation', required=True)
            p.add_argument('--github-output', type=Path)
            p.add_argument('--native-script', type=Path)
            p.add_argument('--native-policy', type=Path)
            p.add_argument('--native-helper', type=Path)
    p = subs.add_parser('run')
    p.add_argument('--plan', type=Path, required=True)
    p.add_argument('--snapshot', type=Path, required=True)
    p.add_argument('--snapshot-id')
    p.add_argument('--snapshot-partition')
    p.add_argument('--uncached-zkir', type=Path)
    p.add_argument('--snapshot-out', type=Path)
    p.add_argument('--max-bytes', type=int)
    args = parser.parse_args(argv)
    for name, value in vars(args).items():
        if isinstance(value, Path):
            setattr(args, name, value.absolute())
    print(json.dumps(dict(bootstrap=bootstrap, plan=plan, run=run)[args.command](args)))


if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError, KeyError, TypeError) as error:
        print(json.dumps(dict(error=str(error))), file=sys.stderr)
        raise SystemExit(1)
