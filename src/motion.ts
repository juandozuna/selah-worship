import type { Block, Layout, Motion, Region, TextAnimSpec, TransitionSpec } from './types';
// Web twin of crates/renderer (transition.rs and the animated text path). Keep formulas identical.

/** Text animation starts this far into a visible transition so the two overlap (same constant in output.rs). */
export const TEXT_START_FRACTION=.5;

export function ease(kind:string,t:number):number {
  t=Math.min(1,Math.max(0,t));
  switch(kind) {
    case 'ease-in': return t*t*t;
    case 'ease-out': return 1-(1-t)**3;
    case 'ease-in-out': return t<.5?4*t*t*t:1-(-2*t+2)**3/2;
    case 'spring': {const c1=1.70158,c3=2.70158;return 1+c3*(t-1)**3+c1*(t-1)**2;}
    default: return t;
  }
}

export function wrapText(ctx:CanvasRenderingContext2D,text:string,width:number) {const result:string[]=[];for(const paragraph of text.split('\n')){let line='';for(const word of paragraph.split(/\s+/)){if(line&&ctx.measureText(`${line} ${word}`).width>width){result.push(line);line=word;}else line=line?`${line} ${word}`:word;}result.push(line);}return result;}

/** Region override, or the resolved default, with preset kinds expanded to a unit and direction. */
export function effectiveAnim(motion:Motion|null|undefined,r:Region):TextAnimSpec|null {
  if(!motion)return null;
  const spec={...motion.text};
  if(r.anim)spec.kind=r.anim;
  switch(spec.kind) {
    case 'none': return null;
    case 'typewriter': spec.unit='letter';break;
    case 'word-rise': spec.unit='word';spec.direction='up';break;
    case 'line-cascade': spec.unit='line';spec.direction='up';break;
  }
  return spec;
}
export function unitCount(text:string,spec:TextAnimSpec):number {
  switch(spec.unit) {
    case 'line': return Math.max(1,text.split('\n').length);
    case 'word': return Math.max(1,text.split(/\s+/).filter(Boolean).length);
    case 'letter': return Math.max(1,[...text].filter(c=>!/\s/.test(c)).length);
    default: return 1;
  }
}
const animTotal=(spec:TextAnimSpec,text:string)=>Math.max(0,unitCount(text,spec)-1)*spec.staggerMs+spec.durationMs;
export function unitRanks(n:number,order:string):number[] {
  const key=(i:number)=>{
    switch(order) {
      case 'backward': return n-1-i;
      case 'center': return Math.floor(Math.abs(i-(n-1)/2)*2)*1000+i;
      case 'random': return (i*2654435761)%1000003*1000+i;
      default: return i;
    }
  };
  const idx=[...Array(n).keys()].sort((a,b)=>key(a)-key(b));
  const rank=Array<number>(n).fill(0);idx.forEach((i,r)=>{rank[i]=r;});return rank;
}
/** Longest text animation among the layout's regions, in ms. */
export function textAnimTotalMs(layout:Layout,block:Block|null,motion:Motion|null|undefined):number {
  if(!block)return 0;let max=0;
  for(const r of layout.regions){const spec=effectiveAnim(motion,r);const text=block[r.binding]??'';if(spec&&text)max=Math.max(max,animTotal(spec,text));}
  return max;
}

