// The four nested runs of the known-failure rule (T20 of the R-02 report review, T28 of the R-03
// one): the gate exits 0 when only the assets of open findings fail with their stated message, 1 on
// any other failure, 1 when an asset fails with another message, and 1 when an asset passes. Each
// starts a Playwright run of its own on canned findings and a canned spec (12 to 14 s each under
// load), so they run in the slow lane, not in the 120 s gate (UXF-025).
import { spawnSync } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { expect, test } from '@playwright/test';
import { findingRecord as record } from './fixtures';

// A fixture project: one open finding whose asset is the test `known`, failing with the message
// "known broken", and a test `other` no finding names. CASE says which of them fail, and how.
function nestedRun(dir: string, testCase: 'known-only' | 'other-too' | 'known-passes' | 'known-other-cause'): number {
  const findings = resolve(__dirname, '../core/findings');
  mkdirSync(join(dir, 'findings'), { recursive: true });
  mkdirSync(join(dir, 'specs'), { recursive: true });
  writeFileSync(join(dir, 'findings', 'UXF-001.md'), record('UXF-001', 'open', 'specs/case.spec.ts :: known', 'known broken'));
  writeFileSync(
    join(dir, 'playwright.config.ts'),
    `export default { testDir: './specs', outputDir: ${JSON.stringify(join(dir, 'out'))}, reporter: 'line', workers: 1, retries: 0 };\n`,
  );
  writeFileSync(
    join(dir, 'specs', 'case.spec.ts'),
    [
      `import { expect, test } from '@playwright/test';`,
      `import { holdKnownFailures } from ${JSON.stringify(findings)};`,
      `holdKnownFailures(test, ${JSON.stringify(join(dir, 'findings'))}, ${JSON.stringify(dir)});`,
      `const testCase = process.env.CASE;`,
      `test('known', () => { expect(testCase === 'known-passes' ? 'fixed' : testCase === 'known-other-cause' ? 'another cause' : 'known broken').toBe('fixed'); });`,
      `test('other', () => { expect(testCase === 'other-too' ? 'broken' : 'fine').toBe('fine'); });`,
      '',
    ].join('\n'),
  );
  const cli = require.resolve('@playwright/test/cli');
  const child = spawnSync(process.execPath, [cli, 'test', '-c', 'playwright.config.ts'], {
    cwd: dir,
    env: { PATH: process.env.PATH ?? '', HOME: process.env.HOME ?? '', CASE: testCase },
    encoding: 'utf8',
    timeout: 50_000,
  });
  return child.status ?? -1;
}

test('L1 known failures: the gate exits 0 when only the assets of open findings fail', ({}, testInfo) => {
  expect(nestedRun(testInfo.outputPath('fixture'), 'known-only'), 'known failures only').toBe(0);
});

test('L1 known failures: the gate exits 1 on a failure no open finding names', ({}, testInfo) => {
  expect(nestedRun(testInfo.outputPath('fixture'), 'other-too'), 'a new failure beside a known one').toBe(1);
});

test('L1 known failures: the gate exits 1 when the asset of an open finding fails with a message its pattern does not match (T28)', ({}, testInfo) => {
  expect(nestedRun(testInfo.outputPath('fixture'), 'known-other-cause'), 'a known test failing for another cause').toBe(1);
});

test('L1 known failures: the gate exits 1 when the asset of an open finding passes', ({}, testInfo) => {
  expect(nestedRun(testInfo.outputPath('fixture'), 'known-passes'), 'an open finding whose asset passes').toBe(1);
});
