import type { Project, Live } from './types';
import { seed } from './data';
import { validateProject } from './model';
let remote=false;
/** True when this page is served by the running desktop app, so commands go over HTTP. */
export async function detectRemote():Promise<void> {
  if(window.__TAURI_INTERNALS__||!location.protocol.startsWith('http'))return;
  try {const r=await fetch('api/ping');remote=r.ok&&(await r.json())==='selah';} catch {remote=false;}
}
export const desktop=()=>Boolean(window.__TAURI_INTERNALS__)||remote;
export async function invoke<T>(command:string,args:Record<string,unknown>={}):Promise<T> {
  if(window.__TAURI_INTERNALS__) return window.__TAURI_INTERNALS__.invoke<T>(command,args);
  if(!remote) throw new Error('This feature needs the desktop app. The browser preview supports local preparation and presentation.');
  const r=await fetch(`api/invoke/${command}`,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(args)});
  const body=await r.json().catch(()=>null);
  if(!r.ok) throw new Error(typeof body==='string'?body:`Request failed (${r.status})`);
  return body as T;
}
export async function load():Promise<Project> {
  if(desktop()) return invoke('load_project',{seed});
  let stored:string|null;
  try {stored=localStorage.getItem('selah.project.v1');}
  catch {throw new Error('Browser storage is unavailable. Enable storage for this page to restore and autosave services. You can still prepare and export a service.');}
  const p:Project=stored===null?structuredClone(seed):JSON.parse(stored);validateProject(p);return p;
}
export async function save(project:Project) { validateProject(project);if(desktop()) await invoke('save_project',{project});else localStorage.setItem('selah.project.v1',JSON.stringify(project)); }
export function hasWelcomed():boolean {try {return localStorage.getItem('selah.welcome')==='1';}catch {return false;}}
export function rememberWelcome():void {try {localStorage.setItem('selah.welcome','1');}catch {/* Onboarding remains usable when browser storage is blocked. */}}
export async function present(live:Live) { if(desktop()) await invoke('set_live',{live}); }
export function download(name:string,text:string) {const url=URL.createObjectURL(new Blob([text],{type:'application/json'}));const a=document.createElement('a');a.href=url;a.download=name;a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);}
