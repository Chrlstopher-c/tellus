// Dock des pouvoirs : un pouvoir actif attend un clic sur la planète pour cibler.
import { Dna, MountainSnow, Snowflake, Sparkle, SunDim, ThermometerSun } from 'lucide-react';
import type { ReactElement } from 'react';
import { POWERS, type PowerId } from './powers';

const ICONS: Record<PowerId, ReactElement> = {
  meteor: <Sparkle />, volcano: <MountainSnow />, climate: <ThermometerSun />,
  species: <Dna />, ice: <Snowflake />, drought: <SunDim />,
};

interface Props {
  active: PowerId | null;
  onPick(id: PowerId | null): void;
}

export function PowersDock(p: Props): ReactElement {
  const active = POWERS.find((x) => x.id === p.active);
  return (
    <>
      <section className="powers glass" data-testid="powers">
        {POWERS.map((x) => (
          <button key={x.id} className={`power${x.id === p.active ? ' active' : ''}`} data-label={x.label}
            aria-label={x.label} onClick={() => p.onPick(x.id === p.active ? null : x.id)}>
            {ICONS[x.id]}
          </button>
        ))}
      </section>
      {active && <p className="power-hint">{active.label} — cliquez sur la planète pour cibler</p>}
    </>
  );
}
