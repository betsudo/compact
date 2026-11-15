#!/usr/bin/env python3
# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Qualify selected maintained DID primitive reducers using exact local artifacts."""
import argparse
import json
import os
from pathlib import Path
import shutil
import sys
import did_proof_gate as did
import did_reducer_proof as reducer
import local_parity_gate as common
import resolve_ledger_test_static as ledger_static
import reviewed_rust_capture as reviewed
import reducer_capture_source as capture_inputs

ROOT = common.ROOT
REGISTRY = ROOT / 'tools/compact-rust-backend/did_primitive_reducers.json'
FROZEN_LEDGER9_SOURCE = ROOT / 'tools/compact-rust-backend/reviewed-ledger9-reducer-source-05912594'
# The retained 0.17.0 captures were made at the published Rust AST base. Later
# Ledger 9 work changes these live runtime files; their reviewed capture must
# remain bound to the source bytes that actually produced it.
FROZEN_LEDGER9_RUNTIME = {
    'runtime/src/built-ins.ts':
        (FROZEN_LEDGER9_SOURCE / 'built-ins.ts',
         '89f0f6a65c7e62f85bbd88044e872d9d3555f6ebeb45dba460e1601c025f2a4b'),
    'runtime/src/compact-types.ts':
        (FROZEN_LEDGER9_SOURCE / 'compact-types.ts',
         '07328c4c793ef7c81171d9835c8535543959bfef1c1f67c730ed8fe65e7576f4'),
}
GEOMETRY = {
    "point-digest": {"point_write_only": (7, 77), "pure_projection_only": (7, 95),
                     "scalar_control": (6, 49), "update": (7, 97)},
    "point-guard": {"check": (8, 128)},
    "alias-set": {"insert_control": (6, 38), "member_control": (7, 62),
                  "mutate": (8, 134), "remove_control": (6, 36)},
    "alias-digest": {"run": (7, 90)},
    "alias-guard": {"local_control": (6, 51), "run": (6, 51), "transitive_control": (6, 51)},
    "service-map": {"remove_service": (7, 60), "set_service": (8, 165)},
    "point-map": {"remove": (7, 60), "upsert": (8, 173)},
    "nested-map": {"remove": (7, 60), "upsert": (8, 154)},
    "enum-map": {"drop": (7, 60), "put": (8, 152)},
}


def require(condition, message):
    if not condition:
        raise common.GateError(message)


def read_json(path):
    value = json.loads(path.read_text())
    require(isinstance(value, dict), f'expected object: {path}')
    return value


def hashes(paths):
    return {str(p): common.sha256(p) for p in sorted(set(paths)) if p.is_file()}


def reviewed_rows():
    value = read_json(REGISTRY)
    require(value.get('format') == 'compact-did-primitive-reducers/v1', 'unknown registry')
    rows = value['reducers']
    require(len(rows) == len({r['kind'] for r in rows}), 'duplicate reviewed reducer')
    for row in rows:
        require(row['proof_cases'] and len(row['proof_cases']) == len({c['case'] for c in row['proof_cases']}),
                'empty or duplicate proof cases')
        operations = {c['operation'] for c in row['operations']}
        require(operations and len(operations) == len(row['operations']), 'empty or duplicate operations')
        require(all(c['operation'] in operations for c in row['proof_cases']), 'unreviewed proof operation')
        for name in ('source', 'fixture'):
            path = Path(row[name])
            require(not path.is_absolute() and '..' not in path.parts, 'invalid registry path')
    return rows


def validate_summary(value, row, *, require_v3=False):
    version = 'v2' if require_v3 else 'v1'
    require(value.get('format') == f'compact-did-primitive-reducer-proof/{version}'
            and value.get('kind') == row['kind'], 'wrong reducer identity')
    require(value.get('status') == 'passed' and value.get('strictness') == 'default',
            'reducer failed default strictness')
    require(value.get('constructor_data_deployed') is True
            and value.get('constructor_execution_proved') is False, 'constructor boundary changed')
    calls = value.get('calls', [])
    require([{'case': c.get('case'), 'operation': c.get('operation')} for c in calls]
            == row['proof_cases'], 'missing, extra, duplicate or reordered proof case/operation')
    for call in calls:
        for flag in ('changed_binding_rejected', 'applied', 'recorded_state_matches', 'replay_unchanged'):
            require(call.get(flag) is True, f'missing {flag}')
        require(call.get('replay_refusal') == 'IntentAlreadyExists', 'wrong replay refusal')
        for count in ('proof_bytes', 'state_bytes'):
            require(type(call.get(count)) is int and call[count] > 0, f'missing {count}')
        if require_v3:
            reducer.validate_call(call)


