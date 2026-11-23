// L1 checks of the built site's own pages that need no browser: what a page says, read from the
// built HTML, the text a reader gets.
import { expect, test } from '@playwright/test';
import { builtPage } from '../adapter';

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
