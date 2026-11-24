// Metric functions of the loop. Pure, no browser, no project code.

export function median(values: number[]): number {
  if (values.length === 0) return 0;
  const sorted = [...values].sort((a, b) => a - b);
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 1 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2;
}

// Nearest-rank percentile: the smallest value with at least p % of the values at or below it.
export function percentile(values: number[], p: number): number {
  if (values.length === 0) return 0;
  const sorted = [...values].sort((a, b) => a - b);
  const rank = Math.max(1, Math.ceil((p / 100) * sorted.length));
  return sorted[rank - 1];
}

export function round(value: number, places = 4): number {
  const factor = 10 ** places;
  return Math.round(value * factor) / factor;
}

// One state of a screen-only task: what the user looked at and did there.
export interface StateRecord {
  step: string;
  glance: 1 | 2 | 3;
  actions: number;
  steers: number;
  words: number;
}

export interface TaskMetrics {
  'screen_only.actions': number;
  'screen_only.looks': number;
  'screen_only.steers': number;
  'screen_only.time_to_act_steps.max': number;
  'glance_cost.median': number;
  'glance_cost.p95': number;
  'reading_load.max': number;
  'reading_load.median': number;
}

// looks: states where the next control was not in the primary region (glance 2 or 3).
// time to act at a state: 1 + the steers spent there before the control could be used.
export function taskMetrics(states: StateRecord[]): TaskMetrics {
  const glances = states.map((s) => s.glance);
  const words = states.map((s) => s.words);
  return {
    'screen_only.actions': states.reduce((sum, s) => sum + s.actions, 0),
    'screen_only.looks': states.filter((s) => s.glance > 1).length,
    'screen_only.steers': states.reduce((sum, s) => sum + s.steers, 0),
    'screen_only.time_to_act_steps.max': Math.max(0, ...states.map((s) => 1 + s.steers)),
    'glance_cost.median': median(glances),
    'glance_cost.p95': percentile(glances, 95),
    'reading_load.max': Math.max(0, ...words),
    'reading_load.median': median(words),
  };
}