def validate_current_capture(path, fixture):
    oracle = ROOT / fixture / "oracle"
    binding_path = oracle / "ledger9-binding.json"
    binding = reviewed.read_json(binding_path)
    retained = oracle / "ledger9-cases.json"
    historical = oracle / "cases.json"
    old_binding = oracle / "reviewed-rust-binding.json"
    require(binding.get("format") == "compact-ledger9-reducer-capture-binding/v1"
            and binding.get("compact_runtime") == "0.17.0"
            and binding.get("onchain_runtime") == "4.0.1", "Ledger 9 reducer capture binding differs")
    require(common.sha256(old_binding) == binding["ledger8_binding_sha256"]
            == reviewed.REVIEWED_BINDINGS[str(fixture)], "historical reducer binding changed")
    old_captures = reviewed.read_json(old_binding)["captures"]
    old_sha = [row["sha256"] for row in old_captures if row["file"] == historical.name]
    require(old_sha == [binding["ledger8_capture_sha256"]]
            and common.sha256(historical) == old_sha[0], "historical reducer capture changed")
    require(common.sha256(path) == common.sha256(retained) == binding["ledger9_capture_sha256"],
            "fresh reducer capture differs from exact reviewed Ledger 9 bytes")
    capture = reviewed.read_json(path)
    require(capture["runtimeVersion"] == binding["compact_runtime"]
            and capture["runtimeJavaScriptSha256"] == binding["runtime_builtins_sha256"]
            and capture["generatedJavaScriptSha256"] == binding["generated_javascript_sha256"],
            "Ledger 9 reducer runtime or generated JavaScript identity changed")
    for row in capture["provenance"]:
        relative = reviewed.relative(row["path"])
        frozen = FROZEN_LEDGER9_RUNTIME.get(str(relative))
        if frozen is not None:
            source, expected_sha = frozen
            require(row["sha256"] == expected_sha,
                    f"reviewed reducer runtime identity changed: {relative}")
        else:
            source = ROOT / relative
        require(common.sha256(source) == row["sha256"], f"reducer capture source changed: {source}")
    return hashes([binding_path, retained, historical, old_binding,
                   *(source for source, _ in FROZEN_LEDGER9_RUNTIME.values())])


def source_inventory(rows):
    files = [REGISTRY, Path(__file__), Path(common.__file__), Path(ledger_static.__file__),
             Path(did.__file__), Path(reducer.__file__),
             Path(reviewed.__file__), Path(__file__).with_name('test_reviewed_rust_capture.py'),
             Path(capture_inputs.__file__), Path(__file__).with_name('test_reducer_capture_source.py'),
             ROOT/reviewed.SUPPORT,
             ROOT/'Cargo.toml', ROOT/'Cargo.lock', ROOT/'tools/compact-rust-proof-smoke/Cargo.toml',
             ROOT/'tools/compact-rust-backend/did_primitive_reducer_capture.mjs',
             ROOT/'tools/compact-rust-backend/did_primitive_reducer_cases.mjs',
             ROOT/'tools/compact-rust-backend/test_did_primitive_reducer_gate.py',
             ROOT/'tools/compact-rust-backend/check_fixture_outputs.py']
    files.extend(source for source, _ in FROZEN_LEDGER9_RUNTIME.values())
    for row in rows:
        files.append(ROOT/row['source'])
        files.append(ROOT/row['fixture']/'oracle/reviewed-rust-binding.json')
        files.extend((ROOT/row['fixture']).rglob('*'))
    for folder in ('runtime-rs', 'runtime-rs-macros', 'testkit-rs', 'tools/compact-rust-proof-smoke/src'):
        files.extend(p for p in (ROOT/folder).rglob('*') if p.suffix in ('.rs','.toml','.json'))
    return hashes(files)


