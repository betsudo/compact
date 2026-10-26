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
const query=r.QueryContext.prototype.query;let active;
r.QueryContext.prototype.query=function(...args){const out=query.apply(this,args);if(active)active.push(json(out.gasCost));return out;};
const args=x=>Array.isArray(x)?x.map(args):typeof x==='number'?BigInt(x):x;
try {for(const spec of specs){
 const source=spec.source,path=pathToFileURL(wrappers[source].path);
 if(!modules.has(source))modules.set(source,await import(path));const {Contract,ledger}=modules.get(source),events=[];
 const operand=(name,value)=>ctx=>{events.push({name,privateState:ctx.privateState});if(spec.fail===name)throw Error(name+' refused');return[ctx.privateState+1,value];};
 const witnesses={seed:operand('seed',1n),items:operand('items',[2n,3n]),left:operand('left',[2n,3n]),right:operand('right',[4n,5n]),step:(ctx,acc,item)=>{events.push({name:'step',privateState:ctx.privateState,acc:String(acc),item:String(item)});if(spec.fail==='step'&&item===7n)throw Error('step refused');return[ctx.privateState+1,acc*10n+item];}};
 const c=new Contract(witnesses),init=c.initialState({initialPrivateState:7,initialZswapLocalState:r.emptyZswapLocalState(key)});
 const ctx=r.createCircuitContext(r.dummyContractAddress(),key,init.currentContractState.data,7,undefined,undefined,0);
 active=[];let out,error;try{out=c.circuits.run(ctx,...spec.args.map(args));}catch(e){error=e.message;}const costs=active;active=undefined;
 if(spec.error){assert.ok(error?.includes(spec.error),spec.id+': '+error);assert.equal(out,undefined);rows.push({...spec,events,observedError:error});continue;}
 assert.equal(error,undefined,spec.id);const expected=typeof spec.expected==='boolean'?spec.expected:BigInt(spec.expected);assert.equal(out.result,expected,spec.id);assert.equal(ledger(out.context.currentQueryContext.state).stored,BigInt(spec.stored),spec.id);assert.equal(out.context.currentPrivateState,spec.private,spec.id);
 rows.push({...spec,events,result:encode(out.proofData.output),stored:String(ledger(out.context.currentQueryContext.state).stored),privateState:out.context.currentPrivateState,privateOutputs:out.proofData.privateTranscriptOutputs.map(encode),effects:json(out.context.currentQueryContext.effects),gas:Object.fromEntries(['readTime','computeTime','bytesWritten','bytesDeleted'].map(k=>[k,costs.reduce((n,c)=>n+BigInt(c[k]),0n).toString()]))});
}}finally{r.QueryContext.prototype.query=query;}
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
writeFileSync(options.output,JSON.stringify({format:'direct-fold-ingress-live-ts/v1',
  runtime:{path:runtime,version:options.version,files:before},
  node:{...nodeIdentity,sharedLibraries:Object.fromEntries(external.map(path=>[path,hash(path)])),
    systemLibraries:system,kernelVirtualLibraries,systemScope:'host OS paths and kernel mapping names retained; their bytes not qualified'},
  wrappers,inputs,capture:{path:realpathSync(process.argv[1]),sha256:selfHash},
  scope:'actual complete generated wrappers, independent expected values, query-summed cost, witness/private/state checks; no proof or Linux claim',rows},null,2)+'\n',{flag:'wx'});
console.log(JSON.stringify({cases:rows.length,success:rows.filter(row=>!row.error).length,failures:rows.filter(row=>row.error).length,output:options.output}));
