// L2 layout contract: in the `gate` lane the template pages, in the `slow` lane every page of the
// built sitemap (tagged into SLOW_L2_SLICES slices, one project each), at both gating viewports, in
// both themes.
// One test per page and viewport, one navigation each; the theme changes in place (the site follows
// the emulated colour scheme). Each test declares what it is expected to measure before it runs,
// and attaches what it measured even when it is cut off, so the reporter counts the rest as
// unmeasured.
import { expect, test } from '@playwright/test';
import { BASE, FINDINGS, GATING, LANE, ORIGIN, REPOSITORY, SITEMAP, SLOW_L2_SLICES, TEMPLATE_PAGES, THEMES, HYDRATED, alwaysVisible, builtPage, hasSearchControl, pageKind } from '../adapter';
import { holdKnownFailures } from '../core/findings';
import { checkConfig, frame, gateFailures, showTheme } from '../core/contract';
import { makeHermetic } from '../core/hermetic';
import { probeMotion, readLayoutShift } from '../core/probes';
import { EXPECT, type Expected, type L2Record } from '../core/reporter';
import { sitemapPaths } from '../core/sitemap';

const pages = LANE === 'gate' ? TEMPLATE_PAGES.map((p) => p.path) : sitemapPaths(SITEMAP).filter((p) => p.startsWith(BASE.replace(/\/$/, '')));
const total = pages.length * GATING.length * THEMES.length;

test.describe.configure({ mode: 'parallel' });
holdKnownFailures(test, FINDINGS, REPOSITORY);

// What a test measured so far; attached after the test even when it timed out.
let record: L2Record | null = null;

test.afterEach(async ({}, testInfo) => {
  if (record) await testInfo.attach('ux-l2', { body: JSON.stringify(record), contentType: 'application/json' });
  record = null;
});

// UXF-001 as one finding: the built template pages hold a search control once a search plugin is
// configured. Red while the site has none; no browser needed.
test('L2 site search: every template page builds a search control (UXF-001)', () => {
  const without = TEMPLATE_PAGES.map((p) => p.path).filter((path) => !hasSearchControl(builtPage(path)));
  expect(without, `UXF-001: ${without.length} of ${TEMPLATE_PAGES.length} template pages build no search control (no search plugin is configured)`).toEqual([]);
});

let index = 0;
for (const path of pages) {
  for (const viewport of GATING) {
    const expected: Expected = { kind: 'l2', path, viewport: viewport.name, themes: [...THEMES], total };
    // Dealt round-robin, so the heavy reference pages, which sit together, fall into different slices.
    const slice = `@slice-${(index++ % SLOW_L2_SLICES) + 1}`;
    test(`L2 ${path} ${viewport.name}`, { tag: slice, annotation: { type: EXPECT, description: JSON.stringify(expected) } }, async ({ browser }) => {
      const current: L2Record = {
        path,
        kind: pageKind(path),
        viewport: viewport.name,
        layoutShift: 0,
        motion: { running: [], smoothScroll: false },
        blocked: {},
        configs: [],
        phases: {},
      };
      record = current;
      const phase = async <T>(name: string, step: () => Promise<T>): Promise<T> => {
        const start = Date.now();
        current.phases![name] = -1; // until the step ends: a cut test shows where
        try {
          return await step();
        } finally {
          current.phases![name] = Date.now() - start;
        }
      };
      const context = await browser.newContext({
        viewport: { width: viewport.width, height: viewport.height },
        colorScheme: THEMES[0],
        reducedMotion: 'reduce',
      });
      const hermetic = await makeHermetic(context, ORIGIN);
      const page = await context.newPage();
      const response = await phase('goto', () => page.goto(ORIGIN + path, { waitUntil: 'load' }));
      expect(response?.status(), `${path} answers`).toBeLessThan(400);
      await phase('hydrate', async () => {
        await page.waitForFunction(HYDRATED, undefined, { timeout: 10_000 });
        await frame(page);
      });
      current.layoutShift = await page.evaluate(readLayoutShift);
      current.motion = await page.evaluate(probeMotion);
      for (const theme of THEMES) {
        await phase(`theme ${theme}`, () => showTheme(page, theme, { hydrated: true }));
        const watched = alwaysVisible(page, viewport, current.kind);
        current.configs.push(
          await checkConfig(page, { viewport: viewport.name, theme, gating: true, watched, axe: theme === THEMES[0] ? 'full' : 'colour', phases: current.phases }),
        );
      }
      current.blocked = Object.fromEntries(hermetic.blocked);
      await context.close();
      const failures = current.configs.flatMap(gateFailures);
      if (current.motion.running.length > 0 || current.motion.smoothScroll) {
        failures.push(`reduced motion: ${current.motion.running.length} running animation(s)${current.motion.smoothScroll ? ', smooth scrolling' : ''}`);
      }
      expect(failures, `${path} at ${viewport.name}: hard gates 1, 2, 5, truncation and reduced motion`).toEqual([]);
    });
  }
}