def run_gate(directory, compiler, scheme, target, selected, environment=None):
    rows = reviewed_rows()
    if selected:
        require(set(selected) <= {r['kind'] for r in rows}, 'unknown requested reducer')
        require(len(selected) == len(set(selected)), 'duplicate requested reducer')
        rows = [r for r in rows if r['kind'] in selected]
    require(rows, 'no reviewed reducers selected')
    directory = directory.resolve()
    directory.mkdir(parents=True, exist_ok=False, mode=0o700)
    receipt = {'format':'compact-did-primitive-reducer-gate/v1', 'status':'failed',
               'commands':[], 'expected_reducers':[r['kind'] for r in rows], 'reducers':[]}
    env = dict(os.environ if environment is None else environment)
    try:
        for key in ('MIDNIGHT_PP','MIDNIGHT_LEDGER_TEST_STATIC_DIR'):
            require(bool(env.get(key)) and Path(env[key]).is_absolute() and Path(env[key]).is_dir(),
                    f'{key} must select an absolute existing directory')
        require("MIDNIGHT_PROOF_SERVER" not in env, "DID reducer gate requires local proving")
        zkir = shutil.which('zkir-v3', path=env.get('PATH'))
        require(zkir is not None, 'zkir-v3 missing')
        dust_files=[]
        for name,digest in ledger_static.FIXTURE_HASHES.items():
            path=Path(env['MIDNIGHT_LEDGER_TEST_STATIC_DIR'])/'dust'/name
            require(path.is_file() and common.sha256(path)==digest, f'Dust fixture mismatch: {name}')
            declaration=path.with_name(path.name+'.sha256')
            require(declaration.read_text().split()[0]==digest, f'Dust declaration mismatch: {name}')
            dust_files += [path,declaration]
        receipt['dust_fixture_hashes']=hashes(dust_files)
        receipt['source_hashes']=source_inventory(rows)
        receipt['git_head']=common.git_head()
        (directory/'bin').mkdir()
        components=receipt['components']={}
        for name,path in (('compactc',compiler),('compactc-scheme',scheme),('zkir',Path(zkir))):
            components[name]=common.stable_copy(path,directory/'bin'/name)
        env.update({'COMPACTC_SCHEME':components['compactc-scheme']['snapshot'],
                    'COMPACT_RUST_RUNTIME_DIR':str(ROOT),'CARGO_TARGET_DIR':str(target.resolve()),
                    'CARGO_INCREMENTAL':'0','RUSTUP_TOOLCHAIN':'1.99.0','PYTHONDONTWRITEBYTECODE':'1'})
        zkir = components["zkir"]["snapshot"]
        receipt["key_generation"] = {"mode": "direct-native-v3", "actual_zkir": zkir,
                                     "scope": "Fresh native keys and proofs."}
        def execute(argv,label,stdout_path=None):
            common.run([str(a) for a in argv],label,directory,receipt,env=env,stdout_path=stdout_path)
        packages=[arg for row in rows for arg in ('-p',row['package'])]
        execute(['cargo','+1.99.0','test','--locked','--offline',*packages], 'test-reducer-behavior')
        execute(['cargo','+1.99.0','build','--profile','proof-checks','--locked','--offline',
                 '-p','compact-rust-proof-smoke'],'build-proof-runner')
        components['proof-runner']=common.stable_copy(
            target.resolve()/'proof-checks/compact-rust-proof-smoke',directory/'bin/compact-rust-proof-smoke')
        runner=components['proof-runner']['snapshot']
        reducer.require_zkir_version(execute, zkir, directory / "zkir-version.txt")
        selected_k = {GEOMETRY[row["kind"]][operation["operation"]][0]
                      for row in rows for operation in row["operations"]}
        receipt["parameter_hashes"] = reducer.prepare_parameters(
            execute, receipt, runner, directory / "proof-material.json", Path(env["MIDNIGHT_PP"]), selected_k)
        receipt['source_inputs'] = []
        for row in rows:
            kind=row['kind']; output=directory/kind
            source = ROOT / row["source"]
            expected_path, expected_sha = capture_inputs.SOURCES[kind]
            require(row["source"] == expected_path and common.sha256(source) == expected_sha
                    == receipt["source_hashes"][str(source)], f"{kind}: capture source closure changed")
            receipt["source_inputs"].append({"kind": kind, "path": str(source), "sha256": expected_sha})
            execute([components['compactc']['snapshot'],'--target','ts','--target','rust',
                     '--feature-zkir-v3','--skip-zk','--rust-runtime-root',ROOT,source,output], f'{kind}-compile')
            execute(['rustfmt','--edition','2024',output/'contract/lib.rs'],f'{kind}-format')
            require((output/'contract/lib.rs').read_bytes()==(ROOT/row['fixture']/'lib.rs').read_bytes(),
                    f'{kind}: generated fixture stale')
            report=read_json(output/'contract/rust-capabilities.json')
            common.validate_report(report,ROOT/row['source'])
            cap=common.proof_cross_tab(report,read_json(output/'compiler/contract-info.json'),ROOT/row['source'])
            for operation in row['operations']:
                selected_cap=[c for c in cap if c['name']==operation['operation']]
                require(len(selected_cap)==1 and all(selected_cap[0].get(k) is True for k in ('proof','recorded','observed_call')),
                        f'{kind}: selected operation is not proof/recording applicable')
            capture=directory/f'{kind}-capture.json'
            execute(['node',ROOT/'tools/compact-rust-backend/did_primitive_reducer_capture.mjs',kind,
                     output/'contract/index.js'],f'{kind}-capture',capture)
            binding_hashes = validate_current_capture(capture, row['fixture'])
            generated = reducer.generated_hashes(output)
            generated.update(hashes([output / "contract/index.js"]))
            model = reducer.model(execute, receipt, zkir, output, GEOMETRY[kind])
            artifacts=[]
            for selection in row['operations']:
                operation=selection['operation']
                reducer.keygen(execute, receipt, zkir, output, operation, model[operation],
                               f'{kind}-{operation}-keygen')
                artifacts.extend(output/f'{folder}/{operation}.{ext}' for folder,ext in [('keys','prover'),('keys','verifier'),('zkir','zkir'),('zkir','bzkir')])
            before=hashes(reducer.checked_file(path) for path in artifacts)
            execute([runner,'--did-primitive-reducer',kind,output],f'{kind}-prove')
            summary=read_json(output/'proof-result.json')
            validate_summary(summary, row, require_v3=True)
            results=[output/'proof-result.json', reducer.checked_file(output/'reducer-deployment.bin'),
                     reducer.checked_file(output/'reducer-ledger-after-deploy.bin')]
            for call in summary['calls']:
                results.extend(reducer.proof_artifacts(output, call["case"], call, call["state_bytes"]))
            require(hashes(artifacts)==before,f'{kind}: proof material changed')
            receipt['reducers'].append({'kind':kind,'summary':summary,'capabilities':report,
                                        'artifact_hashes':before,'result_hashes':hashes(results+[capture]),
                                        'generated_hashes':generated,'binding_hashes':binding_hashes,
                                        'geometry':model})
        require([r['kind'] for r in receipt['reducers']]==receipt['expected_reducers'],'incomplete reducer results')
        for result in receipt['reducers']:
            for field in ('artifact_hashes','result_hashes','generated_hashes','binding_hashes'):
                did.unchanged(result[field])
        did.unchanged(receipt["parameter_hashes"])
        did.unchanged(receipt["dust_fixture_hashes"])
        require(source_inventory(rows)==receipt['source_hashes'] and common.git_head()==receipt['git_head'],
                'source or HEAD changed during gate')
        require(all(common.sha256(Path(r['snapshot']))==r['sha256'] for r in components.values()),'tool changed')
        receipt['status']='passed'
    except (common.GateError,OSError,ValueError,KeyError,IndexError,TypeError) as error:
        receipt['error']=str(error)
    finally:
        (directory/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
    return receipt


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for flag in ('compiler','scheme','target','output'):
        parser.add_argument('--'+flag,type=Path,required=True)
    parser.add_argument('--only',action='append',default=[])
    args=parser.parse_args()
    receipt=run_gate(args.output,args.compiler,args.scheme,args.target,args.only)
    print(json.dumps({'status':receipt['status'],'error':receipt.get('error')}))
    return 0 if receipt['status']=='passed' else 1


if __name__=='__main__':
    sys.exit(main())
