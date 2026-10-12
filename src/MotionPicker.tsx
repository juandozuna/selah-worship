import { useEffect, useState } from 'react';
import type { Block, Layout, Motion, MotionOverride, TextAnimSpec, TransitionSpec } from './types';
import { Canvas } from './Canvas';
import { CUT_MOTION } from './model';
import { en, es } from './i18n';

export const TRANSITION_OPTIONS:[string,string,string][]=[
  ['cut','Cut','Instant change'],['fade','Fade','Cross-dissolve'],['dip','Dip','Fade out, then in'],['slide','Slide','New slides over old'],['push','Push','New pushes old out'],
  ['wipe','Wipe','Soft edge sweeps across'],['reveal','Reveal','Old slides away'],['zoom','Zoom','Scale and fade'],['blur','Blur','Defocus dissolve'],
];
export const TEXT_OPTIONS:[string,string,string][]=[
  ['none','None','Text appears with the block'],['fade','Fade in','Whole text, line, word or letter'],['slide','Slide','Drift in from a direction'],['blur','Blur in','Comes into focus'],
  ['typewriter','Typewriter','Letter by letter'],['word-rise','Word rise','Words lift in one by one'],['line-cascade','Line cascade','Lines follow each other'],
];
const EASINGS=[['linear','Linear'],['ease-in','Ease in'],['ease-out','Ease out'],['ease-in-out','Ease in-out'],['spring','Spring']];
const DIRECTIONS=[['left','Left'],['right','Right'],['up','Up'],['down','Down']];
const moves=new Set(['slide','push','reveal','wipe']);
const slider=(label:string,value:number,min:number,max:number,step:number,unit:string,set:(v:number)=>void)=><label>{label} · {value}{unit}<input type="range" min={min} max={max} step={step} value={value} onChange={e=>set(Number(e.target.value))}/></label>;
const select=(label:string,value:string,options:string[][],set:(v:string)=>void)=><label>{label}<select value={value} onChange={e=>set(e.target.value)}>{options.map(([v,n])=><option key={v} value={v}>{n}</option>)}</select></label>;

export function TransitionControls({value,onChange}:{value:TransitionSpec;onChange:(v:TransitionSpec)=>void}) {
  const patch=(p:Partial<TransitionSpec>)=>onChange({...value,...p});
  return <div className="motion-controls"><div className="motion-tiles" role="radiogroup" aria-label="Transition">{TRANSITION_OPTIONS.map(([id,name,hint])=><button key={id} role="radio" aria-checked={value.kind===id} className={`motion-tile ${value.kind===id?'active':''}`} onClick={()=>patch({kind:id})}><strong>{name}</strong><small>{hint}</small></button>)}</div>
    {value.kind!=='cut'&&<div className="form-grid">{slider('Duration',value.durationMs,100,3000,50,' ms',v=>patch({durationMs:v}))}{select('Easing',value.easing,EASINGS,v=>patch({easing:v}))}{moves.has(value.kind)&&select('Direction',value.direction,DIRECTIONS,v=>patch({direction:v}))}{value.kind==='wipe'&&slider('Soft edge',Math.round(value.feather*100),0,50,5,'%',v=>patch({feather:v/100}))}</div>}</div>;
}
export function TextControls({value,onChange}:{value:TextAnimSpec;onChange:(v:TextAnimSpec)=>void}) {
  const patch=(p:Partial<TextAnimSpec>)=>onChange({...value,...p});
  const preset=(id:string)=>patch(id==='typewriter'?{kind:id,unit:'letter',staggerMs:35,durationMs:200}:id==='word-rise'?{kind:id,unit:'word',staggerMs:120,durationMs:450}:id==='line-cascade'?{kind:id,unit:'line',staggerMs:180,durationMs:500}:{kind:id});
  const free=['fade','slide','blur'].includes(value.kind);
  return <div className="motion-controls"><div className="motion-tiles" role="radiogroup" aria-label="Text animation">{TEXT_OPTIONS.map(([id,name,hint])=><button key={id} role="radio" aria-checked={value.kind===id} className={`motion-tile ${value.kind===id?'active':''}`} onClick={()=>preset(id)}><strong>{name}</strong><small>{hint}</small></button>)}</div>
    {value.kind!=='none'&&<div className="form-grid">{free&&select('Animate by',value.unit,[['block','Whole block'],['line','Line'],['word','Word'],['letter','Letter']],v=>patch({unit:v}))}{value.kind!=='typewriter'&&slider('Duration',value.durationMs,100,3000,50,' ms',v=>patch({durationMs:v}))}{value.kind!=='typewriter'&&select('Easing',value.easing,EASINGS,v=>patch({easing:v}))}{value.kind==='slide'&&select('Direction',value.direction,DIRECTIONS,v=>patch({direction:v}))}{(value.unit!=='block'||value.kind==='typewriter')&&slider('Stagger',value.staggerMs,0,500,5,' ms',v=>patch({staggerMs:v}))}{(value.unit!=='block'||value.kind==='typewriter')&&select('Order',value.order,[['forward','Forward'],['backward','Backward'],['center','From center'],['random','Random']],v=>patch({order:v}))}</div>}</div>;
}

