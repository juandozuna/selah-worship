import { useEffect, useRef, useState } from 'react';
import type { Block, Layout, Motion, Region } from './types';
import { desktop, invoke } from './bridge';
import { composeTransition, ease, paintBlock, textAnimTotalMs, TEXT_START_FRACTION, wrapText } from './motion';
export { wrapText };
const frameKey=(layout:Layout,block:Block|null,testPattern:boolean)=>JSON.stringify([layout,block&&[block.id,block.primary,block.secondary,block.reference],testPattern]);
export function Canvas({layout,block,testPattern=false,editable=false,selected,onSelect,onTransform,onDuplicate,onGesture,guide,motion,animate=false}:{layout:Layout;block:Block|null;testPattern?:boolean;editable?:boolean;selected?:string;onSelect?:(id:string)=>void;onTransform?:(id:string,rect:Rect)=>void;onDuplicate?:(id:string)=>string|undefined;onGesture?:()=>void;guide?:Guide;motion?:Motion|null;animate?:boolean}) {
  const canvas=useRef<HTMLCanvasElement>(null);
  const animating=useRef(false),lastKey=useRef(''),[settled,setSettled]=useState(0);
  useEffect(()=>{const redraw=()=>setSettled(n=>n+1);window.addEventListener('selah:fonts-loaded',redraw);return()=>window.removeEventListener('selah:fonts-loaded',redraw);},[]);
  useEffect(()=>{
    const c=canvas.current;if(!c)return;
    const key=frameKey(layout,block,testPattern),changed=key!==lastKey.current,first=lastKey.current==='';lastKey.current=key;
    const transition=motion?.transition,textTotal=textAnimTotalMs(layout,block,motion);
    const wants=animate&&!editable&&changed&&!first&&!testPattern&&motion&&((transition&&transition.kind!=='cut')||textTotal>0)&&c.width===layout.width&&c.height===layout.height;
    // Capture what is on screen now (even mid-transition) before resizing clears the canvas.
    let from:HTMLCanvasElement|null=null;
    if(wants){from=document.createElement('canvas');from.width=c.width;from.height=c.height;from.getContext('2d')!.drawImage(c,0,0);}
    c.width=layout.width;c.height=layout.height;const ctx=c.getContext('2d');if(!ctx)return;
    if(!from||!motion||!transition){animating.current=false;paintBlock(ctx,layout,block,testPattern,motion,null);return;}
    const toStatic=document.createElement('canvas');toStatic.width=c.width;toStatic.height=c.height;paintBlock(toStatic.getContext('2d')!,layout,block,false,motion,null);
    const toLive=document.createElement('canvas');toLive.width=c.width;toLive.height=c.height;const liveCtx=toLive.getContext('2d')!;
    const transitionMs=transition.kind==='cut'?0:transition.durationMs,textStart=transitionMs*TEXT_START_FRACTION,total=Math.max(transitionMs,textStart+textTotal);
    const start=performance.now();let raf=0,alive=true;animating.current=true;
    const tick=(now:number)=>{
      if(!alive)return;const el=now-start;
      if(el>=total){animating.current=false;paintBlock(ctx,layout,block,false,motion,null);setSettled(n=>n+1);return;}
      const textMs=el-textStart;let to:CanvasImageSource=toStatic;
      if(textTotal>0&&textMs<textTotal){paintBlock(liveCtx,layout,block,false,motion,Math.max(0,textMs));to=toLive;}
      if(el<transitionMs)composeTransition(ctx,from!,to,transition,ease(transition.easing,el/transitionMs));
      else{ctx.clearRect(0,0,c.width,c.height);ctx.drawImage(to,0,0);}
      raf=requestAnimationFrame(tick);
    };
    raf=requestAnimationFrame(tick);
    return()=>{alive=false;cancelAnimationFrame(raf);animating.current=false;};
  },[layout,block,testPattern,motion,animate,editable,settled]);
  useEffect(()=>{
    if(!desktop()||animating.current) return;
    let cancelled=false;
    const timer=setTimeout(()=>{
      invoke<string>('render_preview',{live:{songId:null,blockIndex:null,block,layout,testPattern}}).then(url=>{
        if(cancelled)return;const image=new Image();image.onload=()=>{if(cancelled)return;const c=canvas.current,ctx=c?.getContext('2d');if(c&&ctx){ctx.clearRect(0,0,c.width,c.height);ctx.drawImage(image,0,0,c.width,c.height);}};image.src=url;
      }).catch(()=>{ /* The main live command reports render errors; local previews remain usable. */ });
    },editable?150:0);
    return()=>{cancelled=true;clearTimeout(timer);};
  },[layout,block,testPattern,editable,settled]);
  const drag=useRef<Drag|null>(null);
  const begin=(e:React.PointerEvent<HTMLElement>,r:Region,mode:Mode)=>{
    if(e.button!==0)return;e.stopPropagation();
    const box=(e.currentTarget.closest('.canvas-surface') as HTMLElement).getBoundingClientRect();
    let id=r.id;if(mode==='move'&&e.altKey&&onDuplicate)id=onDuplicate(r.id)??r.id;else onSelect?.(r.id);
    onGesture?.();
    drag.current={id,mode,x:r.x,y:r.y,w:r.width,h:r.height,startX:e.clientX,startY:e.clientY,rect:box};e.currentTarget.setPointerCapture(e.pointerId);
  };
  const track=(e:React.PointerEvent<HTMLElement>)=>{
    const d=drag.current;if(!d||!onTransform)return;
    const scale=layout.width/d.rect.width,snap=e.metaKey||e.ctrlKey?1:8,q=(v:number)=>Math.round(v/snap)*snap;
    const dx=(e.clientX-d.startX)*scale,dy=(e.clientY-d.startY)*scale;
    if(d.mode==='move'){onTransform(d.id,{x:clamp(q(d.x+dx),0,layout.width-d.w),y:clamp(q(d.y+dy),0,layout.height-d.h),width:d.w,height:d.h});return;}
    let left=d.x,top=d.y,right=d.x+d.w,bottom=d.y+d.h;
    if(d.mode.includes('w'))left=clamp(q(d.x+dx),0,right-MIN);
    if(d.mode.includes('e'))right=clamp(q(right+dx),left+MIN,layout.width);
    if(d.mode.includes('n'))top=clamp(q(d.y+dy),0,bottom-MIN);
    if(d.mode.includes('s'))bottom=clamp(q(bottom+dy),top+MIN,layout.height);
    if(e.shiftKey&&d.mode.length===2){ // Shift on a corner keeps the original aspect ratio.
      const ratio=d.w/d.h;let w=right-left,h=w/ratio;if(h>bottom-top){h=bottom-top;w=h*ratio;}
      if(d.mode.includes('w'))left=right-w;else right=left+w;if(d.mode.includes('n'))top=bottom-h;else bottom=top+h;
    }
    onTransform(d.id,{x:Math.round(left),y:Math.round(top),width:Math.round(right-left),height:Math.round(bottom-top)});
  };
  const end=()=>{drag.current=null;};
  return <div className={`canvas-surface ${editable?'editing':''}`} style={{aspectRatio:`${layout.width}/${layout.height}`}}>
    {editable&&guide?.src&&<img className="guide-image" src={guide.src} alt="" draggable={false} style={{opacity:guide.opacity}}/>}
    <canvas ref={canvas} aria-label={block?.primary||'Transparent output canvas'}/>
    {editable&&guide?.grid&&<div className="guide-grid" aria-hidden="true">{['Top third','Middle third','Lower third'].map(n=><span key={n}>{n}</span>)}</div>}
    {editable&&<><div className="safe-area"/>{layout.regions.map(r=><div key={r.id} role="button" tabIndex={0} className={`region-hit ${selected===r.id?'selected':''}`} style={{left:`${r.x/layout.width*100}%`,top:`${r.y/layout.height*100}%`,width:`${r.width/layout.width*100}%`,height:`${r.height/layout.height*100}%`}} onPointerDown={e=>begin(e,r,'move')} onPointerMove={track} onPointerUp={end} onPointerCancel={end} onFocus={()=>onSelect?.(r.id)} aria-label={`Move ${r.name}. Alt-drag to duplicate.`}><span>{r.name} · {Math.round(r.width)} × {Math.round(r.height)}</span>{selected===r.id&&HANDLES.map(h=><i key={h} className={`handle ${h}`} onPointerDown={e=>begin(e,r,h)} onPointerMove={track} onPointerUp={end} onPointerCancel={end}/>)}</div>)}</>}
  </div>;
}
const MIN=32;
const HANDLES=['nw','n','ne','e','se','s','sw','w'] as const;
type Mode='move'|typeof HANDLES[number];
type Drag={id:string;mode:Mode;x:number;y:number;w:number;h:number;startX:number;startY:number;rect:DOMRect};
const clamp=(v:number,lo:number,hi:number)=>Math.max(lo,Math.min(hi,v));
export type Guide={src:string|null;opacity:number;grid:boolean};
export type Rect={x:number;y:number;width:number;height:number};
export function overflowRegions(layout:Layout,block:Block):string[] {
  const ctx=document.createElement('canvas').getContext('2d');if(!ctx)return [];
  return layout.regions.filter(r=>{const text=block[r.binding];if(!text)return false;ctx.font=`600 ${r.fontSize}px "${r.fontFamily}"`;const width=r.width-r.outline*2-8;const lines=wrapText(ctx,text,width);return lines.length*r.fontSize*1.25>r.height||lines.some(line=>ctx.measureText(line).width>width);}).map(r=>r.name);
}
