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

"""Current SDK selected-closure/build gate; no SDK API or network execution."""
import argparse,hashlib,io,json,os,stat,subprocess,tarfile,tomllib
from pathlib import Path,PurePosixPath
LEDGER='6fbc2e3900845a858f70eebf8a0deab1d4d3c0d6';ZK='bca8db4d9e2c1ecaa5d1db5ba7a201935b6c6c8a';ZKIR='3377ad9e48741f60e0aa17010b6f23e34223d906'
FORBIDDEN={'mock-verify','mock-prove','ledger-transaction'}
def selected(data,manifest):
 packages={p['id']:p for p in data['packages']};nodes={n['id']:n for n in data['resolve']['nodes']}
 roots=[p['id']for p in packages.values()if p['manifest_path']==str(manifest)]
 assert len(roots)==1,'exact package manifest selector';seen=set();todo=roots.copy()
 while todo:
  key=todo.pop()
  if key in seen:continue
  seen.add(key);todo.extend(d['pkg']for d in nodes[key]['deps'])
 return roots[0],{key:packages[key]for key in seen},{key:nodes[key]for key in seen}
def read(path,cap=64*2**20):
 path=Path(path);s=path.lstat();assert stat.S_ISREG(s.st_mode)and s.st_nlink==1 and s.st_size<=cap
 identity=lambda s:(s.st_dev,s.st_ino,s.st_size,s.st_mode,s.st_uid,s.st_mtime_ns,s.st_ctime_ns)
 with os.fdopen(os.open(path,os.O_RDONLY|os.O_NOFOLLOW),'rb')as f:
  assert identity(s)==identity(os.fstat(f.fileno()));raw=f.read(cap+1);assert len(raw)==s.st_size and identity(s)==identity(os.fstat(f.fileno()))
 assert identity(s)==identity(path.lstat());return raw

def registry_members(raw,package,checksum):
 # Cargo's locked checksum authenticates the compressed archive, not a mutable
 # checksum table next to unpacked sources. Never extract archive paths.
 assert len(raw)<=64*2**20 and hashlib.sha256(raw).hexdigest()==checksum,'locked registry archive checksum'
 files={};dirs=set();seen=set();total=0
 with tarfile.open(fileobj=io.BytesIO(raw),mode='r|gz')as archive:
  for member in archive:
   name=member.name;path=PurePosixPath(name)
   assert name and not path.is_absolute()and str(path)==name.rstrip('/')and all(x not in {'','.','..'}for x in path.parts)and '\\'not in name,'safe canonical crate member'
   assert path.parts[0]==package and (member.isdir()or member.isreg())and not member.issparse(),'one regular crate tree'
   rel='/'.join(path.parts[1:]);assert rel not in seen,'duplicate archive member';seen.add(rel);assert len(seen)<=100000
   if not rel:assert member.isdir();continue
   if member.isdir():dirs.add(rel);continue
   assert rel not in {'.cargo-ok','.cargo-checksum.json'}and 0<=member.size<=64*2**20,'archive payload bounds'
   total+=member.size;assert total<=256*2**20,'bounded crate contents'
   stream=archive.extractfile(member);assert stream is not None
   body=stream.read(member.size+1);assert len(body)==member.size
   files[rel]=(hashlib.sha256(body).hexdigest(),member.size,member.mode&0o111)
   dirs.update(str(parent)for parent in PurePosixPath(rel).parents if str(parent)!='.')
 assert files and 'Cargo.toml'in files,'complete Cargo crate'
 return files,dirs

def registry_check(root,package,checksum):
 # Standard Cargo registry/src/<index>/<name-version> paired with cache/<index>.
 assert root.name==package and root.parent.parent.name=='src','Cargo registry source layout'
 archive=root.parents[2]/'cache'/root.parent.name/(package+'.crate')
 files,dirs=registry_members(read(archive),package,checksum);actual_files=set();actual_dirs=set()
 def error(exc):raise exc
 for base,children,leaves in os.walk(root,followlinks=False,onerror=error):
  for name in children+leaves:
   path=Path(base)/name;rel=path.relative_to(root).as_posix();mode=path.lstat().st_mode
   if stat.S_ISDIR(mode):actual_dirs.add(rel)
   else:
    assert stat.S_ISREG(mode),'registry link/special entry refused'
    if rel in {'.cargo-ok','.cargo-checksum.json'}:continue
    assert rel in files,'unarchived registry source'
    expected,size,execute=files[rel];body=read(path);assert len(body)==size and hashlib.sha256(body).hexdigest()==expected and mode&0o111==execute,'authenticated registry member drift';actual_files.add(rel)
 assert actual_files==set(files)and actual_dirs==dirs,'complete registry physical membership'
 table=root/'.cargo-checksum.json'
 if os.path.lexists(table):
  info=json.loads(read(table));assert info['package']==checksum and info['files']=={name:value[0]for name,value in files.items()},'optional checksum table must agree with archive'
 return files

