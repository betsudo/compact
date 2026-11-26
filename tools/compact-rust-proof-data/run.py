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

"""Bounded normal generation -> real native capture -> checked SDK proof -> separate verifier."""
import argparse,hashlib,json,os,resource,shutil,signal,stat,subprocess,sys,time,tomllib
from pathlib import Path
HERE=Path(__file__).resolve().parent;REPO=HERE.parents[1]
def ref(root,path):
 raw=path.read_bytes();return dict(path=str(path.relative_to(root)),bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest())
def save(path,obj):
 with os.fdopen(os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600),'w')as f:json.dump(obj,f,indent=2);f.write('\n');f.flush();os.fsync(f.fileno())
def identity(path):
 m=Path(path).lstat();return(m.st_dev,m.st_ino,m.st_uid,stat.S_IMODE(m.st_mode))
def original_owner(path,expected):
 try:return identity(path)==expected and stat.S_ISDIR(Path(path).lstat().st_mode)
 except OSError:return False

def fingerprint(path,cap=512*2**20):
 path=Path(path);before=path.lstat();assert stat.S_ISREG(before.st_mode)and before.st_size<=cap,'bounded ordinary source/image'
 key=lambda m:(m.st_dev,m.st_ino,m.st_uid,m.st_mode,m.st_nlink,m.st_size,m.st_mtime_ns,m.st_ctime_ns)
 digest=hashlib.sha256();size=0
 with os.fdopen(os.open(path,os.O_RDONLY|os.O_NOFOLLOW),'rb')as stream:
  assert key(before)==key(os.fstat(stream.fileno()))
  while block:=stream.read(4*2**20):size+=len(block);assert size<=cap;digest.update(block)
  assert key(before)==key(os.fstat(stream.fileno()))
 assert size==before.st_size and key(before)==key(path.lstat()),'source/image changed while hashing'
 return dict(identity=list(key(before)),bytes=size,sha256=digest.hexdigest())

def source_tree(root):
 root=Path(root);assert not root.is_symlink()and root.is_dir();rows={'.':dict(directory=list(identity(root)))};total=0
 def error(exc):raise exc
 for base,dirs,files in os.walk(root,followlinks=False,onerror=error):
  dirs[:]=[d for d in dirs if d!='__pycache__']
  for name in dirs+files:
   path=Path(base)/name;rel=path.relative_to(root).as_posix();mode=path.lstat().st_mode
   if stat.S_ISDIR(mode):rows[rel]=dict(directory=list(identity(path)))
   else:rows[rel]=fingerprint(path,4*2**20);total+=rows[rel]['bytes']
   assert len(rows)<=4096 and total<=64*2**20,'bounded local source tree'
 return rows

secondary=[]
def exited(p):
 return os.waitid(os.P_PID,p.pid,os.WEXITED|os.WNOHANG|os.WNOWAIT) is not None
def group(p):
 raw=subprocess.check_output(['/bin/ps','-axo','pid=,pgid=,state='],text=True)
 return [dict(pid=int(a),pgid=int(b),state=c) for line in raw.splitlines() for a,b,c in [line.split()] if int(b)==p.pid]
