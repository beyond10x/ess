// The adapter of the UI iteration loop for the ESS documentation website (skill § 1, docs-site
// variant). The only file that knows this site; `core/` knows no site at all.
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import type { Page } from '@playwright/test';
import type { BotConfig, Revealer, Task } from './core/bot';
import type { Watched } from './core/contract';

export const ROOT = __dirname;
export const BUILD = join(ROOT, '..', 'build');
export const PORT = Number(process.env.UX_PORT ?? 4391);
export const BASE = '/ess/';
export const ORIGIN = `http://127.0.0.1:${PORT}`;
export const SITEMAP = join(BUILD, 'sitemap.xml');
export const ROUND = process.env.UX_ROUND ?? '';

export interface Viewport {
  name: string;
  width: number;
  height: number;
  layout: 'desktop' | 'mobile';
}

// Gating: every merge. 1280x800 is the smallest common laptop and shows the desktop layout with
// both the docs sidebar and the table of contents; 390x844 is a current phone and shows the other
// layout Docusaurus ships (below 997 px: menu button, no sidebar, no table of contents). Together
// they cover both layouts at their tightest common width.
export const GATING: Viewport[] = [
  { name: 'desktop-1280', width: 1280, height: 800, layout: 'desktop' },
  { name: 'mobile-390', width: 390, height: 844, layout: 'mobile' },
];

// Measured only: reported in a round, never gated.
export const MEASURED: Viewport[] = [
  { name: 'desktop-1440', width: 1440, height: 900, layout: 'desktop' },
  { name: 'desktop-1024', width: 1024, height: 768, layout: 'desktop' },
  { name: 'tablet-768', width: 768, height: 1024, layout: 'mobile' },
];

export const THEMES = ['light', 'dark'] as const;
export type Theme = (typeof THEMES)[number];

// The page is hydrated once the colour-mode toggle is enabled: Docusaurus renders it disabled
// until React has taken over, and only then does the site follow the colour scheme.
export function HYDRATED(): boolean {
  const button = document.querySelector('button[title*="dark and light mode"], button[aria-label*="dark and light mode"]');
  return !!button && !(button as HTMLButtonElement).disabled;
}

export type PageKind = 'landing' | 'doc' | 'doc-without-sidebar' | 'blog' | 'blog-without-sidebar' | 'other';

// Pages under /docs that sit in no sidebar (Docusaurus renders them without one).
const DOCS_WITHOUT_SIDEBAR = new Set(['/ess/docs/visualise']);
// Release pages the blog renders without its recent-posts sidebar, by design.
const BLOG_WITHOUT_SIDEBAR = new Set(['/ess/releases/archive']);

export function pageKind(path: string): PageKind {
  const p = path.replace(/\/$/, '');
  if (p === '/ess' || p === '') return 'landing';
  if (p.startsWith('/ess/docs')) return DOCS_WITHOUT_SIDEBAR.has(p) ? 'doc-without-sidebar' : 'doc';
  if (p.startsWith('/ess/releases')) return BLOG_WITHOUT_SIDEBAR.has(p) ? 'blog-without-sidebar' : 'blog';
  return 'other';
}

// The always-visible elements (gate 2): the navbar everywhere; the theme toggle and the sidebar at
// desktop sizes (Docusaurus moves the toggle and the sidebar into the menu below 997 px, so there
// the menu button is what must stay visible).
// - Search is not one of them: this site configures no search plugin (`docusaurus.config.ts`), so
//   there is nothing to keep in view. Its absence is one site finding, UXF-001, held by the L2 test
//   "L2 site search" over the built template pages (`hasSearchControl`).
// - The sidebar is gated at the top and the middle of a page. At the end the stock theme's sticky
//   column stops where the footer starts, so the sidebar is pushed up while the reader is at the
//   footer; that state is measured and reported, not gated (decision D15, rounds/R-02/decision.md).
export const SIDEBAR_STATES = ['top', 'middle'];
export function alwaysVisible(page: Page, viewport: Viewport, kind: PageKind | string): Watched[] {
  const watched: Watched[] = [{ name: 'navbar', locator: page.getByRole('navigation', { name: 'Main' }) }];
  if (viewport.layout === 'desktop') {
    watched.push({ name: 'theme toggle', locator: page.getByRole('button', { name: /dark and light mode/i }) });
    if (kind === 'doc') watched.push({ name: 'docs sidebar', locator: page.getByRole('navigation', { name: 'Docs sidebar' }), states: SIDEBAR_STATES });
    if (kind === 'blog') watched.push({ name: 'blog sidebar', locator: page.getByRole('navigation', { name: /blog recent posts/i }), states: SIDEBAR_STATES });
  } else {
    watched.push({ name: 'menu button', locator: page.getByRole('button', { name: /toggle navigation bar/i }) });
  }
  return watched;
}

