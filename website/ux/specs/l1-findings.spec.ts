// L1: an open finding's regression asset is an expected failure (T20 of the R-02 report review).
// The gate exits 0 when only the assets of open findings fail, 1 on any other failure, and 1 when
// the asset of an open finding passes: that finding is closed in the same change. Canned findings
// and a canned spec, run by a nested Playwright; no browser, no server.
import { spawnSync } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { expect, test } from '@playwright/test';
import { knownFailures } from '../core/findings';

const record = (id: string, status: string, asset: string) =>
  `---\nid: ${id}\ntitle: t\nclass: layout\nseverity: S2\nmetric: m\nstatus: ${status}\nfound_by: L2\nevidence_recording: none\nevidence_screenshot: none\nevidence_lines: l\nregression_asset: ${asset}\n---\n\n# ${id}\n`;

test('L1 known failures: an open finding asset is an expected failure; an accepted, closed, baseline-held or unnamed one is not', ({}, testInfo) => {
  const dir = testInfo.outputPath('findings');
  mkdirSync(dir, { recursive: true });
  writeFileSync(join(dir, 'UXF-001.md'), record('UXF-001', 'open', 'website/ux/specs/l2-layout.spec.ts :: L2 /ess/ desktop-1280'));
  writeFileSync(join(dir, 'UXF-002.md'), record('UXF-002', 'accepted', 'website/ux/specs/l2-layout.spec.ts :: L2 /ess/docs desktop-1280'));
  writeFileSync(join(dir, 'UXF-003.md'), record('UXF-003', 'closed', 'website/ux/specs/l1-probes.spec.ts :: L1 x'));
  writeFileSync(join(dir, 'UXF-004.md'), record('UXF-004', 'open', 'website/ux/specs/l3-screen-only.spec.ts :: L3 a desktop-1280 keyboard, held by website/ux/baseline.json'));
  writeFileSync(join(dir, 'UXF-005.md'), record('UXF-005', 'open', 'website/ux/specs/l3-screen-only.spec.ts :: L3 b mobile-390 pointer, held by gate 4: red at 4 of 3'));
  writeFileSync(join(dir, 'UXF-006.md'), record('UXF-006', 'open', 'none yet; proposed: an L2 assertion'));
  writeFileSync(join(dir, 'UXF-007.md'), record('UXF-007', 'open', 'website/ux/specs/l2-layout.spec.ts :: L2 /ess/ desktop-1280'));
  // A finding at both gating viewports names both tests; a proposal after "none yet" names none.
  writeFileSync(
    join(dir, 'UXF-008.md'),
    record('UXF-008', 'open', 'website/ux/specs/l2-layout.spec.ts :: L2 /ess/c desktop-1280; website/ux/specs/l2-layout.spec.ts :: L2 /ess/c mobile-390'),
  );
  writeFileSync(join(dir, 'UXF-009.md'), record('UXF-009', 'open', 'none yet; proposed: website/ux/specs/l1-probes.spec.ts :: L1 y'));
  expect(knownFailures(dir)).toEqual([
    { spec: 'website/ux/specs/l2-layout.spec.ts', title: 'L2 /ess/ desktop-1280', findings: ['UXF-001', 'UXF-007'] },
    { spec: 'website/ux/specs/l2-layout.spec.ts', title: 'L2 /ess/c desktop-1280', findings: ['UXF-008'] },
    { spec: 'website/ux/specs/l2-layout.spec.ts', title: 'L2 /ess/c mobile-390', findings: ['UXF-008'] },
    { spec: 'website/ux/specs/l3-screen-only.spec.ts', title: 'L3 b mobile-390 pointer', findings: ['UXF-005'] },
  ]);
});

// A fixture project: one open finding whose asset is the test `known`, and a test `other` no
// finding names. CASE says which of them fail.
function nestedRun(dir: string, testCase: 'known-only' | 'other-too' | 'known-passes'): number {
  const findings = resolve(__dirname, '../core/findings');
  mkdirSync(join(dir, 'findings'), { recursive: true });
  mkdirSync(join(dir, 'specs'), { recursive: true });
  writeFileSync(join(dir, 'findings', 'UXF-001.md'), record('UXF-001', 'open', 'specs/case.spec.ts :: known'));
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
      `test('known', () => { expect(testCase === 'known-passes' ? 'fixed' : 'broken').toBe('fixed'); });`,
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

test('L1 known failures: the gate exits 1 when the asset of an open finding passes', ({}, testInfo) => {
  expect(nestedRun(testInfo.outputPath('fixture'), 'known-passes'), 'an open finding whose asset passes').toBe(1);
});
