// The round command's last stage (UX_STAGE=report, UX_ROUND=R-nn, UX_RUNS=<dir>,<dir>,...): merges
// the round's runs, each a directory under .out/ holding a raw.json, into rounds/R-nn/round.json;
// takes the worst-moment shots and the contact sheets; and writes report.md and the trend row from
// round.json. Runs of the gate lane are reported for their time and result; every other run is
// the round's measurement, merged configuration by configuration (a rerun replaces, never adds).
import { existsSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { test, type Browser, type Page } from '@playwright/test';
import { GATING, MEASURED, MOMENTS, ORIGIN, ROOT, ROUND, cards, scenarios, type Moment } from '../adapter';
import { frame, showTheme } from '../core/contract';
import { makeHermetic } from '../core/hermetic';
import { mergeRecords, mergeTaskRuns, type Raw } from '../core/reporter';
import { buildRound, mergeRaws, worstConfigs, writeReport, writeTrend, type MeasuredRound, type Shot } from '../core/round';

test.setTimeout(120_000);

const viewportOf = (name: string) => [...GATING, ...MEASURED].find((v) => v.name === name)!;

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

test('round report', async ({ browser }) => {
  if (!ROUND) throw new Error('set UX_ROUND=R-nn');
  const names = (process.env.UX_RUNS ?? '').split(',').map((s) => s.trim()).filter(Boolean);
  if (names.length === 0) throw new Error('set UX_RUNS to the run directories under .out/, comma-separated');
  const runs = names.map((name) => ({ name, raw: JSON.parse(readFileSync(join(ROOT, '.out', name, 'raw.json'), 'utf8')) as Raw }));
  const measurement = runs.filter((r) => r.raw.lane !== 'gate');
  const others = runs.filter((r) => r.raw.lane === 'gate');
  const date = new Date().toISOString().slice(0, 10);
  const round: MeasuredRound = buildRound({
    round: ROUND,
    date,
    measurement,
    others,
    cards: cards(ROUND),
    viewports: { gating: GATING.map((v) => v.name), measured: MEASURED.map((v) => v.name) },
    budgetSeconds: 120,
  });
  const merged = mergeRaws(measurement.map((m) => m.raw));
  const l2 = mergeRecords(merged.l2);
  const l3 = mergeTaskRuns(merged.l3);

  const dir = join(ROOT, 'rounds', ROUND);
  const shots = join(dir, 'shots');
  rmSync(shots, { recursive: true, force: true });
  mkdirSync(shots, { recursive: true });

  // Worst moments: the three worst page configurations, and every failed task where it stopped.
  const worst: Shot[] = [];
  for (const [i, w] of worstConfigs(l2, 3).entries()) {
    const page = await open(browser, w.rec.path, w.c.viewport, w.c.theme as 'light' | 'dark');
    const selectors = (w.c.axe ?? []).filter((v) => v.impact === 'serious' || v.impact === 'critical').flatMap((v) => v.targets).filter((s) => !s.includes(' iframe'));
    await mark(page, selectors);
    const file = `worst-${i + 1}-${w.c.viewport}-${w.c.theme}.png`;
    await page.screenshot({ path: join(shots, file), animations: 'disabled' });
    await page.context().close();
    worst.push({ file, caption: `${w.rec.path} at ${w.c.viewport}, ${w.c.theme}: ${w.why} (magenta outline: the first axe nodes)` });
  }
  for (const { viewport, run } of l3.filter((r) => !r.run.completed)) {
    const page = await open(browser, run.path, viewport, 'light');
    const file = `task-${run.task}-${viewport}-${run.mode}.png`;
    await page.screenshot({ path: join(shots, file), animations: 'disabled' });
    await page.context().close();
    worst.push({ file, caption: `task ${run.task} at ${viewport} (${run.mode}) stopped on ${run.path}: ${run.reason}` });
  }

  // Contact sheets: every moment in each theme, 0.75 s after the moment, scaled into one sheet.
  const contact: Shot[] = [];
  for (const theme of ['light', 'dark'] as const) {
    const frames: Array<{ name: string; data: string; mobile: boolean }> = [];
    for (const m of MOMENTS) frames.push({ name: `${m.name} (${m.viewport.name})`, data: (await moment(browser, m, theme)).toString('base64'), mobile: m.viewport.layout === 'mobile' });
    const context = await browser.newContext({ viewport: { width: 1320, height: 400 } });
    const sheet = await context.newPage();
    await sheet.setContent(
      `<body style="margin:8px;font:12px sans-serif;background:#888;display:flex;flex-wrap:wrap;gap:8px">${frames
        .map((f) => `<figure style="margin:0;background:#fff;padding:4px"><img src="data:image/png;base64,${f.data}" style="height:${f.mobile ? 300 : 200}px;display:block"><figcaption>${f.name}</figcaption></figure>`)
        .join('')}</body>`,
    );
    const file = `contact-${theme}.png`;
    await sheet.screenshot({ path: join(shots, file), fullPage: true });
    await context.close();
    contact.push({ file, caption: `contact sheet, ${theme} theme` });
  }

  const baselineFile = join(ROOT, 'baseline.json');
  const baseline = existsSync(baselineFile) ? JSON.parse(readFileSync(baselineFile, 'utf8')).scenarios : null;
  const rounds = readdirSync(join(ROOT, 'rounds')).filter((d) => /^R-\d+$/.test(d) && d < ROUND).sort();
  const last = rounds.at(-1);
  const previous = last && existsSync(join(ROOT, 'rounds', last, 'round.json')) ? JSON.parse(readFileSync(join(ROOT, 'rounds', last, 'round.json'), 'utf8')) : null;
  writeFileSync(join(dir, 'round.json'), JSON.stringify(round, null, 2) + '\n');
  writeReport(dir, round, {
    baseline,
    previous,
    expectations: Object.fromEntries(scenarios().map((s) => [s.name, s.expect.max_actions])),
    l3,
    worst,
    contact,
  });
  writeTrend(join(ROOT, 'trend.md'), round);
});
