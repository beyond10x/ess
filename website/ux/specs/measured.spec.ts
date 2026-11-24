// Measured-only viewports (round only, never gated): gate 1, gate 2 and truncation on every page,
// in the site's default dark theme, no axe. Like L2, each test declares what it is expected to
// measure and attaches what it measured even when it is cut off.
import { test } from '@playwright/test';
import { BASE, FINDINGS, MEASURED, ORIGIN, REPOSITORY, SITEMAP, alwaysVisible, pageKind } from '../adapter';
import { holdKnownFailures } from '../core/findings';
import { checkConfig, frame, showTheme } from '../core/contract';
import { makeHermetic } from '../core/hermetic';
import { EXPECT, type Expected, type L2Record } from '../core/reporter';
import { sitemapPaths } from '../core/sitemap';

const pages = sitemapPaths(SITEMAP).filter((p) => p.startsWith(BASE.replace(/\/$/, '')));
const THEME = 'dark';

test.describe.configure({ mode: 'parallel' });
holdKnownFailures(test, FINDINGS, REPOSITORY);

let record: L2Record | null = null;

test.afterEach(async ({}, testInfo) => {
  if (record) await testInfo.attach('ux-measured', { body: JSON.stringify(record), contentType: 'application/json' });
  record = null;
});

for (const path of pages) {
  const expected: Expected = { kind: 'measured', path, viewports: MEASURED.map((v) => v.name), themes: [THEME], total: pages.length * MEASURED.length };
  test(`measured ${path}`, { annotation: { type: EXPECT, description: JSON.stringify(expected) } }, async ({ browser }) => {
    const current: L2Record = { path, kind: pageKind(path), viewport: MEASURED[0].name, layoutShift: 0, motion: { running: [], smoothScroll: false }, blocked: {}, configs: [] };
    record = current;
    const context = await browser.newContext({ viewport: MEASURED[0], colorScheme: THEME, reducedMotion: 'reduce' });
    await makeHermetic(context, ORIGIN);
    const page = await context.newPage();
    await page.goto(ORIGIN + path, { waitUntil: 'networkidle' });
    await showTheme(page, THEME);
    for (const viewport of MEASURED) {
      await page.setViewportSize({ width: viewport.width, height: viewport.height });
      await frame(page);
      const watched = alwaysVisible(page, viewport, current.kind);
      current.configs.push(await checkConfig(page, { viewport: viewport.name, theme: THEME, gating: false, watched, axe: false }));
    }
    await context.close();
  });
}
