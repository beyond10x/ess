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
// The finding records, and the repository root their regression assets' paths are relative to.
// An open finding's asset is an expected failure of the gate (core/findings.ts).
export const FINDINGS = join(ROOT, 'findings');
export const REPOSITORY = join(ROOT, '..', '..');
// The worker count of every lane: the same in package.json's `gate` script, Taskfile.yml's
// `UX_WORKERS` and every timed run of a round (T19 of the R-02 report review).
export const WORKERS = 4;
// The lane (UX_LANE): `gate` on every merge (the template pages and the anchor cards, 120 s), `slow`
// for the whole sitemap, every card, the measured-only viewports and the nested runs, as
// SLOW_L2_SLICES + 3 pool jobs.
export const LANE: 'gate' | 'slow' = process.env.UX_LANE === 'slow' ? 'slow' : 'gate';
export const SLOW_L2_SLICES = 6;

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

// The most elements a template page of the gate may build: axe and the probes cost with the page's
// elements, and the 120 s gate step holds only pages a test checks in seconds under load (UXF-025:
// the 3,405-element format history and the 1,897-element diagnostics page took 24 to 32 s a test
// in R-03). Longer pages run in the slow lane.
export const GATE_PAGE_ELEMENTS = 1200;

// Template pages: one page of each layout the site renders, so that every page template and every
// always-visible element is checked on every merge. The full sitemap runs in the `slow` lane.
// Listed slowest first (R-04 gate runs): the gate's workers take tests in this order, so the
// long tests start first and the short ones fill the end of the step.
// R-05 (UXF-025): one page of each kind `pageKind` tells apart, five in all. The release list, the
// status page and the lowering reference (a second release page with a sidebar and two more doc
// pages) run in the slow lane with every other page; no page kind and no always-visible element
// leaves the gate (the L1 test of the template pages holds both).
export const TEMPLATE_PAGES: Array<{ path: string; why: string }> = [
  { path: '/ess/docs/visualise', why: 'doc page without sidebar (interactive presentation)' },
  { path: '/ess/releases/conformance-in-any-language', why: 'release post with the recent-posts sidebar' },
  { path: '/ess/releases/archive', why: 'release archive, no sidebar by design' },
  { path: '/ess/docs/start/install', why: 'doc page with sidebar and table of contents' },
  { path: '/ess/', why: 'landing page (product template)' },
];

// The task runs of the gate lane, by card, gating viewport and mode. The slow lane runs every card
// at both gating viewports with the pointer, and the anchor cards with the keyboard too.
export interface GateTaskRun {
  task: string;
  viewport: string;
  mode: 'pointer' | 'keyboard';
}
// The gate (UXF-025, R-05): each anchor card once with the pointer, one at each gating viewport,
// and one with the keyboard alone at desktop-1280 for gate 5's keyboard half. The phone keyboard
// runs (128 to 131 key presses, 56 to 59 s at load 194 in R-05's first gate run) are slow-lane runs.
export const GATE_TASK_RUNS: GateTaskRun[] = [
  { task: 'install-linux-x86_64', viewport: 'desktop-1280', mode: 'keyboard' },
  { task: 'install-linux-x86_64', viewport: 'mobile-390', mode: 'pointer' },
  { task: 'first-conformance-run', viewport: 'desktop-1280', mode: 'pointer' },
];

// The most browser tests the gate lane runs: one L2 test per template page and gating viewport,
// one L3 test per gate task run (UXF-025, R-05). R-04's gate ran 24 at 4 workers: 17.9 s on a
// quiet host, cut at 120 s at a 1-minute load of 115 to 140, where one test took up to 8.9 times
// its quiet time (`L3 install-linux-x86_64 mobile-390 keyboard`, 5.4 s in gate-13, 49.4 s in
// gate-9). Thirteen hold about half the test-seconds, so that step ends inside 120 s at that load.
export const GATE_BROWSER_TESTS = 13;

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
  // Below 997 px the navbar's links, the theme toggle and the docs sidebar are not shown in place;
  // the menu button opens the menu that holds them. On a doc page the menu opens on the sidebar,
  // and "Back to main menu" shows the navbar's links. A collapsed sidebar category is not here:
  // the screen-only user opens one only when the card names it.
  { hiddenWhen: 'nav.navbar:not(.navbar-sidebar--show)', toggle: { role: 'button', name: 'toggle navigation bar' }, close: { role: 'button', name: 'close navigation bar' } },
  { hiddenWhen: '.navbar-sidebar__items--show-secondary', toggle: { role: 'button', name: 'back to main menu' } },
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
