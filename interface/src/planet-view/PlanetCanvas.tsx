// Toile WebGL de la planète : crée la vue quand le monde existe, rend à chaque image, interpole entre deux trames.
import { useEffect, useRef, type ReactElement } from 'react';
import type { WorldInfo } from '../sim-bridge/protocol';
import { rendererLogger } from '../shared/logger';
import { createPlanetView, type PickResult, type PlanetView } from './planet-view';

const FRAME_MS = 100;

export interface RenderClock {
  lastFrameAt: number;
  spin: number;
  visualRate: number;
}

interface Props {
  world: WorldInfo;
  clock: React.RefObject<RenderClock>;
  onView(view: PlanetView | null): void;
  onPick(result: PickResult): void;
}

function startLoop(view: PlanetView, clock: React.RefObject<RenderClock>): () => void {
  let raf = 0, last = performance.now(), visualTime = 0;
  const frame = (now: number): void => {
    const dt = Math.min(0.05, (now - last) / 1000);
    last = now;
    const c = clock.current;
    visualTime += dt * c.visualRate;
    view.frame({ dt, visualTime, spin: c.spin, alpha: (now - c.lastFrameAt) / FRAME_MS });
    raf = requestAnimationFrame(frame);
  };
  raf = requestAnimationFrame(frame);
  return () => cancelAnimationFrame(raf);
}

function usePlanetView(
  canvas: React.RefObject<HTMLCanvasElement | null>, p: Props, viewRef: React.RefObject<PlanetView | null>,
): void {
  const { world, clock } = p;
  const onView = useRef(p.onView);
  onView.current = p.onView;
  useEffect(() => {
    const el = canvas.current;
    if (!el) return;
    let view: PlanetView;
    try {
      view = createPlanetView(el, world.lookup, world.cells);
    } catch (error: unknown) {
      rendererLogger.error({ err: error }, 'création de la vue planétaire impossible');
      return;
    }
    view.resize();
    const onResize = (): void => view.resize();
    window.addEventListener('resize', onResize);
    viewRef.current = view;
    onView.current(view);
    const stop = startLoop(view, clock);
    return () => {
      stop();
      window.removeEventListener('resize', onResize);
      viewRef.current = null;
      onView.current(null);
      view.dispose();
    };
  }, [canvas, world, clock, viewRef]);
}

export function PlanetCanvas(p: Props): ReactElement {
  const canvas = useRef<HTMLCanvasElement | null>(null);
  const viewRef = useRef<PlanetView | null>(null);
  const down = useRef<[number, number] | null>(null);
  usePlanetView(canvas, p, viewRef);
  return (
    <canvas ref={canvas} className="planet-canvas" data-testid="planet"
      onPointerDown={(e) => { down.current = [e.clientX, e.clientY]; }}
      onPointerUp={(e) => {
        const d = down.current;
        if (!d || Math.hypot(e.clientX - d[0], e.clientY - d[1]) > 4) return;
        const hit = viewRef.current?.pick(e.clientX, e.clientY);
        if (hit) p.onPick(hit);
      }} />
  );
}
