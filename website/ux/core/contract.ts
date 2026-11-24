// The layout contract for one loaded page at one viewport and theme: gate 1 (no horizontal
// overflow), gate 2 (always-visible elements), truncation, gate 5 (axe-core).
import AxeBuilder from '@axe-core/playwright';
import type { Frame, Locator, Page } from '@playwright/test';
import axe from 'axe-core';
import { probeLayout, probeVisible } from './probes';
import type { AlwaysVisibleFailure, AxeViolation, L2Config } from './reporter';

export const AXE_TAGS = ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa'];

export interface Watched {
  name: string;
  locator: Locator;
  // The scroll states in which the element is gated; a failure in any other state is measured
  // and reported, never gated. Every state when absent.
  states?: string[];
}

export async function frame(page: Page): Promise<void> {
  await page.evaluate(
    () => new Promise<void>((resolve) => requestAnimationFrame(() => requestAnimationFrame(() => resolve()))),
  );
}

// The site follows the emulated colour scheme while no theme is stored, so a theme changes in
// place, through the site's own colour-mode code, without a reload.
// `hydrated`: the caller has waited for the site's code, so the change can only be slow (a large
// page re-renders in seconds under load), never missed: wait for it and do not reload. R-01 cut
// that wait at 3 s and reloaded, which cost 6 to 16 s per theme on the long reference pages.
export async function showTheme(page: Page, theme: 'light' | 'dark', options: { hydrated?: boolean } = {}): Promise<void> {
  await page.emulateMedia({ colorScheme: theme, reducedMotion: 'reduce' });
  const shows = (timeout: number) => page.waitForFunction((t) => document.documentElement.dataset.theme === t, theme, { timeout });
  if (options.hydrated) {
    await shows(30_000);
    await frame(page);
    return;
  }
  // Before hydration the site does not listen yet; a reload then reads the scheme at start.
  await shows(3_000).catch(async () => {
    await page.reload({ waitUntil: 'networkidle' });
    await shows(5_000);
  });
  await frame(page);
}

// The states a reader moves through on a page: the top, the middle, the end.
export async function scrollStates(page: Page): Promise<Array<{ state: string; y: number }>> {
  const height = await page.evaluate(() => document.documentElement.scrollHeight - window.innerHeight);
  const states = [{ state: 'top', y: 0 }];
  if (height > 50) states.push({ state: 'middle', y: Math.round(height / 2) }, { state: 'end', y: height });
  return states;
}

export async function checkAlwaysVisible(page: Page, watched: Watched[]): Promise<AlwaysVisibleFailure[]> {
  const failures: AlwaysVisibleFailure[] = [];
  for (const { state, y } of await scrollStates(page)) {
    await page.evaluate((top) => window.scrollTo({ top, behavior: 'instant' }), y);
    await frame(page);
    for (const w of watched) {
      // No waiting: an element the page has not rendered by now is missing.
      const handle = ((await w.locator.elementHandles()) as Array<import('@playwright/test').ElementHandle<Element>>)[0] ?? null;
      const v = handle ? await handle.evaluate(probeVisible) : await page.evaluate(probeVisible, null);
      const gating = !w.states || w.states.includes(state);
      if (!v.present) failures.push({ element: w.name, state, problem: 'missing', gating });
      else if (!v.inViewport) failures.push({ element: w.name, state, problem: 'out of viewport', gating });
      else if (v.covered) failures.push({ element: w.name, state, problem: 'covered', gating });
    }
  }
  await page.evaluate(() => window.scrollTo({ top: 0, behavior: 'instant' }));
  // A missing element is missing in every state: report it once, gated if any of its states is.
  return failures
    .map((f) => (f.problem === 'missing' ? { ...f, gating: failures.some((g) => g.element === f.element && g.problem === 'missing' && g.gating) } : f))
    .filter((f, i, all) => f.problem !== 'missing' || all.findIndex((g) => g.element === f.element && g.problem === 'missing') === i);
}

