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
"""Same-run original Forge Nix material; never a consumer qualification verdict."""
import argparse
import json
from pathlib import Path
import zipfile
import time
import compiler_closure as closure

SOURCE = 'd64f52db7239b0ca4891fb0316dca6adb54a5db5'
PRODUCER = '7118b3e80ef4ac8abe7285fd948c59e706166a68'
REF = 'github:betsudo/forge/' + SOURCE
FILES = {'closure.nar', 'store-paths.json', 'forge-derivation.json', 'flake.json', 'rustc-verbose.txt'}
LIMIT = 2147483648
require = closure.require


def read(path):
    require(path.is_file() and not path.is_symlink() and path.stat().st_size <= 16*1024*1024, 'invalid material JSON')
    def unique(items):
        result = {}
        for k, v in items:
            require(k not in result, 'duplicate material field')
            result[k] = v
        return result
    return json.loads(path.read_text(), object_pairs_hook=unique)


def authenticate(value, args):
    attempt = closure.producer_attempt(args.run_attempt, args.producer_attempt)
    require(str(value['id']) == args.artifact_id and value['expired'] is False
            and str(value['workflow_run']['id']) == args.run_id
            and value['name'] == f'forge-nix-linux-{args.run_id}-{attempt}', 'not expected same-run Forge material')
    require(value['digest'] == 'sha256:' + closure.sha(args.artifact_zip), 'Forge service archive digest mismatch')


def unpack(args):
    authenticate(closure.service_artifact(args), args)
    with zipfile.ZipFile(args.artifact_zip) as bundle:
        require(set(bundle.namelist()) == FILES | {'material.json'} and len(bundle.namelist()) == 6, 'Forge ZIP roster mismatch')
        require(all(bundle.getinfo(n).file_size <= (LIMIT if n == 'closure.nar' else 16*1024*1024) for n in bundle.namelist()), 'Forge ZIP member exceeds bound')
        args.directory.mkdir(parents=True, exist_ok=False)
        for name in sorted(bundle.namelist()):
            with bundle.open(name) as src, (args.directory/name).open('xb') as dst:
                import shutil
                shutil.copyfileobj(src, dst)


