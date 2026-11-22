// The round command's last stage (UX_STAGE=report, UX_ROUND=R-nn): reads the measurement in
// .out/, takes the worst-moment shots and the contact sheets, and writes rounds/R-nn/ and trend.md.
import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { test, type Browser, type Page } from '@playwright/test';
import { GATING, MEASURED, MOMENTS, ORIGIN, ROOT, ROUND, cards, scenarios, type Moment } from '../adapter';
import { frame, showTheme } from '../core/contract';
import { makeHermetic } from '../core/hermetic';
import { summarize } from '../core/reporter';
import { worstConfigs, writeReport, writeTrend, type MeasuredRound, type Raw, type Shot } from '../core/round';

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
  const out = join(ROOT, '.out');
  const raw: Raw = JSON.parse(readFileSync(join(out, 'raw.json'), 'utf8'));
  const measured: MeasuredRound = JSON.parse(readFileSync(join(out, 'round.json'), 'utf8'));
  // The measured-only viewports run apart from the gate (UX_OUT=.out/measured), so the gate's own
  // run time stays its own.
  const measuredOnly = join(out, 'measured', 'raw.json');
  if (existsSync(measuredOnly)) raw.measured = (JSON.parse(readFileSync(measuredOnly, 'utf8')) as Raw).measured;
  // UX_L3_FROM=.out/l3: take L1 and L3 from a run of their own (after a fix to the screen-only user
  // alone) and recompute the gates and metrics from the merged records.
  if (process.env.UX_L3_FROM) {
    const other: Raw = JSON.parse(readFileSync(join(ROOT, process.env.UX_L3_FROM, 'raw.json'), 'utf8'));
    raw.l3 = other.l3;
    raw.l3Timing = other.timing;
    const merged = summarize(raw.l2, raw.l3);
    measured.scenarios = merged.scenarios;
    measured.gates = merged.gates;
  }
  const dir = join(ROOT, 'rounds', ROUND);
  const shots = join(dir, 'shots');
  rmSync(shots, { recursive: true, force: true });
  mkdirSync(shots, { recursive: true });

  // Worst moments: the three worst page configurations, and every failed task where it stopped.
  const worst: Shot[] = [];
  for (const [i, w] of worstConfigs(raw.l2, 3).entries()) {
    const page = await open(browser, w.rec.path, w.c.viewport, w.c.theme as 'light' | 'dark');
    const selectors = [
      ...(w.c.axe ?? []).filter((v) => v.impact === 'serious' || v.impact === 'critical').flatMap((v) => v.targets),
    ].filter((s) => !s.includes(' iframe'));
    await mark(page, selectors);
    const file = `worst-${i + 1}-${w.c.viewport}-${w.c.theme}.png`;
    await page.screenshot({ path: join(shots, file), animations: 'disabled' });
    await page.context().close();
    worst.push({ file, caption: `${w.rec.path} at ${w.c.viewport}, ${w.c.theme}: ${w.why} (magenta outline: the first axe nodes)` });
  }
  for (const { viewport, run } of raw.l3.filter((r) => !r.run.completed)) {
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
    await sheet.setContent(`<body style="margin:8px;font:12px sans-serif;background:#888;display:flex;flex-wrap:wrap;gap:8px">${frames
      .map((f) => `<figure style="margin:0;background:#fff;padding:4px"><img src="data:image/png;base64,${f.data}" style="height:${f.mobile ? 300 : 200}px;display:block"><figcaption>${f.name}</figcaption></figure>`)
      .join('')}</body>`);
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
  const date = new Date().toISOString().slice(0, 10);
  const commit = execFileSync('git', ['rev-parse', '--short=12', 'HEAD'], { cwd: ROOT }).toString().trim();
  const all = scenarios();
  const roundCards = cards(ROUND);
  writeFileSync(
    join(dir, 'round.json'),
    JSON.stringify({ ...measured, round: ROUND, date, commit, cards: roundCards, viewports: { gating: GATING.map((v) => v.name), measured: MEASURED.map((v) => v.name) } }, null, 2) + '\n',
  );
  writeReport(dir, {
    round: ROUND,
    date,
    commit,
    raw,
    measured,
    baseline,
    previous,
    cards: roundCards,
    expectations: Object.fromEntries(all.map((s) => [s.name, s.expect.max_actions])),
    gating: GATING.map((v) => v.name),
    measuredViewports: MEASURED.map((v) => v.name),
    worst,
    contact,
    budgetSeconds: 120,
  });
  writeTrend(join(ROOT, 'trend.md'), ROUND, date, measured);
});
