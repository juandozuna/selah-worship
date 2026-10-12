import type { Candidate } from './types';
export type SearchState = { query:string; results:Candidate[]; busy:boolean; checked:boolean; error:string };
export const songQuery=(query:string)=>query.trim().replace(/\s+/g,' ');

/** Share successful results and in-flight requests, but allow failures to be retried. */
export class SongSearchCache {
  private cache=new Map<string,{at:number;results:Candidate[]}>();
  private pending=new Map<string,Promise<Candidate[]>>();
  constructor(private request:(query:string)=>Promise<Candidate[]>,private now=()=>Date.now(),private timeoutMs=30000) {}
  search(query:string):Promise<Candidate[]> {
    query=songQuery(query);const key=query.toLowerCase(),cached=this.cache.get(key);
    if(cached&&this.now()-cached.at<300000)return Promise.resolve(cached.results);
    const pending=this.pending.get(key);if(pending)return pending;
    let timeout:ReturnType<typeof setTimeout>;
    const expired=new Promise<Candidate[]>((_,reject)=>{timeout=setTimeout(()=>reject(new Error('Search timed out. Check your connection and approve any macOS Keychain prompt for Selah, then retry.')),this.timeoutMs);});
    const request=Promise.race([Promise.resolve().then(()=>this.request(query)),expired]).then(results=>{
      this.cache.delete(key);this.cache.set(key,{at:this.now(),results});
      while(this.cache.size>30)this.cache.delete(this.cache.keys().next().value!);
      return results;
    }).finally(()=>{clearTimeout(timeout);this.pending.delete(key);});
    this.pending.set(key,request);return request;
  }
}

/** A local match cancels automatic fallback. Late responses never replace the current query. */
export class SongSearch {
  private revision=0;
  private timer:ReturnType<typeof setTimeout>|undefined;
  private disposed=false;
  state:SearchState={query:'',results:[],busy:false,checked:false,error:''};
  constructor(private cache:SongSearchCache,private changed:(state:SearchState)=>void,private delay=600) {}
  configure(query:string,hasLocal:boolean,available:boolean) {
    this.cancel();query=songQuery(query);
    this.publish({query,results:[],busy:false,checked:false,error:''});
    if(query.length<2||hasLocal||!available)return;
    const revision=this.revision;
    this.publish({...this.state,busy:true});
    this.timer=setTimeout(()=>{void this.run(query,revision);},this.delay);
  }
  searchNow(query:string) {
    this.cancel();query=songQuery(query);if(query.length<2)return;
    void this.run(query,this.revision);
  }
  dispose() {this.disposed=true;this.cancel();}
  private cancel(){this.revision++;clearTimeout(this.timer);}
  private publish(state:SearchState){this.state=state;if(!this.disposed)this.changed(state);}
  private async run(query:string,revision:number) {
    this.publish({query,results:[],busy:true,checked:false,error:''});
    try {
      const results=await this.cache.search(query);
      if(revision===this.revision&&!this.disposed)this.publish({query,results,busy:false,checked:true,error:''});
    } catch(e) {
      if(revision===this.revision&&!this.disposed)this.publish({query,results:[],busy:false,checked:true,error:String(e)});
    }
  }
}
