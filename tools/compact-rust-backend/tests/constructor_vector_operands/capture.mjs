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

import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {existsSync,readFileSync,readdirSync,realpathSync,statSync,writeFileSync} from 'node:fs';
import {createRequire} from 'node:module';
import {dirname,resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {parseArgs} from 'node:util';
const {values:o}=parseArgs({options:Object.fromEntries(['generated','runtime','version','output'].map(name=>[name,{type:'string'}]))});
for(const key of ['generated','runtime','version','output'])assert.ok(o[key],`missing --${key}`);
const hash=p=>createHash('sha256').update(readFileSync(p)).digest('hex');
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
const cases=JSON.parse(readFileSync(new URL('./cases.json',import.meta.url)));
const inputs=Object.fromEntries(['sources.json','cases.json'].map(name=>{const p=realpathSync(new URL(name,import.meta.url));return[p,hash(p)];}));
const runtime=realpathSync(o.runtime),before=inventory(runtime),selfHash=hash(process.argv[1]);
assert.equal(JSON.parse(readFileSync(resolve(runtime,'package.json'))).version,o.version);
const node={path:realpathSync(process.execPath),version:process.version,sha256:hash(process.execPath)};
const r=await import(pathToFileURL(resolve(runtime,'dist/index.js')));
const encode=value=>({valueAtoms:value.value.map(x=>Array.from(x)),alignment:value.alignment});
const field=n=>encode({value:r.CompactTypeField.toValue(BigInt(n)),alignment:r.CompactTypeField.alignment()});
const json=x=>JSON.parse(JSON.stringify(x,(_,v)=>typeof v==='bigint'?String(v):v instanceof Uint8Array?Array.from(v):v));
const key={bytes:new Uint8Array(32)},wrappers={},modules=new Map(),rows=[];
for(const {role} of definitions){
 const path=realpathSync(resolve(o.generated,role,'contract/index.js'));
 assert.equal(realpathSync(createRequire(path).resolve('@midnight-ntwrk/compact-runtime')),realpathSync(resolve(runtime,'dist/index.js')));
 wrappers[role]={path,sha256:hash(path)};modules.set(role,await import(pathToFileURL(path)));
}
for(const spec of cases){
 const m=modules.get(spec.role),observed=[];
 const callback=(name,value)=>({privateState,ledger})=>{
  observed.push({name,privateState,seen:field(ledger.stored)});
  if(spec.failWitness===name)throw new Error('constructor-witness-stop');
  return [privateState+1,value];
 };
 const callbacks=spec.role==='conditional_witness'?{first:callback('first',[2n,3n]),second:callback('second',[7n,8n])}:
  spec.role==='checked_helper'?{after:callback('after',3n)}:
  ['local_witness','direct_witness','empty_witness'].includes(spec.role)?{items:callback('items',spec.role==='empty_witness'?[]:[2n,3n])}:{};
 const c=new m.Contract(callbacks);
 const input=[2n,3n];
 const args=spec.role==='conditional'?[spec.flag,input,[7n,8n]]:
  spec.role==='nested_capture'?[spec.flag,input]:
  spec.role==='conditional_witness'?[spec.flag]:
  spec.role==='checked_helper'?[input,spec.helperOk]:
  spec.role==='owned_reuse'?[['first','β🚀']]:
  spec.role==='empty_local'?[[]]:
  ['local_literal','local_witness','direct_witness','empty_witness'].includes(spec.role)?[]:[input];
 let init,error;
 try{init=c.initialState({initialPrivateState:5,initialZswapLocalState:r.emptyZswapLocalState(key)},...args);}catch(e){error=e;}
 assert.deepEqual(observed,spec.trace.map(t=>({...t,seen:field(t.seen)})),spec.id+' witness prefix');
 if(spec.error!==null){
  assert.equal(init,undefined,spec.id);assert.ok(error?.message.includes(spec.error),spec.id+': '+error);
  rows.push({...spec,observedError:error.message,observedWitnesses:observed});continue;
 }
 assert.ifError(error);
 const stored=m.ledger(init.currentContractState.data).stored;
 assert.equal(stored,BigInt(spec.expected),spec.id);assert.equal(init.currentPrivateState,spec.privateState,spec.id);
 if(spec.role==='owned_reuse'){
  assert.equal(m.ledger(init.currentContractState.data).temporary,'β🚀');
  assert.equal(m.ledger(init.currentContractState.data).reused,'β🚀');
 }
 const context=r.createCircuitContext(r.dummyContractAddress(),key,init.currentContractState.data,init.currentPrivateState,undefined,undefined,0);
 const read=c.circuits.read(context);assert.equal(read.result,stored);
 const out=c.circuits.advance(read.context,4n);assert.equal(out.result,stored+4n,spec.id);
 assert.equal(m.ledger(out.context.currentQueryContext.state).stored,stored+4n,spec.id);
 assert.equal(out.context.currentPrivateState,spec.privateState,spec.id);assert.deepEqual(out.proofData.privateTranscriptOutputs,[]);
 assert.deepEqual(encode(out.proofData.output),field(stored+4n));
 rows.push({...spec,stored:String(stored),afterAdvance:String(out.result),observedWitnesses:observed,publicTranscript:json(out.proofData.publicTranscript)});
}
assert.equal(rows.length,cases.length);assert.deepEqual(inventory(runtime),before);
for(const {path,sha256} of Object.values(wrappers))assert.equal(hash(path),sha256);
for(const [path,sha256] of Object.entries(inputs))assert.equal(hash(path),sha256);
assert.equal(hash(process.argv[1]),selfHash);assert.equal(hash(process.execPath),node.sha256);
const shared=process.report.getReport().sharedObjects;
const system=shared.filter(p=>p.startsWith('/System/Library/')||p.startsWith('/usr/lib/'));
const virtual=shared.filter(p=>['linux-vdso.so.1','linux-gate.so.1'].includes(p));
node.sharedLibraries=Object.fromEntries(shared.filter(p=>!system.includes(p)&&!virtual.includes(p)).map(p=>[p,hash(p)]));
node.systemLibraries=system;node.kernelVirtualLibraries=virtual;
writeFileSync(o.output,JSON.stringify({status:'passed',format:'constructor-vector-operands-ts/v1',
 scope:'Actual complete TS constructors, ordered witness/error prefixes, state/private/owned checks and later read/advance. No constructor proof or constructor total-cost parity.',
 runtime:{path:runtime,version:o.version,files:before},node,wrappers,inputs,capture:{path:realpathSync(process.argv[1]),sha256:selfHash},rows},null,2)+'\n',{flag:'wx'});
console.log(JSON.stringify({cases:rows.length,success:rows.filter(x=>x.error===null).length,errors:rows.filter(x=>x.error!==null).length}));
