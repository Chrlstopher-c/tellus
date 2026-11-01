// Application Tellus : écran de départ, puis la planète et ses panneaux.
import { useCallback, useEffect, useState, type ReactElement } from 'react';
import { Toast, type Announcement } from './chronicle/Toast';
import { PowersDock } from './god-powers/PowersDock';
import type { PowerId } from './god-powers/powers';
import { Inspector } from './inspector/Inspector';
import type { RegionDetail } from './inspector/region';
import { LayersPanel } from './planet-view/LayersPanel';
import { PlanetCanvas } from './planet-view/PlanetCanvas';
import type { PickResult } from './planet-view/planet-view';
import type { ViewMode } from './planet-view/rig';
import { ViewToggle } from './planet-view/ViewToggle';
import { rendererLogger } from './shared/logger';
import { probeRenderer } from './shared/renderer-probe';
import { TimeBar } from './time-control/TimeBar';
import { Vitals } from './world-status/Vitals';
import { WorldHeader } from './world-status/WorldHeader';
import { colorOf, timeline } from './world-session/chronicle-view';
import { parseRegion } from './world-session/region-parse';
import { useWorldSession, type Session } from './world-session/useWorldSession';
import { StartScreen } from './world-setup/StartScreen';
import { worldTitle } from './world-setup/world-config';

interface ViewState {
  mode: ViewMode;
  layer: number;
  clouds: boolean;
  dual: boolean;
  power: PowerId | null;
}

function useAnnouncements(session: Session): Announcement | null {
  const [item, setItem] = useState<Announcement | null>(null);
  const count = session.state.events.length;
  useEffect(() => {
    const e = session.state.events[count - 1];
    if (!e) return;
    setItem({ key: count, title: e.title, body: e.body, color: colorOf(e.kind) });
    const t = setTimeout(() => setItem(null), 5200);
    return () => clearTimeout(t);
  }, [count, session.state.events]);
  return item;
}

function usePick(session: Session, view: ViewState, setView: (v: ViewState) => void): {
  region: RegionDetail | null; setRegion(r: RegionDetail | null): void; onPick(hit: PickResult): void;
} {
  const [region, setRegion] = useState<RegionDetail | null>(null);
  const onPick = useCallback((hit: PickResult) => {
    const client = session.client;
    if (!client) return;
    if (view.power) {
      client.intervene(view.power, hit.cell, 1)
        .catch((e: unknown) => rendererLogger.error({ err: e }, 'intervention refusée'));
      setView({ ...view, power: null });
    }
    client.inspect(hit.cell).then((json) => setRegion(parseRegion(json, hit.lat, hit.lon)))
      .catch((e: unknown) => rendererLogger.error({ err: e }, 'inspection impossible'));
  }, [session.client, view, setView]);
  return { region, setRegion, onPick };
}

interface HudProps {
  session: Session;
  view: ViewState;
  setView(v: ViewState): void;
}

function Hud({ session, view, setView }: HudProps): ReactElement {
  const { state } = session;
  const pick = usePick(session, view, setView);
  const announcement = useAnnouncements(session);
  const title = state.config ? worldTitle(state.config, state.world?.name ?? '') : { name: '', origin: '' };
  return (
    <>
      {state.world && <PlanetCanvas world={state.world} clock={session.clock} onView={(v) => {
        v?.setMode(view.mode); v?.setLayer(view.layer); v?.setClouds(view.clouds);
      }} onPick={pick.onPick} />}
      <div className="vignette" />
      <WorldHeader name={title.name} origin={title.origin} era={state.vitals?.era ?? ''} ageYears={state.simYears} />
      <Toast item={announcement} />
      <ViewToggle mode={view.mode} dual={view.dual}
        onMode={(mode) => { setView({ ...view, mode }); session.view.current?.setMode(mode); }}
        onDual={() => setView({ ...view, dual: !view.dual })} />
      <LayersPanel active={view.layer} clouds={view.clouds}
        onLayer={(layer) => { setView({ ...view, layer }); session.view.current?.setLayer(layer); }}
        onClouds={(clouds) => { setView({ ...view, clouds }); session.view.current?.setClouds(clouds); }} />
      <Inspector region={pick.region} onClose={() => pick.setRegion(null)} />
      {state.vitals && <Vitals v={state.vitals} />}
      <TimeBar playing={state.playing} speed={state.speed} effectiveRatio={state.effectiveRatio}
        events={timeline(state.events, state.simYears)} onToggle={() => session.control(!state.playing, state.speed)}
        onSpeed={(id) => session.control(true, id)} />
      <PowersDock active={view.power} onPick={(power) => setView({ ...view, power })} />
    </>
  );
}

export function App(): ReactElement {
  const session = useWorldSession();
  const [view, setView] = useState<ViewState>({ mode: 'globe', layer: 0, clouds: true, dual: false, power: null });
  const starting = session.state.phase !== 'running';
  useEffect(() => { void probeRenderer().then((p) => { document.body.dataset.renderer = p.kind; }); }, []);
  useEffect(() => { document.body.classList.toggle('starting', starting); }, [starting]);
  useEffect(() => { document.body.classList.toggle('targeting', view.power !== null); }, [view.power]);
  return (
    <main data-phase={session.state.phase}>
      {session.state.phase === 'running' && <Hud session={session} view={view} setView={setView} />}
      {session.state.phase === 'setup' && <StartScreen onLaunch={session.launch} />}
      {session.state.phase === 'creating' && <p className="start-lead loading">Formation du monde…</p>}
      {session.state.phase === 'error' && <p className="start-lead loading">Simulation indisponible — voir logs/sim.log</p>}
    </main>
  );
}
