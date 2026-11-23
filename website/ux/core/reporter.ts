// The measurement reporter. Runs in Playwright's main process, so it sees every worker's results
// in one place. It writes, under UX_OUT (default `.out/`):
//   raw.json    what every test was expected to measure and every record it attached;
//   round.json  the metric values per scenario (the ratchet's input), the gate counts, the coverage
//               and the run's timing.
// It decides no ratchet: the specs assert the hard gates, the Rust ratchet compares. A page, a
// configuration or a task run that was expected and not measured is counted as unmeasured: it has
// no value and fails its gate. It prints the ten slowest tests, as the organization's test budget
// asks.
import { mkdirSync, writeFileSync } from 'node:fs';
import { loadavg } from 'node:os';
import { join, resolve } from 'node:path';
import type { FullConfig, FullResult, Reporter, Suite, TestCase, TestResult } from '@playwright/test/reporter';
import { engine, type Engine } from './engine';
import { round as r } from './metrics';
import type { TaskRun } from './bot';
import type { LayoutProbe } from './probes';

export interface AlwaysVisibleFailure {
  element: string;
  state: string;
  problem: 'missing' | 'out of viewport' | 'covered';
  // false: a state the adapter does not gate (reported as measured only). Gated when absent.
  gating?: boolean;
}

export const gated = (f: AlwaysVisibleFailure) => f.gating !== false;

export interface AxeViolation {
  id: string;
  impact: string;
  nodes: number;
  help: string;
  targets: string[];
}

export interface L2Config {
  viewport: string;
  theme: string;
  gating: boolean;
  layout: LayoutProbe;
  alwaysVisible: AlwaysVisibleFailure[];
  axe: AxeViolation[] | null;
  axeScope?: string;
}

// One test's measurement of one page: one viewport for L2 (both themes), every measured-only
// viewport for the measured lane. `phases` holds the milliseconds of each step of the test.
export interface L2Record {
  path: string;
  kind: string;
  viewport: string;
  layoutShift: number;
  // The theme the page loaded in, which `layoutShift` and `motion` were measured under.
  loadTheme?: string;
  motion: { running: string[]; smoothScroll: boolean };
  blocked: Record<string, number>;
  configs: L2Config[];
  phases?: Record<string, number>;
}

export interface L3Record {
  viewport: string;
  // The card's `max_actions`: a pointer run that completes with more actions fails gate 4.
  limit?: number;
  run: TaskRun;
}

// What a test is expected to measure, declared as a test annotation (type EXPECT) so the reporter
// knows it before the test runs. `total` is the number of units of this kind the whole lane
// declares, so a merge of runs that lacks one of them can tell.
export interface Expected {
  kind: 'l2' | 'l3' | 'measured';
  path?: string;
  viewport?: string;
  viewports?: string[];
  themes?: string[];
  task?: string;
  mode?: string;
  total: number;
}
export const EXPECT = 'ux-expect';

export const configKey = (path: string, viewport: string, theme: string) => `${path}@${viewport}/${theme}`;
export const taskKey = (task: string, viewport: string, mode: string) => (mode === 'keyboard' ? `${task}/${viewport}/keyboard` : `${task}/${viewport}`);

export function expectedUnits(e: Expected): string[] {
  if (e.kind === 'l3') return [taskKey(e.task ?? '', e.viewport ?? '', e.mode ?? 'pointer')];
  const viewports = e.kind === 'l2' ? [e.viewport ?? ''] : (e.viewports ?? []);
  return viewports.flatMap((v) => (e.themes ?? []).map((t) => configKey(e.path ?? '', v, t)));
}

interface TestTiming {
  title: string;
  project: string;
  duration: number;
  start: number;
  status: string;
}

export interface RunTiming {
  wall_s: number;
  workers: number;
  status: string;
  // Only projects that ran a test are listed.
  projects: Record<string, { tests: number; wall_s: number; sum_s: number }>;
  slowest: Array<{ title: string; project: string; seconds: number; status: string }>;
  // Tests that failed as expected: the regression assets of open findings (core/findings.ts).
  known_failures?: string[];
  // Tests whose outcome was not the expected one: a new failure, or an open finding's asset that
  // passed. Any entry makes the run exit 1.
  unexpected?: string[];
  // The same tests as `[<project>] <title>`, and every test whose outcome was the expected one (it
  // passed, or an open finding's asset failed as its record states): the ratchet takes a failed
  // run only when a later run of the round passed each test it failed (T41 of the R-04 review).
  unexpected_tests?: string[];
  passed?: string[];
  // The host's 1, 5 and 15 minute load averages when the run began and when it ended, so a
  // round's notes can cite the load of a run from round.json (T33 of the R-03 report review).
  load?: { before: number[]; after: number[] };
}

