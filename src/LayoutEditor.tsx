import { useEffect, useRef, useState } from 'react';
import { Plus, Trash2, Check, Copy, Move, AlignCenter, Layers, Undo2, Redo2, ImagePlus, Grid3x3, Monitor, Radio, Film, Image as ImageIcon } from 'lucide-react';
import type { AudioDevice, Layout, MediaAsset, MediaLayer, Project, Region, Block, DisplayOutput } from './types';
import { Canvas, overflowRegions, type Guide, type Rect } from './Canvas';
import { MediaStack } from './MediaStack';
import { desktop } from './bridge';
import { TEXT_OPTIONS } from './MotionPicker';
import { preset } from './data';
import { scaleLayout, validateLayout } from './model';
import { fullBleedLayer, importMedia, listAudioOutputs, removeMediaLayer, upsertMediaLayer } from './mediaHelpers';
import { en, es } from './i18n';
type Target='stream'|'display';
const GUIDE_LIMIT=8_000_000;
export default function LayoutEditor({project,apply,applyDisplay,applyMedia}:{project:Project;apply:(layout:Layout)=>void;applyDisplay:(display:DisplayOutput)=>void;applyMedia:(media:MediaAsset[])=>void}) {
  const [target,setTarget]=useState<Target>('stream');
  const t=project.settings.language==='es'?es:en;
  const display=project.display;
  const tabs=<div className="target-tabs" role="tablist"><button role="tab" aria-selected={target==='stream'} className={target==='stream'?'active':''} onClick={()=>setTarget('stream')}><Radio size={14}/>NDI / Syphon stream</button>{desktop()&&<button role="tab" aria-selected={target==='display'} className={target==='display'?'active':''} onClick={()=>setTarget('display')}><Monitor size={14}/>Physical display</button>}</div>;
  if(target==='display'&&display)return <LayoutBuilder key="display" target="display" tabs={tabs} project={project} initial={display.layout} backdrop={display.background} note={`Fills the selected display. Layouts of a different shape are scaled to fit.`} apply={layout=>applyDisplay({...display,layout})} applyMedia={applyMedia}/>;
  if(target==='display')return <main className="layout-main"><div className="workspace-heading"><div><p className="eyebrow">YOUR CANVAS</p><h1>{t.layout}</h1></div></div>{tabs}<p className="muted">Choose a display in Settings → Display output first. It then gets its own layout, separate from the stream.</p></main>;
  return <LayoutBuilder key="stream" target="stream" tabs={tabs} project={project} initial={project.layout} note="NDI and Syphon send exactly this size, so the frame your receiver gets is the canvas you see here." apply={apply} applyMedia={applyMedia}/>;
}
function LayoutBuilder({target,tabs,project,initial,backdrop,note,apply,applyMedia}:{target:Target;tabs:React.ReactNode;project:Project;initial:Layout;backdrop?:string;note:string;apply:(layout:Layout)=>void;applyMedia:(media:MediaAsset[])=>void}) {
  const t=project.settings.language==='es'?es:en;
  const [layout,setLayout]=useState<Layout>(()=>{const l=structuredClone(initial);if(!l.mediaLayers)l.mediaLayers=[];return l;}),[selected,setSelected]=useState(layout.regions[0].id),[error,setError]=useState(''),[applied,setApplied]=useState(false);
  const [mediaSelected,setMediaSelected]=useState<string|null>(layout.mediaLayers?.[0]?.id??null);
  const [devices,setDevices]=useState<AudioDevice[]>([]);
  const undo=useRef<Layout[]>([]),redo=useRef<Layout[]>([]);
  const guideKey=`selah.guide.${target}`;
  const [guide,setGuide]=useState<Guide>(()=>{try{const g=JSON.parse(localStorage.getItem(guideKey)??'null');if(g&&typeof g.opacity==='number')return {src:typeof g.src==='string'?g.src:null,opacity:g.opacity,grid:Boolean(g.grid)};}catch{/* Guide stays session-only without storage. */}return {src:null,opacity:.5,grid:false};});
  useEffect(()=>{try{localStorage.setItem(guideKey,JSON.stringify(guide));}catch{/* Storage can be full or blocked; the guide still works for this session. */}},[guide,guideKey]);
  useEffect(()=>{listAudioOutputs().then(setDevices).catch(()=>setDevices([]));},[]);
  const loadGuide=(file?:File)=>{if(!file)return;if(!file.type.startsWith('image/')||file.size>GUIDE_LIMIT){setError('Choose an image under 8 MB for the guide.');return;}const reader=new FileReader();reader.onload=()=>setGuide(g=>({...g,src:String(reader.result)}));reader.readAsDataURL(file);};
  const region=layout.regions.find(r=>r.id===selected)??layout.regions[0];
  const mediaLayer=(layout.mediaLayers??[]).find(m=>m.id===mediaSelected)??null;
  const remember=()=>{undo.current=[...undo.current.slice(-49),layout];redo.current=[];};
  const update=(next:Layout,record=true)=>{if(record)remember();setLayout(next);setApplied(false);setError('');};
  const stepHistory=(from:typeof undo,to:typeof undo)=>{const previous=from.current.pop();if(!previous)return;to.current.push(layout);setLayout(previous);setApplied(false);if(!previous.regions.some(r=>r.id===selected))setSelected(previous.regions[0].id);};
  const duplicate=(id:string):string|undefined=>{const source=layout.regions.find(r=>r.id===id);if(!source||layout.regions.length>=32)return undefined;const copy={...source,id:crypto.randomUUID(),name:`${source.name} copy`};update({...layout,regions:[...layout.regions,copy]});setSelected(copy.id);return copy.id;};
  const transform=(id:string,rect:Rect)=>update({...layout,regions:layout.regions.map(r=>r.id===id?{...r,...rect}:r)},false);
  const keys=useRef<(e:KeyboardEvent)=>void>(()=>{});
  keys.current=e=>{
    if((e.target as HTMLElement).closest('input,textarea,select,[contenteditable]')||document.querySelector('dialog[open]'))return;
    const mod=e.metaKey||e.ctrlKey,key=e.key.toLowerCase();
    if(mod&&key==='z'){e.preventDefault();if(e.shiftKey)stepHistory(redo,undo);else stepHistory(undo,redo);return;}
    if(mod&&key==='y'){e.preventDefault();stepHistory(redo,undo);return;}
    if(mod&&key==='d'){e.preventDefault();duplicate(selected);return;}
    if((key==='delete'||key==='backspace')&&layout.regions.length>1){e.preventDefault();const regions=layout.regions.filter(r=>r.id!==selected);update({...layout,regions});setSelected(regions[0].id);return;}
    const arrows:Record<string,[number,number]>={arrowleft:[-1,0],arrowright:[1,0],arrowup:[0,-1],arrowdown:[0,1]};
    const move=arrows[key];if(move&&!mod){e.preventDefault();const step=e.shiftKey?10:1;const r=layout.regions.find(x=>x.id===selected);if(!r)return;update({...layout,regions:layout.regions.map(x=>x.id===r.id?{...x,x:Math.max(0,Math.min(layout.width-x.width,x.x+move[0]*step)),y:Math.max(0,Math.min(layout.height-x.height,x.y+move[1]*step))}:x)});}
  };
  useEffect(()=>{const h=(e:KeyboardEvent)=>keys.current(e);window.addEventListener('keydown',h);return()=>window.removeEventListener('keydown',h);},[]);
  const change=(patch:Partial<Region>)=>update({...layout,regions:layout.regions.map(r=>r.id===region.id?{...r,...patch}:r)});
  const changeMedia=(patch:Partial<MediaLayer>)=>{if(!mediaLayer)return;update(upsertMediaLayer(layout,{...mediaLayer,...patch}));};
  const addMedia=async(role:MediaLayer['role'])=>{
    setError('');
    try{
      const {project:next,asset}=await importMedia(project);
      applyMedia(next.media??[]);
      const layer=fullBleedLayer(role,asset.kind,asset.id,layout,asset.name);
      const layers=layout.mediaLayers??[];
      if(layers.filter(l=>l.role===role).length>=2){setError(`Use at most two ${role} media layers.`);return;}
      update(upsertMediaLayer(layout,layer));
      setMediaSelected(layer.id);
    }catch(e){if(String(e)!=='Error: No file selected')setError(String(e));}
  };
  const sample:Block={id:'sample',section:'Preview',primary:project.library[0]?.blocks[0]?.primary??'A little space for worship',secondary:'Un espacio para adorar',reference:'Psalm 96:1 · WEB'};
  const number=(key:keyof Region,label:string,min=0,max=8192)=><label>{label}<input type="number" min={min} max={max} value={Number(region[key])} onChange={e=>change({[key]:Number(e.target.value)})}/></label>;
  const mediaNumber=(key:'x'|'y'|'width'|'height',label:string)=><label>{label}<input type="number" min={0} max={8192} value={mediaLayer?Math.round(mediaLayer[key]):0} onChange={e=>changeMedia({[key]:Number(e.target.value)})}/></label>;
  return <div className="layout-workspace"><aside className="layout-sidebar"><p className="eyebrow">{t.preset}</p>{[['center',t.center,AlignCenter],['lower',t.lower,Layers],['paired',t.paired,Copy]].map(([id,label,Icon])=><button key={String(id)} className="preset-card" onClick={()=>{const l=preset(String(id));update({...scaleLayout(l,layout.width,layout.height),mediaLayers:layout.mediaLayers??[]});setSelected('primary');}}>{typeof Icon!=='string'&&<Icon size={20}/>}<span>{String(label)}</span><div className={`preset-mini ${id}`}><i/>{id==='paired'&&<i/>}</div></button>)}
    <p className="eyebrow region-heading">MEDIA LAYERS</p>
    <div className="guide-controls">
      <button className="button small full" onClick={()=>addMedia('background')}><ImageIcon size={15}/>Add background media</button>
      <button className="button small full" onClick={()=>addMedia('foreground')}><Film size={15}/>Add foreground media</button>
      <small className="hint">Background sits behind lyrics. Foreground sits in front. Enable “Send media on NDI/Syphon” in Settings to include them in the stream.</small>
      {(layout.mediaLayers??[]).map(m=><button key={m.id} className={`region-row ${m.id===mediaSelected?'active':''}`} onClick={()=>setMediaSelected(m.id)}><span className="region-swatch" style={{background:m.role==='background'?'#4a6b8a':'#8a6a4a'}}/>{m.name}<small style={{marginLeft:'auto',opacity:.7}}>{m.role==='background'?'BG':'FG'}</small></button>)}
    </div>
    <p className="eyebrow region-heading">GUIDE · EDITOR ONLY</p><div className="guide-controls"><label className="button small full file-label"><ImagePlus size={15}/>{guide.src?'Replace guide image':'Add guide image'}<input type="file" accept="image/*" onChange={e=>{loadGuide(e.target.files?.[0]);e.target.value='';}}/></label>{guide.src&&<><label>Guide opacity<input type="range" min="0.1" max="1" step="0.05" value={guide.opacity} onChange={e=>setGuide({...guide,opacity:Number(e.target.value)})}/></label><button className="button small full" onClick={()=>setGuide({...guide,src:null})}>Remove image</button></>}<label className="check-label"><input type="checkbox" checked={guide.grid} onChange={e=>setGuide({...guide,grid:e.target.checked})}/><Grid3x3 size={14}/>Placement grid</label><small className="hint">The guide never appears in the output.</small></div><p className="eyebrow region-heading">{t.regions}</p>{layout.regions.map(r=><button className={`region-row ${r.id===region.id?'active':''}`} key={r.id} onClick={()=>{setSelected(r.id);setMediaSelected(null);}}><span className="region-swatch" style={{background:r.color}}/>{r.name}</button>)}<button className="button small full" onClick={()=>{const id=crypto.randomUUID();update({...layout,regions:[...layout.regions,{...region,id,name:`Region ${layout.regions.length+1}`}]});setSelected(id);}} disabled={layout.regions.length>=32}><Plus size={15}/>{t.newRegion}</button></aside>
    <main className="layout-main"><div className="workspace-heading"><div><p className="eyebrow">YOUR CANVAS</p><h1>{t.layout}</h1><p className="muted">{t.layoutHint}</p></div><div className="heading-actions"><button className="icon-button" aria-label="Undo" title="Undo (Ctrl/Cmd+Z)" onClick={()=>stepHistory(undo,redo)}><Undo2 size={16}/></button><button className="icon-button" aria-label="Redo" title="Redo (Shift+Ctrl/Cmd+Z)" onClick={()=>stepHistory(redo,undo)}><Redo2 size={16}/></button><button className="button primary" onClick={()=>{try{validateLayout(layout);apply(layout);setApplied(true);}catch(e){setError(String(e));}}}><Check size={17}/>{applied?t.saved:t.apply}</button></div></div>{tabs}<div className="layout-canvas-wrap"><div className="canvas-ruler"><span>0</span><span>{Math.round(layout.width/2)} px</span><span>{layout.width} px</span></div><div className="canvas-backdrop" style={backdrop?{background:backdrop}:undefined}><MediaStack layout={layout} media={project.media??[]}><Canvas layout={layout} block={sample} editable selected={region.id} onSelect={id=>{setSelected(id);setMediaSelected(null);}} onTransform={transform} onDuplicate={duplicate} onGesture={remember} guide={guide}/></MediaStack></div><div className="canvas-meta"><span><Move size={14}/>Drag to move · handles to resize · Alt-drag to duplicate · Ctrl/Cmd disables snap · Shift keeps ratio</span><span>{layout.width} × {layout.height} · {backdrop?backdrop.toUpperCase():'Transparent'}</span></div><p className="output-size-note"><Radio size={13}/>{note}</p></div>{error&&<p className="error" role="alert">{error}</p>}{overflowRegions(layout,sample).length>0&&<p className="overflow-warning">{t.overflow}: {overflowRegions(layout,sample).join(', ')}</p>}<div className="layout-note"><Layers size={20}/><div><strong>One canvas. As many places as you need.</strong><p>Each region can repeat the lyrics, show a second language, or display a Scripture reference. Media layers sit behind or in front of that text.</p></div></div></main>
    <aside className="inspector">{mediaLayer?<>
      <p className="eyebrow">MEDIA LAYER</p>
      <label>Name<input value={mediaLayer.name} onChange={e=>changeMedia({name:e.target.value})}/></label>
      <label>Role<select value={mediaLayer.role} onChange={e=>changeMedia({role:e.target.value as MediaLayer['role']})}><option value="background">Background (behind text)</option><option value="foreground">Foreground (in front)</option></select></label>
      <div className="form-grid">{mediaNumber('x','X')}{mediaNumber('y','Y')}{mediaNumber('width','Width')}{mediaNumber('height','Height')}</div>
      <label>Opacity<input type="range" min="0" max="1" step="0.05" value={mediaLayer.opacity} onChange={e=>changeMedia({opacity:Number(e.target.value)})}/></label>
      <label>Fit<select value={mediaLayer.fit??'cover'} onChange={e=>changeMedia({fit:e.target.value as MediaLayer['fit']})}><option value="cover">Cover</option><option value="contain">Contain</option><option value="fill">Fill</option></select></label>
      {mediaLayer.kind==='video'&&<>
        <label className="check-label"><input type="checkbox" checked={mediaLayer.muted} onChange={e=>changeMedia({muted:e.target.checked})}/>Mute video audio</label>
        <label>Audio output<select value={mediaLayer.audioDeviceId??''} onChange={e=>changeMedia({audioDeviceId:e.target.value||null})} disabled={mediaLayer.muted}><option value="">System default</option>{devices.map(d=><option key={d.id} value={d.id}>{d.label}</option>)}</select></label>
      </>}
      <button className="button danger small full" onClick={()=>{update(removeMediaLayer(layout,mediaLayer.id));setMediaSelected((layout.mediaLayers??[]).find(m=>m.id!==mediaLayer.id)?.id??null);}}><Trash2 size={14}/>Remove media layer</button>
    </>:<>
      <p className="eyebrow">{t.dimensions}</p><div className="form-grid"><label>Width<input type="number" min="64" max="8192" value={layout.width} onChange={e=>{const v=Number(e.target.value);if(v>=64&&v<=8192)update(scaleLayout(layout,v,layout.height));}}/></label><label>Height<input type="number" min="64" max="8192" value={layout.height} onChange={e=>{const v=Number(e.target.value);if(v>=64&&v<=8192)update(scaleLayout(layout,layout.width,v));}}/></label></div><hr/><p className="eyebrow">{t.properties}</p><label>Name<input value={region.name} onChange={e=>change({name:e.target.value})}/></label><label>Content<select value={region.binding} onChange={e=>change({binding:e.target.value as Region['binding']})}><option value="primary">{t.primary}</option><option value="secondary">{t.secondary}</option><option value="reference">{t.reference}</option></select></label><div className="form-grid">{number('x','X')}{number('y','Y')}{number('width','Width',1)}{number('height','Height',1)}</div><hr/><label>Font<select value={region.fontFamily} onChange={e=>change({fontFamily:e.target.value})}>{['Arial','Georgia','Verdana','Times New Roman'].map(f=><option key={f}>{f}</option>)}</select></label><div className="form-grid">{number('fontSize','Size',8,512)}<label>Color<input type="color" value={region.color} onChange={e=>change({color:e.target.value})}/></label></div><label>Alignment<select value={region.align} onChange={e=>change({align:e.target.value as Region['align']})}><option value="left">Left</option><option value="center">Center</option><option value="right">Right</option></select></label>{number('outline','Outline',0,20)}<label>Text animation<select value={region.anim??''} onChange={e=>change({anim:e.target.value||null})}><option value="">{t.inherit}</option>{TEXT_OPTIONS.map(([id,name])=><option key={id} value={id}>{name}</option>)}</select></label><label className="check-label"><input type="checkbox" checked={region.shadow} onChange={e=>change({shadow:e.target.checked})}/>Soft shadow</label><button className="button danger small full" disabled={layout.regions.length===1} onClick={()=>{const regions=layout.regions.filter(r=>r.id!==region.id);update({...layout,regions});setSelected(regions[0].id);}}><Trash2 size={14}/>{t.remove}</button>
    </>}</aside>
  </div>;
}
