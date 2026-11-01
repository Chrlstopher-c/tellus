import { useEffect, useState, type ReactElement } from 'react';
import { rendererLogger } from '../shared/logger';
import { getSimProbe, type SimProbe } from './probe-sim';

export function SimDebugLine(): ReactElement {
  const [probe, setProbe] = useState<SimProbe | null>(null);
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    let active = true;
    getSimProbe()
      .then((result) => {
        if (active) setProbe(result);
      })
      .catch((error: unknown) => {
        rendererLogger.error({ err: error }, 'sonde de simulation échouée');
        if (active) setFailed(true);
      });
    return () => {
      active = false;
    };
  }, []);

  if (failed) return <p className="tellus-debug">sim : indisponible</p>;
  if (!probe) return <p className="tellus-debug">sim : connexion</p>;
  return (
    <p className="tellus-debug" data-testid="sim-debug" data-snapshot-length={probe.snapshotLength}>
      sim : {probe.cells} cellules, tick {probe.time}
    </p>
  );
}
