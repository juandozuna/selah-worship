import { test } from 'node:test';
import assert from 'node:assert/strict';
import { LiveTranslator, translationLayout } from '../src/translation';
import { seed, preset } from '../src/data';
import { validateLayout } from '../src/model';
import type { Song } from '../src/types';
const tick=()=>new Promise<void>(resolve=>setImmediate(resolve));
const song=(id:string,texts:string[]):Song=>({...structuredClone(seed.library[0]),id,blocks:texts.map((primary,i)=>({id:String(i),primary,section:'Verse',secondary:null,reference:null}))});
function harness(limit=300) {
  const calls:{text:string;target:string;resolve:(text:string)=>void;reject:(e:Error)=>void}[]=[];
  const controller=new LiveTranslator((text,target)=>new Promise((resolve,reject)=>calls.push({text,target,resolve,reject})),()=>{},limit);
  return {controller,calls};
}
test('nothing is translated until a song is on air and the toggle is enabled',async()=>{
  const {controller,calls}=harness();
  controller.configure(undefined,null,'es',true);
  controller.configure(song('one',['A']),0,'es',false);
  controller.configure({...song('bible',['Scripture']),kind:'bible'},0,'es',true);
  await tick();assert.equal(calls.length,0);
});
test('the current block is first, repeated lyrics are reused, and the library is unchanged',async()=>{
  const {controller,calls}=harness(),s=song('one',['A','B','A']),original=structuredClone(s);
  controller.configure(s,1,'es',true);
  assert.equal(calls[0].text,'B');
  calls[0].resolve('Be');await tick();assert.equal(calls[1].text,'A');
  calls[1].resolve('Ah');await tick();
  assert.equal(calls.length,2);assert.equal(controller.status().ready,3);
  controller.configure(s,0,'es',true);await tick();assert.equal(calls.length,2);
  assert.deepEqual(s,original);
});
test('switching songs drops queued old work and prevents stale translations from being displayed',async()=>{
  const {controller,calls}=harness();
  controller.configure(song('old',['Old','Queued old']),0,'es',true);
  controller.configure(song('new',['New']),0,'es',true);
  calls[0].resolve('Viejo');await tick();
  assert.equal(calls.length,2);assert.equal(calls[1].text,'New');assert.equal(controller.status().text,null);
  calls[1].resolve('Nuevo');await tick();assert.equal(controller.status().text,'Nuevo');
});
test('language changes replace pending work and turning off stops all subsequent requests',async()=>{
  const {controller,calls}=harness(),s=song('one',['A','B']);
  controller.configure(s,0,'es',true);controller.configure(s,0,'fr',true);
  calls[0].resolve('Español');await tick();assert.equal(calls[1].target,'fr');assert.equal(controller.status().text,null);
  controller.configure(s,0,'fr',false);calls[1].resolve('Français');await tick();
  assert.equal(calls.length,2);assert.equal(controller.status().text,null);
});
test('advancing prioritizes the new current block ahead of background work',async()=>{
  const {controller,calls}=harness(),s=song('one',['A','B','C','D']);
  controller.configure(s,0,'es',true);controller.configure(s,2,'es',true);
  calls[0].resolve('Ah');await tick();assert.equal(calls[1].text,'C');
  controller.configure(undefined,null,'es',false);calls[1].resolve('Ce');await tick();
});
test('edited lyrics invalidate cached results and matching languages skip inference',async()=>{
  const {controller,calls}=harness();
  controller.configure(song('one',['A']),0,'en',true);assert.equal(calls.length,0);assert.equal(controller.status().sameLanguage,true);
  controller.configure(song('one',['A']),0,'es',true);calls[0].resolve('Ah');await tick();
  controller.configure(song('one',['Edited']),0,'es',true);assert.equal(calls[1].text,'Edited');assert.equal(controller.status().text,null);
  calls[1].resolve('Editado');await tick();
});
test('failures keep originals available and do not cause an automatic retry loop',async()=>{
  const {controller,calls}=harness();controller.configure(song('one',['A']),0,'es',true);
  calls[0].reject(new Error('Offline'));await tick();
  assert.equal(calls.length,1);assert.equal(controller.status().text,null);assert.match(controller.status().error,/Offline/);
  controller.retry();assert.equal(calls.length,2);calls[1].resolve('Ah');await tick();assert.equal(controller.status().text,'Ah');
});
test('bounded caching does not cause endless retranslations of a large active song',async()=>{
  const {controller,calls}=harness(2);controller.configure(song('one',['A','B','C','D']),0,'es',true);
  for(let i=0;i<4;i++){calls[i].resolve(`Translated ${calls[i].text}`);await tick();}
  assert.equal(calls.length,4);assert.equal(controller.lookup('A','es'),'Translated A');
  assert.equal(controller.lookup('B','es'),null);
});
test('bilingual output is temporary and existing secondary regions are preserved',()=>{
  const layout=preset('center'),original=structuredClone(layout),paired=translationLayout(layout,true);
  validateLayout(paired);assert.equal(paired.regions.filter(r=>r.binding==='secondary').length,1);
  assert.deepEqual(layout,original);assert.equal(translationLayout(layout,false),layout);
  const custom=preset('paired');assert.equal(translationLayout(custom,true),custom);
});
