// The round command's last two stages (UX_ROUND=R-nn, UX_RUNS=<dir>,<dir>,..., each a directory
// under .out/ holding a raw.json). Runs of the gate lane are reported for their time and result;
// every other run is the round's measurement, merged configuration by configuration (a rerun
// replaces, never adds).
//   UX_STAGE=round   merges the runs into rounds/R-nn/round.json, and takes the worst-moment shots
//                    and the contact sheets: four tests, each under the 60 s test bound.
//   UX_STAGE=report  writes report.md and the trend row from round.json, the baseline the ratchet
//                    took the round into, the last round and the hand-written rounds/R-nn/notes.md,
//                    whose every number round.json must hold (T33); no browser. Run it after
//                    `cargo xtask ux-ratchet --round website/ux/rounds/R-nn/round.json --update`:
//                    it refuses a baseline that does not hold this round (T23).
// The round stage replaces only the shots it takes (worst-*, task-*, contact-*): other shots of the
// round directory (a decision's, a change's before and after) stay.
import { existsSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { test, type Browser, type Page } from '@playwright/test';
import { GATING, MEASURED, MOMENTS, ORIGIN, ROOT, ROUND, cards, scenarios, type Moment } from '../adapter';
import { frame, showTheme } from '../core/contract';
import { makeHermetic } from '../core/hermetic';
import { mergeRecords, mergeTaskRuns, type L2Record, type L3Record, type Raw } from '../core/reporter';
import { buildRound, committedBaseline, mergeRaws, worstConfigs, writeReport, writeTrend, type MeasuredRound, type Shot } from '../core/round';

const viewportOf = (name: string) => [...GATING, ...MEASURED].find((v) => v.name === name)!;
const THEMES = ['light', 'dark'] as const;

function inputs() {
  if (!ROUND) throw new Error('set UX_ROUND=R-nn');
  const names = (process.env.UX_RUNS ?? '').split(',').map((s) => s.trim()).filter(Boolean);
  if (names.length === 0) throw new Error('set UX_RUNS to the run directories under .out/, comma-separated');
  const runs = names.map((name) => ({ name, raw: JSON.parse(readFileSync(join(ROOT, '.out', name, 'raw.json'), 'utf8')) as Raw }));
  const measurement = runs.filter((r) => r.raw.lane !== 'gate');
  const others = runs.filter((r) => r.raw.lane === 'gate');
  const merged = mergeRaws(measurement.map((m) => m.raw));
  const dir = join(ROOT, 'rounds', ROUND);
  return { measurement, others, l2: mergeRecords(merged.l2), l3: mergeTaskRuns(merged.l3), dir, shots: join(dir, 'shots') };
}

// The worst moments and their captions: a function of the runs, so the report stage names the
// shots the round stage took without a file between them.
function worstShots(l2: L2Record[], l3: L3Record[]) {
  const configs = worstConfigs(l2, 3).map((w, i) => ({
    w,
    shot: { file: `worst-${i + 1}-${w.c.viewport}-${w.c.theme}.png`, caption: `${w.rec.path} at ${w.c.viewport}, ${w.c.theme}: ${w.why} (magenta outline: the first axe nodes)` },
  }));
  const tasks = l3
    .filter((r) => !r.run.completed)
    .map(({ viewport, run }) => ({ viewport, run, shot: { file: `task-${run.task}-${viewport}-${run.mode}.png`, caption: `task ${run.task} at ${viewport} (${run.mode}) stopped on ${run.path}: ${run.reason}` } }));
  return { configs, tasks, shots: [...configs.map((c) => c.shot), ...tasks.map((t) => t.shot)] as Shot[] };
}
const contactShots: Shot[] = THEMES.map((theme) => ({ file: `contact-${theme}.png`, caption: `contact sheet, ${theme} theme` }));

async function open(browser: Browser, path: string, viewportName: string, theme: 'light' | 'dark'): Promise<Page> {
  const v = viewportOf(viewportName);
  const context = await browser.newContext({ viewport: { width: v.width, height: v.height }, colorScheme: theme, reducedMotion: 'reduce' });
  await makeHermetic(context, ORIGIN);
  const page = await context.newPage();
  await page.goto(ORIGIN + path, { waitUntil: 'networkidle' });
  await showTheme(page, theme);
  return page;
}

// Outline what a finding is about, so the shot shows it.
async function mark(page: Page, selectors: string[]): Promise<void> {
  await page.evaluate((sels) => {
    let first: Element | null = null;
    for (const s of sels) {
      for (const el of Array.from(document.querySelectorAll(s)).slice(0, 5)) {
        (el as HTMLElement).style.outline = '3px solid #ff00ff';
        (el as HTMLElement).style.outlineOffset = '1px';
        first ??= el;
      }
    }
    first?.scrollIntoView({ block: 'center', behavior: 'instant' });
  }, selectors);
  await frame(page);
}

async function moment(browser: Browser, m: Moment, theme: 'light' | 'dark'): Promise<Buffer> {
  const page = await open(browser, m.path, m.viewport.name, theme);
  if (m.act === 'open-menu') await page.getByRole('button', { name: /toggle navigation bar/i }).click();
  if (m.act === 'scroll-middle') await page.evaluate(() => window.scrollTo({ top: (document.documentElement.scrollHeight - innerHeight) / 2, behavior: 'instant' }));
  await page.waitForTimeout(750);
  const png = await page.screenshot({ animations: 'disabled' });
  await page.context().close();
  return png;
}

test('round: merge the runs into round.json', () => {
  const { measurement, others, dir } = inputs();
  const round: MeasuredRound = buildRound({
    round: ROUND,
    date: new Date().toISOString().slice(0, 10),
    measurement,
    others,
    cards: cards(ROUND),
    viewports: { gating: GATING.map((v) => v.name), measured: MEASURED.map((v) => v.name) },
    budgetSeconds: 120,
  });
  mkdirSync(dir, { recursive: true });
  writeFileSync(join(dir, 'round.json'), JSON.stringify(round, null, 2) + '\n');
});

// The three worst page configurations, and every failed task where it stopped.
test('round: worst moments', async ({ browser }) => {
  const { l2, l3, shots } = inputs();
  mkdirSync(shots, { recursive: true });
  for (const f of readdirSync(shots).filter((f) => /^(worst|task)-.*\.png$/.test(f))) rmSync(join(shots, f));
  const worst = worstShots(l2, l3);
  for (const { w, shot } of worst.configs) {
    const page = await open(browser, w.rec.path, w.c.viewport, w.c.theme as 'light' | 'dark');
    const selectors = (w.c.axe ?? []).filter((v) => v.impact === 'serious' || v.impact === 'critical').flatMap((v) => v.targets).filter((s) => !s.includes('iframe'));
    await mark(page, selectors);
    await page.screenshot({ path: join(shots, shot.file), animations: 'disabled' });
    await page.context().close();
  }
  for (const { viewport, run, shot } of worst.tasks) {
    const page = await open(browser, run.path, viewport, 'light');
    await page.screenshot({ path: join(shots, shot.file), animations: 'disabled' });
    await page.context().close();
  }
});

// Every moment in one theme, 0.75 s after the moment, scaled into one sheet.
for (const theme of THEMES) {
  test(`round: contact sheet, ${theme} theme`, async ({ browser }) => {
    const { shots } = inputs();
    mkdirSync(shots, { recursive: true });
    const frames: Array<{ name: string; data: string; mobile: boolean }> = [];
    for (const m of MOMENTS) frames.push({ name: `${m.name} (${m.viewport.name})`, data: (await moment(browser, m, theme)).toString('base64'), mobile: m.viewport.layout === 'mobile' });
    const context = await browser.newContext({ viewport: { width: 1320, height: 400 } });
    const sheet = await context.newPage();
    await sheet.setContent(
      `<body style="margin:8px;font:12px sans-serif;background:#888;display:flex;flex-wrap:wrap;gap:8px">${frames
        .map((f) => `<figure style="margin:0;background:#fff;padding:4px"><img src="data:image/png;base64,${f.data}" style="height:${f.mobile ? 300 : 200}px;display:block"><figcaption>${f.name}</figcaption></figure>`)
        .join('')}</body>`,
    );
    await sheet.screenshot({ path: join(shots, `contact-${theme}.png`), fullPage: true });
    await context.close();
  });
}

test('report: report.md and the trend row from round.json and the committed baseline', () => {
  const { l2, l3, dir } = inputs();
  const round = JSON.parse(readFileSync(join(dir, 'round.json'), 'utf8')) as MeasuredRound;
  const baseline = committedBaseline(readFileSync(join(ROOT, 'baseline.json'), 'utf8'), ROUND);
  const rounds = readdirSync(join(ROOT, 'rounds')).filter((d) => /^R-\d+$/.test(d) && d < ROUND).sort();
  const last = rounds.at(-1);
  const previous = last && existsSync(join(ROOT, 'rounds', last, 'round.json')) ? JSON.parse(readFileSync(join(ROOT, 'rounds', last, 'round.json'), 'utf8')) : null;
  writeReport(dir, round, {
    baseline,
    previous,
    expectations: Object.fromEntries(scenarios().map((s) => [s.name, s.expect.max_actions])),
    l3,
    worst: worstShots(l2, l3).shots,
    contact: contactShots,
    // The hand-written notes: every number one of round.json (T33 of the R-03 report review).
    notes: existsSync(join(dir, 'notes.md')) ? readFileSync(join(dir, 'notes.md'), 'utf8') : undefined,
  });
  writeTrend(join(ROOT, 'trend.md'), round);
});
