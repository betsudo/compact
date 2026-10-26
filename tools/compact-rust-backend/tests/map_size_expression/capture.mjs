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
const nodeIdentity={path:realpathSync(process.execPath),version:process.version,sha256:hash(process.execPath)};
const r=await import(pathToFileURL(runtimePath)), c=await import(pathToFileURL(wrapper));
const json=value=>JSON.parse(JSON.stringify(value,(_,v)=>typeof v==='bigint'?String(v):
  v instanceof Uint8Array?Array.from(v):v instanceof Map?Array.from(v.entries()):v));
const fab=({value,alignment})=>({valueAtoms:value.map(atom=>Array.from(atom)),alignment:json(alignment)});
const field=value=>({value:r.CompactTypeField.toValue(value),alignment:r.CompactTypeField.alignment()});
const key={bytes:new Uint8Array(32)},cases=[],refusals=[],boundaries=[];
let active;
const query=r.QueryContext.prototype.query;
r.QueryContext.prototype.query=function(...args) {
  const row={program:json(args[0])}; if(active) active.push(row);
  try {const out=query.apply(this,args);row.gasCost=json(out.gasCost);return out;}
  catch(error) {row.failure=String(error);throw error;}
};
function context(contract,mode) {
  const initial=contract.initialState({initialPrivateState:7,initialZswapLocalState:r.emptyZswapLocalState(key)});
  let state=initial.currentContractState.data;
  if(mode!=='seed') {
    let array=r.StateValue.newArray();
    if(mode!=='missing') {
      const table=mode==='wrong-kind'?r.StateValue.newCell(field(9n)):
        mode==='empty'?r.StateValue.newMap(new r.StateMap()):state.state.asArray()[0];
      let outer=new r.StateMap();
      if(mode==='nested') for(const k of [3n,4n]) outer=outer.insert(field(k),r.StateValue.newMap(new r.StateMap()));
      array=array.arrayPush(table).arrayPush(r.StateValue.newMap(outer));
    }
    state=new r.ChargedState(array);
  }
  return r.createCircuitContext(r.dummyContractAddress(),key,state,7,undefined,undefined,0);
}
function invoke(id,name,args,expected,{mode='seed',fail=false,boundary=false,refusal=false}={}) {
  const events=[];
  const contract=new c.Contract({offset:ctx=>{
    events.push([ctx.privateState,String(ctx.ledger.table.size())]);
    if(fail) throw new Error('map size witness refused');
    return [ctx.privateState+1,2n];
  }});
  const input=context(contract,mode);active=[];
  if(refusal) {
    let error;
    try {contract.circuits[name](input,...args);} catch(e) {error=e;}
    const queries=active;active=undefined;
    assert.ok(error,`${id} must refuse`);
    if(fail&&mode==='seed') assert.equal(error.message,'map size witness refused');
    if(mode==='missing'||mode==='wrong-kind') {assert.deepEqual(events,[]);assert.equal(queries.length,1);assert.ok(queries[0].failure);}
    refusals.push({id,error:error.message,events,queries});return;
  }
  const out=contract.circuits[name](input,...args),queries=active;active=undefined;
  assert.deepEqual(out.result,expected,id);
  const row={id,result:json(out.result),output:fab(out.proofData.output),
    privateState:out.context.currentPrivateState,events,
    privateOutputs:out.proofData.privateTranscriptOutputs.map(fab),queries,
    lastQueryGasCost:json(out.gasCost??null),
    gasCost:Object.fromEntries(['readTime','computeTime','bytesWritten','bytesDeleted'].map(k=>[
      k,queries.reduce((sum,q)=>sum+BigInt(q.gasCost[k]),0n).toString()])),
    effects:json(out.context.currentQueryContext.effects),publicTranscript:json(out.proofData.publicTranscript)};
  if(boundary) {assert.equal(queries.length,0);boundaries.push(row);return;}
  const ledger=c.ledger(out.context.currentQueryContext.state);
  row.table=[...ledger.table].sort((a,b)=>a[0]<b[0]?-1:a[0]>b[0]?1:0)
    .map(([k,v])=>({key:fab(field(k)),value:fab(field(v))}));
  row.tableSize=String(ledger.table.size());row.outerSize=String(ledger.outer.size());
  cases.push(row);
}
try {
  for(const [mode,size,outer] of [['seed',1n,0n],['empty',0n,0n],['nested',1n,2n]]) {
    invoke(`direct-${mode}`,'direct',[],size,{mode});
    invoke(`empty-${mode}`,'empty',[],size===0n,{mode});
    invoke(`plus_one-${mode}`,'plus_one',[],size+1n,{mode});
    invoke(`snapshot-${mode}`,'snapshot',[],{before:size,after:size,outer},{mode});
  }
  for(const k of [1n,2n]) invoke(`put-${k}`,'put',[k,99n],{before:1n,after:k,outer:0n});
  for(const active of [false,true]) invoke(`choose-${active}`,'choose',[active],active?1n:7n);
  invoke('ordered','ordered',[],3n);
  for(const active of [false,true]) invoke(`witnessed-${active}`,'witnessed',[active],active?2n:1n,{fail:!active});
  for(const name of ['ordered','witnessed']) invoke(`${name}-failure`,name,name==='ordered'?[]:[true],undefined,{fail:true,refusal:true});
  for(const mode of ['missing','wrong-kind']) {
    invoke(`ordered-${mode}`,'ordered',[],undefined,{mode,fail:true,refusal:true});
    invoke(`choose-inactive-${mode}`,'choose',[false],7n,{mode,boundary:true});
  }
} finally {r.QueryContext.prototype.query=query;}
assert.deepEqual(inventory(runtime),before,'runtime production closure changed');
assert.equal(hash(wrapper),wrapperHash);assert.equal(hash(process.argv[1]),selfHash);
assert.equal(realpathSync(process.execPath),nodeIdentity.path);assert.equal(hash(process.execPath),nodeIdentity.sha256);
const sharedObjects=process.report.getReport().sharedObjects;
const system=sharedObjects.filter(path=>path.startsWith('/System/Library/')||path.startsWith('/usr/lib/'));
const kernelNames=new Set(['linux-vdso.so.1','linux-gate.so.1']);
const kernelVirtualLibraries=[...new Set(sharedObjects.filter(path=>kernelNames.has(path)))];
const external=sharedObjects.filter(path=>!system.includes(path)&&!kernelNames.has(path));
assert.ok(external.length);
writeFileSync(options.output,JSON.stringify({format:'compact-map-size-expression-ts/v1',
  runtime:{path:runtime,version:options.version,files:before},
  node:{...nodeIdentity,sharedLibraries:Object.fromEntries(external.map(path=>[path,hash(path)])),
    systemLibraries:system,kernelVirtualLibraries,systemScope:'host OS paths and kernel mapping names retained; their bytes not qualified'},
  wrapper:{path:wrapper,sha256:wrapperHash},capture:{path:realpathSync(process.argv[1]),sha256:selfHash},
  scope:'actual generated wrapper; externally seeded empty/nested/malformed states are explicit execution fixtures; no proof claim',
  cases,refusals,boundaries},null,2)+'\n',{flag:'wx'});
console.log(JSON.stringify({cases:cases.length,refusals:refusals.length,boundaries:boundaries.length,output:options.output}));
