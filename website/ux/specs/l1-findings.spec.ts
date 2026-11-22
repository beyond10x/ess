// L1: which tests the open findings name as expected failures (T20 of the R-02 report review), on
// canned records; no browser, no server. The gate's exit codes on them are held by the nested runs
// of specs/nested-findings.spec.ts, in the slow lane (UXF-025).
import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { expect, test } from '@playwright/test';
import { knownFailures } from '../core/findings';
import { findingRecord as record } from './fixtures';

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

