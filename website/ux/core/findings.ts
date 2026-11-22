// Known failures: the regression asset of an open finding is a test that is red until the finding
// is fixed. Such a test is an expected failure, so the gate's exit code tells a new failure from a
// known one (T20 of the R-02 report review):
//   - only assets of open findings fail: exit 0;
//   - any other test fails: exit 1;
//   - the asset of an open finding passes: exit 1, and the finding is closed in the same change.
// Playwright's own expected-failure mark (`testInfo.fail`) gives exactly these exit codes.
//
// An asset is `<spec path> :: <test title>`, the title ending at `, held by` or the end (a finding
// may name several, `assetTests`). An asset
// "held by" a file (the ratchet's baseline) is held by that file's values, not by the test's result,
// so it is not an expected failure. Generic: the findings directory and the root that asset paths
// are relative to are the caller's.
import { readdirSync, readFileSync } from 'node:fs';
import { join, relative } from 'node:path';
import type { TestInfo, TestType } from '@playwright/test';

export interface KnownFailure {
  spec: string;
  title: string;
  findings: string[];
}

function frontMatter(text: string): Record<string, string> {
  const lines = text.split('\n');
  if (lines[0]?.trim() !== '---') return {};
  const fields: Record<string, string> = {};
  for (const line of lines.slice(1)) {
    if (line.trim() === '---') return fields;
    const at = line.indexOf(':');
    if (at > 0) fields[line.slice(0, at).trim()] = line.slice(at + 1).trim().replace(/^"|"$/g, '');
  }
  return {};
}

// The spec and title an asset names, or null when it names no test or is held by a file.
export function assetTest(asset: string): { spec: string; title: string } | null {
  const split = asset.indexOf('::');
  if (split < 0) return null;
  const spec = asset.slice(0, split).trim();
  if (!spec.includes('/') || /\s/.test(spec)) return null;
  const rest = asset.slice(split + 2);
  const held = rest.indexOf(', held by');
  const title = (held < 0 ? rest : rest.slice(0, held)).trim();
  if (!title) return null;
  if (held >= 0 && /^, held by [^\s,;:]*\/[^\s,;:]*/.test(rest.slice(held))) return null;
  return { spec, title };
}

// The tests a regression_asset field names: one asset, or several when a finding one test cannot
// cover (both gating viewports) separates them by `; ` before the next `<spec path> ::`. A field
// whose first part names no test ("none yet; proposed: …") names none.
export function assetTests(field: string): Array<{ spec: string; title: string }> {
  const parts = field.split(/; (?=[^\s;]+\.spec\.ts :: )/);
  if (!assetTest(parts[0])) return [];
  return parts.map(assetTest).filter((t): t is { spec: string; title: string } => t !== null);
}

// Every open finding's asset that is an expected failure, one entry per test, sorted.
export function knownFailures(findingsDir: string): KnownFailure[] {
  const byTest = new Map<string, KnownFailure>();
  for (const file of readdirSync(findingsDir).filter((f) => /^UXF-.*\.md$/.test(f)).sort()) {
    const fields = frontMatter(readFileSync(join(findingsDir, file), 'utf8'));
    if (fields.status !== 'open') continue;
    for (const named of assetTests(fields.regression_asset ?? '')) {
      const key = `${named.spec} :: ${named.title}`;
      const entry = byTest.get(key) ?? { ...named, findings: [] };
      entry.findings.push(fields.id ?? file.replace(/\.md$/, ''));
      byTest.set(key, entry);
    }
  }
  return [...byTest.entries()].sort(([a], [b]) => a.localeCompare(b)).map(([, v]) => v);
}

// The findings whose asset is this test: `file` absolute, asset paths relative to `root`.
export function findingsOf(known: KnownFailure[], root: string, file: string, title: string): string[] {
  const spec = relative(root, file).split('\\').join('/');
  return known.find((k) => k.spec === spec && k.title === title)?.findings ?? [];
}

// Marks every test of the calling spec file that is an open finding's asset as expected to fail.
// `TestType<any, any>`: any spec's `test`, whatever its fixtures.
export function holdKnownFailures(test: TestType<any, any>, findingsDir: string, root: string): void {
  const known = knownFailures(findingsDir);
  test.beforeEach(({}, testInfo: TestInfo) => {
    const ids = findingsOf(known, root, testInfo.file, testInfo.title);
    if (ids.length > 0) testInfo.fail(true, `known failure, the regression asset of open ${ids.join(', ')}`);
  });
}
