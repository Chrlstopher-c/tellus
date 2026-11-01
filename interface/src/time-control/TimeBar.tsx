// Barre du temps : frise des événements, lecture/pause, vitesses, vitesse effective.
import { Pause, Play } from 'lucide-react';
import type { ReactElement } from 'react';
import { SPEEDS, type SpeedId } from './speeds';

export interface TimelineEvent {
  at: number;
  color: string;
  label: string;
}

interface Props {
  playing: boolean;
  speed: SpeedId;
  effectiveRatio: number;
  events: readonly TimelineEvent[];
  onToggle(): void;
  onSpeed(id: SpeedId): void;
}

function speedLabel(playing: boolean, speed: SpeedId, ratio: number): string {
  if (!playing) return 'en pause';
  const base = SPEEDS.find((s) => s.id === speed)?.description ?? '';
  return ratio < 0.95 ? `${base} · ralenti à ${Math.round(ratio * 100)} %` : base;
}

export function TimeBar(p: Props): ReactElement {
  return (
    <section className="time glass" data-testid="time-bar">
      <div className="timeline">
        <div className="track"><div className="fill" style={{ width: '100%' }} /></div>
        <div className="ticks">
          {p.events.map((e) => (
            <span key={`${e.at}-${e.label}`} className="tick" data-label={e.label}
              style={{ left: `${e.at * 100}%`, background: e.color }} />
          ))}
        </div>
        <div className="ends"><span>Formation</span><span>Aujourd’hui</span></div>
      </div>
      <div className="controls">
        <button className="play" onClick={p.onToggle} aria-label={p.playing ? 'Pause' : 'Lecture'}>
          {p.playing ? <Pause /> : <Play />}
        </button>
        <div className="speeds">
          {SPEEDS.map((s) => (
            <button key={s.id} className={`speed${s.id === p.speed ? ' active' : ''}`} onClick={() => p.onSpeed(s.id)}>
              {s.label}
            </button>
          ))}
        </div>
        <p className="speed-label">{speedLabel(p.playing, p.speed, p.effectiveRatio)}</p>
      </div>
    </section>
  );
}
