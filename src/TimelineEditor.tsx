import type { Song, Live, ControlStatus, PresentationCommand } from './types';
export default function TimelineEditor({song,live,status,enqueue,change}:{song?:Song;live:Live;status:ControlStatus;enqueue:(command:PresentationCommand,origin:'timeline')=>void;change:(song:Song)=>void}) {
  if(!song)return null;
  const cues=song.cues??[], max=Math.max(300,status.timeFrames,...cues.map(c=>c.timeFrames));
  return <section className="timeline-editor"><div className="lyrics-toolbar"><strong>Timeline</strong><span className="connection-pill">{status.armedSource.toUpperCase()} · {status.chasePaused?'Paused':'Ready'}</span></div>
    {status.chasePaused&&<div className="notice">Chase paused after manual control. <button className="button primary small" onClick={()=>enqueue({type:'chaseResync'},'timeline')}>Re-sync</button></div>}
    <div className="timeline-transport"><button className="button small" onClick={()=>enqueue({type:status.timelinePlaying?'timelinePause':'timelinePlay'},'timeline')}>{status.timelinePlaying?'Pause':'Play'}</button><strong>{status.timeFrames} frames</strong><button className="button small" disabled={live.songId!==song.id||live.blockIndex===null} onClick={()=>change({...song,cues:[...cues,{id:crypto.randomUUID(),timeFrames:status.timeFrames,command:{type:'go',songId:song.id,blockIndex:live.blockIndex!}}]})}>Tap set-cue · current block</button></div>
    <label>Scrub playhead<input aria-label="Scrub playhead" type="range" min="0" max={max} value={status.timeFrames} onChange={e=>enqueue({type:'timelineSeek',timeFrames:Number(e.target.value)},'timeline')}/></label><div className="timeline-cues">{cues.map(c=><span key={c.id} className="badge">{c.timeFrames} · {c.command.type}<button aria-label={`Remove cue at ${c.timeFrames}`} onClick={()=>change({...song,cues:cues.filter(x=>x.id!==c.id)})}> ×</button></span>)}</div>
  </section>;
}