type Result = 'pass' | 'fail' | 'not run';
const hostLoad = () => loadavg().map((v) => r(v, 2));
const serious = (v: AxeViolation) => v.impact === 'serious' || v.impact === 'critical';
const sum = (values: number[]) => values.reduce((n, v) => n + v, 0);

// Later records replace earlier ones configuration by configuration: a rerun of a test measures
// the same page, viewport and theme again, and counts once.
export function mergeRecords(records: L2Record[]): L2Record[] {
  const byTest = new Map<string, L2Record>();
  for (const rec of records) {
    const key = `${rec.path}@${rec.viewport}`;
    const seen = byTest.get(key);
    if (!seen) {
      byTest.set(key, { ...rec, configs: [...rec.configs] });
      continue;
    }
    const configs = new Map(seen.configs.map((c) => [`${c.viewport}/${c.theme}`, c]));
    for (const c of rec.configs) configs.set(`${c.viewport}/${c.theme}`, c);
    byTest.set(key, { ...rec, configs: [...configs.values()] });
  }
  for (const rec of byTest.values()) rec.configs.sort((a, b) => `${a.viewport}/${a.theme}`.localeCompare(`${b.viewport}/${b.theme}`));
  return [...byTest.values()].sort((a, b) => `${a.path}@${a.viewport}`.localeCompare(`${b.path}@${b.viewport}`));
}

export function mergeTaskRuns(runs: L3Record[]): L3Record[] {
  const byKey = new Map<string, L3Record>();
  for (const rec of runs) byKey.set(taskKey(rec.run.task, rec.viewport, rec.run.mode), rec);
  return [...byKey.entries()].sort(([a], [b]) => a.localeCompare(b)).map(([, rec]) => rec);
}

function coverageOf(kind: Expected['kind'], expected: Expected[], measured: Set<string>) {
  const these = expected.filter((e) => e.kind === kind);
  const units = new Set(these.flatMap(expectedUnits));
  // Records nobody declared (a run without annotations) still count as measured units.
  for (const m of measured) units.add(m);
  const total = Math.max(0, ...these.map((e) => e.total));
  const missing = [...units].filter((u) => !measured.has(u)).sort();
  const undeclared = total > units.size ? total - units.size : 0;
  return { units, missing, undeclared, expected: Math.max(total, units.size) };
}

