// L1: the screen-only user is held to what is on screen and to each card's limit. Doctored pages
// served from a fake origin; no site, no server.
import { expect, test, type Page } from '@playwright/test';
import { ScreenOnlyUser, type Revealer, type Task } from '../core/bot';
import { summarize, type L3Record } from '../core/reporter';

test.use({ viewport: { width: 400, height: 600 } });

const user = (page: Page, revealers: Revealer[] = []) =>
  new ScreenOnlyUser(page, { regions: [{ name: 'main', selector: 'main' }], primary: ['main'], revealers, mode: 'pointer', maxSteers: 20, maxTabs: 20 });

async function serve(page: Page, pages: Record<string, string>): Promise<void> {
  await page.route('http://ux.test/**', (route) => {
    const path = new URL(route.request().url()).pathname;
    return route.fulfill({ contentType: 'text/html', body: pages[path] ?? '<body>404</body>' });
  });
}

const metrics = (actions: number) => ({
  'screen_only.actions': actions,
  'screen_only.looks': 0,
  'screen_only.steers': 0,
  'screen_only.time_to_act_steps.max': 1,
  'glance_cost.median': 1,
  'glance_cost.p95': 1,
  'reading_load.max': 10,
  'reading_load.median': 10,
});

test('L1 gate 4 fails a pointer run over its card\'s action limit, and holds a run within it', () => {
  const runs: L3Record[] = [
    { viewport: 'd', limit: 3, run: { task: 'over', mode: 'pointer', completed: true, reason: '', path: '/x', states: [], metrics: metrics(5), completion_miss: 0, false_complete: 0 } },
    { viewport: 'd', limit: 3, run: { task: 'within', mode: 'pointer', completed: true, reason: '', path: '/x', states: [], metrics: metrics(3), completion_miss: 0, false_complete: 0 } },
  ];
  const s = summarize([], runs);
  expect(s.gates.gate4_screen_only.over_limit).toEqual(['over/d: 5 actions, limit 3']);
  expect(s.gates.gate4_screen_only.result).toBe('fail');
});

const TABLES = `<body style="margin:0"><main>
  <h2 id="families">Families</h2><table><tr><td>009</td><td>a family</td></tr></table>
  <div style="height:1400px"></div>
  <h2 id="classes">Classes</h2><table><tr><td>009</td><td>0.69.0</td></tr></table>
  </main></body>`;

test('L1 the screen-only user checks the card\'s anchor: a target under another heading is not the truth', async ({ page }) => {
  await serve(page, { '/t': TABLES });
  const task: Task = { name: 'anchor', start: '/t', steps: [{ reveal: { role: 'cell', name: '^009$' } }], truth: { path: '/t', target: { role: 'cell', name: '^009$' }, anchor: 'families' } };
  expect((await user(page).run(task, 'http://ux.test')).completed).toBe(true);
  const wrong = await user(page).run({ ...task, truth: { ...task.truth, anchor: 'classes' } }, 'http://ux.test');
  expect(wrong.completed).toBe(false);
  expect(wrong.reason).toContain('anchor');
});

test('L1 the screen-only user checks the cell the card wants beside the target', async ({ page }) => {
  await serve(page, { '/t': TABLES });
  const truth = { path: '/t', target: { role: 'cell' as const, name: '^009$' }, anchor: 'families', beside: { role: 'cell' as const, name: '^0\\.\\d+\\.\\d+$' } };
  const run = await user(page).run({ name: 'beside', start: '/t', steps: [{ reveal: truth.target }], truth }, 'http://ux.test');
  expect(run.completed).toBe(false);
  expect(run.reason).toContain('beside');
});

test('L1 the screen-only user does not open a container to find a control the card does not name', async ({ page }) => {
  await serve(page, {
    '/a': `<body style="margin:0"><main><ul><li class="cat cat--collapsed"><button class="caret">Reference</button>
      <ul><li><a href="/b" style="display:none">Hidden target</a></li></ul></li></ul></main></body>`,
    '/b': '<body><main><h1>B</h1></main></body>',
  });
  const revealers: Revealer[] = [{ inside: '.cat--collapsed', toggle: ':scope > button.caret' }];
  const run = await user(page, revealers).run({ name: 'unnamed', start: '/a', steps: [{ click: { role: 'link', name: '^Hidden target$' } }], truth: { path: '/b' } }, 'http://ux.test');
  expect(run.completed).toBe(false);
  expect(run.reason).toContain('not on screen');
});

test('L1 the screen-only user scans from where it is, down to the end, then up: it does not know where the control is', async ({ page }) => {
  await serve(page, {
    '/m': `<body style="margin:0"><main><h2 style="margin:50px 0 0;height:40px">Target</h2><div style="height:2910px"></div></main>
      <script>window.scrollTo(0, 1200)</script></body>`,
  });
  const run = await user(page).run({ name: 'scan', start: '/m', steps: [{ reveal: { role: 'heading', name: '^Target$' } }], truth: { path: '/m', target: { role: 'heading', name: '^Target$' } } }, 'http://ux.test');
  expect(run.reason).toBe('');
  // From 1200 px: three screens down to the end (2400 px), then five up to the top. A user that
  // knew the heading's box would have scrolled up at once.
  expect(run.metrics['screen_only.actions']).toBe(8);
});

test('L1 gate 3 can fail: a target that arrives under a sticky bar is a completion miss', async ({ page }) => {
  await serve(page, {
    '/a': '<body style="margin:0"><main><a href="/b">Go</a></main></body>',
    // Doctored: the bar covers the heading at the top of the page.
    '/b': `<body style="margin:0"><header style="position:fixed;top:0;left:0;right:0;height:70px;background:#222;z-index:2">bar</header>
      <main><h1 style="margin:0;padding-top:20px">Arrived</h1><div style="height:3000px"></div></main></body>`,
  });
  const run = await user(page).run(
    { name: 'miss', start: '/a', steps: [{ click: { role: 'link', name: '^Go$' } }, { reveal: { role: 'heading', name: '^Arrived$' } }], truth: { path: '/b', target: { role: 'heading', name: '^Arrived$' } } },
    'http://ux.test',
  );
  expect(run.completion_miss).toBe(1);
  expect(summarize([], [{ viewport: 'd', run }]).gates.gate3_unmissable_completion.result).toBe('fail');
});

test('L1 the screen-only user reads a state\'s words once the page it acts on has rendered', async ({ page }) => {
  // A client-side navigation: the URL changes at once, the new page renders 400 ms later.
  await serve(page, {
    '/a': `<body style="margin:0"><main><p>one two three</p><a href="/b" id="go">Go</a></main><script>
      document.getElementById('go').onclick = (e) => { e.preventDefault(); history.pushState({}, '', '/b'); document.querySelector('main').innerHTML = '';
        setTimeout(() => { document.querySelector('main').innerHTML = '<h1>Arrived</h1><p>' + 'word '.repeat(50) + '</p>'; }, 400); };</script></body>`,
  });
  const run = await user(page).run(
    { name: 'render', start: '/a', steps: [{ click: { role: 'link', name: '^Go$' } }, { reveal: { role: 'heading', name: '^Arrived$' } }], truth: { path: '/b', target: { role: 'heading', name: '^Arrived$' } } },
    'http://ux.test',
  );
  expect(run.reason).toBe('');
  expect(run.states[1].words).toBe(51);
});
