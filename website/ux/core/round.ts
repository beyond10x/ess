// The round report (skill § 3.2). The round command merges the runs of a round (their raw.json)
// into round.json, and renders report.md from round.json alone: the report prints no number that
// round.json does not hold. Shots are taken by the caller and passed in by file name.
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import type { Engine } from './engine';
import { gated, summarize, type L2Config, type L2Record, type Raw, type RunTiming, type Summary } from './reporter';

export interface RunSummary {
  name: string;
  lane: string;
  engine: Engine;
  timing: RunTiming;
  coverage: { pages: string; configs: string; tasks: string; measured_only: string; unmeasured: number };
  results: Record<string, string>;
}

export interface MeasuredOnlyRow {
  viewport: string;
  configs: number;
  expected: number;
  page_scroll: number;
  spills: number;
  always_visible: number;
  truncated: number;
}

export interface MeasuredRound {
  round: string;
  date: string;
  lane: string;
  engine: Engine & { runs_agree: boolean };
  cards: { anchors: string[]; rotating: string };
  viewports: { gating: string[]; measured: string[] };
  runs: RunSummary[];
  coverage: Summary['coverage'];
  gates: Summary['gates'];
  scenarios: Summary['scenarios'];
  measured_only: MeasuredOnlyRow[];
  slowest: Array<{ run: string; title: string; project: string; seconds: number; status: string }>;
  slowest_pages: Array<{ path: string; viewport: string; phases: Record<string, number> }>;
  budget_seconds: number;
}

export interface Shot {
  file: string;
  caption: string;
}

export function mergeRaws(raws: Raw[]): Raw {
  return {
    lane: raws[0]?.lane ?? 'none',
    engine: raws[0]?.engine ?? { digest: 'none', commit: 'none', dirty: false },
    expected: raws.flatMap((r) => r.expected),
    l2: raws.flatMap((r) => r.l2),
    l3: raws.flatMap((r) => r.l3),
    measured: raws.flatMap((r) => r.measured),
    timing: raws[0]?.timing ?? { wall_s: 0, workers: 0, status: 'not run', projects: {}, slowest: [] },
  };
}

const ratio = (c: { expected: number; measured: number }) => `${c.measured} of ${c.expected}`;

export function runSummary(name: string, raw: Raw): RunSummary {
  const s = summarize(raw.l2, raw.l3, raw.expected, raw.measured);
  return {
    name,
    lane: raw.lane,
    engine: raw.engine,
    timing: raw.timing,
    coverage: {
      pages: ratio(s.coverage.pages),
      configs: ratio(s.coverage.configs),
      tasks: ratio(s.coverage.tasks),
      measured_only: ratio(s.coverage.measured_only),
      unmeasured: s.coverage.unmeasured.length,
    },
    results: Object.fromEntries(Object.entries(s.gates).map(([k, g]) => [k, g.result])),
  };
}

function measuredOnlyRows(records: L2Record[], viewports: string[], expected: number): MeasuredOnlyRow[] {
  return viewports.map((viewport) => {
    const configs = records.flatMap((rec) => rec.configs.filter((c) => c.viewport === viewport));
    return {
      viewport,
      configs: configs.length,
      expected,
      page_scroll: configs.filter((c) => c.layout.spills.some((s) => s.kind === 'page')).length,
      spills: configs.filter((c) => c.layout.spills.length > 0).length,
      always_visible: configs.filter((c) => c.alwaysVisible.some(gated)).length,
      truncated: configs.reduce((n, c) => n + c.layout.truncated.length, 0),
    };
  });
}

