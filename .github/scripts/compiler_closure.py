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
"""Same-run .#compactc runtime-closure handoff, not a cached test verdict."""
import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import sys
import urllib.request
import zipfile

ROOT = Path(__file__).resolve().parents[2]
STORE = re.compile(r'/nix/store/[a-z0-9]{32}-[^/\s]+')


def require(condition, message):
    if not condition:
        raise ValueError(message)


def run(argv, **kwargs):
    if 'stdout' not in kwargs and not kwargs.get('capture_output'):
        kwargs['stdout'] = sys.stderr
    return subprocess.run(argv, cwd=ROOT, check=True, **kwargs)


def text(argv):
    # Keep machine-readable stdout separate while streaming build diagnostics.
    # Capturing stderr here hid the actual Nix error behind CalledProcessError.
    return run(argv, stdout=subprocess.PIPE, text=True).stdout.strip()


def sha(path):
    require(path.is_file() and not path.is_symlink(), 'expected regular artifact: ' + str(path))
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def producer_attempt(current, produced):
    require(isinstance(produced, str) and produced.isdigit() and current.isdigit()
            and 1 <= int(produced) <= int(current), 'invalid producer attempt')
    return produced


def identity(args):
    require(re.fullmatch('[0-9a-f]{40}', args.source_sha), 'invalid source SHA')
    require(args.system in ('x86_64-linux', 'aarch64-darwin'), 'unsupported system')
    require(args.run_id.isdigit() and args.run_attempt.isdigit(), 'invalid run identity')
    require(text(['git', 'rev-parse', 'HEAD']) == args.source_sha, 'checkout SHA mismatch')
    require(not text(['git', 'status', '--porcelain', '--untracked-files=no']), 'tracked source mutation')
    require(text(['nix', 'eval', '--impure', '--raw', '--expr', 'builtins.currentSystem']) == args.system,
            'platform mismatch')
    actual_attempt = producer_attempt(args.run_attempt, getattr(args, 'producer_attempt', None) or args.run_attempt)
    return dict(source_sha=args.source_sha, system=args.system, run_id=args.run_id,
                run_attempt=actual_attempt, flake_sha256=sha(ROOT/'flake.nix'),
                lock_sha256=sha(ROOT/'flake.lock'))


def info(paths):
    rows = json.loads(text(['nix', 'path-info', '--json', *paths]))
    require(isinstance(rows, dict) and set(rows) == set(paths), 'unexpected Nix path-info roster')
    return {p: rows[p]['narHash'] for p in sorted(rows)}


def canonical_sri(value):
    if not isinstance(value, str) or not value.startswith('sha256-'):
        return False
    try:
        raw = base64.b64decode(value[7:], validate=True)
        return len(raw) == 32 and base64.b64encode(raw).decode() == value[7:]
    except ValueError:
        return False


def validate_receipt(value, expected, archive):
    require(set(value) == {'schema', 'identity', 'derivation', 'output', 'closure', 'archive_sha256'},
            'unknown/missing receipt fields')
    require(value['schema'] == 1 and value['identity'] == expected, 'handoff identity mismatch')
    require(STORE.fullmatch(value['derivation']) and value['derivation'].endswith('.drv'), 'invalid derivation')
    require(STORE.fullmatch(value['output']), 'invalid output')
    closure = value['closure']
    require(isinstance(closure, dict) and value['output'] in closure and 0 < len(closure) <= 10000,
            'invalid closure membership')
    require(all(STORE.fullmatch(p) and canonical_sri(h) for p, h in closure.items()),
            'invalid closure identities')
    require(sha(archive) == value['archive_sha256'], 'archive digest mismatch')


def smoke(output):
    for name in ('compactc', 'compactc-scheme'):
        require(os.access(Path(output)/'bin'/name, os.X_OK), 'missing compiler sibling ' + name)
    for name in ('runtime-rs', 'runtime-rs-macros'):
        require((Path(output)/'share/compactc'/name/'Cargo.toml').is_file(), 'missing packaged runtime ' + name)
    run([output+'/bin/compactc', '--version'])
    run([output+'/bin/compactc', '--help'])
    with tempfile.TemporaryDirectory(prefix='compiler-producer-smoke-') as directory:
        run([output+'/bin/compactc', '--target', 'rust', '--skip-zk',
             str(ROOT/'examples/rust_backend/effect_plan_counter_only.compact'), directory+'/contract'])
        require((Path(directory)/'contract/contract/lib.rs').is_file(), 'fixture source absent')
        require((Path(directory)/'contract/compiler/contract-manifest.json').is_file(), 'fixture manifest absent')


