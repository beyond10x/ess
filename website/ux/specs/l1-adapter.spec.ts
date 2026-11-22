// L1: the adapter states what this site has, and gate 2 gates only what the adapter gates.
import { expect, test } from '@playwright/test';
import { BASE, GATING, SITEMAP, TEMPLATE_PAGES, alwaysVisible, hasSearchControl, pageKind } from '../adapter';
import { checkAlwaysVisible } from '../core/contract';
import { sitemapPaths } from '../core/sitemap';

test.use({ viewport: { width: 400, height: 600 } });

test('L1 adapter: search is not an always-visible element; the site has none (UXF-001, one finding)', async ({ page }) => {
  await page.setContent('<body></body>');
  for (const viewport of GATING) {
    expect(alwaysVisible(page, viewport, 'doc').map((w) => w.name)).not.toContain('search');
  }
});

test('L1 adapter: the release archive expects no sidebar; a release post expects one', async ({ page }) => {
  await page.setContent('<body></body>');
  const desktop = GATING.find((v) => v.layout === 'desktop')!;
  expect(alwaysVisible(page, desktop, pageKind('/ess/releases/archive')).map((w) => w.name)).not.toContain('blog sidebar');
  expect(alwaysVisible(page, desktop, pageKind('/ess/releases/a-deployment-described')).map((w) => w.name)).toContain('blog sidebar');
});

test('L1 adapter: the sidebar is gated at the top and the middle of a page, not at its end (D15)', async ({ page }) => {
  await page.setContent('<body></body>');
  const desktop = GATING.find((v) => v.layout === 'desktop')!;
  const sidebar = alwaysVisible(page, desktop, 'doc').find((w) => w.name === 'docs sidebar');
  expect(sidebar?.states).toEqual(['top', 'middle']);
});

test('L1 gate 2 gates an element only in the states the adapter names and reports the others as measured only', async ({ page }) => {
  // A sticky column that ends where the footer starts: in view at the top and the middle, pushed
  // out at the end.
  await page.setContent(`<body style="margin:0">
    <div style="display:flex"><aside style="width:100px"><nav aria-label="Side" style="position:sticky;top:0;height:300px;background:#eee">side</nav></aside>
    <main style="height:2400px">text</main></div><footer style="height:900px;background:#333"></footer></body>`);
  const failures = await checkAlwaysVisible(page, [{ name: 'side', locator: page.getByRole('navigation', { name: 'Side' }), states: ['top', 'middle'] }]);
  expect(failures).toEqual([{ element: 'side', state: 'end', problem: 'out of viewport', gating: false }]);
});

test('L1 the site-search check finds a built search control and its absence', () => {
  expect(hasSearchControl('<nav class="navbar"><a class="navbar__item">Docs</a><button>Research notes</button></nav>')).toBe(false);
  expect(hasSearchControl('<div class="navbar__search"><input type="search" class="navbar__search-input"></div>')).toBe(true);
  expect(hasSearchControl('<button type="button" class="DocSearch DocSearch-Button" aria-label="Search">')).toBe(true);
});

test('L1 the template pages of the gate cover every page template and every always-visible element of the sitemap', async ({ page }) => {
  await page.setContent('<body></body>');
  const sitemap = sitemapPaths(SITEMAP).filter((p) => p.startsWith(BASE.replace(/\/$/, '')));
  const templates = TEMPLATE_PAGES.map((t) => t.path);
  const kinds = (paths: string[]) => [...new Set(paths.map(pageKind))].sort();
  const elements = (paths: string[]) => [...new Set(paths.flatMap((p) => GATING.flatMap((v) => alwaysVisible(page, v, pageKind(p)).map((w) => `${v.name} ${w.name}`))))].sort();
  expect(templates.filter((p) => !sitemap.includes(p)), 'every template page is a page of the site').toEqual([]);
  expect(kinds(templates)).toEqual(kinds(sitemap));
  expect(elements(templates)).toEqual(elements(sitemap));
});
