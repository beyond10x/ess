import { defineConfig } from '@playwright/test';
import { ORIGIN, PORT, ROUND } from './adapter';

// UX_STAGE=report runs only the round report (it reads the measurement's .out/raw.json).
// Otherwise: l1 (probe break-tests), l2 (layout contract), l3 (screen-only user), and with
// UX_ROUND set also the measured-only viewports.
const report = process.env.UX_STAGE === 'report';
const measure = [
  { name: 'l1', testMatch: /l1-.*\.spec\.ts/ },
  { name: 'l2', testMatch: /l2-.*\.spec\.ts/ },
  { name: 'l3', testMatch: /l3-.*\.spec\.ts/ },
  ...(ROUND ? [{ name: 'measured', testMatch: /measured\.spec\.ts/ }] : []),
];

export default defineConfig({
  testDir: './specs',
  outputDir: './.out/test-results',
  fullyParallel: true,
  forbidOnly: true,
  retries: 0,
  timeout: 60_000,
  // The step budget: 120 s for the gate. UX_GLOBAL_TIMEOUT lifts it only to measure how far over it is.
  globalTimeout: report ? 0 : Number(process.env.UX_GLOBAL_TIMEOUT ?? (ROUND ? 240_000 : 120_000)),
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
  projects: report ? [{ name: 'round', testMatch: /round\.spec\.ts/ }] : measure,
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
