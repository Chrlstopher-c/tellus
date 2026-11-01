import { useEffect, useState, type ReactElement } from 'react';
import { probeRenderer, type RendererKind } from './shared/renderer-probe';

export function App(): ReactElement {
  const [renderer, setRenderer] = useState<RendererKind | null>(null);

  useEffect(() => {
    let active = true;
    void probeRenderer().then((probe) => {
      document.body.dataset.renderer = probe.kind;
      if (active) setRenderer(probe.kind);
    });
    return () => {
      active = false;
    };
  }, []);

  return (
    <main className="tellus-screen">
      <p className="tellus-title">Tellus</p>
      <p className="tellus-debug">moteur : {renderer ?? 'détection'}</p>
    </main>
  );
}
