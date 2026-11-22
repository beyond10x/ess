// L2 layout contract: every page of the built sitemap, at both gating viewports, in both themes.
// One test per page and viewport, one navigation each; the theme changes in place (the site follows
// the emulated colour scheme). The reporter merges a page's two records.
import { expect, test } from '@playwright/test';
import { BASE, GATING, ORIGIN, SITEMAP, THEMES, HYDRATED, alwaysVisible, pageKind } from '../adapter';
import { checkConfig, frame, gateFailures, showTheme } from '../core/contract';
import { makeHermetic } from '../core/hermetic';
import { probeMotion, readLayoutShift } from '../core/probes';
import type { L2Record } from '../core/reporter';
import { sitemapPaths } from '../core/sitemap';

const pages = sitemapPaths(SITEMAP).filter((p) => p.startsWith(BASE.replace(/\/$/, '')));

test.describe.configure({ mode: 'parallel' });

// What a test measured so far; attached after the test even when it timed out.
let record: L2Record | null = null;

test.afterEach(async ({}, testInfo) => {
  if (record) await testInfo.attach('ux-l2', { body: JSON.stringify(record), contentType: 'application/json' });
  record = null;
});

for (const path of pages) {
  for (const viewport of GATING) {
    test(`L2 ${path} ${viewport.name}`, async ({ browser }) => {
      const context = await browser.newContext({
        viewport: { width: viewport.width, height: viewport.height },
        colorScheme: THEMES[0],
        reducedMotion: 'reduce',
      });
      const hermetic = await makeHermetic(context, ORIGIN);
      const page = await context.newPage();
      const response = await page.goto(ORIGIN + path, { waitUntil: 'load' });
      expect(response?.status(), `${path} answers`).toBeLessThan(400);
      await page.waitForFunction(HYDRATED, undefined, { timeout: 10_000 });
      await frame(page);
      const current: L2Record = {
        path,
        kind: pageKind(path),
        layoutShift: await page.evaluate(readLayoutShift),
        motion: await page.evaluate(probeMotion),
        blocked: {},
        configs: [],
      };
      record = current;
      for (const theme of THEMES) {
        await showTheme(page, theme);
        const watched = alwaysVisible(page, viewport, current.kind);
        current.configs.push(
          await checkConfig(page, { viewport: viewport.name, theme, gating: true, watched, axe: theme === THEMES[0] ? 'full' : 'colour' }),
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
