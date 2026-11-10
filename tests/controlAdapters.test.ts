import { test } from 'node:test';
import assert from 'node:assert/strict';
import { queueControlCommand, applyMidiLearn } from '../src/controlAdapters';
import { DEFAULT_MIDI_MAP } from '../src/controlModel';
import { clearLive } from '../src/model';
import { seed } from '../src/data';
import type { CommandIntent, ControlStatus } from '../src/types';
const status:ControlStatus={armedSource:'none',chasePaused:false,timelinePlaying:false,timeFrames:0};
const live=clearLive(seed.layout);

test('queue waits for publication and sync before preparing the next command, including null live',async()=>{
  const calls:string[]=[];
  let releasePublish!:()=>void, releaseSync!:()=>void;
  const publication=new Promise<void>(r=>{releasePublish=r;});
  const synced=new Promise<void>(r=>{releaseSync=r;});
  const first=queueControlCommand(Promise.resolve(),async()=>{calls.push('prepare1');return {live,status};},async()=>{calls.push('publish1');await publication;},()=>{calls.push('accept1');},async accepted=>{assert.equal(accepted,status);calls.push('sync1');await synced;});
  const nextStatus={...status,timelinePlaying:true};
  const second=queueControlCommand(first,async()=>{calls.push('prepare2');return {live:null,status:nextStatus};},async()=>{assert.fail('null live must not publish');},()=>{calls.push('accept2');},async accepted=>{assert.equal(accepted,nextStatus);calls.push('sync2');});
  await new Promise(r=>setImmediate(r));assert.deepEqual(calls,['prepare1','publish1']);
  releasePublish();await new Promise(r=>setImmediate(r));assert.deepEqual(calls,['prepare1','publish1','accept1','sync1']);
  releaseSync();await second;
  assert.deepEqual(calls,['prepare1','publish1','accept1','sync1','prepare2','accept2','sync2']);
});
test('failed preparation/publication never accepts or syncs, and queue recovers',async()=>{
  for(const failure of ['prepare','publish']){
    const failed=queueControlCommand(Promise.resolve(),async()=>{if(failure==='prepare')throw Error('failed');return {live,status};},async()=>{throw Error('failed');},()=>assert.fail('must not accept'),async()=>assert.fail('must not sync'));
    await assert.rejects(failed,/failed/);
    let recovered=false;
    await queueControlCommand(failed,async()=>({live:null,status}),async()=>{},()=>{},async()=>{recovered=true;});
    assert.ok(recovered);
  }
});
test('sync failure preserves accepted status and the next command can proceed',async()=>{
  let accepted:CommandIntent|null=null;
  const failed=queueControlCommand(Promise.resolve(),async()=>({live,status}),async()=>{},intent=>{accepted=intent;},async()=>{throw Error('sync failed');});
  await assert.rejects(failed,/sync failed/);assert.deepEqual(accepted,{live,status});
  await queueControlCommand(failed,async()=>({live:null,status}),async()=>{},()=>{},async()=>{});
});
test('MIDI learn updates only the matching row and preserves commands and device settings',()=>{
  const map=structuredClone(DEFAULT_MIDI_MAP);map.deviceId='Keyboard';
  const event={bindingId:map.bindings[1].id,channel:16,message:'controlChange' as const,number:0};
  const next=applyMidiLearn(map,event,event.bindingId);
  assert.equal(next.deviceId,'Keyboard');assert.equal(next.bindings[0],map.bindings[0]);
  assert.deepEqual(next.bindings[1],{...map.bindings[1],channel:16,message:'controlChange',number:0});
  assert.equal(map.bindings[1].channel,1);
  assert.equal(applyMidiLearn(map,event,null),map);
  assert.equal(applyMidiLearn(map,{...event,bindingId:'removed'},'removed'),map);
});
test('MIDI learn rejects invalid or duplicate captures without mutating the map',()=>{
  const map=structuredClone(DEFAULT_MIDI_MAP), bindingId=map.bindings[1].id;
  for(const patch of [{channel:0,number:3},{channel:17,number:3},{channel:1,number:128},{channel:1,number:60}]){
    assert.throws(()=>applyMidiLearn(map,{bindingId,message:'note',...patch},bindingId));
  }
  assert.deepEqual(map,DEFAULT_MIDI_MAP);
});
