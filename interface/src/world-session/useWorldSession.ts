// Session de monde : connexion à la simulation, création, trames, événements, commandes. Aucune donnée inventée.
import { useCallback, useEffect, useRef, useState } from 'react';
import type { PlanetView } from '../planet-view/planet-view';
import type { RenderClock } from '../planet-view/PlanetCanvas';
import { getSimClient } from '../sim-bridge/connect-sim';
import type { ChronicleEvent, SimFrame, Vitals, WorldInfo } from '../sim-bridge/protocol';
import type { SimClient } from '../sim-bridge/sim-client';
import { rendererLogger } from '../shared/logger';
import { SPEEDS, type SpeedId } from '../time-control/speeds';
import type { WorldConfig } from '../world-setup/world-config';

export const WORLD_LEVEL = 6;

export interface SessionState {
  phase: 'setup' | 'creating' | 'running' | 'error';
  world: WorldInfo | null;
  config: WorldConfig | null;
  simYears: number;
  effectiveRatio: number;
  vitals: Vitals | null;
  events: ChronicleEvent[];
  playing: boolean;
  speed: SpeedId;
}

const INITIAL: SessionState = {
  phase: 'setup', world: null, config: null, simYears: 0, effectiveRatio: 1, vitals: null, events: [],
  playing: true, speed: 'century',
};

function useFrames(
  client: SimClient | null, view: React.RefObject<PlanetView | null>, clock: React.RefObject<RenderClock>,
  set: React.Dispatch<React.SetStateAction<SessionState>>,
): void {
  useEffect(() => {
    if (!client) return;
    const offFrame = client.onFrame((f: SimFrame) => {
      view.current?.push({ cellCount: f.fields.length / 4, fields: f.fields, simYears: f.simYears }, 0);
      clock.current.lastFrameAt = performance.now();
      set((s) => ({ ...s, simYears: f.simYears, effectiveRatio: f.effectiveRatio, vitals: f.vitals }));
    });
    const offEvents = client.onEvents((events) => set((s) => ({ ...s, events: [...s.events, ...events] })));
    return () => { offFrame(); offEvents(); };
  }, [client, view, clock, set]);
}

export interface Session {
  state: SessionState;
  client: SimClient | null;
  view: React.RefObject<PlanetView | null>;
  clock: React.RefObject<RenderClock>;
  launch(config: WorldConfig): void;
  control(playing: boolean, speed: SpeedId): void;
}

export function useWorldSession(): Session {
  const [state, set] = useState<SessionState>(INITIAL);
  const [client, setClient] = useState<SimClient | null>(null);
  const view = useRef<PlanetView | null>(null);
  const clock = useRef<RenderClock>({ lastFrameAt: performance.now(), spin: 0.035, visualRate: 1.5 });
  useEffect(() => {
    getSimClient().then(setClient).catch((error: unknown) => {
      rendererLogger.error({ err: error }, 'simulation injoignable');
      set((s) => ({ ...s, phase: 'error' }));
    });
  }, []);
  useFrames(client, view, clock, set);
  const launch = useCallback((config: WorldConfig) => {
    if (!client) return;
    set((s) => ({ ...s, phase: 'creating', config }));
    client.createWorld({ config, level: WORLD_LEVEL })
      .then((world) => set((s) => ({ ...s, phase: 'running', world })))
      .catch((error: unknown) => {
        rendererLogger.error({ err: error }, 'création du monde impossible');
        set((s) => ({ ...s, phase: 'error' }));
      });
  }, [client]);
  const control = useCallback((playing: boolean, speed: SpeedId) => {
    set((s) => ({ ...s, playing, speed }));
    const sp = SPEEDS.find((x) => x.id === speed);
    clock.current.spin = playing ? sp?.spin ?? 0.03 : 0.004;
    client?.control(playing, speed).catch((e: unknown) => rendererLogger.error({ err: e }, 'commande refusée'));
  }, [client]);
  return { state, client, view, clock, launch, control };
}
