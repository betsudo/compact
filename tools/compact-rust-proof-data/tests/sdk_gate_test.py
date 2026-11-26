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


"""Pure synthetic metadata/artifact cases. No Cargo/SDK/source scanning."""
import copy,hashlib,importlib.util,io,json,tarfile,tempfile,unittest
from pathlib import Path
spec=importlib.util.spec_from_file_location('sdk_gate',Path(__file__).parents[1]/'sdk_gate.py');gate=importlib.util.module_from_spec(spec);spec.loader.exec_module(gate)
def example():
 manifest=Path('/repo/tools/compact-rust-proof-data/Cargo.toml')
 packages=[dict(id='root',name='compact-rust-proof-data',version='0.1.0',source=None,manifest_path=str(manifest),targets=[dict(name=n,kind=['bin'],src_path='/repo/'+n+'.rs')for n in ['produce','verify']])]
 core=[('midnight-transient-crypto','3.1.0',gate.LEDGER),('midnight-zkir','3.2.1',gate.ZKIR),('midnight-proofs','0.8.2',gate.ZK),('midnight-curves','0.3.1',gate.ZK),('midnight-zk-stdlib','2.4.0',gate.ZK),('midnight-serialize','1.2.1',gate.LEDGER)]
 for name,version,pin in core:packages.append(dict(id=name,name=name,version=version,source='git+https://source#'+pin,manifest_path='/sdk/'+name+'/Cargo.toml',targets=[]))
 packages.append(dict(id='unrelated',name='not-selected',version='1',source=None,manifest_path='/other/Cargo.toml',targets=[]))
 nodes=[dict(id=p['id'],features=[],deps=[dict(pkg=q['id'])for q in packages[1:-1]]if p['id']=='root'else[])for p in packages]
 metadata=dict(packages=packages,resolve=dict(nodes=nodes),workspace_members=['root']+['unrelated']*307)
 lock=dict(package=[dict(name=p['name'],version=p['version'],source=p['source'])for p in packages])
 artifacts=[dict(reason='compiler-artifact',package_id=p['id'],features=[],target=dict(kind=['bin']if p['id']=='root'else['lib'],name=n,src_path='/repo/'+n+'.rs'),executable='/bin/'+n if p['id']=='root'else None)for p in packages[:-1]for n in (['produce','verify']if p['id']=='root'else[p['name']])]
 return metadata,lock,manifest,artifacts
class GateTests(unittest.TestCase):
 def check(self,data):
  m,l,p,a=data;return gate.evaluate(m,l,p,'\n'.join(map(json.dumps,[*a,dict(reason='build-finished',success=True)])),'sdk',False)
 def test_workspace_is_not_selected_sdk_graph(self):
  result=self.check(example());self.assertEqual(result['metadata_reachable_packages'],7);self.assertEqual(result['actual_artifact_packages'],7);self.assertEqual(result['workspace_package_count'],308);self.assertFalse(result['whole_workspace_qualified'])
 def test_both_selected_and_actual_mock_features_refuse(self):
  for feature in ['mock-prove','mock-verify','ledger-transaction']:
   data=example();data[0]['resolve']['nodes'][1]['features']=[feature]
   with self.assertRaises(AssertionError):self.check(data)
   data=example();data[3][2]['features']=[feature]
   with self.assertRaises(AssertionError):self.check(data)
 def test_actual_artifact_must_match_selected_target_and_feature(self):
  for edit in [lambda a:a[0]['target'].update(src_path='/wrong.rs'),lambda a:a[0].update(features=['unexpected']),lambda a:a.append(dict(a[0],package_id='unrelated'))]:
   data=example();edit(data[3])
   with self.assertRaises(AssertionError):self.check(data)
 def test_missing_current_sdk_artifact_refuses(self):
  data=example();data[3].pop()
  with self.assertRaises(AssertionError):self.check(data)
 def test_mixed_sdk_version_refuses(self):
  data=example();p=dict(data[0]['packages'][1],id='old-sdk',version='3.0.1');data[0]['packages'].append(p);data[0]['resolve']['nodes'].extend([dict(id='old-sdk',features=[],deps=[])]);data[0]['resolve']['nodes'][0]['deps'].append(dict(pkg='old-sdk'))
  with self.assertRaises(AssertionError):self.check(data)
 def test_capture_accepts_the_intact_original_map_crate_only(self):
  m,l,p,a=example();m['packages'][0]['name']='portable-proof-data-capture';l['package'][0]['name']='portable-proof-data-capture'
  target=dict(kind=['bin'],name='capture',src_path='/repo/capture.rs');m['packages'][0]['targets']=[target]
  generated=dict(id='generated',name='compact-contract-original-map-field-field',version='0.1.0',source=None,manifest_path='/generated/Cargo.toml',targets=[])
  m['packages'].append(generated);m['resolve']['nodes'][0]['deps'].append(dict(pkg='generated'));m['resolve']['nodes'].append(dict(id='generated',features=[],deps=[]));l['package'].append(dict(name=generated['name'],version='0.1.0',source=None))
  a=[dict(reason='compiler-artifact',package_id='root',features=[],target=target,executable='/bin/capture')]+[x for x in a if x['package_id']!='root']
  log='\n'.join(map(json.dumps,[*a,dict(reason='build-finished',success=True)]))
  self.assertTrue(gate.evaluate(m,l,p,log,'capture',False)['passed'])
  generated['name']='unrelated-generated-contract';l['package'][-1]['name']=generated['name']
  with self.assertRaises(AssertionError):gate.evaluate(m,l,p,log,'capture',False)

