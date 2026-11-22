// L3 screen-only user (skill § 7): in the `slow` lane every task, in the `gate` lane the two anchor
// tasks, at both gating viewports with the pointer, and the anchor tasks with the keyboard alone
// (gate 5). Gate 4: every task completed to its truth within its card's `max_actions`.
// Gate 3: no completion miss and no false completion. Each run declares itself before it runs, so
// a run that never ends is counted as unmeasured.
import { expect, test } from '@playwright/test';
import { FINDINGS, GATING, LANE, ORIGIN, REPOSITORY, botConfig, scenarios } from '../adapter';
import { holdKnownFailures } from '../core/findings';
import { ScreenOnlyUser } from '../core/bot';
import { makeHermetic } from '../core/hermetic';
import { EXPECT, type Expected, type L3Record } from '../core/reporter';

test.describe.configure({ mode: 'parallel' });
holdKnownFailures(test, FINDINGS, REPOSITORY);

const runs = scenarios()
  .filter((scenario) => LANE === 'slow' || scenario.card === 'anchor')
  .flatMap((scenario) =>
    GATING.flatMap((viewport) => [
      { scenario, viewport, mode: 'pointer' as const },
      ...(scenario.card === 'anchor' ? [{ scenario, viewport, mode: 'keyboard' as const }] : []),
    ]),
  );

for (const { scenario, viewport, mode } of runs) {
  const expected: Expected = { kind: 'l3', task: scenario.name, viewport: viewport.name, mode, total: runs.length };
  test(`L3 ${scenario.name} ${viewport.name} ${mode}`, { annotation: { type: EXPECT, description: JSON.stringify(expected) } }, async ({ browser }, testInfo) => {
    const context = await browser.newContext({
      viewport: { width: viewport.width, height: viewport.height },
      colorScheme: 'light',
      reducedMotion: 'reduce',
    });
    await makeHermetic(context, ORIGIN);
    const page = await context.newPage();
    const run = await new ScreenOnlyUser(page, botConfig(mode)).run(scenario, ORIGIN);
    // The card's limit holds the pointer user; the keyboard user is held to completion (gate 5).
    const record: L3Record = { viewport: viewport.name, run, ...(mode === 'pointer' ? { limit: scenario.expect.max_actions } : {}) };
    await testInfo.attach('ux-l3', { body: JSON.stringify(record), contentType: 'application/json' });
    await context.close();
    expect(run.reason, `${scenario.name} at ${viewport.name} (${mode}) reaches its truth`).toBe('');
    expect(run.completed).toBe(true);
    expect({ completion_miss: run.completion_miss, false_complete: run.false_complete }).toEqual({ completion_miss: 0, false_complete: 0 });
    if (mode === 'pointer') {
      expect(run.metrics['screen_only.actions'], `${scenario.name} at ${viewport.name}: actions within the card's limit`).toBeLessThanOrEqual(scenario.expect.max_actions);
    }
  });
}
