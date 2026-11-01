// Bascule Globe / Carte et actions de fenêtre (deux écrans, réglages).
import { Globe2, Map, MonitorSmartphone, Settings2 } from 'lucide-react';
import { useLayoutEffect, useRef, useState, type ReactElement } from 'react';
import type { ViewMode } from './rig';

interface Props {
  mode: ViewMode;
  dual: boolean;
  onMode(mode: ViewMode): void;
  onDual(): void;
}

const MODES: readonly { id: ViewMode; label: string; icon: ReactElement }[] = [
  { id: 'globe', label: 'Globe', icon: <Globe2 /> },
  { id: 'map', label: 'Carte', icon: <Map /> },
];

export function ViewToggle(p: Props): ReactElement {
  const refs = useRef<Record<string, HTMLButtonElement | null>>({});
  const [pill, setPill] = useState({ width: 0, x: 0 });
  useLayoutEffect(() => {
    const b = refs.current[p.mode];
    if (b) setPill({ width: b.offsetWidth, x: b.offsetLeft - 4 });
  }, [p.mode]);
  return (
    <nav className="top-actions">
      <div className="segmented glass" role="tablist">
        {MODES.map((m) => (
          <button key={m.id} ref={(el) => { refs.current[m.id] = el; }} data-mode={m.id}
            className={`seg${m.id === p.mode ? ' active' : ''}`} onClick={() => p.onMode(m.id)}>
            {m.icon}{m.label}
          </button>
        ))}
        <span className="seg-pill" style={{ width: pill.width, transform: `translateX(${pill.x}px)` }} />
      </div>
      <button className={`icon-btn glass${p.dual ? ' on' : ''}`} title="Mode deux écrans" onClick={p.onDual}>
        <MonitorSmartphone />
      </button>
      <button className="icon-btn glass" title="Réglages"><Settings2 /></button>
    </nav>
  );
}
