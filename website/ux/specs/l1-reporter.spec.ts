// L1: the reporter counts what was measured, never what was not. Canned records, no browser.
import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { expect, test } from '@playwright/test';
import { FINDINGS, REPOSITORY } from '../adapter';
import { holdKnownFailures } from '../core/findings';
import { engineDigest } from '../core/engine';
import { mergeRecords, summarize, type Expected, type L2Config, type L2Record, type L3Record, type Raw } from '../core/reporter';
import { buildRound, committedBaseline, mergeRaws, notesNumbersMissing } from '../core/round';

holdKnownFailures(test, FINDINGS, REPOSITORY);

const config = (viewport: string, theme: string, over: Partial<L2Config> = {}): L2Config => ({
  viewport,
  theme,
  gating: true,
  layout: { spills: [], truncated: [] },
  alwaysVisible: [],
  axe: [],
  axeScope: 'full',
  ...over,
});
const record = (path: string, viewport: string, configs: L2Config[]): L2Record => ({
  path,
  kind: 'doc',
  viewport,
  layoutShift: 0,
  motion: { running: [], smoothScroll: false },
  blocked: {},
  configs,
});
const l2 = (path: string, viewport: string, total = 4): Expected => ({ kind: 'l2', path, viewport, themes: ['light', 'dark'], total });
const run = (task: string, viewport: string, completed = true): L3Record => ({
  viewport,
  run: {
    task,
    mode: 'pointer',
    completed,
    reason: completed ? '' : 'stopped',
    path: '/x',
    states: [],
    metrics: {
      'screen_only.actions': 2,
      'screen_only.looks': 0,
      'screen_only.steers': 0,
      'screen_only.time_to_act_steps.max': 1,
      'glance_cost.median': 1,
      'glance_cost.p95': 1,
      'reading_load.max': 10,
      'reading_load.median': 10,
    },
    completion_miss: 0,
    false_complete: 0,
  },
});

test('L1 coverage: a page cut off before its first configuration is unmeasured and fails every page gate', () => {
  const s = summarize([record('/a', 'd', [config('d', 'light'), config('d', 'dark')]), record('/b', 'd', [])], [], [l2('/a', 'd'), l2('/b', 'd')]);
  expect(s.coverage.unmeasured).toEqual(['/b@d/dark', '/b@d/light']);
  expect(s.coverage.pages).toEqual({ expected: 2, measured: 1 });
  expect(s.coverage.configs).toEqual({ expected: 4, measured: 2 });
  for (const gate of [s.gates.gate1_no_horizontal_overflow, s.gates.gate2_always_visible, s.gates.gate5_accessibility, s.gates.truncation]) {
    expect(gate.unmeasured).toBe(1);
    expect(gate.result).toBe('fail');
  }
  // An unmeasured configuration has no value, so the ratchet sees it missing.
  expect(Object.keys(s.scenarios)).toEqual(['/a@d/dark', '/a@d/light']);
});

test('L1 coverage: a test that never started is unmeasured, and so is a declared test no merged run holds', () => {
  const s = summarize([record('/a', 'd', [config('d', 'light'), config('d', 'dark')])], [], [l2('/a', 'd', 8), l2('/c', 'd', 8)]);
  expect(s.coverage.unmeasured).toEqual(['/c@d/dark', '/c@d/light', '4 l2 configuration(s) in no merged run']);
  expect(s.coverage.pages).toEqual({ expected: 2, measured: 1 });
  expect(s.gates.gate1_no_horizontal_overflow.result).toBe('fail');
});

test('L1 coverage: a clean page set with every configuration measured passes, with its denominators', () => {
  const s = summarize([record('/a', 'd', [config('d', 'light'), config('d', 'dark')])], [], [l2('/a', 'd', 2)]);
  expect(s.coverage.unmeasured).toEqual([]);
  expect(s.gates.gate1_no_horizontal_overflow).toMatchObject({ pages: 0, of: 1, unmeasured: 0, result: 'pass' });
  expect(s.coverage.by_config).toEqual({ 'd/dark': { expected: 1, measured: 1 }, 'd/light': { expected: 1, measured: 1 } });
});

test('L1 coverage: a rerun replaces the configurations it measured again, never adds them', () => {
  const spill = { spills: [{ kind: 'page' as const, element: 'html', right: 900, limit: 800 }], truncated: [] };
  const merged = mergeRecords([
    record('/a', 'd', [config('d', 'light', { layout: spill })]),
    record('/a', 'd', [config('d', 'light'), config('d', 'dark')]),
  ]);
  expect(merged).toHaveLength(1);
  expect(merged[0].configs.map((c) => `${c.theme}:${c.layout.spills.length}`)).toEqual(['dark:0', 'light:0']);
});

