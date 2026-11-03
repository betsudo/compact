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
const runtime=realpathSync(options.runtime), wrappers={};
assert.equal(JSON.parse(readFileSync(resolve(runtime,'package.json'),'utf8')).version,options.version);
for(const {source} of definitions) {
  const path=realpathSync(resolve(options.generated,source,'contract/index.js'));
  assert.equal(realpathSync(createRequire(path).resolve('@midnight-ntwrk/compact-runtime')),
    realpathSync(resolve(runtime,'dist/index.js')));
  wrappers[source]={path,sha256:hash(path)};
}
const before=inventory(runtime), selfHash=hash(process.argv[1]);
const inputs=Object.fromEntries(['sources.json','cases.json'].map(name=>{
  const path=realpathSync(new URL(name,import.meta.url));return [path,hash(path)];
}));
const nodeIdentity={path:realpathSync(process.execPath),version:process.version,sha256:hash(process.execPath)};
const r=await import(pathToFileURL(resolve(runtime,'dist/index.js')));
const key={bytes:new Uint8Array(32)},rows=[],modules=new Map();
const encode=({value,alignment})=>({valueAtoms:value.map(x=>Array.from(x)),alignment});
const json=x=>JSON.parse(JSON.stringify(x,(_,v)=>typeof v==='bigint'?String(v):v instanceof Uint8Array?Array.from(v):v instanceof Map?Array.from(v.entries()):v));
const convert=x=>Array.isArray(x)?x.map(convert):typeof x==='string'?BigInt(x):
 x!==null&&typeof x==='object'?Object.fromEntries(Object.entries(x).map(([k,v])=>[k,convert(v)])):x;
const uint=(max,width)=>new r.CompactTypeUnsignedInteger(max,width);
const u16=uint(65535n,2), u32=uint(4294967295n,4);
const vector=(n,t)=>{
 const type=new r.CompactTypeVector(n,t);
 return {alignment:()=>type.alignment(),toValue:v=>{
  assert.ok(Array.isArray(v),'declared Vector must be an array');
  assert.equal(v.length,n,'declared Vector dimension differs');return type.toValue(v);
 }};
};
const tuple=(types)=>({alignment:()=>types.flatMap(t=>t.alignment()),toValue:v=>{
 assert.ok(Array.isArray(v),'declared tuple must be an array');
 assert.equal(v.length,types.length,'declared tuple arity differs');
 return types.flatMap((t,i)=>t.toValue(v[i]));
}});
const u8=uint(255n,1);
const narrowProduct=tuple([vector(1,tuple([vector(2,u8),u8])),u16]);
const wideProduct=tuple([vector(1,tuple([vector(2,u16),u16])),u32]);
const deepNarrow=vector(1,tuple([vector(1,tuple([tuple([vector(1,tuple([vector(2,u8),u16])),u8]),u16])),u8]));
const deepWide=vector(1,tuple([vector(1,tuple([tuple([vector(1,tuple([vector(2,u16),u32])),u16]),u32])),u16]));
const descriptors={narrow:vector(1,narrowProduct),wide:vector(1,wideProduct),
 original_direct:vector(2,tuple([vector(2,tuple([vector(2,u16),u16])),u16])),
 original_nested:vector(1,tuple([vector(1,tuple([tuple([vector(1,u16)]),u16])),u16])),
 deep:deepWide,
 boolean_field:vector(1,tuple([vector(1,tuple([vector(2,r.CompactTypeBoolean),r.CompactTypeField])),r.CompactTypeBoolean])),
 field_identity:vector(1,tuple([vector(1,tuple([vector(2,r.CompactTypeField),u16])),u32])),
 empty:vector(0,wideProduct),
 tuple_empty:vector(1,tuple([vector(1,tuple([vector(2,tuple([])),u16])),u32])),
 tuple_single:vector(1,tuple([vector(1,tuple([tuple([vector(1,u16)]),u32])),u32])),
 empty_leaf:vector(1,tuple([vector(1,tuple([vector(0,u16),u16])),u32]))};
const inputDescriptors={
 original_direct:{run:[vector(2,tuple([vector(2,tuple([vector(2,u8),u8])),u8]))]},
 original_nested:{run:[vector(1,tuple([vector(1,tuple([tuple([vector(1,u8)]),u8])),u8]))]},
 family:{parameter:[descriptors.narrow],helper:[descriptors.narrow],local:[descriptors.narrow],
 checked_helper:[r.CompactTypeBoolean,descriptors.wide],witness_before:[],witness_after:[],
 branch:[r.CompactTypeBoolean,descriptors.narrow,descriptors.narrow],deep:[deepNarrow],
 boolean_field:[vector(1,tuple([vector(1,tuple([vector(2,r.CompactTypeBoolean),u8])),r.CompactTypeBoolean]))],
 field_identity:[vector(1,tuple([vector(1,tuple([vector(2,r.CompactTypeField),u8])),u16]))],
 empty:[vector(0,narrowProduct)],
 tuple_empty:[vector(1,tuple([vector(1,tuple([vector(2,tuple([])),u8])),u16]))],
 tuple_single:[vector(1,tuple([vector(1,tuple([tuple([vector(1,u8)]),u16])),u16]))],
 empty_leaf:[vector(1,tuple([vector(1,tuple([vector(0,u8),u8])),u16]))]}};
