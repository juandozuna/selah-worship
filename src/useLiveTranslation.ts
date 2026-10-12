import { useEffect, useRef, useState } from 'react';
import { invoke } from './bridge';
import type { Settings, Song } from './types';
import { LiveTranslator } from './translation';

export function useLiveTranslation(song:Song|undefined,index:number|null,target:string,enabled:boolean,settings:Settings) {
  const [,update]=useState(0),settingsRef=useRef(settings),mounted=useRef(true);
  settingsRef.current=settings;
  const translator=useRef<LiveTranslator|null>(null);
  if(!translator.current)translator.current=new LiveTranslator(
    (text,target)=>invoke<string>('translate_lyrics',{text,target,settings:settingsRef.current}),
    ()=>{if(mounted.current)update(n=>n+1);},
  );
  const controller=translator.current;
  useEffect(()=>{mounted.current=true;return()=>{mounted.current=false;controller.configure(undefined,null,target,false);};},[controller]);
  useEffect(()=>{controller.configure(song,index,target,enabled);},[controller,song,index,target,enabled]);
  return {controller,status:controller.status()};
}