def drain_and_reap(p):
 # The unreaped session leader pins PID/PGID until every signal is finished.
 result={};problem=None
 try:
  initial=group(p);result['initialGroup']=initial
  residual=[r for r in initial if r['pid']!=p.pid and not r['state'].startswith('Z')]
  if residual:problem=RuntimeError('residual descendants after Cargo leader termination')
  if residual or not exited(p):
   try:os.killpg(p.pid,signal.SIGTERM)
   except ProcessLookupError:pass
   until=time.monotonic()+5
   while time.monotonic()<until:
    if exited(p) and not [r for r in group(p) if r['pid']!=p.pid and not r['state'].startswith('Z')]:break
    time.sleep(.1)
   if not exited(p) or [r for r in group(p) if r['pid']!=p.pid and not r['state'].startswith('Z')]:
    try:os.killpg(p.pid,signal.SIGKILL)
    except ProcessLookupError:pass
  until=time.monotonic()+5
  while not exited(p):
   if time.monotonic()>until:raise RuntimeError('leader did not terminate after held-group drain')
   time.sleep(.1)
  result['beforeReapGroup']=group(p)
  if any(r['pid']!=p.pid and not r['state'].startswith('Z') for r in result['beforeReapGroup']):
   problem=problem or RuntimeError('live descendant remains after held-group drain')
 except BaseException as e:
  secondary.append('group cleanup: '+repr(e));problem=problem or e
  # PID remains held: this is the final allowed signaling opportunity.
  try:os.killpg(p.pid,signal.SIGKILL)
  except ProcessLookupError:pass
  except BaseException as e:secondary.append('held group kill: '+repr(e))
 try:result['exit']=p.wait(timeout=5)
 except BaseException as e:
  secondary.append('leader reap: '+repr(e));problem=problem or e
 # No signal is permitted after wait/reap, including on observation failure.
 try:result['afterReapGroup']=group(p)
 except BaseException as e:secondary.append('postreap group observation: '+repr(e));problem=problem or e
 return result,problem
def reap(p):
 disposition,problem=drain_and_reap(p)
 if disposition.get('afterReapGroup'):problem=problem or RuntimeError('group remains after reap')
 if problem:raise problem
 return disposition['exit']

