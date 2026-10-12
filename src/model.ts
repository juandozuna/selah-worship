import type { Block, Layout, Live, MediaAsset, MediaLayer, Motion, MotionOverride, Project, Song } from './types';
export function splitLyrics(text: string, size = 2): Block[] {
  const blocks: Block[] = []; let section = 'Verse 1', lines: string[] = [];
  const flush = () => { for(let i=0;i<lines.length;i+=size) blocks.push({id:`block-${blocks.length}`,section,primary:lines.slice(i,i+size).join('\n'),secondary:null,reference:null}); lines=[]; };
  for(const line of text.split(/\r?\n/)) { const s=line.trim(); if(!s) flush(); else if(s.startsWith('[')&&s.endsWith(']')) {flush();section=s.slice(1,-1);} else lines.push(s); } flush(); return blocks;
}
export const CUT_MOTION: Motion = {
  transition:{kind:'cut',durationMs:400,easing:'ease-in-out',direction:'left',feather:.1},
  text:{kind:'none',unit:'line',durationMs:500,staggerMs:80,easing:'ease-out',direction:'up',order:'forward'},
};
/** Block over song over project default; missing parts inherit. */
export function resolveMotion(project: Project, song?: Song, block?: Block): Motion {
  const out=structuredClone(project.motion??CUT_MOTION);
  for(const o of [song?.motion,block?.motion] as (MotionOverride|null|undefined)[]) {
    if(o?.transition) out.transition={...o.transition};
    if(o?.text) out.text={...o.text};
  }
  return out;
}
export function clearLive(layout: Layout, motion: Motion|null = null, media: MediaAsset[] = [], streamComposite = false): Live {
  return { songId:null,blockIndex:null,block:null,layout:structuredClone(layout),testPattern:false,motion:motion&&structuredClone(motion),media:structuredClone(media),streamComposite };
}
export function snapshot(project: Project,song: Song,index:number,motionOn=true): Live {
  if(!song.blocks[index]) throw new Error('Block not found');
  return {
    songId:song.id,
    blockIndex:index,
    block:structuredClone(song.blocks[index]),
    layout:structuredClone(project.layout),
    testPattern:false,
    motion:motionOn?resolveMotion(project,song,song.blocks[index]):null,
    media:structuredClone(project.media??[]),
    streamComposite:Boolean(project.streamComposite),
  };
}
export function validateMediaAsset(a: MediaAsset): void {
  if(!a||typeof a.id!=='string'||!a.id||typeof a.name!=='string'||!a.name||typeof a.path!=='string'||!a.path||!['image','video'].includes(a.kind)||typeof a.mime!=='string'||!a.mime||a.mime.length>200) throw new Error('Invalid media asset.');
}
export function validateMediaLayer(layer: MediaLayer, layout: Layout): void {
  if(!layer||typeof layer.id!=='string'||!layer.id||typeof layer.name!=='string'||!layer.name||!['background','foreground'].includes(layer.role)||!['image','video'].includes(layer.kind)||typeof layer.assetId!=='string'||!layer.assetId) throw new Error('Invalid media layer.');
  if([layer.x,layer.y,layer.width,layer.height,layer.opacity].some(v=>!Number.isFinite(v))||layer.x<0||layer.y<0||layer.width<=0||layer.height<=0||layer.x+layer.width>layout.width+.01||layer.y+layer.height>layout.height+.01||layer.opacity<0||layer.opacity>1) throw new Error(`Check the media layer “${layer.name}”.`);
  if(layer.fit!=null&&!['cover','contain','fill'].includes(layer.fit)) throw new Error('Invalid media fit mode.');
  if(layer.audioDeviceId!=null&&(typeof layer.audioDeviceId!=='string'||layer.audioDeviceId.length>500)) throw new Error('Invalid audio device.');
}
export function validateProject(p: Project): void {
  if(!p || p.version!==1 || typeof p.name!=='string' || !Array.isArray(p.library) || !Array.isArray(p.service) || !p.settings) throw new Error('Choose a Selah service file (version 1).');
  validateLayout(p.layout);validateMotion(p.motion);
  if(p.display){validateLayout(p.display.layout);if(!/^#[a-f0-9]{6}$/i.test(p.display.background)||!Number.isInteger(p.display.monitor)||p.display.monitor<0) throw new Error('Invalid display output settings.');}
  if(p.streamComposite!=null&&typeof p.streamComposite!=='boolean') throw new Error('Invalid stream composite setting.');
  const media=p.media??[];
  if(!Array.isArray(media)||media.length>64) throw new Error('Media library is too large.');
  const mediaIds=new Set<string>();
  for(const asset of media){validateMediaAsset(asset);if(mediaIds.has(asset.id))throw new Error('Duplicate media asset.');mediaIds.add(asset.id);}
  const checkLayers=(layout:Layout)=>{for(const layer of layout.mediaLayers??[]){validateMediaLayer(layer,layout);if(!mediaIds.has(layer.assetId)) throw new Error(`Media layer “${layer.name}” references a missing asset.`);const asset=media.find(a=>a.id===layer.assetId);if(asset&&asset.kind!==layer.kind) throw new Error(`Media layer “${layer.name}” kind does not match its asset.`);}};
  checkLayers(p.layout);if(p.display)checkLayers(p.display.layout);
  const ids = new Set<string>();
  if(p.library.length>10000) throw new Error('Library is too large.');
  for(const song of p.library) {
    if(!song||typeof song.id!=='string'||!song.id||ids.has(song.id)||typeof song.title!=='string'||!song.title||typeof song.artist!=='string'||typeof song.language!=='string'||typeof song.kind!=='string'||!song.source||typeof song.source.name!=='string'||typeof song.source.license!=='string'||!(song.source.url===null||typeof song.source.url==='string')||!Array.isArray(song.blocks)||!song.blocks.length||song.blocks.length>10000) throw new Error('Invalid or duplicate content.'); ids.add(song.id);
    validateMotion(song.motion);
    const blockIds=new Set<string>();
    for(const block of song.blocks) {if(!block||typeof block.id!=='string'||!block.id||blockIds.has(block.id)||typeof block.section!=='string'||typeof block.primary!=='string'||block.primary.length>100000||!(block.secondary===null||typeof block.secondary==='string')||!(block.reference===null||typeof block.reference==='string')) throw new Error('Invalid content block.');validateMotion(block.motion);blockIds.add(block.id);}
  }
  if(p.service.some(id=>typeof id!=='string'||!ids.has(id))||new Set(p.service).size!==p.service.length) throw new Error('Service contains missing or duplicate content.');
  if(!['en','es'].includes(p.settings.language)||['feedName','aiProvider','aiModel','aiEndpoint','localServerPath','localModelPath'].some(key=>typeof p.settings[key as keyof typeof p.settings]!=='string')) throw new Error('Invalid interface settings.');
}
const TRANSITIONS=['cut','fade','dip','slide','push','wipe','reveal','zoom','blur'],TEXT_KINDS=['none','fade','slide','blur','typewriter','word-rise','line-cascade'],EASINGS=['linear','ease-in','ease-out','ease-in-out','spring'],DIRECTIONS=['left','right','up','down'],UNITS=['block','line','word','letter'],ORDERS=['forward','backward','center','random'];
export function validateMotion(m: Partial<Motion>|MotionOverride|undefined|null): void {
  if(!m) return;
  const t=m.transition,x=m.text;
  if(t&&(!TRANSITIONS.includes(t.kind)||!EASINGS.includes(t.easing)||!DIRECTIONS.includes(t.direction)||!(t.durationMs>=100&&t.durationMs<=3000)||!(t.feather>=0&&t.feather<=.5))) throw new Error('Invalid transition settings.');
  if(x&&(!TEXT_KINDS.includes(x.kind)||!UNITS.includes(x.unit)||!EASINGS.includes(x.easing)||!DIRECTIONS.includes(x.direction)||!ORDERS.includes(x.order)||!(x.durationMs>=100&&x.durationMs<=3000)||!(x.staggerMs>=0&&x.staggerMs<=1000))) throw new Error('Invalid text animation settings.');
}
export function validateLayout(l: Layout): void {
  if(!l||!Number.isInteger(l.width)||!Number.isInteger(l.height)||l.width<64||l.height<64||l.width>8192||l.height>8192||l.width*l.height>33554432||!Array.isArray(l.regions)||!l.regions.length||l.regions.length>32) throw new Error('Canvas must be 64–8192 pixels per side, at most 32 megapixels, with 1–32 regions.');
  const ids=new Set<string>();
  for(const r of l.regions) {if(!r.id||ids.has(r.id)||typeof r.name!=='string'||typeof r.fontFamily!=='string'||typeof r.shadow!=='boolean'||[r.x,r.y,r.width,r.height,r.fontSize,r.outline].some(v=>!Number.isFinite(v))||r.x<0||r.y<0||r.width<=0||r.height<=0||r.x+r.width>l.width+.01||r.y+r.height>l.height+.01||r.fontSize<8||r.fontSize>512||r.outline<0||r.outline>20||!/^#[a-f0-9]{6}$/i.test(r.color)||!(r.anim==null||TEXT_KINDS.includes(r.anim))||!['primary','secondary','reference'].includes(r.binding)||!['left','center','right'].includes(r.align)) throw new Error(`Check the dimensions and style of “${r.name}”.`);ids.add(r.id);}
  const layers=l.mediaLayers??[];
  if(layers.length>8) throw new Error('Use at most 8 media layers.');
  const mediaIds=new Set<string>();let bg=0,fg=0;
  for(const layer of layers){validateMediaLayer(layer,l);if(mediaIds.has(layer.id))throw new Error('Duplicate media layer.');mediaIds.add(layer.id);if(layer.role==='background')bg++;else fg++;}
  if(bg>2||fg>2) throw new Error('Use at most two background and two foreground media layers.');
}
export function scaleLayout(l:Layout,width:number,height:number): Layout {
  const sx=width/l.width, sy=height/l.height;
  return {
    ...l,width,height,
    regions:l.regions.map(r=>({...r,x:Math.round(r.x*sx),y:Math.round(r.y*sy),width:Math.round(r.width*sx),height:Math.round(r.height*sy),fontSize:Math.max(8,Math.min(512,Math.round(r.fontSize*Math.min(sx,sy))))})),
    mediaLayers:(l.mediaLayers??[]).map(m=>({...m,x:Math.round(m.x*sx),y:Math.round(m.y*sy),width:Math.round(m.width*sx),height:Math.round(m.height*sy)})),
  };
}
export function assetUrl(asset: MediaAsset | undefined): string | null {
  if(!asset?.path) return null;
  if(asset.path.startsWith('data:')||asset.path.startsWith('blob:')||asset.path.startsWith('http')||asset.path.startsWith('asset://')||asset.path.startsWith('file://')) return asset.path;
  return `asset://localhost/${asset.path.replace(/^\/+/,'')}`;
}
export function fold(s:string) {return s.normalize('NFD').replace(/[\u0300-\u036f]/g,'').toLowerCase();}
export function matchSong(s:Song,q:string) {return fold(`${s.title} ${s.artist} ${s.blocks.map(b=>b.primary).join(' ')}`).includes(fold(q));}
export function portableProject(p:Project):Project {return {...structuredClone(p),settings:{...p.settings,aiProvider:'local',aiEndpoint:'',localServerPath:'',localModelPath:''}};}
