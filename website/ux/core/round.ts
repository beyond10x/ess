// The round report (skill § 3.2): from the measurement's raw.json and round.json to report.md,
// round.json and trend.md. Shots are taken by the caller and passed in by file name.
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import type { L2Config, L2Record, L3Record, summarize } from './reporter';

type Summary = ReturnType<typeof summarize>;

export interface Raw {
  l2: L2Record[];
  l3: L3Record[];
  measured: L2Record[];
  timing: {
    wall_s: number;
    projects: Record<string, { tests: number; wall_s: number; sum_s: number }>;
    status: string;
    coverage?: { configs_expected: number; configs_measured: number };
    // Runs that measured what a cut-off run left unrun; their records are merged into this one.
    continuations?: Array<Omit<Raw['timing'], 'continuations'>>;
    continuation?: Omit<Raw['timing'], 'continuations'>;
  };
  l3Timing?: Raw['timing'];
}

export interface MeasuredRound {
  round: string;
  pages: number;
  gates: Summary['gates'];
  scenarios: Summary['scenarios'];
  timing: Raw['timing'];
}

export interface Shot {
  file: string;
  caption: string;
}

export interface ReportInput {
  round: string;
  date: string;
  commit: string;
  raw: Raw;
  measured: MeasuredRound;
  baseline: Record<string, Record<string, { value: number }>> | null;
  previous: MeasuredRound | null;
  cards: { anchors: string[]; rotating: string };
  expectations: Record<string, number>;
  gating: string[];
  measuredViewports: string[];
  worst: Shot[];
  contact: Shot[];
  budgetSeconds: number;
}

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
        const score = axeNodes + 10 * c.layout.spills.length + 5 * c.alwaysVisible.filter((f) => f.problem !== 'missing').length + c.layout.truncated.length;
        const why = [
          axeNodes ? `axe ${axe.map((v) => `${v.id}×${v.nodes}`).join(', ')}` : '',
          c.layout.spills.length ? `${c.layout.spills.length} spill(s): ${c.layout.spills.slice(0, 2).map((s) => s.element).join('; ')}` : '',
          c.alwaysVisible.filter((f) => f.problem !== 'missing').length ? `always-visible: ${c.alwaysVisible.filter((f) => f.problem !== 'missing').map((f) => `${f.element} ${f.problem} at ${f.state}`).join(', ')}` : '',
          c.layout.truncated.length ? `${c.layout.truncated.length} truncated` : '',
        ].filter(Boolean).join('; ');
        return { rec, c, score, why };
      }),
  );
  scored.sort((a, b) => b.score - a.score || a.rec.path.localeCompare(b.rec.path));
  const seen = new Set<string>();
  return scored.filter((s) => s.score > 0 && !seen.has(s.rec.path) && seen.add(s.rec.path)).slice(0, n);
}

