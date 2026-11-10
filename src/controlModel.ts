import type { Project, PresentationCommand, MidiMap, OscSettings, TimecodeSettings } from './types';
export const DEFAULT_MIDI_MAP: MidiMap = { enabled:false, deviceId:null, bindings: ['next','prev','clear'].map((type,i)=>({id:`default-${i}`,channel:1,message:'note',number:60+i,command:{type} as PresentationCommand})) };
export const DEFAULT_OSC_SETTINGS: OscSettings = {enabled:false,port:9000};
export const DEFAULT_TIMECODE_SETTINGS: TimecodeSettings = {fps:30,dropFrame:false,armedSource:'none',pauseOnManual:true};
const frames=(n:number)=>Number.isSafeInteger(n)&&n>=0;
const source=(s:unknown)=>['none','mtc','ltc'].includes(s as string);
export function validatePresentationCommand(c: PresentationCommand, p: Project): void {
  if(!c||typeof c!=='object') throw new Error('Invalid presentation command.');
  switch(c.type){
    case 'next':case 'prev':case 'clear':case 'chaseResync':case 'timelinePlay':case 'timelinePause':return;
    case 'go':if(typeof c.songId==='string'&&Number.isSafeInteger(c.blockIndex)&&c.blockIndex>=0&&p.library.some(s=>s.id===c.songId&&Array.isArray(s.blocks)&&c.blockIndex<s.blocks.length))return;break;
    case 'chaseArm':if(source(c.source))return;break;
    case 'timelineSeek':if(frames(c.timeFrames))return;break;
  }
  throw new Error('Invalid presentation command or target.');
}
/** Missing Wave 1 fields retain legacy defaults; present fields are validated strictly. */
export function validateControlContracts(p:Project):void {
  const midi=p.settings.midi;
  if(midi!==undefined){
    if(!midi||typeof midi.enabled!=='boolean'||!(midi.deviceId===null||typeof midi.deviceId==='string'&&midi.deviceId.length>0&&midi.deviceId.length<=500)||!Array.isArray(midi.bindings)||midi.bindings.length>1024)throw new Error('Invalid MIDI map.');
    const ids=new Set<string>(),inputs=new Set<string>();
    for(const b of midi.bindings){
      if(!b||typeof b.id!=='string'||!b.id||ids.has(b.id)||!Number.isInteger(b.channel)||b.channel<1||b.channel>16||!['note','controlChange'].includes(b.message)||!Number.isInteger(b.number)||b.number<0||b.number>127)throw new Error('Invalid MIDI binding.');
      const input=`${b.channel}:${b.message}:${b.number}`;if(inputs.has(input))throw new Error('Duplicate MIDI input.');ids.add(b.id);inputs.add(input);validatePresentationCommand(b.command,p);
    }
  }
  const osc=p.settings.osc;
  if(osc!==undefined&&(!osc||typeof osc.enabled!=='boolean'||!Number.isInteger(osc.port)||osc.port<1||osc.port>65535))throw new Error('Invalid OSC settings.');
  const tc=p.settings.timecode;
  if(tc!==undefined&&(!tc||![24,25,29.97,30].includes(tc.fps)||typeof tc.dropFrame!=='boolean'||tc.dropFrame&&tc.fps!==29.97||!source(tc.armedSource)||typeof tc.pauseOnManual!=='boolean'))throw new Error('Invalid timecode settings.');
  const fonts=p.fonts===undefined?[]:p.fonts;
  if(!Array.isArray(fonts)||fonts.length>256)throw new Error('Invalid font assets.');
  const fontIds=new Set<string>();
  for(const f of fonts){
    if(!f||typeof f.id!=='string'||!f.id.trim()||fontIds.has(f.id)||typeof f.family!=='string'||!f.family.trim()||typeof f.relativePath!=='string'||!f.relativePath.startsWith('fonts/')||/[\\:\0]/.test(f.relativePath)||f.relativePath.split('/').some(s=>!s||s==='.'||s==='..')||! /\.(ttf|otf)$/i.test(f.relativePath))throw new Error('Invalid font asset path or family.');fontIds.add(f.id);
  }
  const cueIds=new Set<string>();
  for(const song of p.library){
    const cues=song.cues===undefined?[]:song.cues;
    if(!Array.isArray(cues)||cues.length>10000)throw new Error('Invalid timeline cues.');
    for(const c of cues){if(!c||typeof c.id!=='string'||!c.id||cueIds.has(c.id)||!frames(c.timeFrames))throw new Error('Invalid timeline cue.');cueIds.add(c.id);validatePresentationCommand(c.command,p);}
  }
}
