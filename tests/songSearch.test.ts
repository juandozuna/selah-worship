import { test } from 'node:test';
import assert from 'node:assert/strict';
import { SongSearch, SongSearchCache } from '../src/songSearch';
import type { Candidate } from '../src/types';
const wait=(ms=0)=>new Promise(resolve=>setTimeout(resolve,ms));
const result=(title:string):Candidate=>({title,url:'https://example.com/'+title,description:'Lyrics'});
test('automatic fallback is debounced and skips blank, short, unavailable and local matches',async()=>{
  const calls:string[]=[];
  const search=new SongSearch(new SongSearchCache(async q=>{calls.push(q);return [];}),()=>{},10);
  search.configure('',false,true);search.configure('a',false,true);search.configure('Amazing Grace',true,true);search.configure('Missing',false,false);
  await wait(20);assert.deepEqual(calls,[]);
  search.configure('Bless',false,true);search.configure('Blessed Assurance',false,true);
  await wait(30);assert.deepEqual(calls,['Blessed Assurance']);search.dispose();
});
test('late search responses cannot replace a new query or a new local match',async()=>{
  const requests:{q:string;resolve:(value:Candidate[])=>void}[]=[];
  const search=new SongSearch(new SongSearchCache(q=>new Promise(resolve=>requests.push({q,resolve}))),()=>{},0);
  search.searchNow('First');await wait();search.searchNow('Second');await wait();
  requests[1].resolve([result('Second')]);await wait();requests[0].resolve([result('First')]);await wait();
  assert.equal(search.state.results[0].title,'Second');
  search.searchNow('Third');await wait();search.configure('Third',true,true);
  requests[2].resolve([result('Third')]);await wait();assert.deepEqual(search.state.results,[]);search.dispose();
});
test('successful results and in-flight requests are shared with normalized queries',async()=>{
  let calls=0,resolve!:(results:Candidate[])=>void;
  const cache=new SongSearchCache(()=>{calls++;return new Promise(r=>resolve=r);});
  const first=cache.search('  Blessed   Assurance '),second=cache.search('blessed assurance');
  await wait();assert.equal(calls,1);resolve([result('Found')]);assert.deepEqual(await first,await second);
  await cache.search('BLESSED ASSURANCE');assert.equal(calls,1);
});
test('failed requests are retryable and cache entries expire',async()=>{
  let calls=0,now=0;
  const cache=new SongSearchCache(async()=>{if(++calls===1)throw new Error('Missing key');return [result('Found')];},()=>now);
  await assert.rejects(cache.search('Song'));await cache.search('Song');assert.equal(calls,2);
  now=300001;await cache.search('Song');assert.equal(calls,3);
});
test('closing a search cancels its timer and ignores requests already in flight',async()=>{
  let calls=0,updates=0,resolve!:(results:Candidate[])=>void;
  const search=new SongSearch(new SongSearchCache(()=>{calls++;return new Promise(r=>resolve=r);}),()=>updates++,10);
  search.configure('Song',false,true);search.dispose();await wait(20);assert.equal(calls,0);
  const active=new SongSearch(new SongSearchCache(()=>new Promise(r=>resolve=r)),()=>updates++,0);
  active.searchNow('Song');await wait();active.dispose();const previous=updates;resolve([]);await wait();assert.equal(updates,previous);
});
test('a stalled credential or network request times out without keeping the UI busy forever',async()=>{
  const cache=new SongSearchCache(()=>new Promise(()=>{}),()=>Date.now(),10);
  const search=new SongSearch(cache,()=>{},0);
  search.searchNow('Song');await wait(30);
  assert.equal(search.state.busy,false);assert.match(search.state.error,/timed out/);search.dispose();
});
