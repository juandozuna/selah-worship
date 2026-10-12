import type { Layout, Project, Song } from './types';
import { splitLyrics } from './model';
export function preset(name:string): Layout {
  const region = {id:'primary',name:'Lyrics',x:192,y:380,width:1536,height:320,fontSize:76,fontFamily:'Arial',color:'#ffffff',align:'center' as const,binding:'primary' as const,outline:0,shadow:true};
  if(name==='lower') return {width:1920,height:1080,regions:[{...region,y:820,height:190,fontSize:64}],mediaLayers:[]};
  if(name==='paired') return {width:1920,height:1080,regions:[{...region,y:300,height:230,fontSize:68},{...region,id:'secondary',name:'English / Español',y:570,height:230,fontSize:56,binding:'secondary',color:'#d6dfd4'}],mediaLayers:[]};
  return {width:1920,height:1080,regions:[region],mediaLayers:[]};
}
function hymn(id:string,title:string,artist:string,language:string,text:string,secondary?:string[]):Song {
  return {id,title,artist,language,kind:'song',source:{name:'Public-domain hymn text',url:null,license:'Public domain; arrangement not included'},blocks:splitLyrics(text).map((b,i)=>({...b,secondary:secondary?.[i]??null}))};
}
export const seed:Project={version:1,name:'Sunday worship',layout:preset('center'),media:[],streamComposite:false,motion:{transition:{kind:'fade',durationMs:400,easing:'ease-in-out',direction:'left',feather:.1},text:{kind:'none',unit:'line',durationMs:500,staggerMs:80,easing:'ease-out',direction:'up',order:'forward'}},settings:{language:'en',feedName:'Selah Lyrics',aiProvider:'local',aiModel:'Qwen3-4B-Instruct-2507',aiEndpoint:'',localServerPath:'',localModelPath:''},service:['amazing-grace','holy-holy','doxology'],library:[
  hymn('amazing-grace','Amazing Grace','John Newton · 1779','en',`[Verse 1]\nAmazing grace! how sweet the sound\nThat saved a wretch like me!\nI once was lost, but now am found,\nWas blind, but now I see.\n\n[Verse 2]\n’Twas grace that taught my heart to fear,\nAnd grace my fears relieved;\nHow precious did that grace appear\nThe hour I first believed!\n\n[Verse 3]\nThrough many dangers, toils and snares,\nI have already come;\n’Tis grace hath brought me safe thus far,\nAnd grace will lead me home.`),
  hymn('holy-holy','Holy, Holy, Holy','Reginald Heber · 1826','en',`[Verse 1]\nHoly, holy, holy! Lord God Almighty!\nEarly in the morning our song shall rise to thee.\nHoly, holy, holy! merciful and mighty!\nGod in three Persons, blessed Trinity!\n\n[Verse 2]\nHoly, holy, holy! all the saints adore thee,\nCasting down their golden crowns around the glassy sea;\nCherubim and seraphim falling down before thee,\nWhich wert, and art, and evermore shalt be.`),
  hymn('doxology','Doxology','Thomas Ken · 1674','en',`[Doxology]\nPraise God, from whom all blessings flow;\nPraise him, all creatures here below;\nPraise him above, ye heavenly host;\nPraise Father, Son, and Holy Ghost.`)
]};