def crate_bytes(members):
 output=io.BytesIO()
 with tarfile.open(fileobj=output,mode='w:gz')as archive:
  for name,body,kind in members:
   member=tarfile.TarInfo(name);member.mode=0o644
   if kind=='file':member.size=len(body);archive.addfile(member,io.BytesIO(body))
   elif kind=='link':member.type=tarfile.SYMTYPE;member.linkname='../outside';archive.addfile(member)
   else:member.type=tarfile.DIRTYPE;archive.addfile(member)
 return output.getvalue()
class RegistryTests(unittest.TestCase):
 def setUp(self):
  self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup);base=Path(self.temp.name)/'registry';self.root=base/'src/index/pkg-1.0';self.root.mkdir(parents=True);(self.root/'src').mkdir()
  self.contents={'Cargo.toml':b'[package]\nname="pkg"\nversion="1.0"\n','src/lib.rs':b'pub fn value()->u8{1}\n'}
  raw=crate_bytes([('pkg-1.0/'+k,v,'file')for k,v in self.contents.items()]);self.checksum=hashlib.sha256(raw).hexdigest();cache=base/'cache/index';cache.mkdir(parents=True);self.archive=cache/'pkg-1.0.crate';self.archive.write_bytes(raw)
  for name,body in self.contents.items():(self.root/name).write_bytes(body)
 def check(self):return gate.registry_check(self.root,'pkg-1.0',self.checksum)
 def table(self):return {'package':self.checksum,'files':{k:hashlib.sha256(v).hexdigest()for k,v in self.contents.items()}}
 def test_absent_table_is_valid_and_optional_table_is_secondary(self):
  self.assertEqual(set(self.check()),set(self.contents));(self.root/'.cargo-checksum.json').write_text(json.dumps(self.table()));self.assertEqual(set(self.check()),set(self.contents))
 def test_mutated_source_and_forged_matching_table_refuse(self):
  (self.root/'src/lib.rs').write_bytes(b'pub fn value()->u8{2}');table=self.table();table['files']['src/lib.rs']=hashlib.sha256((self.root/'src/lib.rs').read_bytes()).hexdigest();(self.root/'.cargo-checksum.json').write_text(json.dumps(table))
  with self.assertRaises(AssertionError):self.check()
 def test_empty_or_partial_optional_table_refuses(self):
  table=self.table();table['files']={};(self.root/'.cargo-checksum.json').write_text(json.dumps(table))
  with self.assertRaises(AssertionError):self.check()
 def test_extra_and_missing_physical_members_refuse(self):
  extra=self.root/'unarchived.rs';extra.write_text('extra')
  with self.assertRaises(AssertionError):self.check()
  extra.unlink();(self.root/'src/lib.rs').unlink()
  with self.assertRaises(AssertionError):self.check()
 def test_archive_checksum_and_executable_drift_refuse(self):
  original=self.archive.read_bytes();self.archive.write_bytes(original+b'x')
  with self.assertRaises(AssertionError):self.check()
  self.archive.write_bytes(original);(self.root/'src/lib.rs').chmod(0o755)
  with self.assertRaises(AssertionError):self.check()
 def test_link_traversal_duplicate_and_foreign_root_members_refuse(self):
  valid=[('pkg-1.0/Cargo.toml',b'ok','file')]
  for bad in [('pkg-1.0/../outside',b'x','file'),('pkg-1.0/link',b'','link'),('pkg-1.0/Cargo.toml',b'again','file'),('other/file',b'x','file'),('pkg-1.0/a//b',b'x','file')]:
   raw=crate_bytes(valid+[bad])
   with self.assertRaises(AssertionError):gate.registry_members(raw,'pkg-1.0',hashlib.sha256(raw).hexdigest())
if __name__=='__main__':unittest.main()
