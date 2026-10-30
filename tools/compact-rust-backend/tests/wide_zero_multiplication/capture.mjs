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

// Source-bound wide-zero wrapper behavior and precise failure controls.
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
const rows={pure:[],stateful:[],errors:[]};
const values=[0n,1n,1n<<128n,(1n<<200n)-1n], max128=(1n<<128n)-1n;
for(const x of values) {
  for(const [name,args] of [['rightZero',[x]],['leftZero',[x]],['rightParameter',[x,0n]],['leftParameter',[0n,x]],['nested',[x,false]],['nested',[x,true]]]) {
    const result=c.pureCircuits[name](...args);assert.equal(result,0n);
    rows.pure.push({name,input:String(x),result:String(result)});
  }
}
function cast(error,value,maximum) {
  assert.ok(error instanceof r.CompactError,'expected the actual runtime CompactError class');
  assert.match(error.message,/^wide_zero_multiplication\.compact line \d+ char \d+: /);
  assert.ok(error.message.endsWith(`cast from Field or Uint value to smaller Uint value failed: ${value} is greater than ${maximum}`),'exact cast value/maximum/family differs');
}
function assertion(error) {
  assert.ok(error instanceof r.CompactError);
  assert.equal(error.message,'failed assert: zero operand guard');
}
for(const [name,args,kind,value,maximum] of [
  ['checkedLeft',[1n<<128n,1n],'cast',1n<<128n,max128],
  ['checkedRight',[1n,1n<<128n],'cast',1n,0n],
  ['checkedLeft',[1n,1n],'cast',1n,0n],
  ['checkedRight',[0n,1n<<128n],'cast',1n<<128n,max128],
  ['firstErrorLeft',[1n<<128n,1n],'cast',1n<<128n,max128],
  ['firstErrorRight',[1n,1n<<128n],'assert'],
  ['firstErrorLeft',[1n,1n],'assert'],
  ['firstErrorRight',[0n,1n<<128n],'cast',1n<<128n,max128],
]) {
  let error;try {c.pureCircuits[name](...args);}catch(e){error=e;}
  if(kind==='cast') {
    cast(error,value,maximum);
    // Run wrong-family, value and bound controls against the same observed error.
    assert.throws(()=>assertion(error));
    assert.throws(()=>cast(error,value+1n,maximum));
    assert.throws(()=>cast(error,value,maximum+1n));
  } else {assertion(error);assert.throws(()=>cast(error,1n,0n));}
  rows.errors.push({name,kind,message:error.message});
}
for(const name of ['checkedLeft','checkedRight']) {
  const args=name==='checkedLeft'?[max128,0n]:[0n,max128];
  assert.equal(c.pureCircuits[name](...args),0n);
}
const key={bytes:new Uint8Array(32)};
for(const name of ['multiplyRight','multiplyLeft','witnessedRight','witnessedLeft']) {
  for(const value of values) {
    const witnessed=name.startsWith('witnessed');
    for(const fail of witnessed?[false,true]:[false]) {
      const events=[];
      const contract=new c.Contract({wide:ctx=>{
        events.push([ctx.privateState,String(ctx.ledger.calls)]);
        if(fail)throw new r.CompactError('wide witness failure');
        return [ctx.privateState+1,value];
      }});
      const initial=contract.initialState({initialPrivateState:42,initialZswapLocalState:r.emptyZswapLocalState(key)});
      const context=r.createCircuitContext(r.dummyContractAddress(),key,initial.currentContractState.data,42,undefined,undefined,0);
      const invoke=()=>contract.circuits[name](context,...(witnessed?[]:[value]));
      if(fail) {
        assert.throws(invoke,e=>e instanceof r.CompactError && e.message==='wide witness failure');
        assert.deepEqual(events,[[42,'1']]);continue;
      }
      const out=invoke();assert.equal(out.result,0n);
      assert.equal(c.ledger(out.context.currentQueryContext.state).calls,1n);
      assert.equal(out.context.currentPrivateState,witnessed?43:42);
      assert.deepEqual(events,witnessed?[[42,'1']]:[]);
      rows.stateful.push({name,input:String(value),result:String(out.result),calls:'1',privateState:out.context.currentPrivateState,witnesses:events.length});
    }
  }
}
assert.deepEqual(inventory(runtime),before,'runtime closure changed during capture');
assert.equal(hash(wrapper),wrapperHash);assert.equal(hash(process.argv[1]),selfHash);
assert.equal(hash(nodeIdentity.path),nodeIdentity.sha256);
const sharedLibraries={},kernelVirtualLibraries=[],systemLibraries=[];
for(const name of process.report.getReport().sharedObjects) {
  if(['linux-vdso.so.1','linux-gate.so.1'].includes(name))kernelVirtualLibraries.push(name);
  else if(name.startsWith('/System/Library/')||name.startsWith('/usr/lib/'))systemLibraries.push(name);
  else sharedLibraries[realpathSync(name)]=hash(name);
}
writeFileSync(options.output,JSON.stringify({format:'wide-zero-ts/v1',rows,wrapper:{path:wrapper,sha256:wrapperHash},capture:{path:realpathSync(process.argv[1]),sha256:selfHash},runtime:{files:before},node:{...nodeIdentity,sharedLibraries,kernelVirtualLibraries,systemLibraries,systemScope:'host OS paths and kernel mapping names retained; their bytes not qualified'}},null,2)+'\n',{flag:'wx'});