test('L1 coverage: a task run that never ran is unmeasured and fails gates 3 and 4', () => {
  const expected: Expected[] = [
    { kind: 'l3', task: 't', viewport: 'd', mode: 'pointer', total: 2 },
    { kind: 'l3', task: 'u', viewport: 'd', mode: 'pointer', total: 2 },
  ];
  const s = summarize([], [run('t', 'd')], expected);
  expect(s.coverage.tasks).toEqual({ expected: 2, measured: 1 });
  expect(s.coverage.unmeasured).toEqual(['u/d']);
  expect(s.gates.gate4_screen_only).toMatchObject({ tasks: 1, completed: 1, unmeasured: 1, result: 'fail' });
  expect(s.gates.gate3_unmissable_completion.result).toBe('fail');
});

// T34 of the R-04 report review: the page load is measured under the first theme the L2 test
// shows (light), and the value was filed under the first configuration after the merge sorted
// them, dark.
test('L1 metrics: the layout shift of the page load is filed under the theme it was measured in (T34)', () => {
  const rec = { ...record('/a', 'd', [config('d', 'light'), config('d', 'dark')]), layoutShift: 0.05, loadTheme: 'light' };
  const s = summarize([rec], [], [l2('/a', 'd')]);
  expect(s.scenarios['/a@d/light']['layout_shift.load']).toBe(0.05);
  expect(s.scenarios['/a@d/dark']['layout_shift.load']).toBeUndefined();
  expect(s.scenarios['/a@d/light']['motion.running']).toBe(0);
});

test('L1 coverage: measured-only viewports report their coverage with a denominator', () => {
  const measured = [record('/a', 'w', [config('w', 'dark', { gating: false })])];
  const expected: Expected[] = [
    { kind: 'measured', path: '/a', viewports: ['w', 'v'], themes: ['dark'], total: 4 },
    { kind: 'measured', path: '/b', viewports: ['w', 'v'], themes: ['dark'], total: 4 },
  ];
  const s = summarize([], [], expected, measured);
  expect(s.coverage.measured_only).toEqual({ expected: 4, measured: 1 });
});

test('L1 engine: the digest follows the engine files and ignores the round records', () => {
  const root = test.info().outputPath('engine');
  mkdirSync(join(root, 'core'), { recursive: true });
  mkdirSync(join(root, 'rounds', 'R-01'), { recursive: true });
  writeFileSync(join(root, 'core', 'a.ts'), 'one');
  writeFileSync(join(root, 'rounds', 'R-01', 'report.md'), 'x');
  const first = engineDigest(root);
  writeFileSync(join(root, 'rounds', 'R-01', 'report.md'), 'y');
  expect(engineDigest(root)).toBe(first);
  writeFileSync(join(root, 'core', 'a.ts'), 'two');
  expect(engineDigest(root)).not.toBe(first);
});

// T26 of the R-03 report review: the ratchet holds every run's digest to the committed tree's,
// which it computes from git with whole paths sorted (ux_ratchet.rs `engine_digest`, which pins
// the same value). The engine walked directory by directory, which puts `core/b.ts` before
// `core.ts`.
test('L1 engine: the digest sorts whole paths, as the ratchet does', () => {
  const root = test.info().outputPath('engine-order');
  mkdirSync(join(root, 'core'), { recursive: true });
  writeFileSync(join(root, 'core', 'b.ts'), 'two');
  writeFileSync(join(root, 'core.ts'), 'one');
  expect(engineDigest(root)).toBe('804bb99ed3bd75aa');
});

test('L1 round: the slow lane, merged from its jobs, is the full round the ratchet holds every scenario to', () => {
  const raw = (lane: string): Raw => ({ lane, engine: { digest: 'd', commit: 'c', dirty: false }, expected: [], l2: [], l3: [], measured: [], timing: { wall_s: 1, workers: 2, status: 'passed', projects: {}, slowest: [] } });
  const round = (lane: string) => buildRound({ round: 'R-x', date: '2026-11-22', measurement: [{ name: 'a', raw: raw(lane) }, { name: 'b', raw: raw(lane) }], others: [{ name: 'g', raw: raw('gate') }], cards: { anchors: [], rotating: '' }, viewports: { gating: [], measured: [] }, budgetSeconds: 120 });
  expect(round('slow').lane).toBe('full');
  expect(round('gate').lane).toBe('gate');
});

const rawOf = (engine: Raw['engine'], l2: L2Record[] = []): Raw => ({ lane: 'slow', engine, expected: [], l2, l3: [], measured: [], timing: { wall_s: 1, workers: 4, status: 'passed', projects: {}, slowest: [] } });
const roundOf = (measurement: Array<{ name: string; raw: Raw }>) =>
  buildRound({ round: 'R-x', date: '2026-11-22', measurement, others: [], cards: { anchors: [], rotating: '' }, viewports: { gating: [], measured: [] }, budgetSeconds: 120 });