export function summarize(l2In: L2Record[], l3In: L3Record[], expected: Expected[] = [], measuredIn: L2Record[] = []) {
  const l2 = mergeRecords(l2In);
  const l3 = mergeTaskRuns(l3In);
  const measuredRecords = mergeRecords(measuredIn);
  const configs = l2.flatMap((rec) => rec.configs.filter((c) => c.gating).map((c) => ({ rec, c, key: configKey(rec.path, c.viewport, c.theme) })));
  const l2Cov = coverageOf('l2', expected, new Set(configs.map((x) => x.key)));
  const l3Cov = coverageOf('l3', expected, new Set(l3.map(({ viewport, run }) => taskKey(run.task, viewport, run.mode))));
  const mCov = coverageOf(
    'measured',
    expected,
    new Set(measuredRecords.flatMap((rec) => rec.configs.map((c) => configKey(rec.path, c.viewport, c.theme)))),
  );

  // A page is measured when every configuration expected of it was measured.
  const pageOf = (unit: string) => unit.slice(0, unit.lastIndexOf('@'));
  const pages = new Set([...l2Cov.units].map(pageOf));
  const pagesUnmeasured = new Set(l2Cov.missing.map(pageOf));
  const pagesMeasured = [...pages].filter((p) => !pagesUnmeasured.has(p));
  const unmeasuredPages = pagesUnmeasured.size;
  const incomplete = unmeasuredPages > 0 || l2Cov.undeclared > 0;
  const of = pagesMeasured.length;

  const byConfig: Record<string, { expected: number; measured: number }> = {};
  for (const unit of l2Cov.units) {
    const k = unit.slice(unit.lastIndexOf('@') + 1);
    const entry = (byConfig[k] ??= { expected: 0, measured: 0 });
    entry.expected += 1;
    if (!l2Cov.missing.includes(unit)) entry.measured += 1;
  }

  const scenarios: Record<string, Record<string, number>> = {};
  for (const { rec, c, key } of configs) {
    const axe = (c.axe ?? []).filter(serious);
    scenarios[key] = {
      'layout.spills': c.layout.spills.length,
      'layout.truncated_texts': c.layout.truncated.length,
      'always_visible.failures': c.alwaysVisible.filter(gated).length,
      'axe.serious_critical_rules': new Set(axe.map((v) => v.id)).size,
      'axe.serious_critical_nodes': sum(axe.map((v) => v.nodes)),
      // The page load is measured once per test, under the theme it loaded in (`loadTheme`; T34 of
      // the R-04 review: it was filed under the first configuration, dark, while it loaded light).
      ...((rec.loadTheme ? c.theme === rec.loadTheme : c === rec.configs[0]) ? { 'layout_shift.load': r(rec.layoutShift), 'motion.running': rec.motion.running.length + (rec.motion.smoothScroll ? 1 : 0) } : {}),
    };
  }

  const pagesWith = (pred: (c: L2Config, rec: L2Record) => boolean) => new Set(configs.filter(({ rec, c }) => pred(c, rec)).map(({ rec }) => rec.path)).size;
  const pageGate = (pagesFailing: number): Result => (of === 0 && !incomplete ? 'not run' : pagesFailing > 0 || incomplete ? 'fail' : 'pass');

  const gate1Pages = pagesWith((c) => c.layout.spills.length > 0);
  const gate1 = {
    pages: gate1Pages,
    of,
    unmeasured: unmeasuredPages,
    page_scroll: pagesWith((c) => c.layout.spills.some((s) => s.kind === 'page')),
    spills: sum(configs.map(({ c }) => c.layout.spills.length)),
    result: pageGate(gate1Pages),
  };
  const byElement: Record<string, number> = {};
  const measuredOnly: Record<string, number> = {};
  for (const { c } of configs) {
    for (const f of c.alwaysVisible) {
      const into = gated(f) ? byElement : measuredOnly;
      const k = gated(f) ? `${f.element}: ${f.problem}` : `${f.element}: ${f.problem} at ${f.state}`;
      into[k] = (into[k] ?? 0) + 1;
    }
  }
  const gate2Pages = pagesWith((c) => c.alwaysVisible.some(gated));
  const gate2 = {
    pages: gate2Pages,
    of,
    unmeasured: unmeasuredPages,
    failures: sum(Object.values(byElement)),
    by_element: byElement,
    // Failures in states the adapter does not gate: reported, never failing the gate.
    measured_only: measuredOnly,
    result: pageGate(gate2Pages),
  };

  const byRule: Record<string, { nodes: number; pages: number; impact: string; help: string }> = {};
  const rulePages = new Map<string, Set<string>>();
  for (const { rec, c } of configs) {
    for (const v of (c.axe ?? []).filter(serious)) {
      const entry = (byRule[v.id] ??= { nodes: 0, pages: 0, impact: v.impact, help: v.help });
      entry.nodes += v.nodes;
      if (v.impact === 'critical') entry.impact = 'critical';
      if (!rulePages.has(v.id)) rulePages.set(v.id, new Set());
      rulePages.get(v.id)!.add(rec.path);
    }
  }
  for (const [id, set] of rulePages) byRule[id].pages = set.size;
  const gate5Pages = pagesWith((c) => (c.axe ?? []).some(serious));
  const truncPages = pagesWith((c) => c.layout.truncated.length > 0);
  const motionPages = new Set(l2.filter((rec) => rec.motion.running.length > 0 || rec.motion.smoothScroll).map((rec) => rec.path)).size;

  // Task runs: pointer runs for gate 4, keyboard runs for gate 5's keyboard half.
  const unmeasuredTasks = l3Cov.missing.length + l3Cov.undeclared;
  const pointerMissing = l3Cov.missing.filter((u) => !u.endsWith('/keyboard')).length;
  const keyboardMissing = l3Cov.missing.filter((u) => u.endsWith('/keyboard')).length;
  const pointer = l3.filter(({ run }) => run.mode !== 'keyboard');
  const keyboard = l3.filter(({ run }) => run.mode === 'keyboard');
  const failed = pointer.filter(({ run }) => !run.completed).map(({ viewport, run }) => `${taskKey(run.task, viewport, run.mode)}: ${run.reason}`);
  const overLimit = pointer
    .filter(({ run, limit }) => run.completed && limit !== undefined && run.metrics['screen_only.actions'] > limit)
    .map(({ viewport, run, limit }) => `${taskKey(run.task, viewport, run.mode)}: ${run.metrics['screen_only.actions']} actions, limit ${limit}`);
  const taskResult = (runs: number, failing: number, missing: number): Result => (runs === 0 && missing === 0 ? 'not run' : failing > 0 || missing > 0 ? 'fail' : 'pass');
  const completionMiss = sum(l3.map(({ run }) => run.completion_miss));
  const falseComplete = sum(l3.map(({ run }) => run.false_complete));
  const keyboardCompleted = keyboard.filter(({ run }) => run.completed).length;
  const gates = {
    gate1_no_horizontal_overflow: gate1,
    gate2_always_visible: gate2,
    gate3_unmissable_completion: {
      completion_miss: completionMiss,
      false_complete: falseComplete,
      unmeasured: unmeasuredTasks,
      result: taskResult(l3.length, completionMiss + falseComplete, unmeasuredTasks),
    },
    gate4_screen_only: {
      tasks: pointer.length,
      completed: pointer.length - failed.length,
      unmeasured: pointerMissing + (l3Cov.undeclared > 0 ? l3Cov.undeclared : 0),
      failed,
      over_limit: overLimit,
      result: taskResult(pointer.length, failed.length + overLimit.length, pointerMissing + l3Cov.undeclared),
    },
    gate5_accessibility: {
      pages: gate5Pages,
      of,
      unmeasured: unmeasuredPages,
      nodes: sum(Object.values(byRule).map((v) => v.nodes)),
      by_rule: byRule,
      result: pageGate(gate5Pages),
    },
    gate5_keyboard: {
      tasks: keyboard.length,
      completed: keyboardCompleted,
      unmeasured: keyboardMissing,
      result: taskResult(keyboard.length, keyboard.length - keyboardCompleted, keyboardMissing),
    },
    truncation: { pages: truncPages, of, unmeasured: unmeasuredPages, texts: sum(configs.map(({ c }) => c.layout.truncated.length)), result: pageGate(truncPages) },
    reduced_motion: { pages: motionPages, of, unmeasured: unmeasuredPages, result: pageGate(motionPages) },
  };

  for (const { viewport, run } of l3) {
    if (!run.completed) continue; // a failed task has no comparable metrics; gate 4 holds it
    scenarios[taskKey(run.task, viewport, run.mode)] = {
      ...Object.fromEntries(Object.entries(run.metrics).map(([k, v]) => [k, r(v)])),
      completion_miss: run.completion_miss,
      false_complete: run.false_complete,
    };
  }
  const sorted = Object.fromEntries(Object.entries(scenarios).sort(([a], [b]) => a.localeCompare(b)));
  const declared = (cov: { undeclared: number }, kind: string, unit: string) => (cov.undeclared > 0 ? [`${cov.undeclared} ${kind} ${unit}(s) in no merged run`] : []);
  const coverage = {
    pages: { expected: pages.size, measured: of },
    configs: { expected: l2Cov.expected, measured: l2Cov.units.size - l2Cov.missing.length },
    by_config: Object.fromEntries(Object.entries(byConfig).sort(([a], [b]) => a.localeCompare(b))),
    tasks: { expected: l3Cov.expected, measured: l3Cov.units.size - l3Cov.missing.length },
    measured_only: { expected: mCov.expected, measured: mCov.units.size - mCov.missing.length },
    unmeasured: [
      ...l2Cov.missing,
      ...declared(l2Cov, 'l2', 'configuration'),
      ...l3Cov.missing,
      ...declared(l3Cov, 'l3', 'task run'),
      ...mCov.missing.map((u) => `measured ${u}`),
      ...declared(mCov, 'measured-only', 'configuration'),
    ],
  };
  return { scenarios: sorted, gates, coverage, records: { l2, l3, measured: measuredRecords } };
}

