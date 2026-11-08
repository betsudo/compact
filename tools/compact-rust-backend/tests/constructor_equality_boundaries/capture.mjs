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
const [generated,runtimeRoot,casesPath,output]=process.argv.slice(2);
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

const runtime=realpathSync(runtimeRoot),before=inventory(runtime);
assert.equal(JSON.parse(readFileSync(resolve(runtime,'package.json'))).version,'0.16.102');
const cases=JSON.parse(readFileSync(casesPath));
assert.equal(cases.length,30);
const ids=new Set();
const rows=[];const key={bytes:new Uint8Array(32)};
const json=x=>JSON.parse(JSON.stringify(x,(_,v)=>typeof v==='bigint'?String(v):v instanceof Uint8Array?Array.from(v):v));
for(const row of cases){
 const id=`${row.role}/${row.case}`;
 assert.ok(!ids.has(id));ids.add(id);
 const wrapper=realpathSync(resolve(generated,row.role,'contract/index.js'));
 const runtimePath=createRequire(wrapper).resolve('@midnight-ntwrk/compact-runtime');
 assert.equal(realpathSync(runtimePath),realpathSync(resolve(runtime,'dist/index.js')));
 const r=await import(pathToFileURL(runtimePath)),c=await import(pathToFileURL(wrapper));
 const item=v=>row.role.startsWith('mixed_uint_')?BigInt(v):
  row.role.startsWith('tuple_')?[BigInt(v[0]),new Uint8Array(v[1])]:
  row.role.startsWith('vector_')?v.map(BigInt):
  row.role.startsWith('struct_')?{value:BigInt(v.value),tag:new Uint8Array(v.tag)}:
  assert.fail(`unknown role ${row.role}`);
 const args=[row.args[0].map(item),item(row.args[1])];
 const beforeArgs=json(args),contract=new c.Contract({});
 const initialize=()=>contract.initialState({initialPrivateState:42,initialZswapLocalState:r.emptyZswapLocalState(key)},...args);
 const initial=initialize(),value=c.ledger(initial.currentContractState.data).stored;
 assert.equal(value,BigInt(row.expected));assert.equal(initial.currentPrivateState,42);
 const entries={};
 for(const [name,delta] of [['read',0n],['advance',4n],['advance_other',5n]]){
  const fresh=initialize();
  const context=r.createCircuitContext(r.dummyContractAddress(),key,fresh.currentContractState.data,42,undefined,undefined,0);
  const out=contract.circuits[name](context,...(name==='read'?[]:[4n]));
  assert.equal(out.result,value+delta);
  assert.equal(c.ledger(out.context.currentQueryContext.state).stored,value+delta);
  entries[name]={result:String(out.result),publicTranscript:json(out.proofData.publicTranscript)};
 }
 assert.deepEqual(json(args),beforeArgs);
 rows.push({role:row.role,case:row.case,initial:String(value),entries});
}
assert.deepEqual(inventory(runtime),before);
writeFileSync(output,JSON.stringify({rows,runtime:before,node:{path:realpathSync(process.execPath),sha256:hash(process.execPath),version:process.version}},null,2)+'\n');