// `measurement`: the runs whose records are the round's measurement (the full lane, possibly in
// shards and reruns); `others`: runs reported for their time and result only (the gate's).
export function buildRound(input: {
  round: string;
  date: string;
  measurement: Array<{ name: string; raw: Raw }>;
  others: Array<{ name: string; raw: Raw }>;
  cards: MeasuredRound['cards'];
  viewports: MeasuredRound['viewports'];
  budgetSeconds: number;
}): MeasuredRound {
  const merged = mergeRaws(input.measurement.map((m) => m.raw));
  const s = summarize(merged.l2, merged.l3, merged.expected, merged.measured);
  const all = [...input.measurement, ...input.others];
  const digests = new Set(all.map((m) => m.raw.engine.digest));
  const measuredExpected = Math.max(0, ...merged.expected.filter((e) => e.kind === 'measured').map((e) => e.total)) / Math.max(1, input.viewports.measured.length);
  return {
    round: input.round,
    date: input.date,
    lane: merged.lane,
    engine: { ...merged.engine, runs_agree: digests.size <= 1 },
    cards: input.cards,
    viewports: input.viewports,
    runs: all.map((m) => runSummary(m.name, m.raw)),
    coverage: s.coverage,
    gates: s.gates,
    scenarios: s.scenarios,
    measured_only: measuredOnlyRows(s.records.measured, input.viewports.measured, measuredExpected),
    slowest: input.measurement
      .flatMap((m) => m.raw.timing.slowest.map((t) => ({ run: m.name, ...t })))
      .sort((a, b) => b.seconds - a.seconds)
      .slice(0, 10),
    slowest_pages: [...s.records.l2]
      .filter((rec) => rec.phases)
      .sort((a, b) => sumPhases(b) - sumPhases(a))
      .slice(0, 3)
      .map((rec) => ({ path: rec.path, viewport: rec.viewport, phases: rec.phases ?? {} })),
    budget_seconds: input.budgetSeconds,
  };
}

const sumPhases = (rec: L2Record) => Object.values(rec.phases ?? {}).reduce((n, v) => n + Math.max(0, v), 0);

const fmt = (v: number | undefined) => (v === undefined ? '–' : Number.isInteger(v) ? String(v) : v.toFixed(3).replace(/0+$/, '').replace(/\.$/, ''));

function table(head: string[], rows: Array<Array<string | number>>): string {
  const line = (cells: Array<string | number>) => `| ${cells.map((c) => String(c).replace(/\|/g, '\\|')).join(' | ')} |`;
  return [line(head), line(head.map(() => '---')), ...rows.map(line)].join('\n');
}

export function worstConfigs(l2: L2Record[], n: number): Array<{ rec: L2Record; c: L2Config; score: number; why: string }> {
  const scored = l2.flatMap((rec) =>
    rec.configs
      .filter((c) => c.gating)
      .map((c) => {
        const axe = (c.axe ?? []).filter((v) => v.impact === 'serious' || v.impact === 'critical');
        const axeNodes = axe.reduce((m, v) => m + v.nodes, 0);
        const visible = c.alwaysVisible.filter(gated);
        const score = axeNodes + 10 * c.layout.spills.length + 5 * visible.length + c.layout.truncated.length;
        const why = [
          axeNodes ? `axe ${axe.map((v) => `${v.id}×${v.nodes}`).join(', ')}` : '',
          c.layout.spills.length ? `${c.layout.spills.length} spill(s): ${c.layout.spills.slice(0, 2).map((s) => s.element).join('; ')}` : '',
          visible.length ? `always-visible: ${visible.map((f) => `${f.element} ${f.problem} at ${f.state}`).join(', ')}` : '',
          c.layout.truncated.length ? `${c.layout.truncated.length} truncated` : '',
        ]
          .filter(Boolean)
          .join('; ');
        return { rec, c, score, why };
      }),
  );
  scored.sort((a, b) => b.score - a.score || a.rec.path.localeCompare(b.rec.path));
  const seen = new Set<string>();
  return scored.filter((s) => s.score > 0 && !seen.has(s.rec.path) && seen.add(s.rec.path)).slice(0, n);
}

export interface ReportExtras {
  baseline: Record<string, Record<string, { value: number }>> | null;
  previous: MeasuredRound | null;
  expectations: Record<string, number>;
  l3: Raw['l3'];
  worst: Shot[];
  contact: Shot[];
}

