import { useEffect, useState } from 'react';
import type { DisplayOutput, Live } from './types';
import { invoke } from './bridge';
import { Canvas } from './Canvas';
import { MediaStack } from './MediaStack';
import { translationLayout } from './translation';
type Frame={revision:number;live:Live|null;config:DisplayOutput|null};
/** Full-screen window content: media + lyrics over the display background, following live output. */
export default function DisplayView() {
  const [frame,setFrame]=useState<Frame|null>(null);
  useEffect(()=>{
    document.title='Selah Display';let revision=-1,stopped=false;
    const poll=()=>invoke<Frame>('display_frame').then(f=>{if(!stopped&&f.revision!==revision){revision=f.revision;setFrame(f);}}).catch(()=>{});
    poll();const id=setInterval(poll,100);
    const key=(e:KeyboardEvent)=>{if(e.key==='Escape')invoke('close_display').catch(()=>{});};
    window.addEventListener('keydown',key);
    return()=>{stopped=true;clearInterval(id);window.removeEventListener('keydown',key);};
  },[]);
  const config=frame?.config;
  const live=frame?.live;
  const layout=config?translationLayout(config.layout,Boolean(live?.aiTranslation)):null;
  const mediaLayout=layout?{...layout,mediaLayers:(layout.mediaLayers?.length?layout.mediaLayers:live?.layout.mediaLayers)??[]}:null;
  return <div className="display-root" style={{background:config?.background??"#000000",["--ar" as string]:config?config.layout.width/config.layout.height:1}}>
    {config&&layout&&mediaLayout&&<div className="display-stage">
      <MediaStack layout={mediaLayout} media={live?.media??[]} streamComposite>
        <Canvas layout={layout} block={live?.block??null} testPattern={live?.testPattern??false} motion={live?.motion} animate/>
      </MediaStack>
    </div>}
  </div>;
}
