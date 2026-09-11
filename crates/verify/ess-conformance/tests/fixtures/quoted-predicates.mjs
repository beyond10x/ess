import assert from 'node:assert/strict'
import test from 'node:test'
import {admitSuite} from '../../assets/coverage-admission.js'

const scenario = 'quoted.core/authored/operand'
const document = predicate => JSON.stringify({
  provenance: {suite_version: 'ess-conformance/5', system: 'quoted', specification_version: 'v1', spec_digest: 'a'.repeat(64), contract_digest: 'b'.repeat(64)},
  scenarios: {[scenario]: {purpose: 'Check a quoted operand', steps: [{step: 'expect_view', view: 'quoted.core.Rows', expectation: {expect: 'satisfies', predicate}}], source: []}},
  coverage: {
    selection: {scope: {kind: 'system'}, origins: 'authored', filter: {kind: 'all'}},
    knowledge: 'complete_inventory', generated: [], authored: [scenario], outside: [], refused: [],
    authored_sources: {'operand.yaml': {digest: `sha256:${'c'.repeat(64)}`, scenario, disposition: 'accepted'}},
    counts: {generated: 0, authored: 1, outside: 0, refused: 0},
  },
})
const meaning = admitted => admitted.meaning[scenario].steps[0].expectation.predicate

for (const expression of [
  'to == "" or text == ""', "to == '' and text == ''", 'to == "done" trailing',
  'to == "done""next"', 'not to == "" or text == ""', 'to == "unterminated',
  "to == 'unterminated", String.raw`to == "escaped\"`, String.raw`to == "slash\\" or text == ""`,
]) {
  test(`refuse ${expression}`, async () => {
    await assert.rejects(admitSuite(document(expression)), /structured any\/all\/not/)
  })
}

for (const [expression, literal] of [
  ['to == ""', ''], ["to == 'true'", 'true'], ['to == "or and not"', 'or and not'],
  [" \tto == '  spaced  ' \n", '  spaced  '],
  [String.raw`to == "say \"or\""`, String.raw`say \"or\"`],
  [String.raw`to == 'it\'s valid'`, String.raw`it\'s valid`],
  [String.raw`to == "path\\"`, String.raw`path\\`],
  ['to == "雪 and ☃"', '雪 and ☃'], ['to == "a == b or c != d"', 'a == b or c != d'],
]) {
  test(`preserve ${expression}`, async () => {
    assert.deepEqual(meaning(await admitSuite(document(expression))), ['compare', 'to', '==', ['literal', literal]])
  })
}

test('structured composition and prefix not remain admitted', async () => {
  await admitSuite(document({any: ['to == ""', {all: ["text == ''", {not: 'disabled'}]}]}))
  assert.deepEqual(meaning(await admitSuite(document('not to == ""'))), ['not', ['compare', 'to', '==', ['literal', '']]])
})

for (const literal of ['a == b', 'a != b', 'a >= b']) {
  test(`operator in literal ${literal}`, async () => {
    assert.deepEqual(meaning(await admitSuite(document(`to < "${literal}"`))), ['compare', 'to', '<', ['literal', literal]])
  })
}