const bySource=new Map(definitions.map(row=>[row.source,row]));
for(const spec of specs){
 const source=spec.source,path=pathToFileURL(wrappers[source].path);
 if(!modules.has(source))modules.set(source,await import(path));const m=modules.get(source);
 const args=spec.args.map(convert), inputTypes=inputDescriptors[source]?.[spec.entry];
 assert.ok(inputTypes,`missing input descriptor ${source}.${spec.entry}`);assert.equal(inputTypes.length,args.length);
 const argumentValues=args.map((value,i)=>encode({value:inputTypes[i].toValue(value),alignment:inputTypes[i].alignment()}));
 let out,value,error;const observed=[];const stateful=bySource.get(source).stateful;
 if(stateful){
  const c=new m.Contract(source==='family'?{vector_value:({privateState,ledger})=>{
    const seen=ledger.seen;assert.equal(seen,BigInt(spec.witnessSeen),'initializer witness order');
    if(spec.error?.kind==='witness'){
      observed.push({privateState,seen:encode({value:r.CompactTypeField.toValue(seen),alignment:r.CompactTypeField.alignment()})});
      throw new Error(spec.error.message);
    }
    const component=[[BigInt(privateState-4),BigInt(privateState+121)],BigInt(privateState+248)];
    const value=[[[component],BigInt(privateState+65528)]];
    observed.push({privateState,value:json(value),seen:encode({value:r.CompactTypeField.toValue(seen),alignment:r.CompactTypeField.alignment()})});
    return [privateState+1,value];
  }}:{});
  const init=c.initialState({initialPrivateState:7,initialZswapLocalState:r.emptyZswapLocalState(key)},...spec.constructor.map(convert));
  const ctx=r.createCircuitContext(r.dummyContractAddress(),key,init.currentContractState.data,7,undefined,undefined,0);
  try{out=c.circuits[spec.entry](ctx,...args);value=out.result;}catch(e){error=e;}
 }else{try{value=m.pureCircuits[spec.entry](...args);}catch(e){error=e;}}
 if(spec.error){
  assert.ok(error instanceof Error,spec.id+': must return an actual error');
  assert.equal(out,undefined,spec.id+': error must not return execution/proof data');
  const expected=spec.error.kind==='unsigned'?`cast from Field or Uint value to smaller Uint value failed: ${spec.error.value} is greater than ${spec.error.max}`:spec.error.message;
  if(spec.error.kind==='witness'){
    assert.equal(error.message,expected,spec.id);
    assert.deepEqual(observed,[{privateState:7,seen:encode({value:r.CompactTypeField.toValue(BigInt(spec.witnessSeen)),alignment:r.CompactTypeField.alignment()})}],spec.id);
  }else{
    assert.ok(error instanceof r.CompactError,spec.id+': expected runtime CompactError');
    assert.equal(error.message,spec.error.tsMessage,spec.id);
    assert.deepEqual(observed,[],spec.id);
  }
  rows.push({...spec,argumentValues,observedError:error.message,observedWitnesses:observed});continue;
 }
 assert.ifError(error);assert.deepEqual(value,convert(spec.expected),spec.id);
 const descriptorSpec=bySource.get(source).descriptor;
 const name=typeof descriptorSpec==='string'?descriptorSpec:descriptorSpec[spec.entry];
 const descriptor=descriptors[name];assert.ok(descriptor,`missing explicit descriptor ${source}.${spec.entry}`);
 const encoded=encode({value:descriptor.toValue(value),alignment:descriptor.alignment()});
 if(stateful){
  assert.equal(m.ledger(out.context.currentQueryContext.state).seen,BigInt(spec.seen),spec.id);
  assert.equal(out.context.currentPrivateState,spec.privateState??7,spec.id);
  const privateDescriptor=descriptors.narrow;
  const privateValues=(spec.witnessValues??[]).map(value=>encode({value:privateDescriptor.toValue(convert(value)),alignment:privateDescriptor.alignment()}));
  assert.deepEqual(out.proofData.privateTranscriptOutputs.map(encode),privateValues,spec.id);
  assert.deepEqual(observed,(spec.witnessValues??[]).map((value,i)=>({privateState:7+i,value,seen:encode({value:r.CompactTypeField.toValue(BigInt(spec.witnessSeen)),alignment:r.CompactTypeField.alignment()})})),spec.id);
  assert.deepEqual(encode(out.proofData.output),encoded,spec.id);
 }
 rows.push({...spec,argumentValues,result:encoded,privateState:stateful?(spec.privateState??7):null,privateOutputs:stateful?out.proofData.privateTranscriptOutputs.map(encode):[],publicTranscript:stateful?json(out.proofData.publicTranscript):null,observedWitnesses:observed});
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
writeFileSync(options.output,JSON.stringify({status:'passed',format:'component-product-vector-widening-ts/v1',
  runtime:{path:runtime,version:options.version,files:before},
  node:{...nodeIdentity,sharedLibraries:Object.fromEntries(external.map(path=>[path,hash(path)])),
    systemLibraries:system,kernelVirtualLibraries,systemScope:'host OS paths and kernel mapping names retained; their bytes not qualified'},
  wrappers,inputs,capture:{path:realpathSync(process.argv[1]),sha256:selfHash},
  scope:'actual complete wrappers and independent expected values, public transcript equality inputs and state/private checks; actual ordered assertion/cast/witness errors; no native error substitutions, proof or Linux claim',rows},null,2)+'\n',{flag:'wx'});
console.log(JSON.stringify({cases:rows.length,success:rows.filter(row=>!row.error).length,failures:rows.filter(row=>row.error).length,output:options.output}));
