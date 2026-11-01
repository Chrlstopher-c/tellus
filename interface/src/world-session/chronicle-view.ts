// Traduction des événements de la chronique pour l'interface (frise et annonces).
import type { TimelineEvent } from '../time-control/TimeBar';
import type { ChronicleEvent } from '../sim-bridge/protocol';

const KIND_COLOR: Record<string, string> = {
  geology: 'var(--geo)', climate: 'var(--climate)', life: 'var(--life)', intervention: 'var(--god)',
};

export const colorOf = (kind: string): string => KIND_COLOR[kind] ?? 'var(--select)';

/** Position de chaque événement sur la frise, de la formation (0) à aujourd'hui (1). */
export function timeline(events: readonly ChronicleEvent[], nowYears: number): TimelineEvent[] {
  if (nowYears <= 0) return [];
  return events.slice(-40).map((e) => ({
    at: Math.min(1, e.years / nowYears), color: colorOf(e.kind), label: e.title,
  }));
}