export type Summary = ReturnType<typeof summarize>;

// The run's raw output: what one Playwright run expected and measured.
export interface Raw {
  lane: string;
  engine: Engine;
  expected: Expected[];
  l2: L2Record[];
  l3: L3Record[];
  measured: L2Record[];
  timing: RunTiming;
}

export function roundOf(raw: Raw, round: string) {
  const s = summarize(raw.l2, raw.l3, raw.expected, raw.measured);
  return { round, lane: raw.lane, engine: raw.engine, coverage: s.coverage, gates: s.gates, scenarios: s.scenarios, timing: raw.timing };
}

export default class UxReporter implements Reporter {
  private expected: Expected[] = [];
  private l2: L2Record[] = [];
  private l3: L3Record[] = [];
  private measured: L2Record[] = [];
  private timings: TestTiming[] = [];
  private started = Date.now();
  private workers = 0;
  private known: string[] = [];
  private unexpected: string[] = [];
  private unexpectedTests: string[] = [];
  private passed: string[] = [];
  private loadBefore: number[] = [];

  onBegin(config: FullConfig, suite: Suite): void {
    this.started = Date.now();
    this.loadBefore = hostLoad();
    this.workers = config.workers;
    for (const test of suite.allTests()) {
      for (const a of test.annotations) if (a.type === EXPECT && a.description) this.expected.push(JSON.parse(a.description));
    }
  }

