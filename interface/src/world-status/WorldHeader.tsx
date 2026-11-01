// En-tête du monde : identité (nom, origine) et horloge (époque, âge).
import { ChevronDown } from 'lucide-react';
import type { ReactElement } from 'react';
import { formatAge } from '../time-control/speeds';

interface Props {
  name: string;
  origin: string;
  era: string;
  ageYears: number;
}

export function WorldHeader(p: Props): ReactElement {
  const age = formatAge(p.ageYears);
  return (
    <>
      <header className="identity">
        <span className="wordmark">Tellus</span>
        <div className="world"><h1>{p.name}</h1><ChevronDown /></div>
        <p className="world-sub">{p.origin}</p>
      </header>
      <section className="clock" data-testid="clock">
        <p className="era">{p.era}</p>
        <p className="year"><span>{age.value}</span> <span className="unit">{age.unit}</span></p>
      </section>
    </>
  );
}
