// The layout contract for one loaded page at one viewport and theme: gate 1 (no horizontal
// overflow), gate 2 (always-visible elements), truncation, gate 5 (axe-core).
import AxeBuilder from '@axe-core/playwright';
import type { Locator, Page } from '@playwright/test';
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
export async function showTheme(page: Page, theme: 'light' | 'dark'): Promise<void> {
  await page.emulateMedia({ colorScheme: theme, reducedMotion: 'reduce' });
  const shows = (timeout: number) => page.waitForFunction((t) => document.documentElement.dataset.theme === t, theme, { timeout });
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
  const builder = new AxeBuilder({ page }).options({ resultTypes: ['violations'] });
  const result = await (scope === 'full' ? builder.withTags(AXE_TAGS) : builder.withRules(COLOUR_RULES)).analyze();
  return result.violations
    .map((v) => ({
      id: v.id,
      impact: v.impact ?? 'unknown',
      nodes: v.nodes.length,
      help: v.help,
      targets: v.nodes.slice(0, 3).map((n) => n.target.join(' ')),
    }))
    .sort((a, b) => a.id.localeCompare(b.id));
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