const A:Block={id:'a',section:'Verse 1',primary:'Amazing grace! how sweet the sound\nThat saved a wretch like me!',secondary:'Sublime gracia del Señor',reference:null};
const B:Block={id:'b',section:'Verse 2',primary:'’Twas grace that taught my heart to fear,\nAnd grace my fears relieved',secondary:'Su gracia me enseñó a temer',reference:null};
/** Loops between two sample blocks with the real motion driver so the preview matches the output. */
export function MotionPreview({motion,layout}:{motion:Motion;layout:Layout}) {
  const [index,setIndex]=useState(0);
  useEffect(()=>{const id=setInterval(()=>setIndex(i=>i+1),2800+(motion.text.kind==='none'?0:600));return()=>clearInterval(id);},[motion.text.kind]);
  const preview:Layout={...layout,regions:layout.regions.slice(0,2)};
  return <div className="motion-preview"><Canvas layout={preview} block={index%2?B:A} motion={motion} animate/><small>Preview · plays on a loop</small></div>;
}

export function MotionEditor({value,onChange,layout,lang}:{value:Motion;onChange:(m:Motion)=>void;layout:Layout;lang:'en'|'es'}) {
  const t=lang==='es'?es:en;
  return <div className="motion-editor"><MotionPreview motion={value} layout={layout}/><p className="eyebrow">{t.transitionLabel.toUpperCase()}</p><TransitionControls value={value.transition} onChange={transition=>onChange({...value,transition})}/><p className="eyebrow">{t.textAnimLabel.toUpperCase()}</p><TextControls value={value.text} onChange={text=>onChange({...value,text})}/></div>;
}

/** Song or block override: each half inherits until switched on. */
export function MotionOverrideEditor({base,value,onChange,layout,lang}:{base:Motion;value:MotionOverride|null|undefined;onChange:(v:MotionOverride|null)=>void;layout:Layout;lang:'en'|'es'}) {
  const t=lang==='es'?es:en;
  const emit=(next:MotionOverride)=>onChange(next.transition||next.text?next:null);
  const resolved:Motion={transition:value?.transition??base.transition,text:value?.text??base.text};
  return <div className="motion-editor"><MotionPreview motion={resolved} layout={layout}/>
    <label className="check-label"><input type="checkbox" checked={Boolean(value?.transition)} onChange={e=>emit({...value,transition:e.target.checked?{...base.transition,kind:base.transition.kind==='cut'?'fade':base.transition.kind}:null})}/>{t.transitionLabel}: {value?.transition?'custom':t.inherit.toLowerCase()}</label>
    {value?.transition&&<TransitionControls value={value.transition} onChange={transition=>emit({...value,transition})}/>}
    <label className="check-label"><input type="checkbox" checked={Boolean(value?.text)} onChange={e=>emit({...value,text:e.target.checked?{...base.text,kind:base.text.kind==='none'?'fade':base.text.kind}:null})}/>{t.textAnimLabel}: {value?.text?'custom':t.inherit.toLowerCase()}</label>
    {value?.text&&<TextControls value={value.text} onChange={text=>emit({...value,text})}/>}</div>;
}
export const defaultMotion=(m?:Motion)=>m??CUT_MOTION;
