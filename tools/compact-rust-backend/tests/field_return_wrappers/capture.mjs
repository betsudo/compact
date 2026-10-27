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
const specs=JSON.parse(readFileSync(new URL('./cases.json',import.meta.url)));
const key={bytes:new Uint8Array(32)},rows=[];
const neutralEffects=Object.fromEntries(['claimedNullifiers','claimedShieldedReceives','claimedShieldedSpends',
 'claimedContractCalls','shieldedMints','unshieldedMints','unshieldedInputs','unshieldedOutputs',
 'claimedUnshieldedSpends'].map(name=>[name,[]]));
let active;
const query=r.QueryContext.prototype.query;
r.QueryContext.prototype.query=function(...args){const out=query.apply(this,args);if(active)active.push(json(out.gasCost));return out;};
try { for(const spec of specs) {
 const events=[];
 function witness(name,ctx,value) {
  events.push([name,ctx.privateState,String(ctx.ledger.count),String(ctx.ledger.values.size())]);
  if(spec.fail===name)throw new Error(`${name} refused`);
  return [ctx.privateState+1,value];
 }
 const contract=new c.Contract({choose:ctx=>witness('choose',ctx,spec.choice),observe:ctx=>witness('observe',ctx,9n)});
 const initial=contract.initialState({initialPrivateState:7,initialZswapLocalState:r.emptyZswapLocalState(key)});
 const input=r.createCircuitContext(r.dummyContractAddress(),key,initial.currentContractState.data,7,undefined,undefined,0);
 active=[];let out,error;
 try{out=contract.circuits[spec.name](input,...spec.args.map(x=>typeof x==='number'?BigInt(x):x));}catch(e){error=e;}
 const queries=active;active=undefined;
 if(spec.error){assert.ok(error,spec.id);assert.ok(error.message.includes(spec.error==='subtract'?'result of subtraction would be negative':spec.error));
  rows.push({id:spec.id,error:error.message,events,queries});continue;}
 assert.ifError(error);assert.equal(out.result,BigInt(spec.result),spec.id);
 if(spec.name==='expression_only'){assert.equal(queries.length,0);rows.push({id:spec.id,output:fab(field(out.result)),events,queries});continue;}
 const ledger=c.ledger(out.context.currentQueryContext.state);
 const keys=[...ledger.values].map(Number).sort((a,b)=>a-b);
 assert.deepEqual(keys,spec.keys,spec.id);assert.equal(ledger.count,BigInt(spec.count),spec.id);
 assert.equal(ledger.stored,2n);assert.deepEqual(json(out.context.currentQueryContext.effects),neutralEffects,spec.id);
 rows.push({id:spec.id,output:fab(out.proofData.output),keys,count:String(ledger.count),stored:String(ledger.stored),
 privateState:out.context.currentPrivateState,privateOutputs:out.proofData.privateTranscriptOutputs.map(fab),events,queries,
 gasCost:Object.fromEntries(['readTime','computeTime','bytesWritten','bytesDeleted'].map(k=>[k,queries.reduce((sum,q)=>sum+BigInt(q[k]),0n).toString()])),
 effects:json(out.context.currentQueryContext.effects)});
}}finally{r.QueryContext.prototype.query=query;}
assert.deepEqual(inventory(runtime),before,'runtime production closure changed');
assert.equal(hash(wrapper),wrapperHash);assert.equal(hash(process.argv[1]),selfHash);
assert.equal(realpathSync(process.execPath),nodeIdentity.path);assert.equal(hash(process.execPath),nodeIdentity.sha256);
const sharedObjects=process.report.getReport().sharedObjects;
const system=sharedObjects.filter(path=>path.startsWith('/System/Library/')||path.startsWith('/usr/lib/'));
const kernelNames=new Set(['linux-vdso.so.1','linux-gate.so.1']);
const kernelVirtualLibraries=[...new Set(sharedObjects.filter(path=>kernelNames.has(path)))];
const external=sharedObjects.filter(path=>!system.includes(path)&&!kernelNames.has(path));
assert.ok(external.length);
writeFileSync(options.output,JSON.stringify({format:'compact-field-return-wrappers-ts/v1',
  runtime:{path:runtime,version:options.version,files:before},
  node:{...nodeIdentity,sharedLibraries:Object.fromEntries(external.map(path=>[path,hash(path)])),
    systemLibraries:system,kernelVirtualLibraries,systemScope:'host OS paths and kernel mapping names retained; their bytes not qualified'},
  wrapper:{path:wrapper,sha256:wrapperHash},capture:{path:realpathSync(process.argv[1]),sha256:selfHash},
  scope:'actual generated wrapper and source constructor; successful results/state/effects/costs and failure callback prefixes; no failed-context rollback or proof claim',
  rows},null,2)+'\n',{flag:'wx'});
console.log(JSON.stringify({cases:rows.length,output:options.output}));
