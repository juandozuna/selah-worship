import { useEffect, useRef, useState } from 'react';
import { ExternalLink, Globe, LoaderCircle, WandSparkles } from 'lucide-react';
import { desktop, invoke } from './bridge';
import { SongSearch, SongSearchCache, songQuery } from './songSearch';
import type { Candidate } from './types';
import { en, es } from './i18n';

const cache=new SongSearchCache(query=>invoke<Candidate[]>('search_lyrics',{query}));
export function LinesPerBlock({value,change,disabled=false,lang}:{value:number;change:(n:number)=>void;disabled?:boolean;lang:'en'|'es'}) {
  const t=lang==='es'?es:en;
  return <label>{t.linesPerBlock}<select aria-label={t.linesPerBlock} value={value} disabled={disabled} onChange={e=>change(Number(e.target.value))}>{[1,2,3,4,5,6,7,8].map(n=><option key={n}>{n}</option>)}</select></label>;
}
export default function SongWebSearch({query,hasLocal,choose,lines,changeLines,lang,disabled=false}:{query:string;hasLocal:boolean;choose:(candidate:Candidate)=>void;lines:number;changeLines:(n:number)=>void;lang:'en'|'es';disabled?:boolean}) {
  const t=lang==='es'?es:en;
  const [state,setState]=useState({query:'',results:[] as Candidate[],busy:false,checked:false,error:''});
  const search=useRef<SongSearch|null>(null);
  useEffect(()=>{const instance=new SongSearch(cache,setState);search.current=instance;return()=>{instance.dispose();search.current=null;};},[]);
  useEffect(()=>{search.current?.configure(query,hasLocal,desktop()&&!disabled);},[query,hasLocal,disabled]);
  const current=state.query===songQuery(query)?state:{query:'',results:[],busy:false,checked:false,error:''};
  return <section className="song-web-search">
    <div className="organize-row"><LinesPerBlock value={lines} change={changeLines} disabled={disabled} lang={lang}/><button className="button small" disabled={!desktop()||songQuery(query).length<2||current.busy||disabled} onClick={()=>search.current?.searchNow(query)}><Globe size={15}/>{t.web}</button></div>
    <p className="hint" role="status">{current.busy?<><LoaderCircle size={13} className="spin"/>{t.autoSearchBusy}</>:current.error?t.autoSearchFailed:current.checked&&current.results.length===0?t.autoSearchEmpty:t.autoSearchHint}</p>
    {current.error&&<p className="error" role="alert">{current.error}<br/>{t.autoSearchKeyHint}</p>}
    {current.results.length>0&&<><p className="eyebrow">{t.webSources}</p>{current.results.map(result=><div className="web-result" key={result.url}><a href={result.url} target="_blank" rel="noreferrer">{result.title}<ExternalLink size={13}/></a><p>{result.description}</p><button className="button small" disabled={disabled} onClick={()=>choose(result)}><WandSparkles size={15}/>{t.prepareSong}</button></div>)}</>}
  </section>;
}
