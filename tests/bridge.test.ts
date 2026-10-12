import { test } from 'node:test';
import assert from 'node:assert/strict';
import { load, save, hasWelcomed, rememberWelcome } from '../src/bridge';
import { seed } from '../src/data';

// Each test file runs in its own Node worker; these globals model a browser
// without the Tauri adapter. Keep saved bytes visible to check preservation.
Object.defineProperty(globalThis, 'window', { value: {}, configurable: true });
function storage(value:string|null = null) {
  const values=new Map<string,string>();
  if(value!==null)values.set('selah.project.v1',value);
  Object.defineProperty(globalThis,'localStorage',{configurable:true,value:{
    getItem:(key:string)=>values.get(key)??null,
    setItem:(key:string,next:string)=>values.set(key,next),
  }});
  return values;
}

test('first browser launch loads an independent seed and saves edits',async()=>{
  const values=storage();
  const project=await load();
  project.name='Wednesday service';
  await save(project);
  assert.equal((await load()).name,'Wednesday service');
  assert.equal(seed.name,'Sunday worship');
  assert.ok(values.has('selah.project.v1'));
});

test('invalid saved services are rejected without replacing their bytes',async()=>{
  for(const raw of ['{broken','',JSON.stringify({...seed,name:42}),JSON.stringify({...seed,library:[{...seed.library[0],language:null}]})]) {
    const values=storage(raw);
    await assert.rejects(load());
    assert.equal(values.get('selah.project.v1'),raw);
  }
});

test('blocked storage does not crash onboarding or silently pretend to save',async()=>{
  Object.defineProperty(globalThis,'localStorage',{configurable:true,get:()=>{throw new Error('Storage blocked');}});
  assert.equal(hasWelcomed(),false);
  assert.doesNotThrow(rememberWelcome);
  await assert.rejects(load(),/Browser storage is unavailable/);
  await assert.rejects(save(structuredClone(seed)),/Storage blocked/);
});

test('onboarding preference persists when browser storage is available',()=>{
  storage();
  assert.equal(hasWelcomed(),false);
  rememberWelcome();
  assert.equal(hasWelcomed(),true);
});

test('failed autosave leaves the previous service intact',async()=>{
  const original=JSON.stringify(seed);
  const values=storage(original);
  Object.defineProperty(globalThis,'localStorage',{configurable:true,value:{
    getItem:(key:string)=>values.get(key)??null,
    setItem:()=>{throw new Error('Storage quota exceeded');},
  }});
  const project=await load();
  project.name='Unsaved edit';
  await assert.rejects(save(project),/quota/);
  assert.equal(values.get('selah.project.v1'),original);
});
