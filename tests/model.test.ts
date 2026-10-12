import { test } from 'node:test';
import assert from 'node:assert/strict';
import { splitLyrics, snapshot, clearLive, scaleLayout, validateProject, matchSong, portableProject } from '../src/model';
import { seed, preset } from '../src/data';
test('live snapshot survives edits and changed layouts',()=>{const p=structuredClone(seed);const live=snapshot(p,p.library[0],0);p.library[0].blocks[0].primary='Edited';p.layout.regions[0].x=0;assert.notEqual(live.block?.primary,'Edited');assert.equal(live.layout.regions[0].x,192);assert.equal(clearLive(p.layout).block,null);});
test('Spanish accents and repeated sections survive splitting',()=>{const b=splitLyrics('[Coro]\nÉl es fiel\nSanto es\n\n[Coro]\nÉl es fiel\nSanto es');assert.equal(b.length,2);assert.equal(b[0].primary,b[1].primary);assert.equal(b[1].section,'Coro');});
test('portable service refuses missing content and invalid canvas',()=>{const p=structuredClone(seed);validateProject(p);p.service.push('missing');assert.throws(()=>validateProject(p));p.service.pop();p.layout.regions[0].x=2000;assert.throws(()=>validateProject(p));});
test('resizing preserves relative region placement',()=>{const l=scaleLayout(preset('paired'),3840,2160);assert.equal(l.regions[0].x,384);assert.equal(l.regions[0].fontSize,136);validateProject({...structuredClone(seed),layout:l});});
test('search ignores Spanish accents',()=>{const s=structuredClone(seed.library[0]);s.title='Él es fiel';assert.ok(matchSong(s,'el es'));});
test('portable services omit executable paths and cloud endpoints',()=>{const p=structuredClone(seed);p.settings.localServerPath='/private/runtime';p.settings.localModelPath='/private/model';p.settings.aiEndpoint='https://example.com';const exported=portableProject(p);assert.equal(exported.settings.localServerPath,'');assert.equal(exported.settings.localModelPath,'');assert.equal(exported.settings.aiEndpoint,'');assert.equal(p.settings.localServerPath,'/private/runtime');});
test('media layers and stream composite round-trip in projects',()=>{
  const p=structuredClone(seed);
  p.media=[{id:'a1',name:'bg.jpg',path:'/tmp/bg.jpg',kind:'image',mime:'image/jpeg'}];
  p.streamComposite=true;
  p.layout.mediaLayers=[{id:'l1',name:'Background',role:'background',kind:'image',assetId:'a1',x:0,y:0,width:1920,height:1080,opacity:1,muted:true,fit:'cover'}];
  validateProject(p);
  const live=snapshot(p,p.library[0],0);
  assert.equal(live.streamComposite,true);
  assert.equal(live.media?.length,1);
  assert.equal(live.layout.mediaLayers?.length,1);
  p.layout.mediaLayers[0].assetId='missing';
  assert.throws(()=>validateProject(p));
});
test('service imports reject malformed fields used by the interface',()=>{
  for(const change of [
    (p:any)=>p.name=42,
    (p:any)=>p.library[0].language=null,
    (p:any)=>p.library[0].artist={},
    (p:any)=>p.library[0].source.name=[],
    (p:any)=>p.library[0].source.url=false,
    (p:any)=>p.library[0].blocks[0].section={},
    (p:any)=>p.library[0].blocks[0].reference=42,
    (p:any)=>p.library[0].blocks[0]=null,
    (p:any)=>p.service.push(p.service[0]),
  ]) {
    const p=structuredClone(seed);change(p);assert.throws(()=>validateProject(p));
  }
});
