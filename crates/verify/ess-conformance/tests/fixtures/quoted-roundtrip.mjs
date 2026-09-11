import assert from 'node:assert/strict'
import {readFileSync} from 'node:fs'
import {admitSuite, admitReplay} from '../../assets/coverage-admission.js'
const suite = await admitSuite(readFileSync(process.argv[2], 'utf8'))
const expected = JSON.parse(readFileSync(process.argv[3], 'utf8'))
const steps = suite.meaning['quoted.core/authored/operand'].steps
assert.equal(steps.length, expected.length)
for (const [index, step] of steps.entries()) {
  assert.deepEqual(step.expectation.predicate, ['compare', 'to', '==', ['literal', expected[index]]])
}
console.log(`${expected.length} actual Rust-emitted literals preserved by browser admission`)

for (const major of [5, 7]) {
  const original = JSON.parse(suite.original)
  original.provenance.suite_version = `ess-conformance/${major}`
  await assert.rejects(admitSuite(JSON.stringify(original)))
}
for (const badStep of [{step:'establish_entity'}, {step:'expect_response'}]) {
  const original = JSON.parse(suite.original)
  original.scenarios['quoted.core/authored/operand'].steps = [badStep]
  await assert.rejects(admitSuite(JSON.stringify(original)))
}
const original = JSON.parse(suite.original)
original.provenance.suite_version = 'ess-conformance/5'
original.scenarios['quoted.core/authored/operand'].steps = [{step:'expect_view', view:'quoted.core.Rows', expectation:{expect:'satisfies',predicate:{to:{eq:'"busy" status'}}}}]
await admitSuite(JSON.stringify(original))
console.log('Browser refuses old-version corrected operands and unsupported setup/response; compatible fallback stays suite/5')

const parentRef = {version:'ess-conformance/9', digest_profile:'sha256-json-bytes/1', digest:suite.digest}
const child = JSON.parse(suite.original)
child.coverage.selection.filter = {kind:'explicit', ids:Object.keys(child.scenarios), parent:parentRef}
const selected = await admitSuite(JSON.stringify(child))
const p = selected.document.provenance
const replay = {
  format:'ess-conformance-replay/1',
  model:{system:p.system, version:p.specification_version, spec_digest:p.spec_digest, contract_digest:p.contract_digest, entities:[], commands:[], views:[], actors:[], bindings:[]},
  suite:{...parentRef,digest:selected.digest},
  input:{format:'ess-conformance-input/1', suite_json:selected.original, parent_suites:[suite.original]}
}
await admitReplay(JSON.stringify(replay))
replay.suite.version = 'ess-conformance/5'
await assert.rejects(admitReplay(JSON.stringify(replay)))
console.log('Browser replay verifies actual suite/9 parent identity and refuses a false suite/5 reference')