const RESULT = (g: { result: string; unmeasured?: number }) => (g.unmeasured ? `${g.result} (${g.unmeasured} unmeasured)` : g.result);

export function writeReport(dir: string, m: MeasuredRound, x: ReportExtras): void {
  const g = m.gates;
  const c = m.coverage;
  const out: string[] = [];
  out.push(`# UX round ${m.round}: ESS documentation website`, '');
  out.push(
    `Date ${m.date}. Engine: commit \`${m.engine.commit}\`${m.engine.dirty ? ' with uncommitted engine changes' : ''}, digest \`${m.engine.digest}\`${m.engine.runs_agree ? '' : '; the runs of this round used different engines (see Runs)'}. Gating viewports ${m.viewports.gating.join(', ')}; measured only ${m.viewports.measured.join(', ')}; themes light and dark. Hermetic: every request off the local origin aborted.`,
    '',
  );
  out.push(`Cards: anchors ${m.cards.anchors.map((a) => `\`${a}\``).join(', ')}; rotating \`${m.cards.rotating}\`. No person ran them this round: the screen-only user played every task (L3), the questionnaire has no answers, and no session was recorded.`, '');

  out.push('## Coverage', '');
  out.push(
    table(
      ['what', 'measured of expected'],
      [
        ['pages (every configuration measured)', ratio(c.pages)],
        ['page configurations (page × gating viewport × theme)', ratio(c.configs)],
        ...Object.entries(c.by_config).map(([k, v]) => [`  ${k}`, ratio(v)]),
        ['task runs', ratio(c.tasks)],
        ['measured-only configurations', ratio(c.measured_only)],
      ],
    ),
    '',
  );
  out.push(c.unmeasured.length === 0 ? 'Unmeasured: none.' : `Unmeasured (${c.unmeasured.length}, each fails its gate): ${c.unmeasured.slice(0, 20).map((u) => `\`${u}\``).join(', ')}${c.unmeasured.length > 20 ? ', …' : ''}.`, '');

  out.push('## Gates', '');
  const byElement = Object.entries(g.gate2_always_visible.by_element).map(([k, v]) => `${k} ×${v}`).join('; ');
  const measuredOnly = Object.entries(g.gate2_always_visible.measured_only).map(([k, v]) => `${k} ×${v}`).join('; ');
  const byRule = Object.entries(g.gate5_accessibility.by_rule).map(([k, v]) => `${k} (${v.impact}) ${v.nodes} nodes on ${v.pages} pages`).join('; ');
  const pagesOf = (n: number) => `${n} of ${g.gate1_no_horizontal_overflow.of} measured pages`;
  out.push(
    table(
      ['gate', 'result', 'counts'],
      [
        ['1 no horizontal overflow', RESULT(g.gate1_no_horizontal_overflow), `${pagesOf(g.gate1_no_horizontal_overflow.pages)}; ${g.gate1_no_horizontal_overflow.page_scroll} with page scroll; ${g.gate1_no_horizontal_overflow.spills} spills`],
        ['2 always visible', RESULT(g.gate2_always_visible), `${pagesOf(g.gate2_always_visible.pages)}; ${byElement || 'none'}${measuredOnly ? `; measured only, not gated: ${measuredOnly}` : ''}`],
        ['3 unmissable completion', RESULT(g.gate3_unmissable_completion), `completion_miss ${g.gate3_unmissable_completion.completion_miss}, false_complete ${g.gate3_unmissable_completion.false_complete}`],
        ['4 screen-only user', RESULT(g.gate4_screen_only), `${g.gate4_screen_only.completed} of ${g.gate4_screen_only.tasks} pointer runs completed`],
        ['5 accessibility (axe)', RESULT(g.gate5_accessibility), `${g.gate5_accessibility.nodes} serious/critical nodes on ${pagesOf(g.gate5_accessibility.pages)}; ${byRule || 'none'}`],
        ['5 keyboard only', RESULT(g.gate5_keyboard), `${g.gate5_keyboard.completed} of ${g.gate5_keyboard.tasks} keyboard runs completed`],
        ['truncation', RESULT(g.truncation), `${g.truncation.texts} texts on ${pagesOf(g.truncation.pages)}`],
        ['reduced motion', RESULT(g.reduced_motion), `${pagesOf(g.reduced_motion.pages)} animate or scroll smoothly under reduce`],
        ['6 cue truth', 'dropped', 'no guidance cue on a docs site'],
      ],
    ),
    '',
  );
  if (g.gate4_screen_only.failed.length > 0) out.push('Failed task runs:', '', ...g.gate4_screen_only.failed.map((f) => `- ${f}`), '');

  out.push('## Runs', '');
  out.push(`Every run of this round, with its own time against the ${m.budget_seconds} s step budget of the gate lane. A project that ran no test is not listed.`, '');
  out.push(
    table(
      ['run', 'lane', 'engine', 'workers', 'wall s', 'status', 'projects (tests, wall s)', 'pages', 'task runs', 'unmeasured'],
      m.runs.map((r) => [
        r.name,
        r.lane,
        `${r.engine.commit}${r.engine.dirty ? '+' : ''} ${r.engine.digest}`,
        r.timing.workers,
        r.timing.wall_s,
        r.timing.status,
        Object.entries(r.timing.projects).map(([p, v]) => `${p} ${v.tests}, ${v.wall_s}`).join('; ') || 'none',
        r.coverage.pages,
        r.coverage.tasks,
        r.coverage.unmeasured,
      ]),
    ),
    '',
  );
  out.push('Ten slowest tests of the measurement:', '');
  out.push(table(['s', 'status', 'run', 'test'], m.slowest.map((t) => [t.seconds, t.status, t.run, t.title])), '');
  if (m.slowest_pages.length) {
    out.push('Where the slowest pages spent their time (ms per step; -1: the step did not end):', '');
    for (const p of m.slowest_pages) out.push(`- ${p.path} at ${p.viewport}: ${Object.entries(p.phases).map(([k, v]) => `${k} ${v}`).join(', ')}`);
    out.push('');
  }

  out.push('## Screen-only user (L3)', '');
  out.push(
    table(
      ['task', 'viewport', 'mode', 'done', 'actions (limit)', 'steers', 'looks', 'glance p95', 'time to act max', 'reading load max', 'note'],
      x.l3.map(({ viewport, run }) => [
        run.task,
        viewport,
        run.mode,
        run.completed ? 'yes' : 'no',
        `${run.metrics['screen_only.actions']} (${run.mode === 'pointer' ? (x.expectations[run.task] ?? '–') : '–'})`,
        run.metrics['screen_only.steers'],
        run.metrics['screen_only.looks'],
        run.metrics['glance_cost.p95'],
        run.metrics['screen_only.time_to_act_steps.max'],
        run.metrics['reading_load.max'],
        run.reason ? run.reason.slice(0, 120) : '',
      ]),
    ),
    '',
  );

  out.push('## Metrics against the baseline and the last round', '');
  const tasks = Object.entries(m.scenarios).filter(([k]) => !k.startsWith('/'));
  const pages = Object.entries(m.scenarios).filter(([k]) => k.startsWith('/'));
  out.push(`All lower is better. ${pages.length} page configurations and ${tasks.length} task runs hold values; the page values are summed per viewport and theme here, every value is in round.json.`, '');
  const rows: Array<Array<string | number>> = [];
  const groups = new Map<string, Record<string, number>>();
  for (const [scenario, values] of pages) {
    const key = scenario.slice(scenario.lastIndexOf('@') + 1);
    const group = groups.get(key) ?? {};
    for (const [k, v] of Object.entries(values)) group[k] = (group[k] ?? 0) + v;
    groups.set(key, group);
  }
  const prevGroup = (key: string, metric: string) => {
    const prev = x.previous?.scenarios;
    if (!prev) return undefined;
    const these = Object.entries(prev).filter(([s]) => s.startsWith('/') && s.endsWith(`@${key}`));
    return these.length ? these.reduce((n, [, v]) => n + (v[metric] ?? 0), 0) : undefined;
  };
  for (const [key, values] of [...groups.entries()].sort()) {
    for (const [metric, value] of Object.entries(values)) rows.push([`pages @${key} (sum)`, metric, fmt(value), '–', fmt(prevGroup(key, metric))]);
  }
  for (const [scenario, values] of tasks) {
    for (const [metric, value] of Object.entries(values)) {
      rows.push([scenario, metric, fmt(value), fmt(x.baseline?.[scenario]?.[metric]?.value), fmt(x.previous?.scenarios?.[scenario]?.[metric])]);
    }
  }
  out.push(table(['scenario', 'key', 'this round', 'baseline', 'last round'], rows), '');

  out.push('## Measured-only viewports', '');
  out.push(
    table(
      ['viewport', 'pages measured', 'with page scroll', 'with a spill', 'with an always-visible element out of view, covered or missing (in the states gate 2 gates)', 'truncated texts'],
      m.measured_only.map((r) => [r.viewport, `${r.configs} of ${r.expected}`, r.page_scroll, r.spills, r.always_visible, r.truncated]),
    ),
    '',
  );

  out.push('## Worst moments', '');
  for (const s of x.worst) out.push(`- ${s.caption}`, `  ![${s.file}](shots/${s.file})`);
  out.push('', '## Contact sheets', '');
  for (const s of x.contact) out.push(`![${s.caption}](shots/${s.file})`, '');

  out.push('## Missing inputs', '');
  out.push('- L6 recorded live rounds and the questionnaire: no person took part in this round.');
  out.push('- Recordings (rrweb): none; the screen-only user\'s states are in the runs\' raw.json.');
  out.push('- L4 keyed replay, L5 discoverability probe, L7 review agents: run by the coordinator after this report.', '');
  writeFileSync(`${dir}/report.md`, out.join('\n'));
}

export const TREND_HEAD =
  '# UX trend: ESS documentation website\n\nOne row per round. Lower is better in every column but "measured" and "done".\n\n| round | date | engine | pages measured | unmeasured | gate 1 pages | gate 2 pages | axe serious/critical nodes | axe pages | pointer runs done | over the card limit | keyboard runs done | actions (sum, pointer) | steers (sum) |\n| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n';

export function writeTrend(file: string, m: MeasuredRound): void {
  const g = m.gates;
  const tasks = Object.entries(m.scenarios).filter(([k]) => !k.startsWith('/') && !k.endsWith('/keyboard'));
  const sumOf = (key: string) => tasks.reduce((n, [, v]) => n + (v[key] ?? 0), 0);
  const over = 'over_limit' in g.gate4_screen_only ? (g.gate4_screen_only as { over_limit: string[] }).over_limit.length : '–';
  const row = `| ${m.round} | ${m.date} | ${m.engine.commit} | ${m.coverage.pages.measured} of ${m.coverage.pages.expected} | ${m.coverage.unmeasured.length} | ${g.gate1_no_horizontal_overflow.pages} | ${g.gate2_always_visible.pages} | ${g.gate5_accessibility.nodes} | ${g.gate5_accessibility.pages} | ${g.gate4_screen_only.completed}/${g.gate4_screen_only.tasks} | ${over} | ${g.gate5_keyboard.completed}/${g.gate5_keyboard.tasks} | ${sumOf('screen_only.actions')} | ${sumOf('screen_only.steers')} |`;
  const existing = existsSync(file) ? readFileSync(file, 'utf8') : TREND_HEAD;
  const lines = existing.split('\n').filter((l) => !l.startsWith(`| ${m.round} |`));
  writeFileSync(file, lines.join('\n').replace(/\n*$/, '\n') + row + '\n');
}