// A search control in a built page: the classic theme's local search bar, an Algolia DocSearch
// button, or any search input or landmark. Matched on the markup the plugins render, not on a
// name containing "search" (which "Research notes" would match).
const SEARCH_MARKUP = /class="[^"]*\b(navbar__search|DocSearch-Button|DocSearch)\b[^"]*"|<input[^>]*type="search"|role="search"/;
export function hasSearchControl(html: string): boolean {
  return SEARCH_MARKUP.test(html);
}

// The built HTML of a page path.
export function builtPage(path: string): string {
  const rel = path.replace(/^\/ess\/?/, '').replace(/\/$/, '');
  const candidates = rel === '' ? ['index.html'] : [join(rel, 'index.html'), `${rel}.html`];
  for (const c of candidates) {
    try {
      return readFileSync(join(BUILD, c), 'utf8');
    } catch {
      // the other spelling
    }
  }
  throw new Error(`${path}: no built page under ${BUILD}`);
}

// Template pages: one page of each layout the site renders, so that every page template and every
// always-visible element is checked on every merge. The full sitemap runs in the `slow` lane.
export const TEMPLATE_PAGES: Array<{ path: string; why: string }> = [
  { path: '/ess/', why: 'landing page (product template)' },
  { path: '/ess/docs/start/install', why: 'doc page with sidebar and table of contents' },
  { path: '/ess/docs/reference/spec-versions', why: 'long reference table page' },
  { path: '/ess/docs/reference/diagnostics', why: 'generated reference page (cargo xtask diagnostics)' },
  { path: '/ess/docs/status/where-this-stands', why: 'status page' },
  { path: '/ess/docs/visualise', why: 'doc page without sidebar (interactive presentation)' },
  { path: '/ess/releases', why: 'release list with the recent-posts sidebar' },
  { path: '/ess/releases/conformance-in-any-language', why: 'release post' },
  { path: '/ess/releases/archive', why: 'release archive, no sidebar by design' },
];

// Regions the screen-only user tells apart; the first match wins, so nested regions come first.
// Primary: where a reader's eyes already are (the top bar and the article).
export const REGIONS: BotConfig['regions'] = [
  { name: 'menu', selector: '.navbar-sidebar' },
  { name: 'navbar', selector: 'nav.navbar' },
  { name: 'sidebar', selector: 'aside' },
  { name: 'toc', selector: '.theme-doc-toc-desktop, .table-of-contents' },
  { name: 'footer', selector: 'footer' },
  { name: 'main', selector: 'main' },
];
export const PRIMARY = ['navbar', 'main'];

export const REVEALERS: Revealer[] = [
  { inside: '.navbar-sidebar', hiddenWhen: 'nav.navbar:not(.navbar-sidebar--show)', toggle: { role: 'button', name: 'toggle navigation bar' } },
  { inside: '.navbar-sidebar__items > .navbar-sidebar__item:first-child', hiddenWhen: '.navbar-sidebar__items--show-secondary', toggle: { role: 'button', name: 'back to main menu' } },
  // Below 997 px the navbar's links and the docs sidebar are not shown in place; the menu renders
  // them when opened.
  { inside: 'nav.navbar .navbar__items', hiddenWhen: 'nav.navbar:not(.navbar-sidebar--show)', toggle: { role: 'button', name: 'toggle navigation bar' } },
  { inside: 'aside.theme-doc-sidebar-container', hiddenWhen: 'nav.navbar:not(.navbar-sidebar--show)', toggle: { role: 'button', name: 'toggle navigation bar' } },
  {
    inside: '.menu__list-item--collapsed',
    toggle: ':scope > .menu__list-item-collapsible > button.menu__caret, :scope > .menu__list-item-collapsible > a.menu__link--sublist-caret, :scope > a.menu__link--sublist-caret',
  },
];

export function botConfig(mode: BotConfig['mode']): BotConfig {
  return { regions: REGIONS, primary: PRIMARY, revealers: REVEALERS, mode, maxSteers: 40, maxTabs: 250 };
}

// Moments of the contact sheet: the states a reader passes, at a gating viewport.
export interface Moment {
  name: string;
  path: string;
  viewport: Viewport;
  act?: 'open-menu' | 'scroll-middle';
}
export const MOMENTS: Moment[] = [
  { name: 'landing', path: '/ess/', viewport: GATING[0] },
  { name: 'docs-intro', path: '/ess/docs', viewport: GATING[0] },
  { name: 'install', path: '/ess/docs/start/install', viewport: GATING[0] },
  { name: 'cli-middle', path: '/ess/docs/reference/cli', viewport: GATING[0], act: 'scroll-middle' },
  { name: 'release-post-index', path: '/ess/releases', viewport: GATING[0] },
  { name: 'mobile-landing', path: '/ess/', viewport: GATING[1] },
  { name: 'mobile-doc', path: '/ess/docs/start/install', viewport: GATING[1] },
  { name: 'mobile-menu', path: '/ess/docs/start/install', viewport: GATING[1], act: 'open-menu' },
];

// Scenarios: the page list comes from the built sitemap (core/sitemap.ts); the tasks live in
// scenarios/<name>/scenario.json with their start page, route of visible controls, and truth.
export interface Scenario extends Task {
  card: 'anchor' | 'rotating';
  persona: string;
  task: string;
  twists: string[];
  expect: { completion: string; max_actions: number };
}

export function scenarios(): Scenario[] {
  const dir = join(ROOT, 'scenarios');
  return readdirSync(dir)
    .sort()
    .map((name) => JSON.parse(readFileSync(join(dir, name, 'scenario.json'), 'utf8')) as Scenario);
}

// Two fixed anchor cards and one rotating card per round, rotating in name order by round number.
export function cards(round: string): { anchors: string[]; rotating: string } {
  const all = scenarios();
  const anchors = all.filter((s) => s.card === 'anchor').map((s) => s.name);
  const pool = all.filter((s) => s.card === 'rotating').map((s) => s.name);
  const n = Math.max(1, Number(round.replace(/\D/g, '')) || 1);
  return { anchors, rotating: pool[(n - 1) % pool.length] };
}

// The completion state of a task: its truth path, and the target heading or cell in view (or the
// truth theme on the page).
export const COMPLETION = 'the truth path is open and the target element is wholly in view, uncovered';