  onTestEnd(test: TestCase, result: TestResult): void {
    const project = test.parent.project()?.name ?? '';
    this.timings.push({ title: test.title, project, duration: result.duration, start: result.startTime.getTime(), status: result.status });
    // Retries are 0, so each test ends once and its outcome is final.
    // An asset that failed with a message its findings do not state says which (core/findings.ts).
    const why = [...result.annotations, ...test.annotations].find((a) => a.type === 'new failure')?.description;
    const name = `[${project}] ${test.title}`;
    if (test.outcome() === 'unexpected') {
      this.unexpected.push(`${name} (${result.status}, expected ${test.expectedStatus})${why ? `: ${why}` : ''}`);
      this.unexpectedTests.push(name);
    } else if (test.outcome() === 'expected') this.passed.push(name);
    if (test.expectedStatus === 'failed' && result.status === 'failed') this.known.push(`[${project}] ${test.title}`);
    for (const a of result.attachments) {
      if (!a.body) continue;
      if (a.name === 'ux-l2') this.l2.push(JSON.parse(a.body.toString()));
      if (a.name === 'ux-measured') this.measured.push(JSON.parse(a.body.toString()));
      if (a.name === 'ux-l3') this.l3.push(JSON.parse(a.body.toString()));
    }
  }

  onEnd(result: FullResult): void {
    const root = resolve(__dirname, '..');
    const out = resolve(process.env.UX_OUT ?? join(root, '.out'));
    mkdirSync(out, { recursive: true });
    const projects: RunTiming['projects'] = {};
    for (const name of [...new Set(this.timings.map((t) => t.project))].sort()) {
      const these = this.timings.filter((t) => t.project === name);
      const start = Math.min(...these.map((t) => t.start));
      const end = Math.max(...these.map((t) => t.start + t.duration));
      projects[name] = { tests: these.length, wall_s: r((end - start) / 1000, 1), sum_s: r(sum(these.map((t) => t.duration)) / 1000, 1) };
    }
    const slowest = [...this.timings]
      .sort((a, b) => b.duration - a.duration)
      .slice(0, 10)
      .map((t) => ({ title: t.title, project: t.project, seconds: r(t.duration / 1000, 1), status: t.status }));
    const timing: RunTiming = {
      wall_s: r((Date.now() - this.started) / 1000, 1),
      workers: this.workers,
      status: result.status,
      projects,
      slowest,
      known_failures: [...this.known].sort(),
      unexpected: [...this.unexpected].sort(),
      unexpected_tests: [...this.unexpectedTests].sort(),
      passed: [...this.passed].sort(),
      load: { before: this.loadBefore, after: hostLoad() },
    };
    const raw: Raw = { lane: process.env.UX_LANE ?? 'gate', engine: engine(root), expected: this.expected, l2: this.l2, l3: this.l3, measured: this.measured, timing };
    writeFileSync(join(out, 'raw.json'), JSON.stringify(raw, null, 1) + '\n');
    const measuredRound = roundOf(raw, process.env.UX_ROUND ?? raw.lane);
    writeFileSync(join(out, 'round.json'), JSON.stringify(measuredRound, null, 2) + '\n');
    const c = measuredRound.coverage;
    console.log(
      `\nux: lane ${raw.lane}, engine ${raw.engine.commit}${raw.engine.dirty ? ' (dirty)' : ''} ${raw.engine.digest}; pages ${c.pages.measured} of ${c.pages.expected}, configurations ${c.configs.measured} of ${c.configs.expected}, task runs ${c.tasks.measured} of ${c.tasks.expected}, unmeasured ${c.unmeasured.length}; ${this.known.length} known failures, ${this.unexpected.length} new; wall ${timing.wall_s} s, ${timing.workers} workers; 1-minute load ${this.loadBefore[0]} before, ${timing.load?.after[0]} after`,
    );
    for (const t of timing.known_failures ?? []) console.log(`ux: known failure (an open finding's asset): ${t}`);
    for (const t of timing.unexpected ?? []) console.log(`ux: NEW: ${t}`);
    for (const [name, p] of Object.entries(projects)) console.log(`ux: project ${name}: ${p.tests} tests, wall ${p.wall_s} s`);
    console.log('ux: ten slowest tests');
    for (const t of slowest) console.log(`  ${t.seconds.toFixed(1).padStart(6)} s  [${t.project}] ${t.title} (${t.status})`);
    console.log(`ux: wrote ${join(out, 'round.json')}`);
  }
}
