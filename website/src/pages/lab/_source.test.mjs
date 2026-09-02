// The copy in `_source.ts`, held to the file it is a copy of.
//
// The left panel of the lab shows `examples/billing/domains/invoice.yaml` and claims that a line
// number shown there is the line number in the repository. `_spans.ts` computes every highlight
// from that copy, so the claim holds exactly as long as the copy does — and a copy is a thing that
// rots silently. It rotted once: the release that gave `billing.invoice.Account` its `owns`
// relation changed the specification and not the copy, and the panel spent that release pointing
// at lines that had moved.
//
// This check is the reason it cannot rot again. It is separate from `_run.test.mjs` because it
// needs no module: it fails on a stale copy whether or not anything is built, and it says which
// line first differs rather than that some highlight is out of range.
//
// Usage, from the repository root (`task site-lab` runs it):
//
//   node --disable-warning=MODULE_TYPELESS_PACKAGE_JSON website/src/pages/lab/_source.test.mjs
//
// Reading across the tree is this file's whole job, and it is the one place in `website/` that
// does. Nothing the Docusaurus build imports reads outside `website/` — that is what the committed
// copy is for.

import {readFile} from 'node:fs/promises';
import {fileURLToPath} from 'node:url';

import {INVOICE_YAML, INVOICE_YAML_LINES} from './_source.ts';

const SPECIFICATION = fileURLToPath(
  new URL('../../../../examples/billing/domains/invoice.yaml', import.meta.url),
);

const file = await readFile(SPECIFICATION, 'utf8');
if (INVOICE_YAML === file) {
  console.log(
    `the lab's copy of invoice.yaml: verbatim, ${INVOICE_YAML_LINES.length} lines`,
  );
  process.exit(0);
}

const copied = INVOICE_YAML.split('\n');
const actual = file.split('\n');
const at = copied.findIndex((line, index) => line !== actual[index]);
console.error(
  `website/src/pages/lab/_source.ts is not ${SPECIFICATION} any more.\n` +
    (at === -1
      ? `the copy holds ${copied.length} lines and the file ${actual.length}.`
      : `line ${at + 1} first differs:\n  file: ${actual[at] ?? '(the file ends)'}\n  copy: ${copied[at] ?? '(the copy ends)'}`) +
    '\nrefresh it: copy the file into the template literal, escaping ` as \\`.',
);
process.exit(1);
