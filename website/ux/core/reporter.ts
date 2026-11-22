// The measurement reporter. Runs in Playwright's main process, so it sees every worker's results
// in one place. It writes, under UX_OUT (default `.out/`):
//   raw.json    every L2 configuration and every L3 run, as the specs attached them;
//   round.json  the metric values per scenario (the ratchet's input) and the gate counts.
// It decides no gate and no ratchet: the specs assert the hard gates, the Rust ratchet compares.
// It prints the ten slowest tests, as the organization's test budget asks.
import { mkdirSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import type { FullResult, Reporter, TestCase, TestResult } from '@playwright/test/reporter';
import { round as r } from './metrics';
import type { TaskRun } from './bot';
import type { LayoutProbe } from './probes';

export interface AlwaysVisibleFailure {
  element: string;
  state: string;
  problem: 'missing' | 'out of viewport' | 'covered';
}

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

export interface L2Record {
  path: string;
  kind: string;
  layoutShift: number;
  motion: { running: string[]; smoothScroll: boolean };
  blocked: Record<string, number>;
  configs: L2Config[];
}

export interface L3Record {
  viewport: string;
  run: TaskRun;
}

interface TestTiming {
  title: string;
  project: string;
  duration: number;
  start: number;
}

const serious = (v: AxeViolation) => v.impact === 'serious' || v.impact === 'critical';

export function summarize(l2: L2Record[], l3: L3Record[]) {
  const scenarios: Record<string, Record<string, number>> = {};
  const gates = {
    gate1_no_horizontal_overflow: { pages: 0, page_scroll: 0, spills: 0, by_viewport: {} as Record<string, number> },
    gate2_always_visible: { pages: 0, failures: 0, by_element: {} as Record<string, number> },
    gate3_unmissable_completion: { completion_miss: 0, false_complete: 0 },
    gate4_screen_only: { tasks: 0, completed: 0, failed: [] as string[] },
    gate5_accessibility: { pages: 0, nodes: 0, by_rule: {} as Record<string, { nodes: number; pages: number; impact: string; help: string }>, keyboard_tasks: 0, keyboard_completed: 0 },
    truncation: { pages: 0, texts: 0 },
    reduced_motion: { pages: 0 },
  };
  const gatingConfigs = l2.flatMap((rec) => rec.configs.filter((c) => c.gating).map((c) => ({ rec, c })));
  const keys = new Set(gatingConfigs.map(({ c }) => `${c.viewport}/${c.theme}`));
  for (const key of [...keys].sort()) {
    const [viewport, theme] = key.split('/');
    const these = gatingConfigs.filter(({ c }) => c.viewport === viewport && c.theme === theme);
    scenarios[`site/${viewport}/${theme}`] = {
      'layout.page_scroll_pages': these.filter(({ c }) => c.layout.spills.some((s) => s.kind === 'page')).length,
      'layout.spill_pages': these.filter(({ c }) => c.layout.spills.length > 0).length,
      'layout.always_visible_pages': these.filter(({ c }) => c.alwaysVisible.length > 0).length,
      'layout.truncated_texts': these.reduce((n, { c }) => n + c.layout.truncated.length, 0),
      'axe.serious_critical_nodes': these.reduce((n, { c }) => n + (c.axe ?? []).filter(serious).reduce((m, v) => m + v.nodes, 0), 0),
      'axe.serious_critical_pages': these.filter(({ c }) => (c.axe ?? []).some(serious)).length,
    };
  }
  if (l2.length > 0) {
    scenarios['site/load'] = {
      'layout_shift.max': r(Math.max(0, ...l2.map((rec) => rec.layoutShift))),
      'reduced_motion.pages': l2.filter((rec) => rec.motion.running.length > 0 || rec.motion.smoothScroll).length,
    };
  }
  const pagesWith = (pred: (c: L2Config) => boolean) => l2.filter((rec) => rec.configs.some((c) => c.gating && pred(c))).length;
  gates.gate1_no_horizontal_overflow.pages = pagesWith((c) => c.layout.spills.length > 0);
  gates.gate1_no_horizontal_overflow.page_scroll = pagesWith((c) => c.layout.spills.some((s) => s.kind === 'page'));
  for (const { c } of gatingConfigs) {
    gates.gate1_no_horizontal_overflow.spills += c.layout.spills.length;
    if (c.layout.spills.length > 0) {
      const by = gates.gate1_no_horizontal_overflow.by_viewport;
      by[c.viewport] = (by[c.viewport] ?? 0) + 1;
    }
    gates.gate2_always_visible.failures += c.alwaysVisible.length;
    for (const f of c.alwaysVisible) {
      const k = `${f.element}: ${f.problem}`;
      gates.gate2_always_visible.by_element[k] = (gates.gate2_always_visible.by_element[k] ?? 0) + 1;
    }
    gates.truncation.texts += c.layout.truncated.length;
  }
  gates.gate2_always_visible.pages = pagesWith((c) => c.alwaysVisible.length > 0);
  gates.truncation.pages = pagesWith((c) => c.layout.truncated.length > 0);
  gates.reduced_motion.pages = l2.filter((rec) => rec.motion.running.length > 0 || rec.motion.smoothScroll).length;
  const rulePages = new Map<string, Set<string>>();
  for (const { rec, c } of gatingConfigs) {
    for (const v of (c.axe ?? []).filter(serious)) {
      const entry = (gates.gate5_accessibility.by_rule[v.id] ??= { nodes: 0, pages: 0, impact: v.impact, help: v.help });
      entry.nodes += v.nodes;
      if (v.impact === 'critical') entry.impact = 'critical';
      gates.gate5_accessibility.nodes += v.nodes;
      if (!rulePages.has(v.id)) rulePages.set(v.id, new Set());
      rulePages.get(v.id)!.add(rec.path);
    }
  }
  for (const [id, set] of rulePages) gates.gate5_accessibility.by_rule[id].pages = set.size;
  gates.gate5_accessibility.pages = pagesWith((c) => (c.axe ?? []).some(serious));

  for (const { viewport, run } of l3) {
    const name = run.mode === 'keyboard' ? `${run.task}/${viewport}/keyboard` : `${run.task}/${viewport}`;
    if (run.mode === 'keyboard') {
      gates.gate5_accessibility.keyboard_tasks += 1;
      if (run.completed) gates.gate5_accessibility.keyboard_completed += 1;
    } else {
      gates.gate4_screen_only.tasks += 1;
      if (run.completed) gates.gate4_screen_only.completed += 1;
      else gates.gate4_screen_only.failed.push(`${name}: ${run.reason}`);
    }
    gates.gate3_unmissable_completion.completion_miss += run.completion_miss;
    gates.gate3_unmissable_completion.false_complete += run.false_complete;
    if (!run.completed) continue; // a failed task has no comparable metrics; gate 4 holds it
    scenarios[name] = {
      ...Object.fromEntries(Object.entries(run.metrics).map(([k, v]) => [k, r(v)])),
      completion_miss: run.completion_miss,
      false_complete: run.false_complete,
    };
  }
  const sorted = Object.fromEntries(Object.entries(scenarios).sort(([a], [b]) => a.localeCompare(b)));
  return { scenarios: sorted, gates };
}

// One record per page: a page is measured by one test per viewport.
export function mergeByPath(records: L2Record[]): L2Record[] {
  const byPath = new Map<string, L2Record>();
  for (const rec of records) {
    const seen = byPath.get(rec.path);
    if (!seen) {
      byPath.set(rec.path, { ...rec, configs: [...rec.configs], blocked: { ...rec.blocked }, motion: { ...rec.motion, running: [...rec.motion.running] } });
      continue;
    }
    seen.configs.push(...rec.configs);
    seen.layoutShift = Math.max(seen.layoutShift, rec.layoutShift);
    seen.motion.running.push(...rec.motion.running);
    seen.motion.smoothScroll ||= rec.motion.smoothScroll;
    for (const [host, n] of Object.entries(rec.blocked)) seen.blocked[host] = (seen.blocked[host] ?? 0) + n;
  }
  for (const rec of byPath.values()) rec.configs.sort((a, b) => `${a.viewport}/${a.theme}`.localeCompare(`${b.viewport}/${b.theme}`));
  return [...byPath.values()];
}

export default class UxReporter implements Reporter {
  private l2: L2Record[] = [];
  private l3: L3Record[] = [];
  private measured: L2Record[] = [];
  private timings: TestTiming[] = [];
  private started = Date.now();
  private failedTests = 0;

  onBegin(): void {
    this.started = Date.now();
  }

  onTestEnd(test: TestCase, result: TestResult): void {
    const project = test.parent.project()?.name ?? '';
    this.timings.push({ title: test.title, project, duration: result.duration, start: result.startTime.getTime() });
    if (result.status !== 'passed' && result.status !== 'skipped') this.failedTests += 1;
    for (const a of result.attachments) {
      if (!a.body) continue;
      if (a.name === 'ux-l2') (project === 'measured' ? this.measured : this.l2).push(JSON.parse(a.body.toString()));
      if (a.name === 'ux-l3') this.l3.push(JSON.parse(a.body.toString()));
    }
  }

  onEnd(result: FullResult): void {
    const out = resolve(process.env.UX_OUT ?? join(__dirname, '..', '.out'));
    mkdirSync(out, { recursive: true });
    const wall = Date.now() - this.started;
    const projects: Record<string, { tests: number; wall_s: number; sum_s: number }> = {};
    for (const t of this.timings) {
      const p = (projects[t.project] ??= { tests: 0, wall_s: 0, sum_s: 0 });
      p.tests += 1;
      p.sum_s += t.duration / 1000;
    }
    for (const name of Object.keys(projects)) {
      const these = this.timings.filter((t) => t.project === name);
      const start = Math.min(...these.map((t) => t.start));
      const end = Math.max(...these.map((t) => t.start + t.duration));
      projects[name].wall_s = r((end - start) / 1000, 1);
      projects[name].sum_s = r(projects[name].sum_s, 1);
    }
    const sortL2 = (a: L2Record, b: L2Record) => a.path.localeCompare(b.path);
    this.l2 = mergeByPath(this.l2);
    this.l2.sort(sortL2);
    this.measured.sort(sortL2);
    this.l3.sort((a, b) => `${a.run.task}/${a.viewport}/${a.run.mode}`.localeCompare(`${b.run.task}/${b.viewport}/${b.run.mode}`));
    const { scenarios, gates } = summarize(this.l2, this.l3);
    const l2Tests = this.timings.filter((t) => t.project === 'l2').length;
    const coverage = { configs_expected: l2Tests * 2, configs_measured: this.l2.reduce((n, rec) => n + rec.configs.length, 0) };
    const timing = { wall_s: r(wall / 1000, 1), projects, status: result.status, coverage };
    writeFileSync(join(out, 'raw.json'), JSON.stringify({ l2: this.l2, l3: this.l3, measured: this.measured, timing }, null, 1) + '\n');
    writeFileSync(
      join(out, 'round.json'),
      JSON.stringify({ round: process.env.UX_ROUND ?? 'gate', pages: this.l2.length, gates, scenarios, timing }, null, 2) + '\n',
    );
    const slowest = [...this.timings].sort((a, b) => b.duration - a.duration).slice(0, 10);
    console.log(`\nux: ${this.l2.length} pages, ${this.l3.length} task runs, ${this.failedTests} failed tests, wall ${timing.wall_s} s`);
    for (const [name, p] of Object.entries(projects)) console.log(`ux: project ${name}: ${p.tests} tests, wall ${p.wall_s} s`);
    console.log('ux: ten slowest tests');
    for (const t of slowest) console.log(`  ${(t.duration / 1000).toFixed(1).padStart(6)} s  [${t.project}] ${t.title}`);
    console.log(`ux: wrote ${join(out, 'round.json')}`);
  }
}
