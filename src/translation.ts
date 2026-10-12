import type { Layout, Song } from './types';

export const translationLanguages = [
  ['en','English'],['es','Español'],['pt','Português'],['fr','Français'],
  ['de','Deutsch'],['it','Italiano'],['ko','한국어'],['ja','日本語'],['zh','中文'],
] as const;
type Plan = { song: Song; index: number; target: string };
export type TranslationStatus = { text: string|null; busy: boolean; ready: number; total: number; error: string; sameLanguage: boolean };
const key = (text:string,target:string) => JSON.stringify([target,text]);

/** Only the on-air song is queued. A new song replaces pending work, with one request in flight. */
export class LiveTranslator {
  private plan: Plan|null = null;
  private cache = new Map<string,string>();
  private errors = new Map<string,string>();
  private running = false;
  private signature = "";
  private attempted = new Set<string>();
  constructor(private request:(text:string,target:string)=>Promise<string>, private changed:()=>void, private limit=300) {}
  configure(song:Song|undefined,index:number|null,target:string,enabled:boolean) {
    this.plan = enabled && song?.kind==='song' && index!==null && song.blocks[index]
      ? {song:structuredClone(song),index,target} : null;
    const signature=this.plan?JSON.stringify([this.plan.song.id,target,this.plan.song.blocks.map(b=>b.primary)]):'';
    if(signature!==this.signature){this.signature=signature;this.attempted.clear();}
    if(this.plan){const k=key(this.plan.song.blocks[this.plan.index].primary,target);if(!this.cache.has(k))this.attempted.delete(k);}
    this.changed();
    void this.pump();
  }
  lookup(text:string,target:string):string|null { return this.cache.get(key(text,target))??null; }
  status():TranslationStatus {
    const p=this.plan;
    if(!p)return {text:null,busy:false,ready:0,total:0,error:'',sameLanguage:false};
    const sameLanguage=p.song.language.toLowerCase().split('-')[0]===p.target;
    const source=p.song.blocks[p.index].primary;
    return {text:this.lookup(source,p.target),busy:this.running,ready:p.song.blocks.filter(b=>this.cache.has(key(b.primary,p.target))).length,total:p.song.blocks.length,error:this.errors.get(key(source,p.target))??'',sameLanguage};
  }
  retry() {
    if(this.plan)for(const b of this.plan.song.blocks){const k=key(b.primary,this.plan.target);this.errors.delete(k);this.attempted.delete(k);}
    void this.pump();
  }
  private async pump() {
    if(this.running)return;
    while(this.plan) {
      const p=this.plan;
      if(p.song.language.toLowerCase().split('-')[0]===p.target)break;
      const order=[p.song.blocks[p.index],...p.song.blocks.slice(p.index+1),...p.song.blocks.slice(0,p.index)];
      const block=order.find(b=>!this.cache.has(key(b.primary,p.target))&&!this.errors.has(key(b.primary,p.target))&&!this.attempted.has(key(b.primary,p.target)));
      if(!block)break;
      const k=key(block.primary,p.target);
      this.attempted.add(k);
      this.running=true;this.changed();
      try {
        const text=await this.request(block.primary,p.target);
        if(typeof text!=='string'||!text.trim())throw new Error('AI returned an empty translation.');
        this.cache.set(k,text);
        while(this.cache.size>this.limit){
          const active=this.plan?key(this.plan.song.blocks[this.plan.index].primary,this.plan.target):'';
          const oldest=[...this.cache.keys()].find(k=>k!==active)??this.cache.keys().next().value!;
          this.cache.delete(oldest);
        }
      } catch(e) {
        this.errors.set(k,String(e));
        while(this.errors.size>this.limit)this.errors.delete(this.errors.keys().next().value!);
      }
      this.running=false;this.changed();
      // Re-read the plan after every await; never continue preloading a song that went off air.
    }
    this.changed();
  }
}

/** Adapt the live output only; saved layouts and manually paired lyrics remain untouched. */
export function translationLayout(layout:Layout,enabled:boolean):Layout {
  if(!enabled||layout.regions.some(r=>r.binding==='secondary'))return layout;
  const primary=layout.regions.find(r=>r.binding==='primary');
  if(!primary||layout.regions.length>=32)return layout;
  const height=primary.height*.55;
  let id='ai-translation';while(layout.regions.some(r=>r.id===id))id+='-';
  return {...layout,regions:[...layout.regions.map(r=>r===primary?{...r,height,fontSize:Math.max(8,r.fontSize*.8)}:r),{
    ...primary,id,name:'AI translation',binding:'secondary',y:primary.y+height,height:primary.height-height,
    fontSize:Math.max(8,primary.fontSize*.6),color:'#d6dfd4',
  }]};
}
