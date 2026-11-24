// L1: every check can fail. Each probe meets a page built to break it, and a page built to pass it;
// the metric functions meet canned states. No server, no site.
import { expect, test } from '@playwright/test';
import { FINDINGS, REPOSITORY } from '../adapter';
import { holdKnownFailures } from '../core/findings';
import { ScreenOnlyUser } from '../core/bot';
import { checkAlwaysVisible, runAxe } from '../core/contract';
import { median, percentile, taskMetrics } from '../core/metrics';
import { probeLayout, probeMotion } from '../core/probes';

test.use({ viewport: { width: 400, height: 600 } });
holdKnownFailures(test, FINDINGS, REPOSITORY);

test('L1 gate 1 finds a horizontal page scroll and an element past its box', async ({ page }) => {
  await page.setContent(`<body style="margin:0"><main><div style="width:200px"><p style="width:300px">wide</p></div>
    <div style="width:900px">past the viewport</div></main></body>`);
  const probe = await page.evaluate(probeLayout);
  expect(probe.spills.map((s) => s.kind).sort()).toEqual(['box', 'page', 'viewport']);
});

test('L1 gate 1 passes content inside a horizontal scroll container', async ({ page }) => {
  await page.setContent(`<body style="margin:0"><main><pre style="overflow-x:auto;width:300px"><code style="display:block;width:900px">long</code></pre></main></body>`);
  expect((await page.evaluate(probeLayout)).spills).toEqual([]);
});

test('L1 truncation finds an ellipsis without the full value, and passes one with a title', async ({ page }) => {
  await page.setContent(`<body><span style="display:block;width:60px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap">a long label that is cut</span>
    <span title="a long label that is cut too" style="display:block;width:60px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap">a long label that is cut too</span></body>`);
  const probe = await page.evaluate(probeLayout);
  expect(probe.truncated.map((t) => t.text)).toEqual(['a long label that is cut']);
});

test('L1 gate 2 finds a missing, an out-of-viewport and a covered element', async ({ page }) => {
  await page.setContent(`<body style="margin:0;height:3000px">
    <nav aria-label="Main" style="position:sticky;top:0;height:40px;background:#eee">bar</nav>
    <button style="position:absolute;top:900px">below</button>
    <a href="#" style="position:fixed;top:100px;left:10px">covered</a>
    <div style="position:fixed;top:90px;left:0;width:200px;height:40px;background:red"></div></body>`);
  const failures = await checkAlwaysVisible(page, [
    { name: 'navbar', locator: page.getByRole('navigation', { name: 'Main' }) },
    { name: 'search', locator: page.getByRole('searchbox') },
    { name: 'below', locator: page.getByRole('button', { name: 'below' }) },
    { name: 'covered', locator: page.getByRole('link', { name: 'covered' }) },
  ]);
  const at = (name: string) => failures.filter((f) => f.element === name).map((f) => f.problem);
  expect(at('navbar')).toEqual([]);
  expect(at('search')).toEqual(['missing']);
  expect(at('below')).toContain('out of viewport');
  expect(at('covered')).toContain('covered');
});

// UXF-021: the R-02 engine ran axe in legacy mode, which checks the top document only, so the
// violations inside the visualise page's frame (UXF-007, UXF-008) went unreported.
test('L1 gate 5 reports a control nested in a control and a scroll region the keyboard cannot reach, inside a frame (UXF-021)', async ({ page }) => {
  // Everything in the frame's view: text cut off at the frame's edge makes color-contrast incomplete.
  const framed = `<!doctype html><html lang="en"><head><title>framed</title></head><body style="margin:4px"><main>
    <p style="color:#aaa;background:#fff;margin:0">faint text</p>
    <div role="button" tabindex="0">outer <button>inner</button></div>
    <div style="height:60px;overflow:auto">${'<div>a line in a scrolling region with nothing focusable</div>'.repeat(12)}</div>
    </main></body></html>`;
  await page.setContent(`<!doctype html><html lang="en"><head><title>host</title></head><body style="margin:0"><main><h1 style="margin:0">host</h1>
    <iframe title="framed" style="width:380px;height:300px" srcdoc="${framed.replace(/"/g, '&quot;')}"></iframe></main></body></html>`);
  await page.frameLocator('iframe').getByRole('button', { name: 'inner', exact: true }).waitFor();
  const full = (await runAxe(page, 'full')).map((v) => v.id);
  expect(full).toEqual(expect.arrayContaining(['color-contrast', 'nested-interactive', 'scrollable-region-focusable']));
  expect((await runAxe(page, 'colour')).map((v) => v.id)).toContain('color-contrast');
});

