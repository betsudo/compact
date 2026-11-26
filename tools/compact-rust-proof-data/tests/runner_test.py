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


"""Source-only finite runner controls. No compiler, SDK or process launch."""
import ast,copy,importlib.util,io,json,tempfile,types,unittest
from pathlib import Path
from unittest import mock
spec=importlib.util.spec_from_file_location('runner',Path(__file__).parents[1]/'run.py');runner=importlib.util.module_from_spec(spec);spec.loader.exec_module(runner)
SOURCE=(Path(__file__).parents[1]/'run.py').read_text();MAIN=next(n for n in ast.parse(SOURCE).body if isinstance(n,ast.FunctionDef)and n.name=='main')
class RunnerTests(unittest.TestCase):
 def test_normal_generation_passes_the_workspace_runtime_root(self):
  outer=next(n for n in MAIN.body if isinstance(n,ast.Try)and n.finalbody)
  assignment=next(n for n in outer.body if isinstance(n,ast.Assign)and any(isinstance(t,ast.Name)and t.id=='argv'for t in n.targets))
  workspace=Path('/fixture/workspace')
  ns=dict(a=types.SimpleNamespace(compiler='/fixture/compactc'),REPO=workspace,source=workspace/'original.compact',generated=Path('/fixture/output'),str=str)
  exec(compile(ast.Module(body=[assignment],type_ignores=[]),'actual-generation-arguments','exec'),ns)
  args=ns['argv'];self.assertEqual(args[args.index('--rust-runtime-root')+1],str(workspace))
 def test_real_finalizer_never_writes_or_reads_under_replaced_owner(self):
  final=next(n.finalbody for n in MAIN.body if isinstance(n,ast.Try)and n.finalbody)
  primary=RuntimeError('first');events=[];out=Path('/virtual/owner')
  ns=dict(first=primary,out=out,roots={out:('original',)},original_owner=lambda *args:False,print=lambda *args,**kw:events.append('constant-stderr'),sys=types.SimpleNamespace(stderr=None),RuntimeError=RuntimeError)
  exec(compile(ast.Module(body=final,type_ignores=[]),'actual-finalizer','exec'),ns)
  self.assertIs(ns['first'],primary);self.assertEqual(events,['constant-stderr'])
 def test_real_check_segment_refuses_before_any_parent_copy(self):
  outer=next(n for n in MAIN.body if isinstance(n,ast.Try)and n.finalbody)
  start=next(i for i,n in enumerate(outer.body)if isinstance(n,ast.Expr)and ast.unparse(n).startswith("save(bundle / 'check-input.json'"))
  end=next(i for i,n in enumerate(outer.body)if isinstance(n,ast.Assign)and ast.unparse(n).startswith('manifest = dict(manifest, parent='))
  segment=ast.Module(body=outer.body[start:end],type_ignores=[])
  for measured_k,check_ok,should_copy in [(13,True,False),(7,False,False),(7,True,True)]:
   events=[]
   class FakePath:
    def __init__(self,name):self.name=name
    def __truediv__(self,name):return FakePath(self.name+'/'+str(name))
    @property
    def parent(self):return FakePath(self.name+'/..')
    def __str__(self):return self.name
    def read_text(self):
     if self.name.endswith('model.json'):return json.dumps(dict(k=measured_k,cases=[dict(id='one',SDKCheckPassed=check_ok)]))
     return json.dumps(dict(packages=[dict(name='midnight-transient-crypto',version='3.1.0',source='git#pin',manifest_path='/sdk/Cargo.toml')]))
    def chmod(self,*args):events.append('chmod-parent')
   def run(label,*args,**kw):
    events.append(label)
    if measured_k>8:raise RuntimeError('actual child over-budget')
   ns=dict(bundle=FakePath('bundle'),out=FakePath('out'),manifest=dict(parent=None),flags=[],exe={'produce':'producer'},run=run,save=lambda *a:events.append('save-check-input'),ref=lambda *a:{'cached':True},a=types.SimpleNamespace(max_k=8,allow_large_k=False),cases=[dict(id='one')],json=json,sdkmeta=FakePath('metadata'),data={'ledger':'pin'},Path=FakePath,shutil=types.SimpleNamespace(copyfile=lambda *a:events.append('parent-copy')))
   try:exec(compile(segment,'actual-check-segment','exec'),ns)
   except (RuntimeError,AssertionError):self.assertFalse(should_copy)
   self.assertEqual('parent-copy'in events,should_copy)
 def test_bound_source_tree_detects_byte_and_membership_drift(self):
  with tempfile.TemporaryDirectory()as directory:
   root=Path(directory);(root/'lib.rs').write_text('original');before=runner.source_tree(root)
   (root/'lib.rs').write_text('changed');self.assertNotEqual(runner.source_tree(root),before)
   (root/'extra.rs').write_text('extra');self.assertNotEqual(set(runner.source_tree(root)),set(before))
 def test_real_source_guard_refuses_owner_before_any_source_reader(self):
  node=next(n for n in MAIN.body if isinstance(n,ast.FunctionDef)and n.name=='sources');events=[]
  ns=dict(out='out',roots={'out':'before'},original_owner=lambda *args:False,source_files={'source':{}},fingerprint=lambda *args:events.append('read'),source_trees={},harness_revision=None)
  exec(compile(ast.Module(body=[node],type_ignores=[]),'actual-source-guard','exec'),ns)
  with self.assertRaises(AssertionError):ns['sources']()
  self.assertEqual(events,[])
 def test_compiler_source_is_not_attributed_to_harness_head(self):
  self.assertIn("source_revision='UNKNOWN_EXTERNAL_COMPILER_SOURCE_REVISION'",SOURCE)
  self.assertIn("harness_git_revision=harness_revision",SOURCE)
  self.assertIn('source_trees[generated]=source_tree(generated)',SOURCE)
  self.assertIn('source_trees[capture]=source_tree(capture)',SOURCE)
 def test_all_case_selectors_use_the_correct_fixture_capture_and_operation(self):
  outer=next(n for n in MAIN.body if isinstance(n,ast.Try)and n.finalbody)
  names=['source','capture_source','operation','original_ir']
  nodes=[n for n in outer.body if isinstance(n,ast.Assign)and any(isinstance(t,ast.Name)and t.id in names for t in n.targets)]
  manifest=next(n.value for n in outer.body if isinstance(n,ast.Assign)and any(isinstance(t,ast.Name)and t.id=='manifest'for t in n.targets))
  profile=next(k.value for k in manifest.keywords if k.arg=='profile')
  for case,module,operation,expected_profile in [('map-member','map.rs','member','map-member'),('map-lookup','map_lookup.rs','lookup','map-lookup'),('map-is-empty','map_is_empty.rs','is_empty','map-is-empty'),('map-size','map_size.rs','size','unit-uint64'),('set-is-empty','set_is_empty.rs','is_empty','set-is-empty'),('set-member','set_member.rs','member','set-member'),('set-size','set_size.rs','size','set-size'),('tiny-clear','tiny.rs','clear','tiny-clear'),('tiny-set','tiny.rs','set','tiny-set')]:
   ns=dict(a=types.SimpleNamespace(case=case),REPO=Path('/fixture/workspace'),bundle=Path('/fixture/bundle'),generated=Path('/fixture/generated'))
   exec(compile(ast.Module(body=nodes,type_ignores=[]),'actual-case-selection','exec'),ns)
   self.assertEqual(ns['capture_source'],module);self.assertEqual(ns['operation'],operation)
   self.assertEqual(eval(compile(ast.Expression(body=copy.deepcopy(profile)),'actual-profile-routing','eval'),ns),expected_profile)
   self.assertEqual(ns['original_ir'],Path('/fixture/generated/zkir')/(operation+'.zkir'))
   self.assertEqual(ns['source'].name,'original-map-field-field.compact'if case.startswith('map-')else'original-set-field.compact'if case in ['set-is-empty','set-member','set-size']else'original-tiny.compact')
 def test_actual_parser_accepts_all_cases_and_preserves_default_budget(self):
  prefix=[]
  for node in MAIN.body:
   prefix.append(node)
   if isinstance(node,ast.Assign)and any(isinstance(t,ast.Name)and t.id=='a'for t in node.targets):break
  for case in ['map-member','map-lookup','map-is-empty','map-size','set-is-empty','set-member','set-size','tiny-clear','tiny-set',None]:
   extra=['--case',case]if case else[];expected=case or'map-member'
   with mock.patch.object(runner.sys,'argv',['run.py','--compiler','/virtual/compactc','--out','/virtual/fresh',*extra]):
    ns=dict(argparse=runner.argparse,Path=Path)
    exec(compile(ast.Module(body=prefix,type_ignores=[]),'actual-cli-prefix','exec'),ns)
   self.assertEqual(ns['a'].case,expected);self.assertEqual(ns['a'].max_k,8)
   self.assertFalse(ns['a'].allow_large_k);self.assertEqual(ns['a'].out,Path('/virtual/fresh'))
 def test_set_domain_admission_refuses_map_carrier_and_changed_shape(self):
  ir=json.loads((runner.REPO/'tools/compact-rust-backend/fixtures/upstream-672/export-set-field/compact-rust-ir.json').read_text())
  raw=(runner.HERE/'src/producer/map_is_empty.json').read_bytes()
  caps=dict(schema_version=3,circuits=[dict(name=c['name'],recorded=True,observed_call=True,**(dict(standalone_proof_data=dict(api_version=1,method='is_empty_proof_data'))if c['name']=='is_empty'else{}))for c in ir['stateful_circuits']])
  self.assertEqual(runner.admit_set_is_empty(ir,caps,raw)['profile'],'set-is-empty')
  for pointer,value in [('declaration',dict(kind='map',key=dict(kind='field'),value=dict(kind='field'))),('id','other'),('index',1),('path',[0,1])]:
   changed=copy.deepcopy(ir);changed['ledger_fields'][0][pointer]=value
   with self.assertRaises(AssertionError):runner.admit_set_is_empty(changed,caps,raw)
  for field,value in [('parameters',[dict(name='key',ty=dict(kind='field'))]),('actions',[dict(kind='set_reset',field='c',index=0)]),('result',dict(kind='field')),('return_value',dict(kind='map_is_empty',field='c',index=0))]:
   changed=copy.deepcopy(ir);next(c for c in changed['stateful_circuits']if c['name']=='is_empty')[field]=value
   with self.assertRaises(AssertionError):runner.admit_set_is_empty(changed,caps,raw)
  changed=copy.deepcopy(caps);next(c for c in changed['circuits']if c['name']=='is_empty')['standalone_proof_data']['api_version']=2
  with self.assertRaises(AssertionError):runner.admit_set_is_empty(ir,changed,raw)
  with self.assertRaises(AssertionError):runner.admit_set_is_empty(ir,caps,raw+b' ')
 def test_set_read_typed_domains_and_helper_inventory_refuse_cross_subjects(self):
  ir=json.loads((runner.REPO/'tools/compact-rust-backend/fixtures/upstream-672/export-set-field/compact-rust-ir.json').read_text())
  caps=dict(schema_version=3,circuits=[dict(name=c['name'],recorded=True,observed_call=True,**(dict(standalone_proof_data=dict(api_version=1,method=c['name']+'_proof_data'))if c['name']in['is_empty','member','size']else{}))for c in ir['stateful_circuits']])
  for profile,name in [('set-member','member'),('set-size','size')]:
   raw=(runner.HERE/'src/producer'/('set_'+name+'.json')).read_bytes()
   self.assertEqual(runner.admit_set_read(profile,ir,caps,raw)['profile'],profile)
   for key,value in [('id','other'),('index',1),('path',[0,1]),('declaration',dict(kind='map',key=dict(kind='field'),value=dict(kind='field')))]:
    changed=copy.deepcopy(ir);changed['ledger_fields'][0][key]=value
    with self.assertRaises(AssertionError):runner.admit_set_read(profile,changed,caps,raw)
   for key,value in [('actions',[dict(kind='set_reset',field='c',index=0)]),('result',dict(kind='field')),('parameters',[]if name=='member'else[dict(name='extra',ty=dict(kind='field'))])]:
    changed=copy.deepcopy(ir);next(c for c in changed['stateful_circuits']if c['name']==name)[key]=value
    with self.assertRaises(AssertionError):runner.admit_set_read(profile,changed,caps,raw)
   changed=copy.deepcopy(caps);next(c for c in changed['circuits']if c['name']==name)['standalone_proof_data']['api_version']=2
   with self.assertRaises(AssertionError):runner.admit_set_read(profile,ir,changed,raw)
   with self.assertRaises(AssertionError):runner.admit_set_read(profile,ir,caps,raw+b' ')
   swapped=raw.replace(b'%value.0',b'%key.0')if name=='member'else(runner.HERE/'src/producer/set_member.json').read_bytes()
   with self.assertRaises(AssertionError):runner.admit_set_read(profile,ir,caps,swapped)
 def test_set_is_empty_keeps_one_helper_history_and_accepts_exact_three_helper_successor(self):
  ir=json.loads((runner.REPO/'tools/compact-rust-backend/fixtures/upstream-672/export-set-field/compact-rust-ir.json').read_text())
  raw=(runner.HERE/'src/producer/map_is_empty.json').read_bytes()
  for names in [['is_empty'],['is_empty','member','size']]:
   caps=dict(schema_version=3,circuits=[dict(name=c['name'],recorded=True,observed_call=True,**(dict(standalone_proof_data=dict(api_version=1,method=c['name']+'_proof_data'))if c['name']in names else{}))for c in ir['stateful_circuits']])
   self.assertEqual(runner.admit_set_is_empty(ir,caps,raw)['profile'],'set-is-empty')
  caps['circuits'][-1]['standalone_proof_data']['method']='other'
  with self.assertRaises(AssertionError):runner.admit_set_is_empty(ir,caps,raw)
if __name__=='__main__':unittest.main()
