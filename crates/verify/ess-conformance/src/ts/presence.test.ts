import assert from 'node:assert/strict';
import test from 'node:test';
import { admitSuite, holds } from './runtime.js';

// A field's presence policy (beyond10x/ess#139) travels in suite/24 and /25 as a `presence` key on
// a payload leaf. This runtime reads it (beyond10x/ess#188): an absent value is held to the one
// spelling the field declares. A runtime that ignored it would pass an implementation that sends
// `null` where the specification says the key is omitted.
function suiteText(version: string, presence: string): string {
  return JSON.stringify({
    provenance: {
      suite_version: version,
      system: 'example',
      specification_version: 'v1',
      spec_digest: 'a'.repeat(64),
      contract_digest: 'b'.repeat(64),
    },
    scenarios: {
      'example.Place/outcome/placed': {
        purpose: 'A placed order publishes its partner reference',
        source: [],
        steps: [
          { step: 'execute_command', command: 'example.Place' },
          {
            step: 'expect_event',
            event: 'example.Placed',
            shape: {
              partner_ref: { holds: 'primitive', kind: 'string', optional: true, presence },
            },
          },
        ],
      },
    },
  });
}

test('suite/24 admits a presence policy, and an older suite refuses it by version', () => {
  for (const presence of ['null_when_absent', 'omitted_when_absent']) {
    assert.equal(
      admitSuite(suiteText('ess-conformance/24', presence)).provenance.suite_version,
      'ess-conformance/24',
    );
    assert.throws(
      () => admitSuite(suiteText('ess-conformance/22', presence)),
      /field presence policies require suite\/24 or \/25/,
    );
  }
  assert.throws(
    () => admitSuite(suiteText('ess-conformance/24', 'sometimes')),
    /presence must be null_when_absent or omitted_when_absent/,
  );
});

test('an absent value is held to the spelling its field declares', () => {
  const leaf = (presence?: string) => ({
    partner_ref: {
      holds: 'primitive',
      kind: 'string',
      optional: true,
      ...(presence === undefined ? {} : { presence }),
    },
  });
  // No policy: both spellings of absence conform.
  assert.equal(holds({}, leaf()), '');
  assert.equal(holds({ partner_ref: null }, leaf()), '');
  // `null_when_absent`: the key is always sent.
  assert.equal(holds({ partner_ref: null }, leaf('null_when_absent')), '');
  assert.match(holds({}, leaf('null_when_absent')), /left out `partner_ref`/);
  // `omitted_when_absent`: the key is never `null`.
  assert.equal(holds({}, leaf('omitted_when_absent')), '');
  assert.match(holds({ partner_ref: null }, leaf('omitted_when_absent')), /as null/);
  // A present value is checked as it always was, whatever the policy.
  assert.equal(holds({ partner_ref: 'P-1' }, leaf('null_when_absent')), '');
  assert.match(holds({ partner_ref: 7 }, leaf('omitted_when_absent')), /partner_ref/);
});
