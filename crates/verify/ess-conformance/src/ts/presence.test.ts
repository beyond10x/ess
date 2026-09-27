import assert from 'node:assert/strict';
import test from 'node:test';
import { admitSuite } from './runtime.js';

// A field's presence policy (beyond10x/ess#139) travels in suite/24 and /25 as a `presence` key on
// a payload leaf. This runtime does not read it, and a runtime that ignored it would pass an
// implementation that sends `null` where the specification says the key is omitted, so both
// envelopes are refused by version before anything in them is read.
function suiteText(version: string): string {
  return JSON.stringify({
    provenance: {
      suite_version: version,
      system: 'example',
      specification_version: 'v1',
      spec_digest: 'a'.repeat(64),
      contract_digest: 'b'.repeat(64),
    },
    scenarios: {},
  });
}

test('suites carrying presence policies are refused by version', () => {
  for (const version of ['ess-conformance/24', 'ess-conformance/25']) {
    assert.throws(
      () => admitSuite(suiteText(version)),
      new RegExp(`unsupported suite version "${version.replace('/', '\\/')}"`),
    );
  }
});