def main():
 ap=argparse.ArgumentParser();ap.add_argument('--compiler',required=True);ap.add_argument('--case',choices=['map-member','tiny-clear','tiny-set'],default='map-member');ap.add_argument('--out',type=Path,required=True);ap.add_argument('--max-k',type=int,default=8);ap.add_argument('--allow-large-k',action='store_true');ap.add_argument('--binding-le',default=(17).to_bytes(32,'little').hex());ap.add_argument('--randomness-le',default=(29).to_bytes(32,'little').hex());a=ap.parse_args()
 assert 1<=a.max_k<=13 and(a.max_k<=8 or a.allow_large_k)
 assert REPO not in a.out.resolve().parents and a.out.resolve()!=REPO,'output must be outside repository sources'
 assert not a.out.exists();a.out.mkdir(mode=0o700);out=a.out.resolve();target=out/'target';target.mkdir(mode=0o700);bundle=out/'inputs';bundle.mkdir(mode=0o700)
 start=time.monotonic();rows=[];first=None;child=None;materialized=[];admitted_input=None;data=None;source_files={};source_trees={};harness_revision=None;compiler_image=None;check_input=None;check_model=None
 roots={p:identity(p)for p in [out,target,bundle]};last_census=0
 env={k:v for k,v in os.environ.items()if k in ['HOME','CARGO_HOME','RUSTUP_HOME','PATH']};env.update(CARGO_TARGET_DIR=str(target),CARGO_NET_OFFLINE='true',CARGO_INCREMENTAL='0',CARGO_BUILD_JOBS='2',RAYON_NUM_THREADS='2',CARGO_PROFILE_DEV_DEBUG='0',RUSTUP_TOOLCHAIN='1.99.0',LANG='C',LC_ALL='C')
 def cancel(*_):raise KeyboardInterrupt('cancelled')
 for sig in [signal.SIGINT,signal.SIGTERM]:signal.signal(sig,cancel)
 def sources():
  assert original_owner(out,roots[out]),'Owner replaced before source reader'
  for path,before in source_files.items():assert fingerprint(path)==before,'bound source/image drift: '+str(path)
  for path,before in source_trees.items():assert source_tree(path)==before,'bound source tree/overlay drift: '+str(path)
  if harness_revision is not None:assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=REPO,text=True,timeout=15).strip()==harness_revision,'harness Git revision changed'
 def caps(raws,strict=False):
  nonlocal last_census
  for p,expected in roots.items():assert identity(p)==expected and p.is_dir()and not p.is_symlink(),'creation identity'
  assert shutil.disk_usage(out).free>=50*2**30 and time.monotonic()-start<2400
  for p in raws:assert p.stat().st_size<=16*2**20
  if not strict and time.monotonic()-last_census<1:return
  def missing(e):
   if not strict and isinstance(e,FileNotFoundError)and e.filename and Path(e.filename)!=target and target in Path(e.filename).parents:return
   raise e
  totals=[0,0];entries=0
  for base,dirs,files in os.walk(out,followlinks=False,onerror=missing):
   for name in dirs+files:
    path=Path(base)/name
    try:m=path.lstat()
    except FileNotFoundError:
     if not strict and target in path.parents:continue
     raise
    entries+=1;assert entries<=300000,'finite owned entries'
    # Cargo fingerprint/cache symlinks are data, never followed by this census.
    if stat.S_ISREG(m.st_mode):totals[int(target in path.parents)]+=m.st_size
    else:assert stat.S_ISDIR(m.st_mode)or(stat.S_ISLNK(m.st_mode)and target in path.parents),'ordinary owned output'
    assert totals[0]<=128*2**20 and totals[1]<=16*2**30,'Owner128MiB/target16GiB'
  last_census=time.monotonic()
 def run(label,argv,cwd=REPO,timeout=900,compiler=False):
  nonlocal child
  stdout=out/(label+'.stdout');stderr=out/(label+'.stderr');since=time.monotonic();pid=None;code=None;error=None
  def setup():os.umask(0o022 if compiler else 0o077);resource.setrlimit(resource.RLIMIT_CORE,(0,0))
  try:
   caps([]);sources()
   with stdout.open('xb')as o,stderr.open('xb')as e:
    stdout.chmod(0o600);stderr.chmod(0o600);child=subprocess.Popen(argv,cwd=cwd,env=env,stdin=subprocess.DEVNULL,stdout=o,stderr=e,start_new_session=True,preexec_fn=setup);pid=child.pid
    while not exited(child):
     caps([stdout,stderr]);assert time.monotonic()-since<timeout
     raw=subprocess.check_output(['ps','-axo','pgid=,rss='],text=True);rss=sum(int(b)*1024 for line in raw.splitlines()for gid,b in [line.split()]if int(gid)==pid);assert rss<=4*2**30;time.sleep(.1)
    status=os.waitid(os.P_PID,child.pid,os.WEXITED|os.WNOHANG|os.WNOWAIT);code=status.si_status if status.si_code==os.CLD_EXITED else -status.si_status
    if code:error=RuntimeError('first nonzero '+label+': '+str(code))
    held=child
    try:code=reap(held)
    except BaseException as close:
     if error:secondary.append('group closeout: '+repr(close))
     else:error=close
    finally:child=None
    try:caps([stdout,stderr],strict=True);sources();assert time.monotonic()-since<timeout
    except BaseException as close:
     if error:secondary.append('terminal bounds: '+repr(close))
     else:error=close
    if error:raise error
  except BaseException as exc:
   error=exc
   if child is not None:
    try:code=reap(child)
    except BaseException as close:rows.append(dict(secondaryCloseout=repr(close)))
    child=None
   raise
  finally:rows.append(dict(phase=label,argv=[str(x)for x in argv],pid=pid,exit=code,error=repr(error)if error else None,seconds=time.monotonic()-since))
  return stdout
 try:
  harness_revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=REPO,text=True,timeout=15).strip()
  source=REPO/'tools/compact-rust-backend/fixtures/upstream-672'/('export-map-field-field/original-map-field-field.compact'if a.case=='map-member'else'tiny-get/original-tiny.compact')
  for tree in [HERE,REPO/'runtime-rs/src',REPO/'runtime-rs-macros/src']:source_trees[tree]=source_tree(tree)
  for path in [REPO/'Cargo.toml',REPO/'Cargo.lock',REPO/'runtime-rs/Cargo.toml',REPO/'runtime-rs-macros/Cargo.toml',source]:source_files[path]=fingerprint(path)
  a.compiler=str(Path(a.compiler).resolve(strict=True));compiler_image=fingerprint(Path(a.compiler));source_files[Path(a.compiler)]=compiler_image
  source_files[Path(sys.executable).resolve()]=fingerprint(Path(sys.executable).resolve())
  if sys.platform=='darwin':
   env['SDKROOT']=subprocess.check_output(['xcrun','--sdk','macosx','--show-sdk-path'],text=True).strip();env.update(CC='/usr/bin/clang',CXX='/usr/bin/clang++',CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER='/usr/bin/clang')
  cargo=shutil.which('cargo');assert cargo
  for toolpath in [Path(cargo).resolve(),Path(shutil.which('rustc')).resolve()]:source_files[toolpath]=fingerprint(toolpath)
  assert run('cargo-version',[cargo,'--version'],timeout=30).read_text().startswith('cargo 1.99.')
  rust=run('rustc-version',['rustc','-vV'],timeout=30).read_text();assert 'release: 1.99.'in rust
  host=next(line.removeprefix('host: ')for line in rust.splitlines()if line.startswith('host: '));assert host
  version=run('compiler-version',[a.compiler,'--version'],timeout=30).read_text().strip();assert version
  for flag in ['--rust-standalone-proof-data','--rust-require-recording']:assert flag in run('compiler-help-'+flag[2:],[a.compiler,'--help'],timeout=30).read_text()
  tool=HERE/'Cargo.toml';sdkmeta=run('sdk-metadata',[cargo,'metadata','--locked','--offline','--format-version','1','--filter-platform',host,'--manifest-path',str(tool)])
  sdkbuild=run('sdk-build',[cargo,'build','--package','compact-rust-proof-data','--bin','produce','--bin','verify','--locked','--offline','--profile','proof-checks','--message-format=json-render-diagnostics','--jobs','2'])
  run('sdk-gate',[sys.executable,str(HERE/'sdk_gate.py'),str(sdkmeta),str(REPO/'Cargo.lock'),str(tool),str(sdkbuild),str(bundle/'sdk-gate.json'),'--kind','sdk'])
  data=json.loads((bundle/'sdk-gate.json').read_text());materialized.extend(ref(out,p)for p in [sdkmeta,sdkbuild,bundle/'sdk-gate.json']);messages=[json.loads(x)for x in sdkbuild.read_text().splitlines()if x.startswith('{')]
  exe={m['target']['name']:m['executable']for m in messages if m.get('reason')=='compiler-artifact'and m['target']['kind']==['bin']and m.get('executable')and m['target']['name']in ['produce','verify']};assert set(exe)=={'produce','verify'}
  for path in exe.values():source_files[Path(path).resolve()]=fingerprint(Path(path).resolve())
  generated=out/'generated';argv=[a.compiler,'--target','rust','--target','ts','--feature-zkir-v3','--skip-zk','--ts-runtime-profile','released-0.21.0','--rust-runtime-root',str(REPO/'runtime-rs'),'--rust-require-recording','--rust-standalone-proof-data',str(source),str(generated)]
  run('normal-generation',argv,compiler=True,timeout=300)
  source_trees[generated]=source_tree(generated)
  generated_manifest=generated/'contract/Cargo.toml';g=tomllib.loads(generated_manifest.read_text());assert g['lib']['path']=='lib.rs'
  capture=out/'capture-crate';capture.mkdir(mode=0o700)
  shutil.copyfile(HERE/'capture/common.rs',capture/'common.rs');shutil.copyfile(HERE/'capture'/('map.rs'if a.case=='map-member'else'tiny.rs'),capture/'main.rs')
  q=lambda x:json.dumps(str(x))
  # This is an EXPLICIT temporary native consumer, not a silently rewritten SDK
  # tool manifest. Bind materialized Cargo bytes in the run receipt below.
  text='[package]\nname="portable-proof-data-capture"\nversion="0.0.0"\nedition="2024"\nrust-version="1.99"\n[workspace]\n[[bin]]\nname="capture"\npath="main.rs"\n[dependencies]\n'
  text+='generated={package='+q(g['package']['name'])+',path='+q(generated/'contract')+'}\ncompact-rust-proof-data={path='+q(HERE)+'}\nanyhow="=1.0.103"\nserde_json="=1.0.149"\nmidnight-onchain-vm="=4.0.1"\nmidnight-serialize="=1.2.1"\nmidnight-transient-crypto="=3.1.0"\n'
  # Preserve exact normal-generator patch selectors, not another SDK graph.
  normal=generated_manifest.read_text();text+=normal[normal.index('[patch'):]
  text+='\n[profile.proof-checks]\ninherits="dev"\ndebug=0\n[profile.proof-checks.package."*"]\nopt-level=3\n'
  assert (capture/'common.rs').read_bytes()==(HERE/'capture/common.rs').read_bytes()and (capture/'main.rs').read_bytes()==(HERE/'capture'/('map.rs'if a.case=='map-member'else'tiny.rs')).read_bytes(),'exact capture overlay'
  (capture/'Cargo.toml').write_text(text);(capture/'Cargo.toml').chmod(0o600);shutil.copyfile(REPO/'Cargo.lock',capture/'Cargo.lock')
  run('capture-resolve',[cargo,'metadata','--offline','--format-version','1','--filter-platform',host,'--manifest-path',str(capture/'Cargo.toml')])
  metadata=run('capture-metadata',[cargo,'metadata','--locked','--offline','--format-version','1','--filter-platform',host,'--manifest-path',str(capture/'Cargo.toml')])
  # Finite temporary lock may add local capture/generated owners only; every
  # external name/version/source/checksum must remain the current root's.
  key=lambda p:(p['name'],p['version'],p.get('source'));anchor={key(p):p.get('checksum')for p in tomllib.loads((REPO/'Cargo.lock').read_text())['package']if p.get('source')}
  for p in tomllib.loads((capture/'Cargo.lock').read_text())['package']:
   if p.get('source'):assert key(p)in anchor and anchor[key(p)]==p.get('checksum')
  materialized.extend(ref(out,p)for p in [capture/'Cargo.toml',capture/'Cargo.lock',capture/'main.rs',capture/'common.rs'])
  source_trees[capture]=source_tree(capture)
  build=run('capture-build',[cargo,'build','--locked','--offline','--profile','proof-checks','--manifest-path',str(capture/'Cargo.toml'),'--bin','capture','--message-format=json-render-diagnostics','--jobs','2'])
  run('capture-gate',[sys.executable,str(HERE/'sdk_gate.py'),str(metadata),str(capture/'Cargo.lock'),str(capture/'Cargo.toml'),str(build),str(out/'capture-sdk-gate.json'),'--kind','capture'])
  messages=[json.loads(x)for x in build.read_text().splitlines()if x.startswith('{')];cexe=[m['executable']for m in messages if m.get('reason')=='compiler-artifact'and m['target']['name']=='capture'and m.get('executable')];assert len(cexe)==1
  source_files[Path(cexe[0]).resolve()]=fingerprint(Path(cexe[0]).resolve())
  captures=bundle/'capture';operation={'map-member':'member','tiny-clear':'clear','tiny-set':'set'}[a.case]
  run('capture',[cexe[0],operation,str(captures),a.binding_le,a.randomness_le],timeout=300)
  cases=json.loads((captures/'cases.json').read_text())
  for c in cases:
   for field in ['preimage','public_stream']:c[field]['path']='capture/'+c[field]['path']
  original_ir=generated/'zkir'/(operation+'.zkir');shutil.copyfile(original_ir,bundle/'original.zkir');(bundle/'original.zkir').chmod(0o600)
  copied=bundle/'generation';copied.mkdir(mode=0o700);files=[]
  for path in generated.rglob('*'):
   if not path.is_file():continue
   rel=path.relative_to(generated);dst=copied/rel;dst.parent.mkdir(mode=0o700,parents=True,exist_ok=True);shutil.copyfile(path,dst);dst.chmod(0o600);files.append(ref(bundle,dst))
  # A separate immutable check input has no parameter reference or body copy.
  # The real SDK model/check + explicit lane must succeed before touching SRS.
  generation=dict(compiler_version=version,source_revision='UNKNOWN_EXTERNAL_COMPILER_SOURCE_REVISION',files=files)
  manifest=dict(schema='compact-rust-proof-data-input/v1',profile=a.case,sdk=dict(ledger=data['ledger'],zk=data['zk'],zkir=data['zkir'],gate=ref(bundle,bundle/'sdk-gate.json')),raw_ir=ref(bundle,bundle/'original.zkir'),parent=None,binding_le=a.binding_le,randomness_le=a.randomness_le,generation=generation,cases=cases)
  flags=['--max-k',str(a.max_k)]+(['--allow-large-k']if a.allow_large_k else[])
  save(bundle/'check-input.json',manifest);check_input=ref(out,bundle/'check-input.json')
  run('check',[exe['produce'],'check',str(bundle/'check-input.json'),str(out/'check'),*flags],timeout=300)
  measured=json.loads((out/'check/model.json').read_text());assert measured['k']>0 and measured['k']<=a.max_k and(measured['k']<=8 or a.allow_large_k)and [c['id']for c in measured['cases']]==[c['id']for c in cases]and all(c['SDKCheckPassed']for c in measured['cases'])
  check_model=ref(out,out/'check/model.json')
  meta=json.loads(sdkmeta.read_text());tc=[p for p in meta['packages']if p['name']=='midnight-transient-crypto'and p['version']=='3.1.0'and p['source'].endswith('#'+data['ledger'])];assert len(tc)==1
  parent=Path(tc[0]['manifest_path']).parent/'static/bls_midnight_2p14';shutil.copyfile(parent,bundle/'parent14.raw');(bundle/'parent14.raw').chmod(0o600)
  manifest=dict(manifest,parent=ref(bundle,bundle/'parent14.raw'))
  save(bundle/'prove-input.json',manifest);admitted_input=ref(out,bundle/'prove-input.json')
  run('produce',[exe['produce'],'prove',str(bundle/'prove-input.json'),str(out/'proofs'),*flags],timeout=900)
  run('verify',[exe['verify'],str(bundle/'prove-input.json'),str(out/'proofs/produced.json'),str(out/'verification.json'),*flags],timeout=600)
 except BaseException as exc:first=exc
 finally:
  # Identity refusal authorizes neither postguard writes nor receipt creation
  # under the replacement. Cached first error survives without following it.
  if not original_owner(out,roots[out]):
   first=first or RuntimeError('Owner replaced during execution')
   print('OWNER_IDENTITY_REFUSAL_NO_CLOSEOUT_WRITE',file=sys.stderr)
  else:
   if data is not None:
    try:run('sdk-postgate',[sys.executable,str(HERE/'sdk_gate.py'),str(sdkmeta),str(REPO/'Cargo.lock'),str(tool),str(sdkbuild),str(out/'sdk-postgate.json'),'--kind','sdk'])
    except BaseException as exc:secondary.append('SDK source postguard: '+repr(exc));first=first or exc
   try:caps([],strict=True);sources()
   except BaseException as exc:secondary.append('final bounds/source: '+repr(exc));first=first or exc
   record=dict(schema='compact-rust-proof-data-run/v1',case=a.case,check_input=check_input,check_model=check_model,input_manifest=admitted_input,harness_git_revision=harness_revision,compiler_source_revision='UNKNOWN',compiler_image=compiler_image,bound_sources={str(k):v for k,v in source_files.items()},bound_source_trees={str(k):v for k,v in source_trees.items()},materialized=materialized,secondary_errors=secondary,error=repr(first)if first else None,rows=rows,seconds=time.monotonic()-start,owned_rebuildable_target=str(target),normal_generation_attempted=any(r.get('phase')=='normal-generation'for r in rows),qualified=False,limits='No Linux/releasedSDK1/chain/transaction/fullmatrix qualification')
   try:
    assert original_owner(out,roots[out]),'Owner replaced before receipt'
    save(out/'run.json',record)
   except BaseException as exc:print('SECONDARY_RUN_RECEIPT_FAILURE',file=sys.stderr);first=first or exc
 if first:raise first
 print(json.dumps(dict(complete=True,run=str(out))))
if __name__=='__main__':main()