const TEST_COLORS=['#ffffff','#e5ce57','#57cbce','#57ce70','#cd57ce','#ce5757','#5768ce'];
/** Draws one frame of the output. `animMs` is time into the text animation; null draws the settled text. */
export function paintBlock(ctx:CanvasRenderingContext2D,layout:Layout,block:Block|null,testPattern:boolean,motion?:Motion|null,animMs:number|null=null) {
  const c=ctx.canvas;ctx.clearRect(0,0,c.width,c.height);
  if(testPattern){TEST_COLORS.forEach((color,i)=>{ctx.fillStyle=color;ctx.fillRect(i*c.width/7,0,c.width/7,c.height);});ctx.fillStyle='#151917';ctx.font=`bold ${c.width/35}px Arial`;ctx.fillText(`SELAH · ${c.width} × ${c.height}`,c.width*.05,c.height*.9);return;}
  if(!block)return;
  for(const r of layout.regions){
    const text=block[r.binding]??'';if(!text)continue;
    ctx.save();ctx.beginPath();ctx.rect(r.x,r.y,r.width,r.height);ctx.clip();
    ctx.font=`600 ${r.fontSize}px "${r.fontFamily}"`;ctx.textAlign=r.align;ctx.textBaseline='middle';ctx.fillStyle=r.color;ctx.lineJoin='round';
    const lines=wrapText(ctx,text,r.width-r.outline*2-8),lineHeight=r.fontSize*1.25;
    const start=r.y+(r.height-(lines.length-1)*lineHeight)/2;
    const x=r.align==='left'?r.x+4:r.align==='right'?r.x+r.width-4:r.x+r.width/2;
    const spec=animMs===null?null:effectiveAnim(motion,r);
    if(spec&&animMs!==null&&animMs<animTotal(spec,text))paintAnimated(ctx,r,lines,start,lineHeight,spec,animMs);
    else lines.forEach((line,n)=>drawLine(ctx,r,line,x,start+n*lineHeight));
    ctx.restore();
  }
}
function drawLine(ctx:CanvasRenderingContext2D,r:Region,line:string,x:number,y:number) {
  if(r.shadow){ctx.shadowColor='rgba(0,0,0,.65)';ctx.shadowBlur=r.fontSize*.1;ctx.shadowOffsetY=3;}
  if(r.outline){ctx.strokeStyle='#000000';ctx.lineWidth=r.outline*2;ctx.strokeText(line,x,y);}
  ctx.fillText(line,x,y);
}
function paintAnimated(ctx:CanvasRenderingContext2D,r:Region,lines:string[],start:number,lineHeight:number,spec:TextAnimSpec,ms:number) {
  type Unit={text:string;x:number;y:number;index:number};
  const units:Unit[]=[];let counter=0;
  ctx.textAlign='left';
  lines.forEach((line,n)=>{
    const y=start+n*lineHeight,lw=ctx.measureText(line).width;
    const left=r.align==='left'?r.x+4:r.align==='right'?r.x+r.width-4-lw:r.x+r.width/2-lw/2;
    if(spec.unit==='line'){units.push({text:line,x:left,y,index:counter++});return;}
    if(spec.unit==='word'||spec.unit==='letter') {
      const pattern=spec.unit==='word'?/\S+/gu:/\S/gu;
      for(const m of line.matchAll(pattern))units.push({text:m[0],x:left+ctx.measureText(line.slice(0,m.index)).width,y,index:counter++});
      return;
    }
    units.push({text:line,x:left,y,index:0});
  });
  const ranks=unitRanks(spec.unit==='block'?1:Math.max(1,counter),spec.order),dist=r.fontSize*.6;
  const blurOk=typeof (ctx as {filter?:string}).filter==='string';
  for(const u of units){
    const p=Math.min(1,Math.max(0,(ms-ranks[u.index]*spec.staggerMs)/spec.durationMs)),e=ease(spec.easing,p);
    let opacity=Math.min(1,Math.max(0,e)),dx=0,dy=0,blur=0;
    switch(spec.kind) {
      case 'typewriter': opacity=ms>=ranks[u.index]*spec.staggerMs?1:0;break;
      case 'blur': blur=Math.max(0,1-e)*r.fontSize*.25;break;
      case 'fade': break;
      default:
        if(spec.direction==='left')dx=(1-e)*dist;else if(spec.direction==='right')dx=-(1-e)*dist;else if(spec.direction==='down')dy=-(1-e)*dist;else dy=(1-e)*dist;
    }
    if(opacity<=.001)continue;
    ctx.save();ctx.globalAlpha=opacity;ctx.translate(dx,dy);
    if(blur>.5&&blurOk)(ctx as unknown as {filter:string}).filter=`blur(${blur}px)`;
    drawLine(ctx,r,u.text,u.x,u.y);ctx.restore();
  }
}

