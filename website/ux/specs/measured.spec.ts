// Measured-only viewports (round only, never gated): gate 1, gate 2 and truncation on every page,
// in the site's default dark theme, no axe.
import { test } from '@playwright/test';
import { BASE, MEASURED, ORIGIN, SITEMAP, alwaysVisible, pageKind } from '../adapter';
import { checkConfig, frame, showTheme } from '../core/contract';
import { makeHermetic } from '../core/hermetic';
import type { L2Record } from '../core/reporter';
import { sitemapPaths } from '../core/sitemap';

const pages = sitemapPaths(SITEMAP).filter((p) => p.startsWith(BASE.replace(/\/$/, '')));

test.describe.configure({ mode: 'parallel' });

for (const path of pages) {
  test(`measured ${path}`, async ({ browser }, testInfo) => {
    const context = await browser.newContext({ viewport: MEASURED[0], colorScheme: 'dark', reducedMotion: 'reduce' });
    await makeHermetic(context, ORIGIN);
    const page = await context.newPage();
    await page.goto(ORIGIN + path, { waitUntil: 'networkidle' });
    await showTheme(page, 'dark');
    const record: L2Record = { path, kind: pageKind(path), layoutShift: 0, motion: { running: [], smoothScroll: false }, blocked: {}, configs: [] };
    for (const viewport of MEASURED) {
      await page.setViewportSize({ width: viewport.width, height: viewport.height });
      await frame(page);
      const watched = alwaysVisible(page, viewport, record.kind);
      record.configs.push(await checkConfig(page, { viewport: viewport.name, theme: 'dark', gating: false, watched, axe: false }));
    }
    await testInfo.attach('ux-l2', { body: JSON.stringify(record), contentType: 'application/json' });
    await context.close();
  });
}
