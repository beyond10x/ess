// L1 checks of the built site's own pages that need no browser: what a page says, read from the
// built HTML, the text a reader gets.
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { expect, test } from '@playwright/test';
import { BUILD, builtPage } from '../adapter';

// A highlighted code block splits a token into spans, so code is read with its tags dropped, not
// replaced by a space.
const text = (path: string, tag = ' ') => builtPage(path).replace(/<[^>]*>/g, tag).replace(/\s+/g, ' ');

// UXF-016: the navbar's "Status" named a release older than the one the install page installs
// (0.51.0 against 0.97.1 in R-01, 0.97.3 against 0.97.4 in R-05).
test('L1 site: the status page names as the latest release the version the install page installs (UXF-016)', () => {
  const latest = /latest published release observed on [^.]*? is (\d+\.\d+\.\d+)/.exec(text('/ess/docs/status/where-this-stands'))?.[1];
  const installs = [...new Set([...text('/ess/docs/start/install', '').matchAll(/version=(\d+\.\d+\.\d+)/g)].map((m) => m[1]))];
  expect(installs, 'the install page installs one version').toHaveLength(1);
  expect(latest, 'the status page names the latest release').toBe(installs[0]);
});

// UXF-027: the index lists of the two reference pages were a paragraph of links with no name, the
// diagnostics list offered bare numbers, and a jump landed under the navbar with no mark.
test('L1 site: each index list of a reference page is a named navigation that says what it covers, and a jump lands below the navbar marked (UXF-027)', () => {
  const nav = /<nav aria-label="?([^">]+)"?>([\s\S]*?)<\/nav>/;
  const plain = (html: string) => html.replace(/<[^>]*>/g, '').replace(/\s+/g, ' ');
  const versions = nav.exec(builtPage('/ess/docs/reference/spec-versions'));
  expect(versions?.[1], 'the versions list is a named navigation').toBe('Versions of the ess/ format');
  expect(versions?.[2], 'it links ess/31').toMatch(/href="?#ess-31"?/);
  expect(plain(versions?.[2] ?? ''), 'it says which family it covers').toMatch(/specification format ess\/; the other formats/);
  const classes = nav.exec(builtPage('/ess/docs/reference/diagnostics'));
  expect(classes?.[1], 'the class list is a named navigation').toBe('Classes of the specification diagnostics');
  expect(plain(classes?.[2] ?? ''), 'each number shows its class').toContain('009 UNSUPPORTED');
  const dir = join(BUILD, 'assets', 'css');
  const css = readdirSync(dir).map((f) => readFileSync(join(dir, f), 'utf8')).join('\n');
  expect(css, 'a jump target clears the navbar').toMatch(/\.markdown a\[id\]:empty\{scroll-margin-top:calc\(var\(--ifm-navbar-height\) ?\+ ?1rem\)\}/);
  expect(css, 'the row a jump lands on is marked').toMatch(/\.markdown tr:has\(a\[id\]:target\)\{outline:/);
});