const scratch:HTMLCanvasElement[]=[];
function temp(i:number,w:number,h:number) {
  let c=scratch[i];if(!c){c=scratch[i]=document.createElement('canvas');}
  if(c.width!==w||c.height!==h){c.width=w;c.height=h;}
  const ctx=c.getContext('2d')!;ctx.setTransform(1,0,0,1,0,0);ctx.globalAlpha=1;ctx.globalCompositeOperation='source-over';ctx.clearRect(0,0,w,h);return {c,ctx};
}
const vec=(d:string):[number,number]=>d==='right'?[1,0]:d==='up'?[0,-1]:d==='down'?[0,1]:[-1,0];
/** Mixes two frames; `t` is already eased progress. Frames are premultiplied so cross-fades keep correct alpha. */
export function composeTransition(ctx:CanvasRenderingContext2D,from:CanvasImageSource,to:CanvasImageSource,spec:TransitionSpec,t:number) {
  const {width:w,height:h}=ctx.canvas;t=Math.min(1,Math.max(0,t));
  const [dx,dy]=vec(spec.direction);
  const draw=(img:CanvasImageSource,alpha=1,x=0,y=0,op:GlobalCompositeOperation='source-over')=>{ctx.globalAlpha=alpha;ctx.globalCompositeOperation=op;ctx.drawImage(img,Math.round(x),Math.round(y));};
  ctx.save();ctx.setTransform(1,0,0,1,0,0);ctx.clearRect(0,0,w,h);
  switch(spec.kind) {
    case 'fade': draw(from,1-t);draw(to,t,0,0,'lighter');break;
    case 'dip': if(t<.5)draw(from,1-2*t);else draw(to,2*t-1);break;
    case 'slide': draw(from);draw(to,1,-dx*w*(1-t),-dy*h*(1-t));break;
    case 'push': draw(from,1,dx*w*t,dy*h*t);draw(to,1,dx*w*t-dx*w,dy*h*t-dy*h,'lighter');break;
    case 'reveal': draw(to);draw(from,1,dx*w*t,dy*h*t);break;
    case 'zoom': {
      const scaled=(img:CanvasImageSource,s:number,alpha:number,op:GlobalCompositeOperation)=>{ctx.globalAlpha=alpha;ctx.globalCompositeOperation=op;ctx.setTransform(s,0,0,s,w/2*(1-s),h/2*(1-s));ctx.drawImage(img,0,0);ctx.setTransform(1,0,0,1,0,0);};
      scaled(from,1+.3*t,1-t,'source-over');scaled(to,.8+.2*t,t,'lighter');break;
    }
    case 'blur': {
      const max=Math.max(w,h)/40,blurOk=typeof (ctx as {filter?:string}).filter==='string';
      if(blurOk)ctx.filter=`blur(${t*max}px)`;draw(from,1-t);if(blurOk)ctx.filter=`blur(${(1-t)*max}px)`;draw(to,t,0,0,'lighter');ctx.filter='none';break;
    }
    case 'wipe': {
      const f=Math.max(spec.feather,.0001),edge=t*(1+f);
      // u runs 0→1 along the direction of travel; mask alpha for the incoming frame is 1 below edge-f, 0 above edge.
      const horizontal=spec.direction==='left'||spec.direction==='right';
      const [x0,y0,x1,y1]=spec.direction==='left'?[w,0,0,0]:spec.direction==='right'?[0,0,w,0]:spec.direction==='up'?[0,h,0,0]:[0,0,0,h];
      const mask=(incoming:boolean)=>{const g=ctx.createLinearGradient(x0,y0,x1,y1);const a=(v:number)=>incoming?v:1-v;const lo=Math.min(1,Math.max(0,edge-f)),hi=Math.min(1,Math.max(0,edge));g.addColorStop(0,`rgba(0,0,0,${a(1)})`);g.addColorStop(lo,`rgba(0,0,0,${a(1)})`);g.addColorStop(hi,`rgba(0,0,0,${a(0)})`);g.addColorStop(1,`rgba(0,0,0,${a(0)})`);return g;};
      void horizontal;
      const layer=(i:number,img:CanvasImageSource,incoming:boolean)=>{const {c,ctx:l}=temp(i,w,h);l.drawImage(img,0,0);l.globalCompositeOperation='destination-in';l.fillStyle=mask(incoming);l.fillRect(0,0,w,h);return c;};
      const a=layer(0,from,false),b=layer(1,to,true);
      draw(a);draw(b,1,0,0,'lighter');break;
    }
    default: draw(to);
  }
  ctx.restore();
}
