import { useEffect, useRef } from 'react';
import { X } from 'lucide-react';
export function Modal({title,close,children,wide=false}:{title:string;close:()=>void;children:React.ReactNode;wide?:boolean}) {
  const dialog=useRef<HTMLDialogElement>(null);
  useEffect(()=>{dialog.current?.showModal();return()=>dialog.current?.close();},[]);
  return <dialog ref={dialog} className={`modal ${wide?'wide':''}`} onCancel={e=>{e.preventDefault();close();}} onClick={e=>{if(e.target===dialog.current)close();}}><div className="modal-header"><h2>{title}</h2><button className="icon-button" onClick={close} aria-label="Close / Cerrar"><X size={20}/></button></div>{children}</dialog>;
}