test('L1 reduced motion finds a running animation', async ({ page }) => {
  await page.setContent(`<style>@keyframes spin{to{transform:rotate(1turn)}} .s{animation:spin 2s infinite}</style><div class="s">x</div>`);
  expect((await page.evaluate(probeMotion)).running.length).toBe(1);
});

const PAGE_A = `<body style="margin:0"><main><div style="height:1500px"></div><a href="/b">Next</a>
  <a href="/b" style="display:none">Hidden</a></main></body>`;
const PAGE_B = `<body style="margin:0"><main><div style="height:2000px"></div><h2>Target</h2><div style="height:1000px"></div></main></body>`;
const probeUser = (page: import('@playwright/test').Page) =>
  new ScreenOnlyUser(page, {
    regions: [{ name: 'main', selector: 'main' }],
    primary: ['main'],
    revealers: [],
    mode: 'pointer',
    maxSteers: 10,
    maxTabs: 10,
  });

test('L1 the screen-only user scrolls to a control below the fold and counts the steers', async ({ page }) => {
  await page.route('http://ux.test/**', (route) =>
    route.fulfill({ contentType: 'text/html', body: route.request().url().endsWith('/b') ? PAGE_B : PAGE_A }),
  );
  const run = await probeUser(page).run(
    {
      name: 'probe',
      start: '/a',
      steps: [{ click: { role: 'link', name: '^Next$' } }, { reveal: { role: 'heading', name: '^Target$' } }],
      truth: { path: '/b', target: { role: 'heading', name: '^Target$' } },
    },
    'http://ux.test',
  );
  expect(run.reason).toBe('');
  expect(run.states[0].glance).toBe(3);
  expect(run.states[0].steers).toBeGreaterThan(0);
  expect(run.metrics['screen_only.actions']).toBe(run.states.reduce((n, s) => n + s.actions, 0));
});

test('L1 the screen-only user fails on a control nothing on screen reveals', async ({ page }) => {
  await page.route('http://ux.test/**', (route) => route.fulfill({ contentType: 'text/html', body: PAGE_A }));
  const run = await probeUser(page).run(
    { name: 'probe', start: '/a', steps: [{ click: { role: 'link', name: '^Hidden$' } }], truth: { path: '/b' } },
    'http://ux.test',
  );
  expect(run.completed).toBe(false);
  expect(run.reason).toContain('nothing on screen reveals it');
});

test('L1 metric functions', () => {
  expect(median([3, 1, 2])).toBe(2);
  expect(median([1, 2, 3, 4])).toBe(2.5);
  expect(percentile([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3], 95)).toBe(1);
  expect(percentile([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 3], 95)).toBe(3);
  const m = taskMetrics([
    { step: 'a', glance: 1, actions: 1, steers: 0, words: 100 },
    { step: 'b', glance: 3, actions: 3, steers: 2, words: 300 },
  ]);
  expect(m['screen_only.actions']).toBe(4);
  expect(m['screen_only.looks']).toBe(1);
  expect(m['screen_only.steers']).toBe(2);
  expect(m['screen_only.time_to_act_steps.max']).toBe(3);
  expect(m['glance_cost.p95']).toBe(3);
  expect(m['reading_load.median']).toBe(200);
});