// `full`: every rule of AXE_TAGS. `colour`: only the rules whose result depends on the colours,
// for the second theme of a viewport whose structure the full run has already checked.
export const COLOUR_RULES = ['color-contrast', 'link-in-text-block'];
export async function runAxe(page: Page, scope: 'full' | 'colour' = 'full'): Promise<AxeViolation[]> {
  // Legacy mode runs axe in the top document only: it never enters a frame. The default mode runs
  // each frame partly and aggregates the results in a new blank page, which covers frames but on
  // the 17,337-element ess-ui reference page took 92 to 112 s for one rule, against 16 to 30 s for
  // every rule in legacy mode (R-02, under load 55 to 85). R-02 ran legacy mode alone and lost every
  // violation inside the visualise page's frame (UXF-021).
  // R-03 first used the default mode on a framed page; under load it reported nothing for the frame
  // in 2 of 6 runs, because @axe-core/playwright drops a frame whose run fails without a word. So
  // each frame runs axe on its own document, in legacy mode's way, and an error there fails the test.
  const builder = new AxeBuilder({ page }).setLegacyMode(true).options({ resultTypes: ['violations'] });
  const top = (await (scope === 'full' ? builder.withTags(AXE_TAGS) : builder.withRules(COLOUR_RULES)).analyze()).violations;
  const framed: axe.Result[] = [];
  for (const child of page.frames().filter((f) => f !== page.mainFrame() && !f.isDetached() && !f.url().startsWith('chrome-error:'))) {
    framed.push(...(await axeInFrame(child, scope)));
  }
  const byId = new Map<string, AxeViolation>();
  for (const v of [...top, ...framed]) {
    const seen = byId.get(v.id);
    const targets = v.nodes.map((n) => n.target.join(' '));
    if (seen) {
      seen.nodes += v.nodes.length;
      seen.targets = [...seen.targets, ...targets].slice(0, 3);
    } else {
      byId.set(v.id, { id: v.id, impact: v.impact ?? 'unknown', nodes: v.nodes.length, help: v.help, targets: targets.slice(0, 3) });
    }
  }
  return [...byId.values()].sort((a, b) => a.id.localeCompare(b.id));
}

// axe-core in one frame's own document; the frames inside it are found by page.frames().
async function axeInFrame(child: Frame, scope: 'full' | 'colour'): Promise<axe.Result[]> {
  await child.evaluate((source) => {
    if (!(window as unknown as { axe?: unknown }).axe) (0, eval)(source);
  }, axe.source);
  const runOnly = scope === 'full' ? { type: 'tag' as const, values: AXE_TAGS } : { type: 'rule' as const, values: COLOUR_RULES };
  return child.evaluate(
    (options) => (window as unknown as { axe: typeof axe }).axe.run(document, options as axe.RunOptions).then((r) => r.violations),
    { runOnly, resultTypes: ['violations'], iframes: false },
  );
}

export async function checkConfig(
  page: Page,
  options: { viewport: string; theme: string; gating: boolean; watched: Watched[]; axe: 'full' | 'colour' | false; phases?: Record<string, number> },
): Promise<L2Config> {
  // Each step's milliseconds, per theme, so a slow page says where its time went.
  const timed = async <T>(name: string, step: () => Promise<T>): Promise<T> => {
    const start = Date.now();
    // -1 until the step ends: a test cut off inside a step shows which.
    if (options.phases) options.phases[`${name} ${options.theme}`] = -1;
    try {
      return await step();
    } finally {
      if (options.phases) options.phases[`${name} ${options.theme}`] = Date.now() - start;
    }
  };
  const layout = await timed('layout', () => page.evaluate(probeLayout));
  const alwaysVisible = await timed('visible', () => checkAlwaysVisible(page, options.watched));
  const axe = options.axe ? await timed('axe', () => runAxe(page, options.axe as 'full' | 'colour')) : null;
  return { viewport: options.viewport, theme: options.theme, gating: options.gating, layout, alwaysVisible, axe, axeScope: options.axe || 'none' };
}

export function gateFailures(config: L2Config): string[] {
  const where = `${config.viewport}/${config.theme}`;
  const out: string[] = [];
  for (const s of config.layout.spills) out.push(`gate 1 ${where}: ${s.kind} spill ${s.element} right ${s.right} > ${s.limit}`);
  for (const f of config.alwaysVisible.filter((f) => f.gating !== false)) out.push(`gate 2 ${where}: ${f.element} ${f.problem} at ${f.state}`);
  for (const t of config.layout.truncated) out.push(`truncation ${where}: ${t.element} "${t.text}"`);
  for (const v of config.axe ?? []) {
    if (v.impact === 'serious' || v.impact === 'critical') out.push(`gate 5 ${where}: axe ${v.id} (${v.impact}) ${v.nodes} node(s): ${v.targets.join(' | ')}`);
  }
  return out;
}
