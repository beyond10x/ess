// A selection comparing a Boolean item leaf with `true` or `false` (ess/27, downstream report,
// 2026-10-18): the observation is admitted, decides each item by the flag, and a text literal
// against the Boolean leaf is refused, as `src/go/runtime.go` and the native `selection.rs` answer.
import assert from 'node:assert/strict';
import test from 'node:test';

import { admitSelection } from './runtime.js';

/** The observation synthesis writes for `first: {in: legs, where: item.held == false}`. */
const OBSERVATION = `{"event_type":"demo.calls.CallEnded","plan":{"inputs":[{"name":"legs","source":{"kind":"field","field":{"name":"legs","type":"List<demo.calls.Leg>"}},"list_type":"List<demo.calls.Leg>","item_type":"demo.calls.Leg","optional_items":false,"conversion":null}],"selectors":[{"name":"free","input":0,"operation":{"kind":"first","excluding":[],"predicate":"item.held == false","reads":{"item.held":{"segments":["item","held"],"root":{"name":"item","type":"demo.calls.Leg"},"start":1,"nodes":[{"source":"Boolean","shape":{"kind":"primitive","name":"boolean"},"position":2,"operation":{"kind":"leaf"},"leaf":"Boolean","may_miss":false,"depth":1},{"source":"demo.calls.Leg","shape":{"kind":"object"},"position":1,"operation":{"kind":"field","field":{"name":"held","type":"Boolean"},"next":0},"leaf":"Boolean","may_miss":false,"depth":2}]}}}}]},"declarations":{"demo.calls.Leg":{"kind":"struct","fields":[{"name":"id","type":"String"},{"name":"held","type":"Boolean"}]}},"selector":0,"projection":{"projection":{"segments":["item","id"],"root":{"name":"item","type":"demo.calls.Leg"},"start":1,"nodes":[{"source":"String","shape":{"kind":"primitive","name":"string"},"position":2,"operation":{"kind":"leaf"},"leaf":"String","may_miss":false,"depth":1},{"source":"demo.calls.Leg","shape":{"kind":"object"},"position":1,"operation":{"kind":"field","field":{"name":"id","type":"String"},"next":0},"leaf":"String","may_miss":false,"depth":2}]},"target":"Optional<String>","types":{"nodes":{}}}}`;

function observation(predicate = 'item.held == false') {
  const value = JSON.parse(OBSERVATION);
  value.plan.selectors[0].operation.predicate = predicate;
  return value;
}

function leg(id: string, held: boolean) {
  return { id, held };
}

test('a Boolean item leaf selects the first item whose flag the predicate names', () => {
  const selection = admitSelection(observation());
  assert.deepEqual(selection.evaluate({ legs: [leg('a', true), leg('b', false)] }), ['b', true]);
  assert.equal(selection.evaluate({ legs: [leg('a', true)] })[1], false);
  const negated = admitSelection(observation('item.held != false'));
  assert.deepEqual(negated.evaluate({ legs: [leg('a', false), leg('b', true)] }), ['b', true]);
});

test('a text literal against a Boolean item leaf is refused', () => {
  assert.throws(
    () => admitSelection(observation('item.held == "false"')),
    /selection Boolean leaf needs a Boolean literal/,
  );
});
