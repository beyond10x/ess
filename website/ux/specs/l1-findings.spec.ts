// L1: which tests the open findings name as expected failures (T20 of the R-02 report review), on
// canned records; no browser, no server. The gate's exit codes on them are held by the nested runs
// of specs/nested-findings.spec.ts, in the slow lane (UXF-025).
import { mkdirSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { expect, test } from '@playwright/test';
import { assetTests, knownFailures, unmatchedErrors } from '../core/findings';
import { findingRecord as record } from './fixtures';

test('L1 known failures: an open finding asset is an expected failure; an accepted, closed, baseline-held or unnamed one is not', ({}, testInfo) => {
  const dir = testInfo.outputPath('findings');
  mkdirSync(dir, { recursive: true });
  writeFileSync(join(dir, 'UXF-001.md'), record('UXF-001', 'open', 'website/ux/specs/l2-layout.spec.ts :: L2 /ess/ desktop-1280', 'axe color-contrast'));
  writeFileSync(join(dir, 'UXF-002.md'), record('UXF-002', 'accepted', 'website/ux/specs/l2-layout.spec.ts :: L2 /ess/docs desktop-1280'));
  writeFileSync(join(dir, 'UXF-003.md'), record('UXF-003', 'closed', 'website/ux/specs/l1-probes.spec.ts :: L1 x'));
  writeFileSync(join(dir, 'UXF-004.md'), record('UXF-004', 'open', 'website/ux/specs/l3-screen-only.spec.ts :: L3 a desktop-1280 keyboard, held by website/ux/baseline.json'));
  writeFileSync(join(dir, 'UXF-005.md'), record('UXF-005', 'open', 'website/ux/specs/l3-screen-only.spec.ts :: L3 b mobile-390 pointer, held by gate 4: red at 4 of 3', "actions within the card's limit"));
  writeFileSync(join(dir, 'UXF-006.md'), record('UXF-006', 'open', 'none yet; proposed: an L2 assertion'));
  writeFileSync(join(dir, 'UXF-007.md'), record('UXF-007', 'open', 'website/ux/specs/l2-layout.spec.ts :: L2 /ess/ desktop-1280', 'axe nested-interactive'));
  // A finding at both gating viewports names both tests; a proposal after "none yet" names none.
  writeFileSync(
    join(dir, 'UXF-008.md'),
    record('UXF-008', 'open', 'website/ux/specs/l2-layout.spec.ts :: L2 /ess/c desktop-1280; website/ux/specs/l2-layout.spec.ts :: L2 /ess/c mobile-390', 'gate 2 \\S+: docs sidebar'),
  );
  writeFileSync(join(dir, 'UXF-009.md'), record('UXF-009', 'open', 'none yet; proposed: website/ux/specs/l1-probes.spec.ts :: L1 y'));
  expect(knownFailures(dir)).toEqual([
    { spec: 'website/ux/specs/l2-layout.spec.ts', title: 'L2 /ess/ desktop-1280', findings: ['UXF-001', 'UXF-007'], patterns: ['axe color-contrast', 'axe nested-interactive'] },
    { spec: 'website/ux/specs/l2-layout.spec.ts', title: 'L2 /ess/c desktop-1280', findings: ['UXF-008'], patterns: ['gate 2 \\S+: docs sidebar'] },
    { spec: 'website/ux/specs/l2-layout.spec.ts', title: 'L2 /ess/c mobile-390', findings: ['UXF-008'], patterns: ['gate 2 \\S+: docs sidebar'] },
    { spec: 'website/ux/specs/l3-screen-only.spec.ts', title: 'L3 b mobile-390 pointer', findings: ['UXF-005'], patterns: ["actions within the card's limit"] },
  ]);
});

// T28 of the R-03 report review: a failure of an open finding's asset was held whatever its cause.
// A failed asset is held only when each of its errors matches the failure pattern of a finding it
// is the asset of; any other error is a new failure. Colour codes do not count.
test('L1 known failures: a failed asset is held only when each of its errors matches a failure pattern of its findings (T28)', () => {
  const landing = { patterns: ['gate 5 \\S+/light: axe label-content-name-mismatch', 'truncation mobile-390/\\w+: div\\.b10x-terminal__bar'] };
  const gate5 = 'Error: /ess/ at mobile-390, hard gate\n\nExpected: "none"\nReceived: "gate 5 mobile-390/light: axe label-content-name-mismatch (serious) 2 node(s): g"';
  const cut = 'Error: /ess/ at mobile-390, hard gate\n\nExpected: "none"\nReceived: "truncation mobile-390/dark: div.b10x-terminal__bar > span.b10x-terminal__title \\"ess\\""';
  expect(unmatchedErrors(landing, [gate5, cut])).toEqual([]);
  expect(unmatchedErrors(landing, [`\u001b[31m${gate5}\u001b[39m`])).toEqual([]);
  const contrast = 'Error: /ess/ at mobile-390, hard gate\n\nExpected: "none"\nReceived: "gate 5 mobile-390/light: axe color-contrast (serious) 1 node(s): h1"';
  const thrown = 'page.goto: net::ERR_CONNECTION_REFUSED at http://127.0.0.1:4391/ess/';
  expect(unmatchedErrors(landing, [gate5, contrast, thrown])).toEqual([contrast, thrown]);
  expect(unmatchedErrors({ patterns: ['none'] }, ['none'])).toEqual(['none']);
});

// Every open record states its failure pattern: a regular expression where its asset is an
// expected failure, `none` where it is not (no test yet, or a test held by a file).
test('L1 known failures: every open record states a failure pattern, a regular expression for an expected failure and none otherwise', () => {
  const dir = resolve(__dirname, '../findings');
  const expected = new Set(knownFailures(dir).flatMap((k) => k.findings));
  const wrong: string[] = [];
  for (const file of readdirSync(dir).filter((f) => /^UXF-.*\.md$/.test(f)).sort()) {
    const text = readFileSync(join(dir, file), 'utf8');
    if (!/^status: open$/m.test(text)) continue;
    const pattern = /^failure_pattern: (.*)$/m.exec(text)?.[1]?.trim();
    const id = file.replace(/\.md$/, '');
    if (!pattern) wrong.push(`${id}: no failure_pattern`);
    else if (expected.has(id)) {
      if (pattern === 'none') wrong.push(`${id}: an expected failure with pattern none`);
      else
        try {
          new RegExp(pattern);
        } catch (e) {
          wrong.push(`${id}: ${String(e)}`);
        }
    } else if (pattern !== 'none') wrong.push(`${id}: no expected failure, so the pattern is none, not ${pattern}`);
  }
  expect(wrong).toEqual([]);
  expect(assetTests('none yet; proposed: x')).toEqual([]);
});

// The L2 test reports each hard-gate failure as an error of its own, so a pattern matches one
// failure and a second failure in the same test is a second error.
test('L1 known failures: the L2 test reports each hard-gate failure as an error of its own (T28)', () => {
  const spec = readFileSync(resolve(__dirname, 'l2-layout.spec.ts'), 'utf8');
  expect(spec, 'one soft assertion per failure').toMatch(/for \(const failure of failures\) expect\.soft\(failure/);
  expect(spec, 'no assertion over the list of failures').not.toMatch(/expect\(failures[,)]/);
});

