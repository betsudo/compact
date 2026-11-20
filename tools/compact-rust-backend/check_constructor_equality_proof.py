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

"""Whole constructor equality sources, intact models and selected fresh proof/application qualification.

Combines the maintained source/API, model and authenticated native preflight
owners. Later-entrypoint proofs do not prove constructor initialization.
"""
from pathlib import Path
import argparse, contextlib, datetime, hashlib, json, os, re, shutil, subprocess, sys, time, traceback
if not __debug__ or sys.flags.optimize or os.environ.get('PYTHONOPTIMIZE'):
    raise RuntimeError('optimized Python invocation is not admitted')
import check_constructor_equality as source_owner
import check_constructor_equality_models as model_owner
import check_constructor_equality_scope as scope_owner
import check_constructor_equality_proof_preflight as preflight_owner
import check_functional_source_sdk as sdk
import check_passport_current_profile as process_owner
import constructor_proof_dependencies as dependencies
import check_constructor_vector_proof as vector_proof
from check_constructor_vector_models import GEOMETRY

W = sdk.ROOT
sha = sdk.digest
git = lambda *a: subprocess.check_output(['git',*a],cwd=W,text=True).strip()

def prove_selected(E, environment, *, boundaries=False, tuple_guards=False):
    assert not (boundaries and tuple_guards), 'one finite cohort required'
    extended = boundaries or tuple_guards
    source_count = 16 if tuple_guards else 8
    case_count = 105 if tuple_guards else 30 if boundaries else 16
    fixture = W / "tools/compact-rust-backend/tests" / ("constructor_equality_tuple_guards" if tuple_guards else "constructor_equality_boundaries" if boundaries else "constructor_equality")
    P = E/'preflight'
    D = E/'proofs'
    preflight_path=P/'receipt.json'
    model_path=E/'models/receipt.json'
    preflight=json.loads(preflight_path.read_text())
    models=json.loads(model_path.read_text())
    assert preflight['status']==models['status']=='passed'
    assert len(preflight['sources'])==source_count and preflight['selected_cases']==case_count
    assert models['observed_models']==source_count*3
    assert models['observed_k'] == [6, 7], 'exact constructor parameter roster required'
    rows = models['inventory']['circuits']
    assert models['inventory']['format'] == model_owner.inventory_format(
        boundaries=boundaries, tuple_guards=tuple_guards)
    assert len(rows) == source_count*3 and {(r['source'], r['circuit']) for r in rows} == {
        (role, entry) for role in preflight['sources'] for entry in GEOMETRY}
    assert all((r['k'], r['rows']) == GEOMETRY[r['circuit']] for r in rows)
    assert preflight['producer_head']==preflight['qualifier_head']==models['producer_source']==git('rev-parse','HEAD')
    D.mkdir()
    env = dict(environment)
    env.pop('LD_LIBRARY_PATH', None)
    pins = dict(preflight['frozen_inputs'])
    producer_path = E / 'protocol/receipt.json'
    producer = json.loads(producer_path.read_text())
    assert producer['status'] == 'passed' and producer['head'] == preflight['qualifier_head']
    pins.update(models['inputs'])
    pins.update(producer['inputs'])
    pins[str(producer_path)] = sha(producer_path)
    pins.update({str(p):sha(p) for p in [preflight_path,model_path,Path(__file__),Path(vector_proof.__file__)]})
    config = sdk.cargo_configuration(env)
    assert config == preflight['cargo_configuration']
    R = dict(format='constructor-tuple-equality-fresh-proofs/v2' if tuple_guards else 'constructor-equality-boundaries-fresh-proofs/v2' if boundaries else 'constructor-equality-fresh-proofs/v2', status='running', started_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(), producer_head=preflight['producer_head'], qualifier_head=preflight['qualifier_head'], preflight=str(preflight_path), commands=[], sources={}, frozen_inputs=pins, proof_cache=False, automatic_k_cap=8, scope='selected later-entrypoint proofs and separate funded DEFAULT strict applications, not constructor/deployment/wallet proof or integration acceptance')
    save = lambda: (D/'receipt.json').write_text(json.dumps(R,indent=2)+'\n')
    save()
    def unchanged():
        assert git('rev-parse','HEAD') == R['qualifier_head'] and not git('status','--porcelain')
        assert sdk.cargo_configuration(env) == config
        for name,digest in pins.items(): assert sha(name)==digest,name
        for name,inventory in preflight['frozen_directories'].items(): sdk.equal_inventory(Path(name),inventory)
        dependencies.unchanged(preflight['dependency_sources'])
    def run(label,cmd):
        unchanged();start=time.monotonic()
        try: p=(process_owner if extended else sdk).bounded_command(cmd,env)
        except subprocess.TimeoutExpired as error:
            for name,data in [('stdout',error.stdout),('stderr',error.stderr)]:
                (D/(label+'.'+name)).write_text(data.decode(errors='replace') if isinstance(data,bytes) else data or '')
            row=dict(label=label,argv=list(map(str,cmd)),timed_out=True)
            if extended:row['timeout_seconds']=process_owner.COMMAND_TIMEOUT_SECONDS
            R['commands'].append(row);save();raise
        row=dict(label=label,argv=list(map(str,cmd)),exit=p.returncode,seconds=time.monotonic()-start)
        if extended:row['timeout_seconds']=process_owner.COMMAND_TIMEOUT_SECONDS
        for name in ['stdout','stderr']:
            path=D/(label+'.'+name);path.write_text(getattr(p,name));row[name]=str(path);row[name+'_sha256']=sha(path)
        R['commands'].append(row);save();assert p.returncode==0,(label,p.stderr[-2000:])
        return p
    try:
        material_runner=json.loads(model_path.read_text())['binary']['path']
        material = json.loads(run('verify-material', [
            material_runner, '--prepare-proof-material', '--verify-only', '--parameter-k', '6', '--parameter-k', '7',
        ]).stdout)
        assert material['mode'] == 'verify-only'
        assert material['verification'] == 'all files checked against locked upstream SHA256 tables'
        assert Path(material['cache_directory']).resolve() == Path(env['MIDNIGHT_PP']).resolve()
        assert {6, 7}.issubset(x['k'] for x in material['parameters'])
        R['verified_material'] = material
        cases_path=fixture/'proof-cases.json'
        cases=json.loads(cases_path.read_text())
        for role,row in preflight['sources'].items():
            lane=D/role;lane.mkdir();material_root=lane/'material';material_root.mkdir()
            shutil.copytree(P/role/'generated/zkir',material_root/'zkir')
            (material_root/'keys').mkdir()
            assert set(x.name for x in (material_root/'zkir').iterdir())=={'read.zkir','advance.zkir','advance_other.zkir'}
            for f in (material_root/'zkir').iterdir():
                pins[str(f)] = sha(f)
                assert pins[str(f)] == pins[str(P/role/'generated/zkir'/f.name)]
            keys={}
            for entry in ['advance','advance_other']:
                observed=[m for m in models['inventory']['circuits'] if m['source']==role and m['circuit']==entry]
                assert len(observed)==1
                expected_geometry = (observed[0]['k'], observed[0]['rows'])
                result=run(role+'-keygen-'+entry,[preflight['prerequisites']['zkir'],'compile',material_root/'zkir'/(entry+'.zkir'),material_root/'keys'/(entry+'.prover'),material_root/'keys'/(entry+'.verifier')])
                match=re.search(r'\(k=(\d+), rows=(\d+)\)',result.stdout+result.stderr)
                assert match and tuple(map(int, match.groups())) == expected_geometry, (role,entry,'automatic geometry')
                keys[entry]=dict(k=int(match[1]),rows=int(match[2]))
                for f in [material_root/'zkir'/(entry+'.bzkir'),material_root/'keys'/(entry+'.prover'),material_root/'keys'/(entry+'.verifier')]:
                    assert f.is_file();pins[str(f)]=sha(f)
            assert pins[str(material_root/'keys/advance.verifier')]!=pins[str(material_root/'keys/advance_other.verifier')]
            binary=row['binary']['snapshot'];assert sha(binary)==row['binary']['sha256'] and not Path(binary).stat().st_mode&0o222
            result=run(role+'-proof-application',[binary,'--prove',P/role/'generated',role,cases_path,material_root,lane/'applications'])
            value=json.loads(result.stdout)
            expected = [x['id'] for x in cases if x['role']==role and x['error'] is None]
            assert value['status']=='passed' and [x['case'] for x in value['results']]==expected
            for result in value['results']:
                path = lane / 'applications' / result['case']
                vector_proof.verify_case(result, path, pins, run, role+'-'+result['case']+'-verify', binary, material_root)
            R['sources'][role]=dict(keys=keys,result=value,artifacts=sdk.inventory(lane))
            save()
        unchanged()
        assert len(R['sources'])==source_count and sum(len(x['result']['results']) for x in R['sources'].values())==case_count
        R.update(status='passed',fresh_key_pairs=source_count*2,fresh_standalone_proofs=case_count,separate_standalone_verifications=case_count,separately_proved_applications=case_count,source_bytes_unchanged=True)
    except BaseException as error:
        R.update(status='failed',error=repr(error));raise
    finally:
        save();print(json.dumps({k:v for k,v in R.items() if k not in ['frozen_inputs','sources','commands','verified_material','verified_material_by_k']},indent=2),flush=True)
    return R

