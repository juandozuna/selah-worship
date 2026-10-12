import React from 'react';
import { createRoot } from 'react-dom/client';
import App from './App';
import DisplayView from './DisplayView';
import { detectRemote } from './bridge';
import './styles.css';
detectRemote().then(()=>createRoot(document.getElementById('root')!).render(<React.StrictMode>{new URLSearchParams(location.search).has('display')?<DisplayView/>:<App/>}</React.StrictMode>));
