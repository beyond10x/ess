import assert from 'node:assert/strict';
import test from 'node:test';
import { admitFixtureSteps, admitFixtures, fixtureValues } from './fixtures.js';
import { admitSuite, equal, JsonNumber, newHarness, ScenarioRun } from './runtime.js';
import type { Node, Target, TestScope } from './runtime.js';

const contract = {
  fields: [{ name: 'current-principal', type: 'example.Principal' }],
  declarations: {
    'example.Principal': { kind: 'struct', fields: [{ name: 'email', type: 'String' }] },
  },
};
const fixture = { kind: 'fixture', fixture: 'current-principal' };
const steps = [
  { step: 'resolve_fixtures', fixtures: contract },
  { step: 'execute_command', command: 'example.Open', input: { principal: fixture } },
  { step: 'expect_event_values', event: 'example.Opened', payload: { principal: fixture } },
];
function suiteText(selected: Node[] = steps): string {
  return JSON.stringify({
    provenance: {
      suite_version: 'ess-conformance/18',
      system: 'example',
      specification_version: 'v1',
      spec_digest: 'a'.repeat(64),
      contract_digest: 'b'.repeat(64),
    },
    scenarios: {
      'example.Open/outcome/opened': {
        purpose: 'A pre-execution principal',
        source: [{ kind: 'command', name: 'example.Open' }],
        steps: selected,
      },
    },
  });
}
const scope: TestScope = { async test() {}, diagnostic() {}, skip() {} };

test('fixture values reject incomplete data, invalid nested types and unrelated authority', () => {
  const admitted = admitFixtures(contract);
  for (const values of [
    {},
    { 'current-principal': null },
    { 'current-principal': { email: false } },
    { 'current-principal': { email: 'a' }, extra: 'b' },
    { 'current-principal': { email: undefined } },
  ]) {
    assert.throws(() => fixtureValues(admitted, values));
  }
  assert.throws(() =>
    admitFixtures({
      ...contract,
      declarations: {
        ...contract.declarations,
        'example.Unused': { kind: 'newtype', of: 'String' },
      },
    }),
  );
  assert.throws(() =>
    admitFixtures({ ...contract, fields: [{ name: 'current-principal', type: 'Binary64' }] }),
  );
});

test('suite admission refuses old vocabulary, misplaced preludes and undeclared fixture references', () => {
  admitSuite(suiteText());
  for (let major = 1; major < 18; major++)
    assert.throws(() =>
      admitSuite(suiteText().replace('ess-conformance/18', `ess-conformance/${major}`)),
    );
  for (const selected of [steps.slice(1), [...steps, steps[0]], [steps[1], steps[0], steps[2]]]) {
    assert.throws(() => admitSuite(suiteText(selected)));
  }
  // A literal's contents are data, even when they resemble a reference.
  admitFixtureSteps([
    { step: 'execute_command', input: { data: { kind: 'literal', value: fixture } } },
  ]);
});

// Suite/18 implies every major below it, and suites 12–17 carry retained results, string operators
// and aggregate views, which this runtime runs (beyond10x/ess#188). A fixture suite admits that
// vocabulary as the Rust reader does, and still refuses a malformed step of it.
test('fixture suites admit the retained-result, string-operator and aggregate vocabulary', () => {
  for (const step of ['capture_command_result', 'expect_replay_result']) {
    assert.throws(() => admitSuite(suiteText([...steps, { step }])), /missing field capture/);
  }
  admitSuite(suiteText([...steps, { step: 'expect_no_events' }]));
  const satisfies = {
    step: 'expect_view',
    view: 'example.Opened',
    expectation: { expect: 'satisfies', predicate: { principal: { starts_with: 'x' } } },
  };
  admitSuite(suiteText([...steps, satisfies]));
  const aggregate = JSON.parse(suiteText());
  aggregate.scenarios['example.Opened/aggregate'] =
    aggregate.scenarios['example.Open/outcome/opened'];
  admitSuite(JSON.stringify(aggregate));
});

test('fixture snapshots survive provider and request mutation, and wrong observed values fail', async () => {
  for (const mode of ['valid', 'wrong-output', 'mutate-input', 'wrong-then-right']) {
    const suite = admitSuite(suiteText());
    const provided = { 'current-principal': { email: 'original@example.com' } };
    let resolved = 0,
      begun = 0,
      ended = 0;
    const target: Target = {
      identity: () => ({ name: 'fixture-test', version: '1' }),
      fixtureValues: (_context, given) => {
        resolved++;
        given.fields[0]!.type = 'Boolean';
        return provided;
      },
      beginScenario: () => {
        assert.equal(resolved, 1);
        begun++;
        provided['current-principal'].email = 'changed@example.com';
      },
      endScenario: () => {
        ended++;
      },
      executeCommand: (request) => {
        assert.equal(request.input.principal.email, 'original@example.com');
        if (mode === 'mutate-input') request.input.principal.email = 'mutated@example.com';
        const principal =
          mode === 'wrong-output' ? { email: 'wrong@example.com' } : request.input.principal;
        const directEvents = [{ event: 'example.Opened', payload: { principal } }];
        if (mode === 'wrong-then-right')
          directEvents.unshift({
            event: 'example.Opened',
            payload: { principal: { email: 'wrong@example.com' } },
          });
        return {
          outcome: 'opened',
          directEvents,
        };
      },
      queryView: () => ({ rows: [] }),
      observeEvents: () => [],
      observeInvocations: () => [],
      configureExternalOutcome() {},
      redeliverEvent() {},
    };
    const run = new ScenarioRun(scope, target, newHarness('example'), 'fixture-test');
    await run.execute(
      'example.Open/outcome/opened',
      suite.scenarios['example.Open/outcome/opened']!,
    );
    assert.equal(run.status, mode === 'valid' ? 'passed' : 'failed', mode);
    assert.equal(begun, 1);
    assert.equal(ended, 1);
  }
});