def source_check(packages,locked):
 roots={};files=0;digest=hashlib.sha256()
 for p in sorted(packages.values(),key=lambda p:p['id']):
  if p['source']is None:continue
  manifest=Path(p['manifest_path']);source=p['source']
  if source.startswith('git+'):
   revision=source.rsplit('#',1)[1];assert revision in {LEDGER,ZK,ZKIR}
   root=Path(subprocess.check_output(['git','rev-parse','--show-toplevel'],cwd=manifest.parent,text=True).strip())
   assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()==revision
   if revision in roots:continue
   tree=subprocess.check_output(['git','ls-tree','-rz','--full-tree',revision],cwd=root)
   for row in tree.split(b'\0'):
    if not row:continue
    header,name=row.split(b'\t',1);mode,kind,oid=header.split();assert kind==b'blob'
    path=root/os.fsdecode(name)
    if mode==b'120000':assert path.is_symlink();raw=os.fsencode(os.readlink(path))
    else:raw=read(path)
    actual=hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest().encode();assert actual==oid,'SDK Git source drift'
    digest.update(revision.encode()+name+oid);files+=1
   roots[revision]=True
  else:
   key=(p['name'],p['version'],source);checksum=locked[key];assert isinstance(checksum,str)and len(checksum)==64
   members=registry_check(manifest.parent,p['name']+'-'+p['version'],checksum)
   for name,(expected,_,_)in sorted(members.items()):digest.update(p['id'].encode()+name.encode()+expected.encode());files+=1
 assert set(roots)=={LEDGER,ZK,ZKIR},'all current SDK source roots'
 return files,digest.hexdigest()

def evaluate(metadata,lock,manifest,buildlog,kind,check_sources=True):
 root,packages,nodes=selected(metadata,manifest);local=[p for p in packages.values()if p['source']is None]
 if kind=='sdk':assert len(local)==1 and local[0]['id']==root and local[0]['name']=='compact-rust-proof-data'
 else:
  assert packages[root]['name']=='portable-proof-data-capture'
  assert all(p['name']in {'portable-proof-data-capture','compact-rust-proof-data','compact-contract-original-tiny','compact-contract-map-field-field','midnight-compact-runtime','midnight-compact-runtime-macros'}for p in local)
 for n in nodes.values():assert not(FORBIDDEN&set(n['features'])),'forbidden selected feature'
 for p in packages.values():assert p['name']not in {'midnight-ledger-v9','midnight-ledger-v8','midnight-transaction'}
 expected={'midnight-transient-crypto':('3.1.0',LEDGER),'midnight-zkir':('3.2.1',ZKIR),'midnight-proofs':('0.8.2',ZK),'midnight-curves':('0.3.1',ZK),'midnight-zk-stdlib':('2.4.0',ZK),'midnight-serialize':('1.2.1',LEDGER)}
 for name,(version,pin)in expected.items():
  matches=[p for p in packages.values()if p['name']==name];assert len(matches)==1 and matches[0]['version']==version and matches[0]['source'].endswith('#'+pin),'current SDK package/source pin'
 tuples={(p['name'],p['version'],p.get('source')):p.get('checksum')for p in lock['package']}
 for p in packages.values():assert(p['name'],p['version'],p['source'])in tuples,'selected package absent from locked graph'
 messages=[json.loads(line)for line in buildlog.splitlines()if line.startswith('{')];assert any(m.get('reason')=='build-finished'and m['success']for m in messages)
 artifacts=[m for m in messages if m.get('reason')=='compiler-artifact'];by_id={}
 for artifact in artifacts:
  ident=artifact['package_id'];assert ident in packages,'artifact outside selected dependency closure'
  assert not(FORBIDDEN&set(artifact['features'])),'forbidden ACTUAL artifact feature';assert set(artifact['features'])<=set(nodes[ident]['features']),'actual artifact features absent from selected resolution';by_id.setdefault(ident,[]).append(artifact)
 bins={'produce','verify'}if kind=='sdk'else{'capture'}
 actual=[a for a in by_id.get(root,[])if a['target']['kind']==['bin']and a.get('executable')]
 assert {a['target']['name']for a in actual}==bins
 declared={t['name']:t['src_path']for t in packages[root]['targets']if t['kind']==['bin']}
 for artifact in actual:assert artifact['target']['src_path']==declared[artifact['target']['name']]
 for name,(version,_)in expected.items():
  ident=next(p['id']for p in packages.values()if p['name']==name and p['version']==version);assert ident in by_id,'actual SDK build artifact missing'
 files,digest=source_check(packages,tuples)if check_sources else(0,'PURE_CONTROL_ONLY')
 return dict(schema='compact-rust-proof-data-sdk-gate/v1',passed=True,kind=kind,ledger=LEDGER,zk=ZK,zkir=ZKIR,selected_root=root,metadata_reachable_packages=len(packages),metadata_reachable_ids=sorted(packages),actual_artifact_packages=len(by_id),actual_artifact_ids=sorted(by_id),actual_bins=sorted(bins),source_files_hashed=files,source_digest=digest,workspace_package_count=len(metadata['workspace_members']),graph_scope='PACKAGE_SELECTED_METADATA_REACHABLE_AND_ACTUAL_BUILD_ARTIFACTS_SEPARATELY',whole_workspace_qualified=False)
def main():
 ap=argparse.ArgumentParser();ap.add_argument('metadata');ap.add_argument('lock');ap.add_argument('manifest');ap.add_argument('buildlog');ap.add_argument('output');ap.add_argument('--kind',choices=['sdk','capture'],required=True);a=ap.parse_args()
 result=evaluate(json.loads(read(a.metadata)),tomllib.loads(read(a.lock).decode()),Path(a.manifest).resolve(),read(a.buildlog,16*2**20).decode(),a.kind)
 with os.fdopen(os.open(a.output,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600),'w')as f:json.dump(result,f,indent=2);f.write('\n')
if __name__=='__main__':main()
