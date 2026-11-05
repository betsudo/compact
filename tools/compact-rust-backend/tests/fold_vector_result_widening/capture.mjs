// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Execute the genuine generated TS wrapper against an explicitly selected runtime.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {existsSync, readFileSync, readdirSync, realpathSync, statSync, writeFileSync} from 'node:fs';
import {createRequire} from 'node:module';
import {dirname, resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {parseArgs} from 'node:util';

const {values:options}=parseArgs({options:Object.fromEntries(
  ['generated','runtime','version','output'].map(name=>[name,{type:'string'}]))});
for (const name of ['generated','runtime','version','output']) assert.ok(options[name],`missing --${name}`);
const hash=path=>createHash('sha256').update(readFileSync(path)).digest('hex');
function inventory(folder) {
  const files=new Set(), packages=new Set();
  function walk(path) {
    for(const entry of readdirSync(path,{withFileTypes:true})) {
      if(['node_modules','.git'].includes(entry.name)) continue;
      const child=resolve(path,entry.name);
      if(entry.isDirectory()) walk(child);
      else if(statSync(child).isFile()) files.add(realpathSync(child));
    }
  }
  function visit(path) {
    path=realpathSync(path);
    if(packages.has(path)) return;
    packages.add(path); walk(path);
    const manifest=resolve(path,'package.json'), req=createRequire(manifest);
    for(const name of Object.keys(JSON.parse(readFileSync(manifest,'utf8')).dependencies??{})) {
      const candidate=req.resolve.paths(name).map(p=>resolve(p,name,'package.json')).find(existsSync);
      assert.ok(candidate,`missing runtime dependency ${name}`); visit(dirname(candidate));
    }
  }
  visit(folder);
  return Object.fromEntries([...files].sort().map(path=>[path,hash(path)]));
}
const definitions=JSON.parse(readFileSync(new URL('./sources.json',import.meta.url)));
const specs=JSON.parse(readFileSync(new URL('./cases.json',import.meta.url)));
const legacy=JSON.parse(readFileSync(new URL('./legacy-order-baseline.json',import.meta.url)));
assert.deepEqual(specs.filter(row=>row.nativeError).map(row=>row.id).sort(),legacy.case_ids.slice().sort());
for(const row of specs.filter(row=>row.nativeError)){
 assert.equal(row.entry,'value');assert.equal(row.args[0],false);assert.equal(row.args[1],false);
 assert.deepEqual(row.error,{kind:'assert',message:legacy.typescript_error});
 assert.deepEqual(row.nativeError,{kind:'assert',message:legacy.native_error});
 assert.ok(['source_checked','source_checked_control'].includes(row.source));
}
const runtime=realpathSync(options.runtime), wrappers={};
assert.equal(JSON.parse(readFileSync(resolve(runtime,'package.json'),'utf8')).version,options.version);
for(const {source} of definitions) {
  const path=realpathSync(resolve(options.generated,source,'contract/index.js'));
  assert.equal(realpathSync(createRequire(path).resolve('@midnight-ntwrk/compact-runtime')),
    realpathSync(resolve(runtime,'dist/index.js')));
  wrappers[source]={path,sha256:hash(path)};
}
const before=inventory(runtime), selfHash=hash(process.argv[1]);
const inputs=Object.fromEntries(['sources.json','cases.json','legacy-order-baseline.json'].map(name=>{
  const path=realpathSync(new URL(name,import.meta.url));return [path,hash(path)];
}));
const nodeIdentity={path:realpathSync(process.execPath),version:process.version,sha256:hash(process.execPath)};
const r=await import(pathToFileURL(resolve(runtime,'dist/index.js')));
const key={bytes:new Uint8Array(32)},rows=[],modules=new Map();
const encode=({value,alignment})=>({valueAtoms:value.map(x=>Array.from(x)),alignment});
const json=x=>JSON.parse(JSON.stringify(x,(_,v)=>typeof v==='bigint'?String(v):v instanceof Uint8Array?Array.from(v):v instanceof Map?Array.from(v.entries()):v));
const convert=x=>Array.isArray(x)?x.map(convert):typeof x==='string'?BigInt(x):x;
const uint=(max,width)=>new r.CompactTypeUnsignedInteger(max,width);
const u16=uint(65535n,2), u32=uint(4294967295n,4), vector=(n,t)=>new r.CompactTypeVector(n,t);
const descriptors={
 field:vector(2,r.CompactTypeField),wide:vector(2,uint((1n<<129n)-1n,17)),
 wide_to_wide:vector(2,uint((1n<<248n)-1n,31)),nested:vector(2,vector(2,u16)),
 product:vector(2,{alignment:()=>[...u16.alignment(),...u32.alignment()],toValue:v=>[...u16.toValue(v[0]),...u32.toValue(v[1])]}),
 alias_bool:vector(2,r.CompactTypeBoolean),scalar_control:u16
};
for(const spec of specs){
 const source=spec.source,path=pathToFileURL(wrappers[source].path);
 if(!modules.has(source))modules.set(source,await import(path));const m=modules.get(source);
 let out,value,error;
 if(source==='helper'){
  const c=new m.Contract({}),init=c.initialState({initialPrivateState:7,initialZswapLocalState:r.emptyZswapLocalState(key)});
  const ctx=r.createCircuitContext(r.dummyContractAddress(),key,init.currentContractState.data,7,undefined,undefined,0);
  try{out=c.circuits[spec.entry](ctx,...spec.args.map(convert));value=out.result;}catch(e){error=e;}
 }else{try{value=m.pureCircuits[spec.entry](...spec.args.map(convert));}catch(e){error=e;}}
 if(spec.error){
  assert.equal(value,undefined,spec.id);
  const suffix=spec.error.kind==='unsigned'?`cast from Field or Uint value to smaller Uint value failed: ${spec.error.value} is greater than ${spec.error.max}`:spec.error.message;
  assert.ok(error?.message.includes(suffix),spec.id+': '+error);
  rows.push({...spec,observedError:error.message});continue;
 }
 assert.ifError(error);assert.deepEqual(value,convert(spec.expected),spec.id);
 const descriptor=descriptors[source]??vector(2,u16);
 const encoded=encode({value:descriptor.toValue(value),alignment:descriptor.alignment()});
 if(source==='helper'){
  assert.equal(m.ledger(out.context.currentQueryContext.state).seen,BigInt(spec.seen),spec.id);
  assert.equal(out.context.currentPrivateState,7,spec.id);assert.deepEqual(out.proofData.privateTranscriptOutputs,[],spec.id);
  assert.deepEqual(encode(out.proofData.output),encoded,spec.id);
 }
 rows.push({...spec,result:encoded,privateState:source==='helper'?7:null,privateOutputs:[]});
}
assert.deepEqual(inventory(runtime),before,'runtime production closure changed');
for(const {path,sha256} of Object.values(wrappers)) assert.equal(hash(path),sha256);
for(const [path,sha256] of Object.entries(inputs)) assert.equal(hash(path),sha256);assert.equal(hash(process.argv[1]),selfHash);
assert.equal(realpathSync(process.execPath),nodeIdentity.path);assert.equal(hash(process.execPath),nodeIdentity.sha256);
const sharedObjects=process.report.getReport().sharedObjects;
const system=sharedObjects.filter(path=>path.startsWith('/System/Library/')||path.startsWith('/usr/lib/'));
const kernelNames=new Set(['linux-vdso.so.1','linux-gate.so.1']);
const kernelVirtualLibraries=[...new Set(sharedObjects.filter(path=>kernelNames.has(path)))];
const external=sharedObjects.filter(path=>!system.includes(path)&&!kernelNames.has(path));
assert.ok(external.length);
writeFileSync(options.output,JSON.stringify({status:'passed',format:'fold-vector-result-widening-ts/v1',
  runtime:{path:runtime,version:options.version,files:before},
  node:{...nodeIdentity,sharedLibraries:Object.fromEntries(external.map(path=>[path,hash(path)])),
    systemLibraries:system,kernelVirtualLibraries,systemScope:'host OS paths and kernel mapping names retained; their bytes not qualified'},
  wrappers,inputs,capture:{path:realpathSync(process.argv[1]),sha256:selfHash},
  scope:'actual complete wrappers and independent expected values, accumulator failures/state/private checks; declared historical native/TS operand-order differences are not parity claims; no proof or Linux claim',rows},null,2)+'\n',{flag:'wx'});
console.log(JSON.stringify({cases:rows.length,success:rows.filter(row=>!row.error).length,failures:rows.filter(row=>row.error).length,output:options.output}));
