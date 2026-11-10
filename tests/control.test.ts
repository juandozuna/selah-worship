import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import type { Project } from '../src/types';
import { validateProject } from '../src/model';
import { DEFAULT_MIDI_MAP, DEFAULT_OSC_SETTINGS, DEFAULT_TIMECODE_SETTINGS } from '../src/controlModel';
const legacy=()=>JSON.parse(readFileSync(new URL('./fixtures/control-project.json',import.meta.url),'utf8')) as Project;
test('legacy defaults and Wave 1 contracts round-trip',()=>{
  const p=legacy();validateProject(p);
  p.settings.midi=structuredClone(DEFAULT_MIDI_MAP);p.settings.osc={...DEFAULT_OSC_SETTINGS};p.settings.timecode={...DEFAULT_TIMECODE_SETTINGS,fps:29.97,dropFrame:true,armedSource:'mtc'};
  p.library[0].cues=[{id:'cue',timeFrames:900,command:{type:'go',songId:'song',blockIndex:1}}];
  p.fonts=[{id:'font',family:'Family',relativePath:'fonts/family.otf'}];
  validateProject(JSON.parse(JSON.stringify(p)));assert.deepEqual(p.settings.midi.bindings.map(b=>b.number),[60,61,62]);
});
test('invalid control settings, cues, commands and font paths are rejected',()=>{
  const changes:((p:any)=>void)[]=[
    p=>p.settings.osc={enabled:true,port:0},p=>p.settings.osc={enabled:true,port:65536},
    p=>p.settings.timecode={...DEFAULT_TIMECODE_SETTINGS,fps:26},p=>p.settings.timecode={...DEFAULT_TIMECODE_SETTINGS,dropFrame:true},
    p=>p.settings.timecode={...DEFAULT_TIMECODE_SETTINGS,armedSource:['mtc','ltc']},
    p=>{p.settings.midi=structuredClone(DEFAULT_MIDI_MAP);p.settings.midi.bindings[0].channel=0;},
    p=>{p.settings.midi=structuredClone(DEFAULT_MIDI_MAP);p.settings.midi.bindings.push({...p.settings.midi.bindings[0],id:'duplicate-input'});},
    ...['fonts/../bad.ttf','/tmp/f.ttf','fonts/a\\b.ttf','fonts/C:f.ttf','fonts/f.woff','fonts//f.otf'].map(relativePath=>(p:any)=>p.fonts=[{id:'f',family:'Family',relativePath}]),
    ...[{type:'go',songId:'missing',blockIndex:0},{type:'go',songId:'song',blockIndex:2},{type:'timelineSeek',timeFrames:1.5},{type:'unknown'},{type:'chaseArm',source:'both'}].map(command=>(p:any)=>p.library[0].cues=[{id:'cue',timeFrames:0,command}]),
    p=>p.library[0].cues=[{id:'cue',timeFrames:Number.MAX_SAFE_INTEGER+1,command:{type:'clear'}}],
    p=>p.library[0].cues=[{id:'cue',timeFrames:0,command:{type:'clear'}},{id:'cue',timeFrames:1,command:{type:'clear'}}],
    p=>p.fonts=null,p=>p.library[0].cues=null,
  ];
  for(const change of changes){const p=legacy();change(p);assert.throws(()=>validateProject(p));}
});
