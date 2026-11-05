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
const rows=[];const key={bytes:new Uint8Array(32)};
for(const row of JSON.parse(readFileSync(casesPath))){
 const wrapper=realpathSync(resolve(generated,row.role,'contract/index.js'));
 const runtimePath=createRequire(wrapper).resolve('@midnight-ntwrk/compact-runtime');
 assert.equal(realpathSync(runtimePath),realpathSync(resolve(runtime,'dist/index.js')));
 const r=await import(pathToFileURL(runtimePath)),c=await import(pathToFileURL(wrapper));
 const convert=(v)=>Array.isArray(v)?v.map(convert):typeof v==='number'?BigInt(v):v;
 const args=row.args.map(convert);
 if(row.role==='bytes_pair_eq'){args[0]=row.args[0].map(x=>new Uint8Array(x));args[1]=new Uint8Array(row.args[1]);}
 const contract=new c.Contract({});
 const initial=contract.initialState({initialPrivateState:42,initialZswapLocalState:r.emptyZswapLocalState(key)},...args);
 const value=c.ledger(initial.currentContractState.data).stored;
 assert.equal(value,BigInt(row.expected));
 const context=r.createCircuitContext(r.dummyContractAddress(),key,initial.currentContractState.data,42,undefined,undefined,0);
 const next=contract.circuits.advance(context,4n);assert.equal(next.result,value+4n);
 rows.push({role:row.role,case:row.case,initial:String(value),advanced:String(next.result)});
}
assert.deepEqual(inventory(runtime),before);
writeFileSync(output,JSON.stringify({rows,runtime:before,node:{path:realpathSync(process.execPath),sha256:hash(process.execPath),version:process.version}},null,2)+'\n');
