// L1: the reporter counts what was measured, never what was not. Canned records, no browser.
import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { expect, test } from '@playwright/test';
import { engineDigest } from '../core/engine';
import { mergeRecords, summarize, type Expected, type L2Config, type L2Record, type L3Record } from '../core/reporter';

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
