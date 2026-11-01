import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import '@fontsource/geist/300.css';
import '@fontsource/geist/400.css';
import '@fontsource/geist/500.css';
import '@fontsource/geist/600.css';
import '@fontsource/geist-mono/400.css';
import '@fontsource/geist-mono/500.css';
import '@fontsource/instrument-serif/400.css';
import '@fontsource/instrument-serif/400-italic.css';
import { App } from './App';
import './shared/design/tellus.css';

const container = document.getElementById('root');
if (!container) throw new Error('élément #root absent');

createRoot(container).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
