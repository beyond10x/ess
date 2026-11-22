import { defineConfig } from '@playwright/test';
import { LANE, ORIGIN, PORT, SLOW_L2_SLICES, WORKERS } from './adapter';

// Two lanes (UX_LANE). `gate`, every merge, within 120 s: the L1 break-tests, the template pages
// at both gating viewports and both themes, and the two anchor cards. `slow`: the whole sitemap in
// SLOW_L2_SLICES projects `l2-<k>`, every card (`l3`), the measured-only viewports (`measured`)
// and the tests that start a Playwright run of their own (`nested`), each project one pool job
// under 30 minutes (`--project=<name>`).
// UX_STAGE=round merges the runs named in UX_RUNS into the round and takes its shots; UX_STAGE=report
// then writes the report from the round and the baseline the ratchet took it into (specs/round.spec.ts).
const stage = process.env.UX_STAGE === 'round' || process.env.UX_STAGE === 'report' ? process.env.UX_STAGE : '';
const report = stage !== '';
// A project's grep is matched against the test's title path joined by spaces (project name, spec
// file, title), never the title alone, so a stage's tests are those whose title starts with
// `<stage>: ` after a space.
export const stageGrep = (name: string): RegExp => new RegExp(`(^|\\s)${name}: `);
// The gate's projects, the slowest tests first (the screen-only user's runs, then the pages, then
// the L1 tests of a second or less), so the workers end the step together (R-04, UXF-025).
export const gateProjects = [
  { name: 'l3', testMatch: /l3-.*\.spec\.ts/ },
  { name: 'l2', testMatch: /l2-.*\.spec\.ts/ },
  { name: 'l1', testMatch: /l1-.*\.spec\.ts/ },
];
export const slowProjects = [
  ...Array.from({ length: SLOW_L2_SLICES }, (_, i) => ({ name: `l2-${i + 1}`, testMatch: /l2-layout\.spec\.ts/, grep: new RegExp(`@slice-${i + 1}(?!\\d)`) })),
  { name: 'l3', testMatch: /l3-.*\.spec\.ts/ },
  { name: 'measured', testMatch: /measured\.spec\.ts/ },
  { name: 'nested', testMatch: /nested-.*\.spec\.ts/ },
];

export default defineConfig({
  testDir: './specs',
  outputDir: './.out/test-results',
  fullyParallel: true,
  // The commands pass --workers too (package.json, Taskfile.yml); an L1 test holds all three equal.
  workers: WORKERS,
  forbidOnly: true,
  retries: 0,
  // No test runs over 60 s; the step ends at its budget: 120 s for the gate, 28 minutes for a job
  // of the slow lane (the pool ends a job at 30). UX_GLOBAL_TIMEOUT lifts it only to measure how
  // far over it is.
  timeout: 60_000,
  globalTimeout: report ? 0 : Number(process.env.UX_GLOBAL_TIMEOUT ?? (LANE === 'gate' ? 120_000 : 1_680_000)),
  reporter: report ? [['list']] : [['dot'], ['./core/reporter.ts']],
  use: {
    baseURL: ORIGIN,
    browserName: 'chromium',
    headless: true,
    deviceScaleFactor: 1,
    reducedMotion: 'reduce',
    serviceWorkers: 'block',
    launchOptions: process.env.UX_CHROMIUM ? { executablePath: process.env.UX_CHROMIUM } : {},
  },
  projects: report ? [{ name: stage, testMatch: /round\.spec\.ts/, grep: stageGrep(stage) }] : LANE === 'gate' ? gateProjects : slowProjects,
  webServer: {
    command: `npx docusaurus serve --dir build --host 127.0.0.1 --port ${PORT} --no-open`,
    cwd: '..',
    url: `${ORIGIN}/ess/`,
    reuseExistingServer: true,
    timeout: 30_000,
    stdout: 'ignore',
    stderr: 'pipe',
  },
});