test('fixture numeric equality preserves exact integers without coercing strings', () => {
  assert.equal(
    equal(new JsonNumber('9007199254740993'), new JsonNumber('9007199254740992')),
    false,
  );
  assert.equal(equal(new JsonNumber('1.0'), 1), true);
  assert.equal(equal(new JsonNumber('1'), '1'), false);
});

test('bad provider data and missing capability prevent session startup', async () => {
  for (const mode of ['invalid', 'missing']) {
    let begun = 0;
    const target: Target = {
      identity: () => ({ name: 'fixture-test', version: '1' }),
      ...(mode === 'invalid' ? { fixtureValues: () => ({ 'current-principal': false }) } : {}),
      beginScenario: () => {
        begun++;
      },
      endScenario() {},
      executeCommand: () => ({}),
      queryView: () => ({}),
      observeEvents: () => [],
      observeInvocations: () => [],
      configureExternalOutcome() {},
      redeliverEvent() {},
    };
    const suite = admitSuite(suiteText());
    const run = new ScenarioRun(scope, target, newHarness('example'), 'fixture-test');
    await assert.rejects(
      run.execute('example.Open/outcome/opened', suite.scenarios['example.Open/outcome/opened']!),
    );
    assert.equal(begun, 0);
    assert.equal(run.status, mode === 'missing' ? 'unsupported' : 'error');
    assert.equal(run.callbacksComplete, true);
  }
});

test('delivery invocation selectors and expected inputs admit fixture references', () => {
  for (const position of ['selecting', 'input']) {
    const observation = {
      step: 'expect_every_invocation',
      binding: 'received',
      command: 'example.Open',
      input: {},
      [position]: { principal: fixture },
    };
    assert.doesNotThrow(() =>
      admitFixtureSteps([{ step: 'resolve_fixtures', fixtures: contract }, observation]),
    );
    assert.throws(() => admitFixtureSteps([observation]), /fixture declarations/);
  }
});

// beyond10x/ess#416: a type that reaches itself only through Optional, List or Map has finite
// values, which a fixture may supply; one with no such boundary has none and stays refused.
test('recursive fixture contracts admit terminated recursion and refuse required recursion', () => {
  const value = 'probe.Value';
  const recursive = (declarations: Record<string, Node>) => ({
    fields: [{ name: 'finite-value', type: value }],
    declarations,
  });
  const struct = (...fields: [string, string][]) => ({
    kind: 'struct',
    fields: fields.map(([name, type]) => ({ name, type })),
  });
  const list = admitFixtures(
    recursive({ [value]: struct(['text', 'String'], ['children', `List<${value}>`]) }),
  );
  for (const declarations of [
    { [value]: struct(['text', 'String'], ['next', `Optional<${value}>`]) },
    { [value]: struct(['named', `Map<String, ${value}>`]) },
    {
      [value]: { kind: 'union', tag: 'kind', variants: { leaf: 'String', pair: 'probe.Pair' } },
      'probe.Pair': struct(['left', value], ['right', value]),
    },
  ]) {
    admitFixtures(recursive(declarations));
  }
  for (const declarations of [
    { [value]: struct(['text', 'String'], ['self', value]) },
    {
      [value]: struct(['text', 'String'], ['other', 'probe.Other']),
      'probe.Other': struct(['back', value]),
    },
  ]) {
    assert.throws(
      () => admitFixtures(recursive(declarations)),
      /fixture input finite-value has no finite value/,
    );
  }
  const tree = {
    text: 'root',
    children: [
      { text: 'a', children: [] },
      { text: 'b', children: [{ text: 'c', children: [] }] },
    ],
  };
  const resolved = fixtureValues(list, { 'finite-value': tree });
  assert.deepEqual(JSON.parse(JSON.stringify(resolved)), { 'finite-value': tree });
  const wrong = JSON.parse(JSON.stringify(tree).replace('"text":"c"', '"text":5'));
  assert.throws(() => fixtureValues(list, { 'finite-value': wrong }));
  let deep: Node = { text: 'deep', children: [] };
  for (let level = 0; level < 70; level++) deep = { text: 'deep', children: [deep] };
  // Either depth guard may stop it first: the copy's JSON nesting or the typed walk's.
  assert.throws(() => fixtureValues(list, { 'finite-value': deep }), /resource|JSON nesting/);
});
