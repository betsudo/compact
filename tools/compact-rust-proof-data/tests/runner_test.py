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
if __name__=='__main__':unittest.main()