// T25 of the R-03 report review: the merged round took its engine from the first run, so the two
// reruns from an engine copy outside Git (commit "unknown", dirty) left no trace in its engine.
test('L1 round: a merged round keeps the engine of every run, and a dirty or unknown one marks the round', () => {
  const merged = mergeRaws([rawOf({ digest: 'd', commit: 'c', dirty: false }), rawOf({ digest: 'd', commit: 'unknown', dirty: true })]);
  expect(merged.engines).toEqual([
    { digest: 'd', commit: 'c', dirty: false },
    { digest: 'd', commit: 'unknown', dirty: true },
  ]);
  expect(merged.engine).toEqual({ digest: 'd', commit: 'unknown', dirty: true });
  const round = roundOf([
    { name: 'a', raw: rawOf({ digest: 'd', commit: 'c', dirty: false }) },
    { name: 'b', raw: rawOf({ digest: 'd', commit: 'unknown', dirty: true }) },
  ]);
  expect(round.engine).toMatchObject({ commit: 'unknown', dirty: true, runs_agree: true });
});

// T27: a rerun replaced the first run's value of a configuration and nothing kept the first one
// (layout_shift.load 0.0253, then 0.0000423, and round.json held 0).
test('L1 round: a value a rerun replaced is kept beside the value that replaced it', () => {
  const shifted = (shift: number) => ({ ...record('/p', 'm', [config('m', 'dark'), config('m', 'light')]), layoutShift: shift });
  const round = roundOf([
    { name: 'slow-l2-4', raw: rawOf({ digest: 'd', commit: 'c', dirty: false }, [shifted(0.0253)]) },
    { name: 'rerun-shift', raw: rawOf({ digest: 'd', commit: 'c', dirty: false }, [shifted(0.0000423)]) },
  ]);
  expect(round.scenarios['/p@m/dark']['layout_shift.load']).toBe(0);
  expect(round.replaced).toEqual({
    '/p@m/dark': {
      'layout_shift.load': [
        { run: 'slow-l2-4', value: 0.0253 },
        { run: 'rerun-shift', value: 0 },
      ],
    },
  });
});

// T33 of the R-03 report review: the report's hand-written notes held numbers neither round.json nor
// the logs gave. A number of the notes is one round.json holds, as a value or inside a string; a
// number that is part of a name (UXF-006, R-03, gate-4, mobile-390) or in a code span is not a
// number of the notes. A run records the host load before and after it, so the notes can cite it.
test('L1 report: the hand-written notes carry no number round.json does not hold (T33)', () => {
  const round = {
    runs: [{ name: 'gate-4', timing: { wall_s: 45.1, load: { before: [53.48, 52.1, 40.03], after: [50.19, 51.2, 40.5] } } }],
    scenarios: { '/p@m/dark': { 'layout_shift.load': 0.0253 } },
    gates: { gate4_screen_only: { over_limit: ['ess-31-release/desktop-1280: 25 of 6'] } },
  };
  const notes = [
    '- Host load. `gate-4` took 45.1 s at a load of 53.48 before and 50.19 after it (UXF-025, R-03).',
    '- `ess/31` at mobile-390 and l2-1: 25 actions of 6; layout shift 0.0253.',
    '- Three runs were cut at 119 s, and the day cost 1,900 minutes.',
  ].join('\n');
  expect(notesNumbersMissing(notes, round)).toEqual(['119', '1,900']);
  expect(notesNumbersMissing('No number here, only `R-04` and gate-5.', round)).toEqual([]);
});

// T23 of the R-02 report review: the report's baseline column was read from whatever baseline.json
// held when the report was written, before the ratchet took the round in.
test('L1 report: the baseline column is the baseline the ratchet took this round into, never an older one', () => {
  const baseline = (sources: Record<string, string> | undefined) =>
    JSON.stringify({ about: 'canned', ...(sources ? { sources } : {}), scenarios: { 't/d': { 'screen_only.actions': { value: 2, tolerance: 'count' } } }, tolerances: {} });
  const taken = committedBaseline(baseline({ full: 'R-03', gate: 'R-03' }), 'R-03');
  expect(taken.scenarios['t/d']['screen_only.actions'].value).toBe(2);
  expect(taken.about).toBe('canned');
  expect(() => committedBaseline(baseline({ full: 'R-02' }), 'R-03')).toThrow(/R-02/);
  expect(() => committedBaseline(baseline(undefined), 'R-03')).toThrow(/ux-ratchet/);
});
