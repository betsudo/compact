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
const wrapper=realpathSync(options.generated), runtime=realpathSync(options.runtime);
assert.equal(JSON.parse(readFileSync(resolve(runtime,'package.json'),'utf8')).version,options.version);
const runtimePath=createRequire(wrapper).resolve('@midnight-ntwrk/compact-runtime');
assert.equal(realpathSync(runtimePath),realpathSync(resolve(runtime,'dist/index.js')));
const before=inventory(runtime), wrapperHash=hash(wrapper), selfHash=hash(process.argv[1]);
assert.equal(before[realpathSync(runtimePath)],hash(runtimePath),'runtime entry omitted from closure inventory');
const nodeIdentity={path:realpathSync(process.execPath),version:process.version,sha256:hash(process.execPath)};
const r=await import(pathToFileURL(runtimePath)), c=await import(pathToFileURL(wrapper));
const json=value=>JSON.parse(JSON.stringify(value,(_,v)=>typeof v==='bigint'?String(v):
  v instanceof Uint8Array?Array.from(v):v instanceof Map?Array.from(v.entries()):v));
const fab=({value,alignment})=>({valueAtoms:value.map(atom=>Array.from(atom)),alignment:json(alignment)});
const fields=[17n,41n], texts=['α\0first','β\0last'];
const pure=[];
for(const [name,input,expected] of [
  ['first',fields,17n],['last',fields,41n],['singleton',[99n],99n],
  ['first_text',texts,texts[0]],['temporary_text',texts,texts[1]],
]) {
  const result=c.pureCircuits[name](input); assert.deepEqual(result,expected,name);
  const type=typeof result==='bigint'?r.CompactTypeField:r.CompactTypeOpaqueString;
  pure.push({id:name,result:json(result),output:fab({value:type.toValue(result),alignment:type.alignment()})});
}
const key={bytes:new Uint8Array(32)}, cases=[], refusals=[];
// The TS context exposes the last query's cost. Rust reports the sum of all
// executed queries, including witness-view reads. Retain both observations;
// compute the sum from actual query calls, without subtracting or estimating.
let active;
const query=r.QueryContext.prototype.query;
r.QueryContext.prototype.query=function(...args) {
  const out=query.apply(this,args);
  if(active) active.push({program:json(args[0]),gasCost:json(out.gasCost)});
  return out;
};
try {
for(const [id,name,args,expected,stored,witnesses,fail] of [
  ['choose-last','choose',[fields,false],41n,41n,0,false],
  ['choose-first','choose',[fields,true],17n,17n,0,false],
  ['owned','owned',[texts],{first:texts[0],repeated:texts[0],original:texts},1n,0,false],
  ['witnessed-last','witnessed',[false],41n,41n,1,false],
  ['witnessed-first','witnessed',[true],17n,17n,1,false],
  ['temporary','temporary',[],41n,41n,1,false],
  ['conditional-inactive','conditional',[false],0n,0n,0,true],
  ['conditional-failure','conditional',[true],undefined,undefined,1,true],
]) {
  const events=[];
  const contract=new c.Contract({values:ctx=>{
    events.push([ctx.privateState,String(ctx.ledger.stored)]);
    if(fail) throw new Error('vector witness refused');
    return [ctx.privateState+1,fields];
  }});
  const initial=contract.initialState({initialPrivateState:7,initialZswapLocalState:r.emptyZswapLocalState(key)});
  const context=r.createCircuitContext(r.dummyContractAddress(),key,initial.currentContractState.data,7,
    undefined,undefined,0);
  if(id==='conditional-failure') {
    active=[];
    assert.throws(()=>contract.circuits[name](context,...args),{message:'vector witness refused'});
    const queries=active; active=undefined;
    assert.deepEqual(events,[[7,'0']]);
    refusals.push({id,error:'vector witness refused',events,queries}); continue;
  }
  active=[];
  const out=contract.circuits[name](context,...args);
  const queries=active; active=undefined;
  assert.deepEqual(out.result,expected,id);
  assert.equal(c.ledger(out.context.currentQueryContext.state).stored,stored,id);
  assert.equal(out.context.currentPrivateState,7+witnesses,id);
  assert.deepEqual(events,witnesses?[[7,'0']]:[],id);
  const privateOutputs=out.proofData.privateTranscriptOutputs.map(fab);
  assert.deepEqual(privateOutputs,witnesses?[fab({value:r.CompactTypeField.toValue(17n).concat(
    r.CompactTypeField.toValue(41n)),alignment:new r.CompactTypeVector(2,r.CompactTypeField).alignment()})]:[],id);
  cases.push({id,result:json(out.result),stored:String(stored),storedFab:fab({value:r.CompactTypeField.toValue(stored),
    alignment:r.CompactTypeField.alignment()}),privateState:out.context.currentPrivateState,
    events,privateOutputs,lastQueryGasCost:json(out.gasCost),queries,
    gasCost:Object.fromEntries(['readTime','computeTime','bytesWritten','bytesDeleted'].map(k=>[
      k,queries.reduce((sum,q)=>sum+BigInt(q.gasCost[k]),0n).toString()])),
    effects:json(out.context.currentQueryContext.effects),
    publicTranscript:json(out.proofData.publicTranscript),input:fab(out.proofData.input),output:fab(out.proofData.output)});
}
} finally {r.QueryContext.prototype.query=query;}
for(const [id,input] of [['wrong-length',[17n]],['wrong-element',[17n,'41']]]) {
  assert.throws(()=>c.pureCircuits.first(input)); refusals.push({id});
}
assert.deepEqual(inventory(runtime),before,'runtime production closure changed');
assert.equal(hash(wrapper),wrapperHash,'generated wrapper changed');
assert.equal(hash(process.argv[1]),selfHash,'capture source changed');
assert.equal(realpathSync(process.execPath),nodeIdentity.path,'Node executable path changed');
assert.equal(hash(process.execPath),nodeIdentity.sha256,'Node executable bytes changed');
const sharedObjects=process.report.getReport().sharedObjects;
const system=sharedObjects.filter(path=>path.startsWith('/System/Library/')||path.startsWith('/usr/lib/'));
// Linux reports these kernel mappings by SONAME, with no backing disk file.
// Keep their names explicit; every other non-system entry must still hash.
const kernelNames=new Set(['linux-vdso.so.1','linux-gate.so.1']);
const kernelVirtualLibraries=[...new Set(sharedObjects.filter(path=>kernelNames.has(path)))];
const external=sharedObjects.filter(path=>!system.includes(path)&&!kernelNames.has(path));
assert.ok(external.length,'no loaded Node libraries recorded');
writeFileSync(options.output,JSON.stringify({format:'compact-vector-projection-ts/v1',
  runtime:{path:runtime,version:options.version,files:before},
  node:{...nodeIdentity,
    sharedLibraries:Object.fromEntries(external.map(path=>[path,hash(path)])),
    systemLibraries:system,kernelVirtualLibraries,
    systemScope:'host OS paths and kernel virtual-library names retained; their bytes not qualified'},
  wrapper:{path:wrapper,sha256:wrapperHash},capture:{path:realpathSync(process.argv[1]),sha256:selfHash},
  pure,cases,refusals},null,2)+'\n',{flag:'wx'});
console.log(JSON.stringify({pure:pure.length,cases:cases.length,refusals:refusals.length,output:options.output}));
