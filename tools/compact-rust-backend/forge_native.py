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

"""Explicit pinned native provider for one current test circuit, never proof results."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile

PINS = ('COMPACT_ZKIR_NATIVE_HELPER', 'COMPACT_ZKIR_NATIVE_HELPER_SHA256',
        'COMPACT_ZKIR_NATIVE_POLICY', 'COMPACT_ZKIR_NATIVE_POLICY_SHA256')


def require(value, message):
    if not value:
        raise ValueError(message)


def digest(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


MAX_JSON = 16 * 1024 * 1024


def read_bounded(path, limit=MAX_JSON):
    require(path.stat().st_size <= limit, 'native input exceeds finite bound')
    with path.open('rb') as stream:
        raw = stream.read(limit + 1)
    require(len(raw) <= limit, 'native input grew beyond finite bound')
    return raw


def declared_digest(path, size):
    require(path.stat().st_size == size, 'native artifact declared size changed')
    total, hasher = 0, hashlib.sha256()
    with path.open('rb') as stream:
        while True:
            chunk = stream.read(min(1024 * 1024, size - total + 1))
            if not chunk:
                break
            total += len(chunk)
            require(total <= size, 'native artifact grew beyond declaration')
            hasher.update(chunk)
    require(total == size, 'native artifact shortened')
    return hasher.hexdigest()


def copy_verified(source, destination, expected):
    """Copy at most declared size plus one byte; never replace on mismatch."""
    require(source.stat().st_size == expected['size'], 'native artifact declared size changed')
    hasher = hashlib.sha256()
    total = 0
    with source.open('rb') as incoming, destination.open('wb') as outgoing:
        while True:
            chunk = incoming.read(min(1024 * 1024, expected['size'] - total + 1))
            if not chunk:
                break
            total += len(chunk)
            require(total <= expected['size'], 'native artifact grew beyond declaration')
            hasher.update(chunk)
            outgoing.write(chunk)
    require(total == expected['size'] and hasher.hexdigest() == expected['digest'], 'native artifact changed while copying')


def bounded_process(helper, env):
    # Disk spools prevent unbounded Python capture; the pinned helper separately
    # bounds its own JSON writer. Read each spool with the same finite limit.
    with tempfile.TemporaryFile() as stdout, tempfile.TemporaryFile() as stderr:
        result = subprocess.run([str(helper)], env=env, stdout=stdout, stderr=stderr, check=False)
        values = []
        for stream in [stdout, stderr]:
            require(stream.tell() <= MAX_JSON, 'native child output exceeds finite bound')
            stream.seek(0)
            raw = stream.read(MAX_JSON + 1)
            require(len(raw) <= MAX_JSON, 'native child output grew beyond finite bound')
            values.append(raw.decode('utf-8'))
        if result.returncode:
            raise subprocess.CalledProcessError(result.returncode, [str(helper)], output=values[0], stderr=values[1])
        return values


def fields(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, 'duplicate native JSON field: ' + key)
        result[key] = value
    return result


def decode(raw):
    require(len(raw) <= 16 * 1024 * 1024, 'native JSON too large')
    def invalid_constant(value):
        raise ValueError('non-finite native JSON number: ' + value)
    return json.loads(raw, object_pairs_hook=fields, parse_constant=invalid_constant)


def selected(env):
    present = [key in env for key in PINS]
    require(not any(present) or all(present), 'all four native provider pins required')
    if any(present):
        require(env.get('COMPACT_ZKIR_MODE', 'uncached') == 'forge', 'native provider requires explicit forge mode')
        require(all(env[key] for key in PINS), 'empty native provider pin')
    return any(present)


def exact(value, keys):
    require(isinstance(value, dict) and set(value) == set(keys), 'unexpected native receipt fields')


def overlap(left, right):
    left, right = Path(left).resolve(), Path(right).resolve()
    return left.is_relative_to(right) or right.is_relative_to(left)


def compile_circuit(source, prover, verifier, real_zkir, env):
    require(selected(env), 'native provider not selected')
    original_source = Path(source)
    original_helper, original_policy = [Path(env[key]) for key in (PINS[0], PINS[2])]
    source = original_source.resolve(strict=True)
    helper, policy_path = [p.resolve(strict=True) for p in (original_helper, original_policy)]
    expected_helper, expected_policy = env[PINS[1]], env[PINS[3]]
    require(all(re.fullmatch('[0-9a-f]{64}', x) for x in (expected_helper, expected_policy)), 'invalid native pin hash')
    policy_bytes = read_bounded(policy_path)
    require(digest(helper) == expected_helper and hashlib.sha256(policy_bytes).hexdigest() == expected_policy, 'native helper/policy pin mismatch')
    policy = decode(policy_bytes)
    original_zkir = Path(real_zkir)
    zkir = original_zkir.resolve(strict=True)
    require(zkir == Path(policy['zkir']['path']).resolve(strict=True)
            and digest(zkir) == policy['zkir']['sha256'], 'actual ZKIR differs from native policy')
    original_params = Path(env['MIDNIGHT_PP'])
    params = original_params.resolve(strict=True)
    require(params == Path(policy['parameter_directory']).resolve(strict=True), 'actual parameters differ from native policy')
    original_store = Path(env.get('MIDNIGHT_ZKIR_FORGE_DIR', str(Path.home() / '.midnight-zkir-forge')))
    store = original_store.resolve()
    destinations = [source.with_suffix('.bzkir'), Path(prover), Path(verifier)]
    protected = [helper, policy_path, zkir, Path(policy['forge']['path']), params, store, source]
    require(len({p.resolve() for p in destinations}) == 3, 'native output paths must be distinct')
    for destination in destinations:
        require(not destination.is_symlink() and all(not overlap(destination, p) for p in protected),
                'native output aliases a protected input/store')
    source_bytes = read_bounded(source)
    source_sha = hashlib.sha256(source_bytes).hexdigest()
    bindings = [(p, p.resolve()) for p in [original_source, original_helper, original_policy, original_zkir,
                original_params, original_store, Path(policy['zkir']['path']),
                Path(policy['parameter_directory']), Path(policy['forge']['path']), *destinations]]
    parameter_hashes = {}

    def unchanged():
        require(digest(helper) == expected_helper and read_bounded(policy_path) == policy_bytes,
                'native helper/policy changed')
        require(hashlib.sha256(read_bounded(source)).hexdigest() == source_sha and digest(zkir) == policy['zkir']['sha256'], 'native source/tool changed')
        require(all(p.resolve() == canonical for p, canonical in bindings), 'native input alias changed')
        require(all(digest(path) == sha for path, sha in parameter_hashes.items()), 'native parameters changed')
        for destination in destinations:
            require(not destination.is_symlink() and all(not overlap(destination, p) for p in protected), 'native output alias changed')

    with tempfile.TemporaryDirectory(prefix='compact-native-circuit-') as private:
        private = Path(private)
        inputs, keys = private / 'inputs', private / 'keys'
        inputs.mkdir()
        current = inputs / (source.stem + '.zkir')
        current.write_bytes(read_bounded(source))
        require(hashlib.sha256(read_bounded(current)).hexdigest() == source_sha, 'native source changed before copy')
        child = dict(env, COMPACT_ZKIR_CACHE_REQUEST=json.dumps({'schema': 1, 'inputs': str(inputs),
                     'keys': str(keys), 'policy': str(policy_path), 'cache': str(store), 'final_output': None, 'report': True}))
        unchanged()
        stdout, _stderr = bounded_process(helper, child)
        value = decode(stdout)
        exact(value, ['schema', 'circuits'])
        require(type(value['schema']) is int and value['schema'] == 1 and isinstance(value['circuits'], list) and len(value['circuits']) == 1,
                'native receipt requires one circuit')
        row = value['circuits'][0]
        exact(row, ['name', 'source_sha256', 'k', 'rows', 'parameter_sha256', 'acquisition_path', 'artifacts'])
        require(row['name'] == source.stem and row['source_sha256'] == source_sha, 'native receipt source mismatch')
        require(type(row['k']) is int and 0 <= row['k'] <= 255 and type(row['rows']) is int and 0 <= row['rows'] < 2**64, 'invalid fresh native model')
        require(row['acquisition_path'] in ['forge-hit', 'forge-miss', 'auto-bypass', 'unavailable-keygen'], 'unknown native acquisition path')
        require(row['parameter_sha256'] == policy['parameters'].get(str(row['k'])), 'native parameter digest differs from policy')
        parameter_path = params / ('bls_midnight_2p' + str(row['k']))
        require(digest(parameter_path) == row['parameter_sha256'], 'native parameter bytes changed')
        parameter_hashes[parameter_path] = row['parameter_sha256']
        require(isinstance(row['artifacts'], list) and len(row['artifacts']) == 3, 'native artifact count')
        by_kind = {}
        paths = {'binary_ir': current.with_suffix('.bzkir'), 'prover': keys / (source.stem + '.prover'), 'verifier': keys / (source.stem + '.verifier')}
        for artifact in row['artifacts']:
            exact(artifact, ['kind', 'digest', 'size'])
            kind = artifact['kind']
            require(kind in paths and kind not in by_kind, 'duplicate/unknown native artifact')
            require(type(artifact['size']) is int and 0 < artifact['size'] <= 1024 ** 3
                    and re.fullmatch('[0-9a-f]{64}', artifact['digest']), 'invalid native artifact identity')
            require(not paths[kind].is_symlink() and paths[kind].stat().st_size == artifact['size']
                    and declared_digest(paths[kind], artifact['size']) == artifact['digest'], 'native artifact bytes differ from receipt')
            by_kind[kind] = artifact
        require(hashlib.sha256(read_bounded(current)).hexdigest() == source_sha, 'private native input changed')
        unchanged()
        for kind, destination in zip(['binary_ir', 'prover', 'verifier'], destinations):
            unchanged()
            destination.parent.mkdir(parents=True, exist_ok=True)
            with tempfile.NamedTemporaryFile(dir=destination.parent, prefix='.native-', delete=False) as stream:
                temporary = Path(stream.name)
            try:
                copy_verified(paths[kind], temporary, by_kind[kind])
                temporary.replace(destination)
            finally:
                temporary.unlink(missing_ok=True)
        unchanged()
    return {'provider': 'pinned native artifact helper', 'source_sha256': source_sha, 'k': row['k'], 'rows': row['rows'],
            'parameter_digest': row['parameter_sha256'], 'acquisition_path': row['acquisition_path'],
            'model_provider': 'actual zkir mock-compile (fresh once in native helper; not keygen)',
            'materialization': 'verified private copies; individual replacements, not a three-file transaction',
            'native_helper_sha256': expected_helper, 'native_policy_sha256': expected_policy, 'artifacts': row['artifacts']}