def validate(manifest, args):
    require(manifest['schema'] == 1 and manifest['payload_source'] == SOURCE and manifest['producer_source'] == PRODUCER
            and manifest['system'] == 'x86_64-linux' and manifest['producer_repository'] == 'betsudo/forge'
            and manifest['repository'] == args.repository and manifest['run_id'] == args.run_id
            and manifest['run_attempt'] == args.producer_attempt and manifest['max_bytes'] == LIMIT,
            'Forge material provenance mismatch')
    require(set(manifest['files']) == FILES, 'Forge file inventory mismatch')
    for name, row in manifest['files'].items():
        path = args.directory/name
        require(type(row['bytes']) is int and path.stat().st_size == row['bytes']
                and closure.sha(path) == row['sha256'], 'Forge material file changed: '+name)
    inventory = read(args.directory/'store-paths.json')
    require(isinstance(inventory, dict) and 0 < len(inventory) == manifest['store_paths'] <= 10000, 'Forge NAR roster mismatch')
    for path, row in inventory.items():
        require(closure.STORE.fullmatch(path) and closure.canonical_sri(row['narHash'])
                and type(row['narSize']) is int and row['narSize'] >= 0, 'invalid Forge NAR identity')
    require(sum(row['narSize'] for row in inventory.values()) == manifest['nar_bytes'] <= LIMIT, 'Forge NAR size mismatch')
    require((args.directory/'closure.nar').stat().st_size <= LIMIT, 'Forge export exceeds bound')
    require(all(manifest[k] in inventory for k in ('root','rust_root','source_root','build_source_root')), 'Forge root absent from closure')
    return inventory


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('repository','run-id','run-attempt','producer-attempt','artifact-id'):
        parser.add_argument('--'+name, required=True)
    parser.add_argument('--artifact-zip', required=True, type=Path)
    parser.add_argument('--directory', required=True, type=Path)
    parser.add_argument('--github-env', required=True, type=Path)
    args = parser.parse_args()
    started = time.monotonic()
    require(closure.text(['nix','eval','--impure','--raw','--expr','builtins.currentSystem']) == 'x86_64-linux', 'native Linux consumer required')
    unpack(args)
    manifest = read(args.directory/'material.json')
    inventory = validate(manifest, args)
    # Evaluate exact public source roots independently; no build/substitute fallback.
    metadata = json.loads(closure.text(['nix','flake','metadata',REF,'--json']))
    require(metadata['locked']['rev'] == SOURCE and metadata['path'] == manifest['source_root'], 'independent Forge source differs')
    root = closure.text(['nix','eval','--raw',REF+'#packages.x86_64-linux.default.outPath'])
    drv = closure.text(['nix','eval','--raw',REF+'#packages.x86_64-linux.default.drvPath'])
    expression = ('let f = builtins.getFlake "'+REF+'"; pkgs = import f.inputs.nixpkgs { system = "x86_64-linux"; overlays = [(import f.inputs.rust-overlay)]; }; in (pkgs.rust-bin.stable."1.98.1".minimal.override { extensions = ["rustfmt" "clippy"]; }).outPath')
    rust = closure.text(['nix','eval','--impure','--raw','--expr',expression])
    independent = json.loads(closure.text(['nix','derivation','show',drv]))
    def row(value):
        rows = value.get('derivations',value)
        found = [r for r in rows.values() if r.get('env',{}).get('out') == root]
        require(len(found) == 1, 'exact Forge derivation required')
        return found[0]
    actual = row(independent)
    require(root == manifest['root'] and rust == manifest['rust_root'] and actual['env']['src'] == manifest['build_source_root']
            and actual == row(read(args.directory/'forge-derivation.json'))
            and actual['system'] == 'x86_64-linux' and rust in actual['env']['nativeBuildInputs'].split(), 'independent Forge derivation differs')
    with (args.directory/'closure.nar').open('rb') as stream:
        imported = closure.text_import(stream)
    require(len(imported) == len(set(imported)) and set(imported) == set(inventory), 'Forge imported roster mismatch')
    roots = [manifest[k] for k in ('root','rust_root','source_root','build_source_root')]
    require(set(closure.text(['nix-store','--query','--requisites',*roots]).splitlines()) == set(inventory), 'Forge runtime/source/tool closure incomplete')
    require(closure.info(sorted(inventory)) == {p:r['narHash'] for p,r in inventory.items()}, 'Forge imported NAR changed')
    required = {'zkir-forge','midnight-zkir-forge','midnight-zkir-forge-ci','midnight-zkir-forge-zkir3-worker','midnight-zkir-forge-zkir31-worker'}
    require(required <= {Path(p).name for p in manifest['binaries']}, 'Forge executable roster missing')
    for p, digest in manifest['binaries'].items():
        require(Path(p).parent == Path(root)/'bin' and closure.sha(Path(p).resolve(strict=True)) == digest, 'Forge executable mismatch')
    import sys
    sys.path.insert(0,str(closure.ROOT/'tools/compact-rust-backend'))
    import qualify_linux_forge as finite
    source_manifest = read(closure.ROOT/'tools/compact-rust-backend/forge_linux_source.json')
    finite.check_derivation(independent,Path(root)/'bin/zkir-forge',Path(rust),source_manifest)
    verbose = closure.text([rust+'/bin/rustc','-vV'])+'\n'
    require(verbose == (args.directory/'rustc-verbose.txt').read_text(), 'Forge Rust executable differs')
    finite.fingerprint(Path(metadata['path']),source_manifest,verbose)
    validate(manifest,args)
    with args.github_env.open('a') as out:
        for k,v in {'FORGE':root+'/bin/zkir-forge','FORGE_HELPER':root+'/bin/midnight-zkir-forge-ci','FORGE_SOURCE':metadata['path'],'FORGE_RUSTC':rust+'/bin/rustc','FORGE_MATERIAL_DERIVATION':str(args.directory/'forge-derivation.json')}.items():
            require('\n' not in v and '\r' not in v,'invalid environment path')
            out.write(k+'='+v+'\n')
    print(json.dumps({'scope':'same-run original Nix material only; finite consumer qualification still required','root':root,'paths':len(inventory),'archive_sha256':closure.sha(args.artifact_zip),'repository':args.repository,'run_id':args.run_id,'producer_attempt':args.producer_attempt,'artifact_id':args.artifact_id,'payload_source':SOURCE,'producer_source':PRODUCER,'material_files':manifest['files'],'complete_import_wall_seconds':time.monotonic()-started}))


if __name__ == '__main__':
    main()