export function writeReport(dir: string, input: ReportInput): void {
  const { measured: m, raw } = input;
  const g = m.gates;
  const out: string[] = [];
  out.push(`# UX round ${input.round}: ESS documentation website`, '');
  out.push(`Date ${input.date}, commit \`${input.commit}\`, ${m.pages} pages from the built sitemap. Gating viewports ${input.gating.join(', ')}; measured only ${input.measuredViewports.join(', ')}; themes light and dark. Hermetic: every request off the local origin aborted.`, '');
  out.push(`Cards: anchors ${input.cards.anchors.map((a) => `\`${a}\``).join(', ')}; rotating \`${input.cards.rotating}\`. No person ran them this round: the screen-only user played every task (L3), the questionnaire has no answers, and no session was recorded.`, '');

  out.push('## Gates', '');
  const l2Wall = raw.timing.projects.l2?.wall_s ?? 0;
  const l3Wall = raw.timing.projects.l3?.wall_s ?? 0;
  const byElement = Object.entries(g.gate2_always_visible.by_element).map(([k, v]) => `${k} ×${v}`).join('; ');
  const byRule = Object.entries(g.gate5_accessibility.by_rule).map(([k, v]) => `${k} (${v.impact}) ${v.nodes} nodes on ${v.pages} pages`).join('; ');
  out.push(
    table(
      ['gate', 'result', 'counts'],
      [
        ['1 no horizontal overflow', g.gate1_no_horizontal_overflow.pages === 0 ? 'pass' : 'fail', `${g.gate1_no_horizontal_overflow.pages} of ${m.pages} pages; ${g.gate1_no_horizontal_overflow.page_scroll} with page scroll; ${g.gate1_no_horizontal_overflow.spills} spills; configurations by viewport ${JSON.stringify(g.gate1_no_horizontal_overflow.by_viewport)}`],
        ['2 always visible', g.gate2_always_visible.pages === 0 ? 'pass' : 'fail', `${g.gate2_always_visible.pages} of ${m.pages} pages; ${byElement || 'none'}`],
        ['3 unmissable completion', g.gate4_screen_only.tasks === 0 ? 'not run' : g.gate3_unmissable_completion.completion_miss + g.gate3_unmissable_completion.false_complete === 0 ? 'pass' : 'fail', `completion_miss ${g.gate3_unmissable_completion.completion_miss}, false_complete ${g.gate3_unmissable_completion.false_complete}`],
        ['4 screen-only user', g.gate4_screen_only.tasks === 0 ? 'not run' : g.gate4_screen_only.completed === g.gate4_screen_only.tasks ? 'pass' : 'fail', `${g.gate4_screen_only.completed} of ${g.gate4_screen_only.tasks} task runs completed`],
        ['5 accessibility (axe)', g.gate5_accessibility.pages === 0 ? 'pass' : 'fail', `${g.gate5_accessibility.nodes} serious/critical nodes on ${g.gate5_accessibility.pages} of ${m.pages} pages; ${byRule || 'none'}`],
        ['5 keyboard only', g.gate5_accessibility.keyboard_tasks === 0 ? 'not run' : g.gate5_accessibility.keyboard_completed === g.gate5_accessibility.keyboard_tasks ? 'pass' : 'fail', `${g.gate5_accessibility.keyboard_completed} of ${g.gate5_accessibility.keyboard_tasks} anchor runs completed`],
        ['truncation', g.truncation.pages === 0 ? 'pass' : 'fail', `${g.truncation.texts} texts on ${g.truncation.pages} pages`],
        ['reduced motion', g.reduced_motion.pages === 0 ? 'pass' : 'fail', `${g.reduced_motion.pages} pages animate or scroll smoothly under reduce`],
        ['6 cue truth', 'dropped', 'no guidance cue on a docs site'],
      ],
    ),
    '',
  );
  if (g.gate4_screen_only.failed.length > 0) {
    out.push('Failed task runs:', '', ...g.gate4_screen_only.failed.map((f) => `- ${f}`), '');
  }
  out.push(`Run time (workers 2, one run of l1, l2 and l3 together): L2 ${l2Wall} s, L3 ${l3Wall} s, whole step ${raw.timing.wall_s} s against the ${input.budgetSeconds} s budget; run status ${raw.timing.status}. Configurations measured: ${raw.timing.coverage?.configs_measured ?? '?'} of ${raw.timing.coverage?.configs_expected ?? '?'} (a test cut off at 60 s keeps what it measured).`, '');
  const rest = raw.timing.continuations ?? [];
  if (rest.length) {
    const total = raw.timing.wall_s + rest.reduce((s, c) => s + c.wall_s, 0);
    out.push(`That run was cut off, so ${rest.length} more run(s) measured the L2 tests it left unrun: ${rest.map((c) => `${c.projects.l2?.tests ?? 0} tests in ${c.wall_s} s (${c.status})`).join(', ')}. L2 over all runs: ${Math.round(total)} s of wall time.`, '');
  }
  if (raw.l3Timing) {
    const p = raw.l3Timing.projects;
    const more = raw.l3Timing.continuation;
    out.push(`L1 and L3 above come from a run of their own (after a fix to the screen-only user): L1 ${p.l1?.wall_s ?? 0} s, L3 ${p.l3?.wall_s ?? 0} s, that run ${raw.l3Timing.wall_s} s, status ${raw.l3Timing.status}.${more ? ` A second run measured the ${more.projects.l3?.tests ?? 0} task runs it left unrun in ${more.wall_s} s (${more.status}).` : ''}`, '');
  }
  const blocked: Record<string, number> = {};
  for (const rec of raw.l2) for (const [h, n] of Object.entries(rec.blocked)) blocked[h] = (blocked[h] ?? 0) + n;
  out.push(`Requests aborted off the local origin: ${Object.keys(blocked).length === 0 ? 'none' : Object.entries(blocked).map(([h, n]) => `${h} ×${n}`).join(', ')}.`, '');

  out.push('## Screen-only user (L3)', '');
  out.push(
    table(
      ['task', 'viewport', 'mode', 'done', 'actions (bar)', 'steers', 'looks', 'glance p95', 'time to act max', 'reading load max', 'note'],
      raw.l3.map(({ viewport, run }) => [
        run.task,
        viewport,
        run.mode,
        run.completed ? 'yes' : 'no',
        `${run.metrics['screen_only.actions']} (${input.expectations[run.task] ?? '–'})`,
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
  out.push('All lower is better. The bar of a site key is 0 (a hard gate); a task\'s action bar is its card\'s expectation.', '');
  const rows: Array<Array<string | number>> = [];
  for (const [scenario, values] of Object.entries(m.scenarios)) {
    for (const [key, value] of Object.entries(values)) {
      rows.push([scenario, key, fmt(value), fmt(input.baseline?.[scenario]?.[key]?.value), fmt(input.previous?.scenarios?.[scenario]?.[key])]);
    }
  }
  out.push(table(['scenario', 'key', 'this round', 'baseline', 'last round'], rows), '');

  out.push('## Measured-only viewports', '');
  const mv = input.measuredViewports.map((name) => {
    const configs = raw.measured.flatMap((rec) => rec.configs.filter((c) => c.viewport === name).map((c) => ({ rec, c })));
    return [
      name,
      configs.length,
      configs.filter(({ c }) => c.layout.spills.some((s) => s.kind === 'page')).length,
      configs.filter(({ c }) => c.layout.spills.length > 0).length,
      configs.filter(({ c }) => c.alwaysVisible.some((f) => f.problem !== 'missing')).length,
      configs.reduce((n, { c }) => n + c.layout.truncated.length, 0),
    ];
  });
  out.push(table(['viewport', 'pages measured', 'pages with page scroll', 'pages with a spill', 'pages with an always-visible element out of view or covered', 'truncated texts'], mv), '');

  out.push('## Worst moments', '');
  for (const s of input.worst) out.push(`- ${s.caption}`, `  ![${s.file}](shots/${s.file})`);
  out.push('', '## Contact sheets', '');
  for (const s of input.contact) out.push(`![${s.caption}](shots/${s.file})`, '');

  out.push('## Missing inputs', '');
  out.push('- L6 recorded live rounds and the questionnaire: no person took part in this round.');
  out.push('- Recordings (rrweb): none; the screen-only user\'s states are in `round.json` instead.');
  out.push('- L4 keyed replay, L5 discoverability probe, L7 review agents: run by the coordinator after this report.');
  out.push('- Baseline and last round: this is the first round, so both columns repeat or lack this round\'s values.', '');
  writeFileSync(`${dir}/report.md`, out.join('\n'));
}

export function writeTrend(file: string, round: string, date: string, m: MeasuredRound): void {
  const head = '# UX trend: ESS documentation website\n\nOne row per round. Lower is better in every column but "tasks done".\n\n| round | date | pages | gate 1 pages | gate 2 pages | axe serious/critical nodes | axe pages | tasks done | keyboard done | actions (sum, gating) | steers (sum) |\n| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n';
  const g = m.gates;
  const tasks = Object.entries(m.scenarios).filter(([k]) => !k.startsWith('site/') && !k.endsWith('/keyboard'));
  const sum = (key: string) => tasks.reduce((n, [, v]) => n + (v[key] ?? 0), 0);
  const row = `| ${round} | ${date} | ${m.pages} | ${g.gate1_no_horizontal_overflow.pages} | ${g.gate2_always_visible.pages} | ${g.gate5_accessibility.nodes} | ${g.gate5_accessibility.pages} | ${g.gate4_screen_only.completed}/${g.gate4_screen_only.tasks} | ${g.gate5_accessibility.keyboard_completed}/${g.gate5_accessibility.keyboard_tasks} | ${sum('screen_only.actions')} | ${sum('screen_only.steers')} |`;
  const existing = existsSync(file) ? readFileSync(file, 'utf8') : head;
  const lines = existing.split('\n').filter((l) => !l.startsWith(`| ${round} |`));
  const body = lines.join('\n').replace(/\n*$/, '\n');
  writeFileSync(file, body + row + '\n');
}