def main(*, boundaries=False, tuple_guards=False):
    assert not (boundaries and tuple_guards), 'one finite cohort required'
    extended = boundaries or tuple_guards
    source_count = 16 if tuple_guards else 8
    case_count = 105 if tuple_guards else 30 if boundaries else 16
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler-receipt',type=Path,required=True)
    parser.add_argument('--ts-runtime',type=Path,required=True)
    parser.add_argument('--zkir',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    if extended:
        parser.add_argument('--preflight-only',action='store_true',help='Run source/scope/models/native preflight only; never create keys or proofs')
    args=parser.parse_args()
    output=args.output.resolve()
    if output.exists() or args.output.is_symlink() or output.is_relative_to(W):
        raise ValueError('fresh output outside source required')
    env=dict(os.environ,RUSTUP_TOOLCHAIN='1.99.0',CARGO_INCREMENTAL='0',PYTHONDONTWRITEBYTECODE='1',CARGO_CACHE_AUTO_CLEAN_FREQUENCY='never')
    if 'MIDNIGHT_PROOF_SERVER' in env:
        raise ValueError('constructor equality requires local proving without MIDNIGHT_PROOF_SERVER')
    output.mkdir(parents=True,mode=0o700)
    env.setdefault('CARGO_BUILD_JOBS','2')
    env.setdefault('CARGO_TARGET_DIR',str(output/'cargo-target'))
    if Path(env['CARGO_TARGET_DIR']).resolve().is_relative_to(W):
        raise ValueError('shared Cargo target must stay outside source')
    zkir=args.zkir.resolve(strict=True)
    env.pop('LD_LIBRARY_PATH',None)
    head=git('rev-parse','HEAD');tree=git('rev-parse','HEAD^{tree}')
    if git('status','--porcelain'):raise ValueError('clean committed source required')
    recipe_path=args.compiler_receipt.resolve(strict=True)
    recipe=json.loads(recipe_path.read_text())
    if recipe['status']!='passed' or recipe['head']!=head or recipe['tree']!=tree:
        raise ValueError('current clean original compiler receipt required')
    pins={str(recipe_path):sha(recipe_path),str(zkir):sha(zkir),str(Path(__file__)):sha(__file__)}
    configuration=sdk.cargo_configuration(env)
    report=dict(format='constructor-tuple-equality-complete-proof-gate/v2' if tuple_guards else 'constructor-equality-boundaries-complete-proof-gate/v2' if boundaries else 'constructor-equality-complete-proof-gate/v2',status='running',head=head,tree=tree,stages=[],proof_execution=False,
                scope='Whole current constructor source/API/refusal/model and selected lateradvance proofs/separatefundedDEFAULTstrictapps; notconstructor/deployment/wallet/totalcost/fullCI/integration')
    save=lambda:(output/'receipt.json').write_text(json.dumps(report,indent=2)+'\n')
    def unchanged():
        assert git('rev-parse','HEAD')==head and not git('status','--porcelain')
        assert sdk.cargo_configuration(env)==configuration
        for path,digest in pins.items():assert sha(path)==digest,path
    def invoke(label,owner,argv, *, boundary_mode=False, tuple_mode=False):
        unchanged();start=time.monotonic();stdout=output/(label+'.stdout');stderr=output/(label+'.stderr')
        prior_env=dict(os.environ);prior_argv=sys.argv
        row=dict(label=label,kind='in-process-maintained-owner',owner=owner.__file__,owner_sha256=sha(owner.__file__),argv=list(map(str,argv)))
        try:
            with stdout.open('x') as out,stderr.open('x') as err,contextlib.redirect_stdout(out),contextlib.redirect_stderr(err):
                os.environ.clear();os.environ.update(env);sys.argv=[owner.__file__,*map(str,argv)]
                try:
                    if tuple_mode:owner.main(tuple_guards=True)
                    elif boundary_mode:owner.main(boundaries=True)
                    else:owner.main()
                except BaseException:traceback.print_exc();raise
            row['exit']=0
        except BaseException:
            row['exit']=1;raise
        finally:
            os.environ.clear();os.environ.update(prior_env);sys.argv=prior_argv
            row.update(seconds=time.monotonic()-start,stdout=str(stdout),stderr=str(stderr),stdout_sha256=sha(stdout),stderr_sha256=sha(stderr))
            report['stages'].append(row);save()
        unchanged()
    save()
    try:
        protocol=output/'protocol'
        invoke('source-api',source_owner,['--compiler-receipt',recipe_path,'--ts-runtime',args.ts_runtime,'--output',protocol],boundary_mode=boundaries,tuple_mode=tuple_guards)
        producer_path=protocol/'receipt.json';producer=json.loads(producer_path.read_text())
        assert producer['status']=='passed' and producer['head']==head and len(producer['sources'])==source_count
        pins.update(producer['inputs']);pins[str(producer_path)]=sha(producer_path)
        model_args=['--producer-root',protocol/'generated','--producer-receipt',producer_path,'--output',output/'models']
        if extended:
            invoke('source-scope',scope_owner,['--compiler-receipt',recipe_path,'--output',output/'scope'])
            scope_path=output/'scope/receipt.json'
            scope=json.loads(scope_path.read_text());assert scope['status']=='passed' and scope['head']==head
            pins[str(scope_path)]=sha(scope_path);pins.update(scope['inputs'])
            model_args += ['--scope-receipt',scope_path]
        if tuple_guards:
            invoke('tuple-source-scope',scope_owner,['--compiler-receipt',recipe_path,'--output',output/'tuple-scope'],tuple_mode=True)
            tuple_scope_path=output/'tuple-scope/receipt.json'
            tuple_scope=json.loads(tuple_scope_path.read_text())
            assert tuple_scope['status']=='passed' and tuple_scope['head']==head
            pins[str(tuple_scope_path)]=sha(tuple_scope_path);pins.update(tuple_scope['inputs'])
            model_args += ['--tuple-scope-receipt',tuple_scope_path]
        invoke('models',model_owner,model_args,boundary_mode=boundaries,tuple_mode=tuple_guards)
        model_path=output/'models/receipt.json';models=json.loads(model_path.read_text());assert models['status']=='passed'
        pins[str(model_path)]=sha(model_path)
        invoke('preflight',preflight_owner,['--producer-root',protocol/'generated','--producer-receipt',producer_path,
                    '--compiler-receipt',recipe_path,'--model-receipt',model_path,'--zkir',zkir,'--output',output/'preflight'],boundary_mode=boundaries,tuple_mode=tuple_guards)
        preflight=json.loads((output/'preflight/receipt.json').read_text())
        assert Path(preflight['prerequisites']['zkir']).resolve()==zkir
        if extended:
            pins[str(output/'preflight/receipt.json')]=sha(output/'preflight/receipt.json')
        if extended and args.preflight_only:
            assert preflight['status']=='passed' and preflight['selected_cases']==case_count
            for path,digest in preflight['frozen_inputs'].items():
                assert sha(path)==digest,path
            for path,inventory in preflight['frozen_directories'].items():
                sdk.equal_inventory(Path(path),inventory)
            dependencies.unchanged(preflight['dependency_sources'])
            unchanged()
            report.update(status='passed-key-free-preflight',proof_execution=False,key_generation=False,selected_cases=case_count,
                observed_k=models['observed_k'],verified_material_k=[k for k in models['observed_k'] if k in {row['k'] for row in preflight['verified_material']['parameters']}],
                source_receipt=dict(path=str(producer_path),sha256=sha(producer_path)),
                scope_receipt=dict(path=str(scope_path),sha256=sha(scope_path)),
                model_receipt=dict(path=str(model_path),sha256=sha(model_path)),
                preflight_receipt=dict(path=str(output/'preflight/receipt.json'),sha256=sha(output/'preflight/receipt.json')))
            if tuple_guards:
                report['tuple_scope_receipt']=dict(path=str(tuple_scope_path),sha256=sha(tuple_scope_path))
            return
        result=prove_selected(output,env,tuple_guards=True) if tuple_guards else prove_selected(output,env,boundaries=True) if boundaries else prove_selected(output,env)
        assert result['status']=='passed' and result['fresh_standalone_proofs']==result['separate_standalone_verifications']==result['separately_proved_applications']==case_count
        unchanged()
        report.update(status='passed',proof_execution=True,fresh_keypairs=source_count*2,standalone_proof_cases=case_count,separate_standalone_verifications=case_count,separate_default_strict_applications=case_count,
                      source_receipt=dict(path=str(producer_path),sha256=sha(producer_path)),model_receipt=dict(path=str(model_path),sha256=sha(model_path)),
                      preflight_receipt=dict(path=str(output/'preflight/receipt.json'),sha256=sha(output/'preflight/receipt.json')),
                      proof_receipt=dict(path=str(output/'proofs/receipt.json'),sha256=sha(output/'proofs/receipt.json')))
    except BaseException as error:
        report.update(status='failed',error=repr(error));raise
    finally:
        save();print(json.dumps({k:v for k,v in report.items() if k not in ['stages']},indent=2))

if __name__=='__main__':
    main()