def service_artifact(args):
    """Authenticate producer material against current-run GitHub service metadata.

    The caller downloads this exact artifact ID from the same run, never searches
    older runs. The manifest's self-hash is corruption checking, not provenance.
    """
    require(re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+', args.repository), 'invalid repository')
    require(args.artifact_id.isdigit(), 'invalid artifact ID')
    token = os.environ.get('GH_TOKEN') or os.environ.get('GITHUB_TOKEN')
    require(token, 'artifact metadata requires a read-only GitHub token')
    request = urllib.request.Request(
        f'https://api.github.com/repos/{args.repository}/actions/artifacts/{args.artifact_id}',
        headers={'Authorization': 'Bearer ' + token, 'Accept': 'application/vnd.github+json',
                 'X-GitHub-Api-Version': '2022-11-28'})
    with urllib.request.urlopen(request, timeout=30) as response:
        raw = response.read(1024*1024 + 1)
    require(len(raw) <= 1024*1024, 'artifact service metadata too large')
    return json.loads(raw)


def unpack_artifact(args, directory):
    value = service_artifact(args)
    actual_attempt = producer_attempt(args.run_attempt, args.producer_attempt)
    expected_name = f'compiler-{args.run_id}-{actual_attempt}-{args.system}-{args.source_sha}'
    require(str(value['id']) == args.artifact_id and not value['expired']
            and str(value['workflow_run']['id']) == args.run_id and value['name'] == expected_name,
            'artifact is not the expected same-run platform producer')
    require(value.get('digest') == 'sha256:' + sha(args.artifact_zip), 'service artifact digest mismatch')
    # Only the two fixed material files are accepted; no path extraction API.
    with zipfile.ZipFile(args.artifact_zip) as bundle:
        require(sorted(bundle.namelist()) == ['closure.nar-export', 'manifest.json'], 'artifact ZIP roster mismatch')
        require(bundle.getinfo('manifest.json').file_size <= 4*1024*1024, 'manifest exceeds bound')
        require(bundle.getinfo('closure.nar-export').file_size <= 16*1024**3, 'closure exceeds transfer bound')
        directory.mkdir(parents=True, exist_ok=False)
        for name in ('manifest.json', 'closure.nar-export'):
            with bundle.open(name) as source, (directory/name).open('xb') as target:
                import shutil
                shutil.copyfileobj(source, target)


def consume(args, expected, directory, derivation, expected_output):
    unpack_artifact(args, directory)
    archive, receipt = directory/'closure.nar-export', directory/'manifest.json'
    require(receipt.is_file() and not receipt.is_symlink() and receipt.stat().st_size <= 4*1024*1024,
            'invalid manifest file')
    def unique(pairs):
        result = {}
        for key, value in pairs:
            require(key not in result, 'duplicate manifest field')
            result[key] = value
        return result
    value = json.loads(receipt.read_text(), object_pairs_hook=unique)
    validate_receipt(value, expected, archive)
    require(value['derivation'] == derivation, 'source derivation mismatch')
    require(value['output'] == expected_output, 'source output mismatch')
    require(identity(args) == expected, 'source changed before import')
    with archive.open('rb') as stream:
        imported = text_import(stream)
    require(len(imported) == len(set(imported)) and set(imported) == set(value['closure']), 'imported closure membership mismatch')
    actual = text(['nix-store', '--query', '--requisites', value['output']]).splitlines()
    require(set(actual) == set(value['closure']), 'runtime requisite roster mismatch')
    require(info(sorted(value['closure'])) == value['closure'], 'NAR identity mismatch')
    # Local evaluation independently produced the exact source derivation;
    # closure-only export does not claim to transfer its build-time dependencies.
    require(text(['nix-store', '--query', '--deriver', value['output']]) == derivation,
            'imported declared deriver mismatch')
    smoke(value['output'])
    require(identity(args) == expected, 'source changed during consumer import')
    print(value['output'])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('mode', choices=['export', 'import'])
    for name in ('source-sha', 'system', 'run-id', 'run-attempt'):
        parser.add_argument('--'+name, required=True)
    parser.add_argument('--directory', type=Path, required=True)
    parser.add_argument('--producer-attempt')
    parser.add_argument('--repository')
    parser.add_argument('--artifact-id')
    parser.add_argument('--artifact-zip', type=Path)
    args = parser.parse_args()
    if args.mode == 'import':
        require(all((args.repository, args.artifact_id, args.artifact_zip, args.producer_attempt)), 'import requires same-run service artifact identity and ZIP')
    expected = identity(args)
    directory = args.directory.resolve()
    archive, receipt = directory/'closure.nar-export', directory/'manifest.json'
    derivation = text(['nix', 'eval', '--raw', '.#compactc.drvPath'])
    expected_output = text(['nix', 'eval', '--raw', '.#compactc.outPath'])
    if args.mode == 'export':
        directory.mkdir(parents=True, exist_ok=False)
        output = text(['nix', 'build', '.#compactc', '--no-link', '--print-out-paths'])
        require(STORE.fullmatch(output) and output == expected_output, 'unexpected build output')
        smoke(output)
        paths = sorted(text(['nix-store', '--query', '--requisites', output]).splitlines())
        closure = info(paths)
        with archive.open('xb') as stream:
            run(['nix-store', '--export', *paths], stdout=stream)
        value = dict(schema=1, identity=expected, derivation=derivation, output=output,
                     closure=closure, archive_sha256=sha(archive))
        receipt.write_text(json.dumps(value, sort_keys=True, indent=2)+'\n')
        require(identity(args) == expected, 'source changed during producer')
    else:
        consume(args, expected, directory, derivation, expected_output)


def text_import(stream):
    return run(['nix-store', '--import'], stdin=stream, capture_output=True, text=True).stdout.splitlines()


if __name__ == '__main__':
    main()
