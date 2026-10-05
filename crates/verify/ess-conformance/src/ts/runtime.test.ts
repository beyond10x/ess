// Cases for the parts of the runtime that are pure logic, plus one end-to-end run.
//
// Not emitted: `mod.rs` names the files the package carries, and this is not one of them. It is
// the evidence that the port answers what the Go runtime answers, held beside the source it
// checks so a change to one is a change in the same directory as the other.

import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { existsSync, mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';

import {
  ErrUnsupported,
  JsonNumber,
  accountForEveryScenario,
  admitSuiteDocument,
  asNumber,
  byteCompare,
  canonicalUUID,
  compare,
  countCanonical,
  describeRow,
  asJSON,
  equal,
  exactDecimal,
  exactNumbers,
  exactInteger,
  executionContextConfiguration,
  executionContextDocument,
  goMarshal,
  holds,
  integral,
  isUnsupported,
  lookup,
  matches,
  newHarness,
  paddedBase64,
  primitive,
  publicBuild,
  ranked,
  reduce,
  render,
  reportConfiguration,
  runWith,
  scenarioIdentity,
  strictJSON,
  unsupported,
  UNEXECUTED_STEPS,
  admitAccessor,
  writeReport,
} from './runtime.js';
import type {
  CommandRequest,
  CommandResult,
  Identity,
  Node,
  Row,
  Scenario,
  ScenarioContext,
  Step,
  Target,
  TestScope,
  ViewRequest,
  ViewResult,
} from './runtime.js';

// ---- a recording stand-in for the test context -------------------------------------------------

class Recorder implements TestScope {
  name: string;
  logs: string[] = [];
  skipped: string | undefined;
  failure: unknown;
  children: Recorder[] = [];

  constructor(name: string) {
    this.name = name;
  }

  async test(name: string, fn: (t: TestScope) => Promise<void> | void): Promise<unknown> {
    const child = new Recorder(name);
    this.children.push(child);
    try {
      await fn(child);
    } catch (error) {
      child.failure = error;
    }
    return undefined;
  }

  diagnostic(message: string): void {
    this.logs.push(message);
  }

  skip(message?: string): void {
    this.skipped = message ?? '';
  }

  verdicts(): string[] {
    return this.children.map((child) =>
      child.failure !== undefined ? 'failed' : child.skipped !== undefined ? 'skipped' : 'passed',
    );
  }
}

// ---- the suite the end-to-end cases run --------------------------------------------------------

const digest = 'a'.repeat(64);

// Suites 12–17 carry retained results, string operators and aggregate views, and every later major
// implies them, so this runtime admits and runs them (beyond10x/ess#188). What stays refused is the
// construct in a forged older envelope: refused by its version, before any target callback.
test('suites 12 to 17 are admitted and each is run only with an explicit report format', async () => {
  for (const major of [12, 13, 14, 15, 16, 17]) {
    const raw = JSON.parse(suiteText());
    raw.provenance.suite_version = `ess-conformance/${major}`;
    if (major % 2 === 1) {
      // A coverage major carries its inventory; its absence is refused on its own.
      assert.throws(() => admitSuiteDocument(JSON.stringify(raw), false), /coverage is required/);
      continue;
    }
    assert.equal(
      admitSuiteDocument(JSON.stringify(raw), false).provenance.suite_version,
      `ess-conformance/${major}`,
    );
    await assert.rejects(
      runWith(new Recorder('report'), () => ({}) as Target, JSON.stringify(raw)),
      /require explicit ESS_REPORT_FORMAT=2/,
    );
  }
});

test('retained-result steps in a forged older envelope refuse before target callbacks', async () => {
  let callbacks = 0;
  const target = (): Target => {
    callbacks += 1;
    throw new Error('target must not be constructed');
  };
  for (const [step, reason] of [
    ['capture_command_result', /retained results require suite\/12 or \/13/],
    ['expect_replay_result', /retained results require suite\/12 or \/13/],
    ['expect_no_events', /`expect_no_events` requires suite\/12 or \/13/],
  ] as const) {
    const raw = JSON.parse(suiteText());
    raw.provenance.suite_version = 'ess-conformance/10';
    raw.scenarios['billing.CreateInvoice/outcome/created'].steps.push({ step });
    await assert.rejects(runWith(new Recorder('retained'), target, JSON.stringify(raw)), reason);
  }
  assert.equal(callbacks, 0);
});

// String operators (beyond10x/ess#95) are suite/14 vocabulary: a forged older suite carrying one is
// refused by its version, as the Rust reader refuses it.
test('string operators in a forged older envelope refuse before target callbacks', async () => {
  let callbacks = 0;
  const target = (): Target => {
    callbacks += 1;
    throw new Error('target must not be constructed');
  };
  const satisfies = {
    step: 'expect_view',
    view: 'billing.Invoices',
    expectation: { expect: 'satisfies', predicate: { not: { customer: { starts_with: '+44' } } } },
  };
  for (const operator of ['starts_with', 'ends_with', 'contains']) {
    const raw = JSON.parse(suiteText());
    raw.provenance.suite_version = 'ess-conformance/10';
    raw.scenarios['billing.CreateInvoice/outcome/created'].steps.push({
      ...satisfies,
      expectation: { expect: 'satisfies', predicate: { customer: { [operator]: 'x' } } },
    });
    await assert.rejects(
      runWith(new Recorder('strings'), target, JSON.stringify(raw)),
      /string predicate operators require suite\/14 or \/15/,
    );
  }
  assert.equal(callbacks, 0);
});

// An aggregate scenario (beyond10x/ess#96) is suite/16 vocabulary: a forged older suite carrying one
// is refused by its id, and from suite/16 on the id is well formed.
test('aggregate scenarios in a forged older envelope refuse before target callbacks', async () => {
  let callbacks = 0;
  const target = (): Target => {
    callbacks += 1;
    throw new Error('target must not be constructed');
  };
  const scenario = JSON.parse(suiteText()).scenarios['billing.CreateInvoice/outcome/created'];
  const forged = JSON.parse(suiteText());
  forged.scenarios['billing.Invoices/aggregate'] = scenario;
  await assert.rejects(
    runWith(new Recorder('aggregates'), target, JSON.stringify(forged)),
    /aggregate views require suite\/16 or \/17/,
  );
  assert.throws(
    () => scenarioIdentity('billing.Invoices/aggregate', 15),
    /aggregate views require suite\/16 or \/17/,
  );
  assert.doesNotThrow(() => scenarioIdentity('billing.Invoices/aggregate', 16));
  assert.equal(callbacks, 0);
});

// A construct this runtime cannot execute, from suite/12 on, refuses the one scenario carrying it
// by name and runs every other (beyond10x/ess#188); below suite/12 it refuses the suite, as it
// always did. `UNEXECUTED_STEPS` is empty today, so the case registers a construct of its own.
test('a construct refused by name skips its scenario, and only it', async () => {
  UNEXECUTED_STEPS['future_step'] = { major: 26, what: 'a step from a later build' };
  try {
    const raw = JSON.parse(suiteText());
    raw.provenance.suite_version = 'ess-conformance/26';
    // Two scenarios: the one that runs, and the one carrying the construct.
    delete raw.scenarios['billing.CreateInvoice/outcome/refused'];
    delete raw.scenarios['billing.CreateInvoice/outcome/unanswered'];
    raw.scenarios['billing.CreateInvoice/outcome/deferred'] = {
      purpose: 'A scenario carrying a construct this runtime does not execute',
      source: [],
      steps: [
        ...raw.scenarios['billing.CreateInvoice/outcome/created'].steps,
        { step: 'future_step' },
      ],
    };
    const suite = admitSuiteDocument(JSON.stringify(raw), false);
    assert.match(
      suite.scenarios['billing.CreateInvoice/outcome/deferred']?.refused ?? '',
      /step 2: the `future_step` step \(a step from a later build\)/,
    );
    assert.equal(suite.scenarios['billing.CreateInvoice/outcome/created']?.refused, undefined);

    const recorder = new Recorder('refused');
    let commands = 0;
    const target = (): Target =>
      ({
        identity: () => ({ name: 'refusal', version: '1' }),
        beginScenario: () => {},
        endScenario: () => {},
        executeCommand: () => {
          commands += 1;
          return { outcome: 'created' };
        },
      }) as unknown as Target;
    const previous = process.env.ESS_REPORT_FORMAT;
    process.env.ESS_REPORT_FORMAT = '2';
    try {
      await runWith(recorder, target, JSON.stringify(raw));
    } finally {
      if (previous === undefined) delete process.env.ESS_REPORT_FORMAT;
      else process.env.ESS_REPORT_FORMAT = previous;
    }
    assert.deepEqual(recorder.verdicts(), ['passed', 'skipped']);
    assert.match(recorder.children[1]?.skipped ?? '', /does not execute step 2: the `future_step`/);
    assert.equal(commands, 1, 'the refused scenario reached no target callback');

    raw.provenance.suite_version = 'ess-conformance/10';
    assert.throws(
      () => admitSuiteDocument(JSON.stringify(raw), false),
      /deferred: unsupported step future_step/,
    );
  } finally {
    delete UNEXECUTED_STEPS['future_step'];
  }
});

function suiteText(): string {
  return JSON.stringify({
    provenance: {
      suite_version: 'ess-conformance/1',
      system: 'billing',
      specification_version: 'v3',
      spec_digest: digest,
      contract_digest: 'b'.repeat(64),
    },
    scenarios: {
      'billing.CreateInvoice/outcome/created': {
        purpose: '`billing.CreateInvoice` answers `created`',
        source: [{ kind: 'command', name: 'billing.CreateInvoice' }],
        steps: [
          { step: 'execute_command', command: 'billing.CreateInvoice' },
          {
            step: 'expect_outcome',
            outcome: { command: 'billing.CreateInvoice', outcome: 'created' },
          },
        ],
      },
      'billing.CreateInvoice/outcome/refused': {
        purpose: '`billing.CreateInvoice` answers `refused`',
        source: [{ kind: 'command', name: 'billing.CreateInvoice' }],
        steps: [
          { step: 'execute_command', command: 'billing.RefuseInvoice' },
          {
            step: 'expect_outcome',
            outcome: { command: 'billing.RefuseInvoice', outcome: 'refused' },
          },
        ],
      },
      'billing.CreateInvoice/outcome/unanswered': {
        purpose: '`billing.CreateInvoice` answers `unanswered`',
        source: [{ kind: 'command', name: 'billing.CreateInvoice' }],
        steps: [
          { step: 'execute_command', command: 'billing.Unanswerable' },
          {
            step: 'expect_outcome',
            outcome: { command: 'billing.Unanswerable', outcome: 'unanswered' },
          },
        ],
      },
    },
  });
}

class ExampleTarget implements Target {
  identity(): Identity {
    return { name: 'example', version: '0.1.0' };
  }

  beginScenario(_scenario: ScenarioContext): void {}

  endScenario(_scenario: ScenarioContext): void {}

  executeCommand(request: CommandRequest): CommandResult {
    if (request.command === 'billing.Unanswerable') {
      throw unsupported('`billing.Unanswerable` has no caller a target can be');
    }
    if (request.command === 'billing.RefuseInvoice') {
      return { outcome: 'accepted', directEvents: [] };
    }
    return { outcome: 'created', directEvents: [] };
  }

  queryView(_request: ViewRequest): ViewResult {
    throw unsupported('no views');
  }

  observeEvents(): never {
    throw unsupported('no events');
  }

  configureExternalOutcome(): never {
    throw unsupported('no external outcomes');
  }

  redeliverEvent(): never {
    throw unsupported('no redelivery');
  }

  observeInvocations(): never {
    throw unsupported('no invocations');
  }
}

function nestedIncrementSuite(): string {
  const raw = JSON.parse(suiteText());
  raw.provenance.suite_version = 'ess-conformance/32';
  raw.scenarios = {
    'nested.counter/authored/increment': {
      purpose: 'A nested increment reads its own previous location',
      source: [],
      steps: [
        { step: 'query_view', view: 'nested.counter.Counters' },
        {
          step: 'expect_view',
          view: 'nested.counter.Counters',
          expectation: {
            expect: 'contains',
            fields: {
              amount: { kind: 'literal', value: 3 },
              packet: {
                kind: 'members',
                members: { amount: { kind: 'literal', value: 101 } },
              },
            },
          },
        },
      ],
    },
  };
  return JSON.stringify(raw);
}

class NestedIncrementTarget extends ExampleTarget {
  private readonly nestedAmount: number;

  constructor(nestedAmount: number) {
    super();
    this.nestedAmount = nestedAmount;
  }

  queryView(_request: ViewRequest): ViewResult {
    return { rows: [{ amount: 3, packet: { amount: this.nestedAmount } }] };
  }
}

test('nested increment suite passes the full path and fails the old-leaf mutant', async () => {
  await withEnvironmentAsync({ ESS_REPORT_FORMAT: '2', ESS_REPORT_OUT: undefined }, async () => {
    const correct = new Recorder('correct nested location');
    await runWith(correct, () => new NestedIncrementTarget(101), nestedIncrementSuite());
    assert.deepEqual(correct.verdicts(), ['passed']);

    // The former defect incremented the unrelated top-level `amount` (3 + 1) and stored that 4 in
    // `packet.amount`; this target is the deliberate regression mutant.
    const oldLeafMutant = new Recorder('old leaf mutant');
    await runWith(oldLeafMutant, () => new NestedIncrementTarget(4), nestedIncrementSuite());
    assert.deepEqual(oldLeafMutant.verdicts(), ['failed']);
  });
});

function setEnvironment(
  values: Record<string, string | undefined>,
): Record<string, string | undefined> {
  const held: Record<string, string | undefined> = {};
  for (const key of Object.keys(values)) {
    held[key] = process.env[key];
    if (values[key] === undefined) {
      delete process.env[key];
    } else {
      process.env[key] = values[key];
    }
  }
  return held;
}

function restoreEnvironment(held: Record<string, string | undefined>): void {
  for (const key of Object.keys(held)) {
    if (held[key] === undefined) {
      delete process.env[key];
    } else {
      process.env[key] = held[key];
    }
  }
}

function withEnvironment<T>(values: Record<string, string | undefined>, body: () => T): T {
  const held = setEnvironment(values);
  try {
    return body();
  } finally {
    restoreEnvironment(held);
  }
}

/**
 * The asynchronous form, which the runner needs: a `finally` around a promise that is returned
 * rather than awaited restores the environment before the run has read a variable out of it.
 */
async function withEnvironmentAsync<T>(
  values: Record<string, string | undefined>,
  body: () => Promise<T>,
): Promise<T> {
  const held = setEnvironment(values);
  try {
    return await body();
  } finally {
    restoreEnvironment(held);
  }
}

// ---- exact integers ----------------------------------------------------------------------------

test('an integer token binary64 cannot hold keeps its digits', () => {
  assert.equal(exactInteger('9007199254740993'), '9007199254740993');
  assert.equal(exactInteger('9007199254740992'), '9007199254740992');
  assert.equal(exactInteger('9007199254740991'), null);
  assert.equal(exactInteger('42'), null);
  assert.equal(exactInteger('-00009007199254740993'), '-9007199254740993');
  assert.equal(exactInteger('9007199254740993.000'), '9007199254740993');
  assert.equal(exactInteger('9007199254740993.5'), null);
  assert.equal(exactInteger('1e400'), null);
});

// ---- strict JSON ---------------------------------------------------------------------------------

test('admission refuses a duplicate key, trailing input and a lone surrogate', () => {
  assert.throws(() => strictJSON('{"a":1,"a":2}'), /duplicate key/);
  assert.throws(() => strictJSON('{} {}'), /trailing JSON input/);
  assert.throws(() => strictJSON('"\\ud800"'), /lone high surrogate/);
  assert.throws(() => strictJSON('"\\udc00"'), /lone low surrogate/);
  assert.doesNotThrow(() => strictJSON('"\\ud83d\\ude00"'));
});

test('admission keeps an integer token exactly rather than through binary64', () => {
  const value = strictJSON('{"n":9007199254740993}') as Record<string, Node>;
  assert.ok(value.n instanceof JsonNumber);
  assert.equal((value.n as JsonNumber).toString(), '9007199254740993');
});

// ---- identity and names --------------------------------------------------------------------------

test('a scenario identity is one of the seven shapes', () => {
  assert.doesNotThrow(() => scenarioIdentity('billing.CreateInvoice/outcome/created'));
  assert.doesNotThrow(() => scenarioIdentity('acd.Agent/state/Ready/refuses/acd.Hangup'));
  assert.doesNotThrow(() => scenarioIdentity('pay-gateway/binding/delivery'));
  assert.throws(() => scenarioIdentity('nope'), /malformed scenario ID/);
  assert.throws(() => scenarioIdentity('pay-gateway/binding/other'), /malformed scenario ID/);
});

// A scenario per selected refusal (beyond10x/ess#269) arrived in suite/36 and /37.
test('a selected refusal scenario identity requires suite/36', () => {
  assert.throws(
    () => scenarioIdentity('notify-ledger/binding/refusal/at-limit', 35),
    /a scenario per selected refusal requires suite\/36 or \/37/,
  );
  assert.doesNotThrow(() => scenarioIdentity('notify-ledger/binding/refusal/at-limit', 36));
  assert.doesNotThrow(() => scenarioIdentity('notify-ledger/binding/refusal/at-limit', 37));
  assert.throws(
    () => scenarioIdentity('notify-ledger/binding/refusal/At_Limit', 36),
    /malformed scenario ID/,
  );
  assert.throws(
    () => scenarioIdentity('notify-ledger/binding/policy/at-limit', 36),
    /malformed scenario ID/,
  );
});

test('a system name reduces to identifier segments', () => {
  assert.equal(reduce('Agent Call Distribution!'), 'Agent-Call-Distribution');
  assert.equal(reduce('!!!'), 'ess');
  const harness = newHarness('billing');
  assert.equal(harness.correlation(), 'billing-000001');
  assert.equal(harness.correlation(), 'billing-000002');
  assert.equal(harness.deadline().attempts, 8);
});

// ---- the value grammar ---------------------------------------------------------------------------

test('a primitive is a grammar and not merely a shape', () => {
  assert.equal(primitive('uuid', '0f8fad5b-d9cb-469f-a165-70867728950e'), '');
  assert.notEqual(primitive('uuid', 'not-a-uuid'), '');
  assert.equal(canonicalUUID('0F8FAD5B-D9CB-469F-A165-70867728950E'), true);
  assert.equal(canonicalUUID('urn:uuid:0f8fad5b-d9cb-469f-a165-70867728950e'), false);
  assert.equal(paddedBase64('AA=A'), false);
  assert.equal(paddedBase64('QUJD'), true);
  assert.equal(paddedBase64('QQ=='), true);
  assert.equal(integral(9223372036854775808), true);
  assert.equal(integral(1.5), false);
  assert.equal(primitive('integer', 3), '');
  assert.notEqual(primitive('integer', 3.5), '');
  assert.notEqual(primitive('boolean', 'true'), '');
});

test('a dotted path answers value, absent or blocked', () => {
  const payload: Record<string, Node> = { amount: { currency: 'EUR' }, scalar: 3, nulled: null };
  assert.deepEqual(lookup(payload, 'amount.currency'), ['EUR', 'value', 'amount.currency']);
  assert.deepEqual(lookup(payload, 'amount.missing'), [null, 'absent', 'amount.missing']);
  assert.deepEqual(lookup(payload, 'scalar.currency'), [3, 'blocked', 'scalar']);
  assert.deepEqual(lookup(payload, 'nulled.currency'), [null, 'absent', 'nulled']);
});

test('optional excuses absence and never a blocked prefix or a wrong kind', () => {
  assert.equal(holds({ a: 'x' }, { a: { holds: 'primitive', kind: 'string' } }), '');
  assert.equal(holds({}, { a: { holds: 'primitive', kind: 'string', optional: true } }), '');
  assert.equal(
    holds({ a: null }, { a: { holds: 'primitive', kind: 'string', optional: true } }),
    '',
  );
  assert.equal(holds({}, { a: { holds: 'primitive', kind: 'string' } }), 'did not carry `a`');
  assert.equal(
    holds({ a: 3 }, { 'a.b': { holds: 'primitive', kind: 'string', optional: true } }),
    '`a` holds 3, so `a.b` is not there to read',
  );
  assert.equal(
    holds({ a: 'Fax' }, { a: { holds: 'enum', variants: ['Email', 'Post'] } }),
    '`a` holds "Fax" and the specification declares one of Email, Post',
  );
  assert.equal(holds({ a: 'Post' }, { a: { holds: 'enum', variants: ['Email', 'Post'] } }), '');
  assert.equal(holds({ a: 3 }, { a: { holds: 'list' } }), '');
});

// ---- comparison and ordering -----------------------------------------------------------------------

test('values compare structurally and numbers compare across carriers', () => {
  assert.equal(equal({ a: [1, 'x', null] }, { a: [1, 'x', null] }), true);
  assert.equal(equal({ a: 1 }, { a: 1, b: 2 }), false);
  assert.equal(equal(1, new JsonNumber('1')), true);
  assert.equal(asNumber(new JsonNumber('1.5'))[0], 1.5);
  assert.equal(asNumber('1.5')[1], false);
  assert.equal(matches({ a: 1, b: 2 }, { a: 1 }), true);
  assert.equal(matches({ a: 1 }, { a: 1, b: 2 }), false);
  assert.deepEqual(compare(1, 2), [-1, true]);
  assert.deepEqual(compare('b', 'a'), [1, true]);
  assert.deepEqual(compare(false, true), [-1, true]);
  assert.deepEqual(compare(1, 'a'), [0, false]);
});

test('ranking reads adjacent pairs and holds for fewer than two rows', () => {
  const rows: Row[] = [{ at: 1 }, { at: 2 }, { at: 3 }];
  assert.deepEqual(ranked('v', ['at'], rows), [true, '', false]);
  assert.deepEqual(ranked('v', ['at desc'], rows)[0], false);
  assert.deepEqual(ranked('v', ['at'], [{ at: 1 }]), [true, '', false]);
  assert.equal(ranked('v', ['missing'], rows)[2], true);
});

test('a rendered value is the shortest decimal and never an exponent', () => {
  assert.equal(render(1e21), '1000000000000000000000');
  assert.equal(render(1.5), '1.5');
  assert.equal(render(-0), '-0');
  assert.equal(render(null), 'null');
  assert.equal(render('x'), '"x"');
  assert.equal(render(true), 'true');
  assert.equal(describeRow({ b: 2, a: 1 }), 'a = 1, b = 2');
});

test('sorting follows byte order rather than UTF-16 code units', () => {
  assert.ok(byteCompare('', '\u{10000}') < 0);
  assert.ok('' > '\u{10000}');
});

// ---- the unsupported answer --------------------------------------------------------------------------

test('unsupported survives wrapping and is not any other error', () => {
  assert.equal(isUnsupported(ErrUnsupported), true);
  assert.equal(isUnsupported(unsupported('cannot invoke `billing.Charge`')), true);
  assert.equal(isUnsupported(new Error('deliberate', { cause: ErrUnsupported })), true);
  assert.equal(isUnsupported(new Error('deliberate', { cause: new Error('other') })), false);
  assert.equal(isUnsupported(new Error('deliberate')), false);
  assert.match(unsupported('cannot invoke `billing.Charge`').message, /cannot invoke/);
});

// ---- the report ----------------------------------------------------------------------------------------

test('a filtered run refuses a report rather than writing a partial one', () => {
  const suite = admitSuiteDocument(suiteText(), false);
  assert.equal(accountForEveryScenario(suite, 3), null);
  const refusal = accountForEveryScenario(suite, 1);
  assert.match(String(refusal), /incomplete execution: 1 of 3 scenarios reached a conclusion/);
});

test('report/2 is canonical: sorted keys, two-space indent, one trailing newline', () => {
  const encoded = countCanonical({ b: 'two', a: new JsonNumber('1'), c: [new JsonNumber('2')] });
  assert.equal(encoded, '{\n  "a": 1,\n  "b": "two",\n  "c": [\n    2\n  ]\n}\n');
});

test('the report format is selected explicitly and strictness needs format 2', () => {
  withEnvironment(
    {
      ESS_REPORT_FORMAT: undefined,
      ESS_CONFORMANCE_STRICT: undefined,
      ESS_CONFORMANCE_ALLOW_INCOMPLETE: undefined,
    },
    () => {
      assert.deepEqual(reportConfiguration(), { version: '1', strict: false });
    },
  );
  withEnvironment({ ESS_REPORT_FORMAT: '3' }, () => {
    assert.throws(() => reportConfiguration(), /ESS_REPORT_FORMAT must be 1 or 2/);
  });
  withEnvironment({ ESS_REPORT_FORMAT: '1', ESS_CONFORMANCE_STRICT: '1' }, () => {
    assert.throws(
      () => reportConfiguration(),
      /strict conformance requires explicit ESS_REPORT_FORMAT=2/,
    );
  });
  withEnvironment(
    { ESS_REPORT_FORMAT: '2', ESS_CONFORMANCE_STRICT: '1', ESS_CONFORMANCE_ALLOW_INCOMPLETE: '1' },
    () => {
      assert.throws(() => reportConfiguration(), /strict and allow-incomplete conflict/);
    },
  );
});

// ---- admission -------------------------------------------------------------------------------------------

test('a suite admits only what the format declares', () => {
  const suite = admitSuiteDocument(suiteText(), false);
  assert.equal(suite.provenance.suite_version, 'ess-conformance/1');
  assert.equal(Object.keys(suite.scenarios).length, 3);
  const scenario = suite.scenarios['billing.CreateInvoice/outcome/created'] as Scenario;
  const steps: Step[] = scenario.steps;
  assert.equal(steps[0]?.step, 'execute_command');
  assert.equal(steps[1]?.outcome?.outcome, 'created');

  const withCoverage = JSON.parse(suiteText()) as Record<string, Node>;
  withCoverage.coverage = { knowledge: 'unknown' };
  assert.throws(
    () => admitSuiteDocument(JSON.stringify(withCoverage), false),
    /coverage is required exactly for suite\/5, suite\/7, suite\/9 and suite\/11/,
  );

  const unknownStep = JSON.parse(suiteText()) as any;
  unknownStep.scenarios['billing.CreateInvoice/outcome/created'].steps[0].step = 'teleport';
  assert.throws(
    () => admitSuiteDocument(JSON.stringify(unknownStep), false),
    /unsupported step teleport/,
  );
});

// ---- the whole run ----------------------------------------------------------------------------------------

test('one target per scenario, and passed, failed and skipped are three different facts', async () => {
  const built: string[] = [];
  const recorder = new Recorder('conformance');
  await withEnvironmentAsync({ ESS_REPORT_FORMAT: undefined, ESS_REPORT_OUT: undefined }, () =>
    runWith(
      recorder,
      () => {
        built.push('target');
        return new ExampleTarget();
      },
      suiteText(),
    ),
  );

  assert.equal(built.length, 4, 'one target for the identity and one per scenario');
  assert.deepEqual(recorder.verdicts(), ['passed', 'failed', 'skipped']);
  assert.match(recorder.logs[0] ?? '', /billing v3, 3 scenario\(s\), spec digest/);
});

test('report/1 names every scenario that did not pass', async () => {
  const directory = mkdtempSync(join(tmpdir(), 'essconform-'));
  const path = join(directory, 'report.json');
  try {
    const recorder = new Recorder('conformance');
    await withEnvironmentAsync({ ESS_REPORT_FORMAT: '1', ESS_REPORT_OUT: path }, () =>
      runWith(recorder, () => new ExampleTarget(), suiteText()),
    );
    const document = JSON.parse(readFileSync(path, 'utf8')) as Record<string, Node>;
    assert.equal(document.format, 'ess-conformance-report/1');
    assert.equal(document.status, 'failed');
    assert.equal(document.scenarios_total, 3);
    assert.equal(document.scenarios_failed, 2);
    assert.deepEqual(document.failed_scenarios, [
      'failed billing.CreateInvoice/outcome/refused',
      'skipped billing.CreateInvoice/outcome/unanswered',
    ]);
    assert.equal(document.implementation, 'example 0.1.0');
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

/** captureStderr runs body and returns what it wrote to process.stderr, which it does not pass on. */
async function captureStderr(body: () => Promise<void>): Promise<string> {
  const original = process.stderr.write.bind(process.stderr);
  let written = '';
  process.stderr.write = ((chunk: string | Uint8Array): boolean => {
    written += typeof chunk === 'string' ? chunk : Buffer.from(chunk).toString('utf8');
    return true;
  }) as typeof process.stderr.write;
  try {
    await body();
  } finally {
    process.stderr.write = original;
  }
  return written;
}

const skipNotice =
  '1 scenario(s) skipped; set ESS_REPORT_FORMAT=2 (or --report-format 2) for passed, failed and ' +
  'skipped counts\n';

test('report/1 with a skip points once at report/2 for the counts (ess#110)', async () => {
  const directory = mkdtempSync(join(tmpdir(), 'essconform-'));
  const path = join(directory, 'report.json');
  try {
    const written = await captureStderr(() =>
      withEnvironmentAsync({ ESS_REPORT_FORMAT: '1', ESS_REPORT_OUT: path }, () =>
        runWith(new Recorder('conformance'), () => new ExampleTarget(), suiteText()),
      ),
    );
    assert.equal(written, skipNotice);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

test('report/1 without a skip, and report/2, print no notice', async () => {
  const directory = mkdtempSync(join(tmpdir(), 'essconform-'));
  const path = join(directory, 'report.json');
  try {
    const counted = await captureStderr(() =>
      withEnvironmentAsync({ ESS_REPORT_FORMAT: '2', ESS_REPORT_OUT: path }, () =>
        runWith(new Recorder('conformance'), () => new ExampleTarget(), suiteText()),
      ),
    );
    assert.equal(counted, '');
    const unpublished = await captureStderr(() =>
      withEnvironmentAsync({ ESS_REPORT_FORMAT: '1', ESS_REPORT_OUT: undefined }, () =>
        runWith(new Recorder('conformance'), () => new ExampleTarget(), suiteText()),
      ),
    );
    assert.equal(unpublished, '');
    const suite = admitSuiteDocument(suiteText(), false);
    const clean = await captureStderr(async () =>
      withEnvironmentAsync({ ESS_REPORT_OUT: path }, async () =>
        writeReport(
          new Recorder('conformance'),
          suite,
          { name: 'example', version: '0.1.0' },
          [
            { id: 'a', status: 'passed' },
            { id: 'b', status: 'failed' },
          ],
          Object.keys(suite.scenarios).length,
        ),
      ),
    );
    assert.equal(clean, '');
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

test('report/2 counts every terminal verdict and is refused for an incomplete run', async () => {
  const directory = mkdtempSync(join(tmpdir(), 'essconform-'));
  const path = join(directory, 'report.json');
  try {
    const recorder = new Recorder('conformance');
    await withEnvironmentAsync({ ESS_REPORT_FORMAT: '2', ESS_REPORT_OUT: path }, () =>
      runWith(recorder, () => new ExampleTarget(), suiteText()),
    );
    const document = JSON.parse(readFileSync(path, 'utf8')) as any;
    assert.equal(document.format, 'ess-conformance-report/2');
    assert.deepEqual(document.counts, {
      total: 3,
      passed: 1,
      failed: 1,
      error: 0,
      unsupported: 1,
      skipped: 0,
    });
    assert.equal(document.execution_status, 'failed');
    assert.equal(document.conformance_status, 'failed');
    assert.equal(document.producer_profile, 'go-scenario-status/2');
    assert.equal(document.coverage.knowledge, 'unknown');
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

test('the marshaller sorts object keys and escapes as Go does', () => {
  assert.equal(goMarshal({ b: 1, a: '<&>' }, true), '{"a":"\\u003c\\u0026\\u003e","b":1}');
  assert.equal(goMarshal({ b: 1, a: '<&>' }, false), '{"a":"<&>","b":1}');
  assert.equal(goMarshal(' '), '"\\u2028"');
});

// Target answers are read as JSON reads them (beyond10x/ess#188): a key holding `undefined` is
// absent, an `undefined` list item is null, and a JsonNumber passes untouched.
test('asJSON reads a target answer as JSON does', () => {
  const exact = new JsonNumber('9007199254740993');
  assert.deepEqual(asJSON({ a: undefined, b: null, c: [undefined, { d: undefined }], e: exact }), {
    b: null,
    c: [null, {}],
    e: exact,
  });
  assert.equal(Object.hasOwn(asJSON({ a: undefined }) as object, 'a'), false);
});

test('asJSON preserves special own object keys without changing the prototype', () => {
  const input = JSON.parse('{"__proto__":{"audit":"private"},"constructor":"data"}');
  const observed = asJSON(input) as Record<string, Node>;
  assert.deepEqual(observed, input);
  assert.equal(Object.hasOwn(observed, '__proto__'), true);
  assert.equal(Object.getPrototypeOf(observed), Object.prototype);
  assert.equal(Object.hasOwn(Object.prototype, 'audit'), false);
});

// Everything is read exactly as `JSON.parse(JSON.stringify(x))` reads it, with two documented
// exceptions that keep a number exact: a JsonNumber passes as is, and a BigInt — which
// JSON.stringify refuses — becomes the JsonNumber of its digits.
test('asJSON is JSON.stringify then JSON.parse, JsonNumber and BigInt aside', () => {
  class Row {
    id = 'a';
    note: unknown = undefined;
    hidden = (): number => 1;
    get computed(): number {
      return 2;
    }
  }
  class Answer {
    readonly rows: unknown[];
    constructor(rows: unknown[]) {
      this.rows = rows;
    }
  }
  const withToJSON = { toJSON: (key: string) => ({ key, via: 'toJSON' }) };
  const holey: unknown[] = [1, , 3]; // eslint-disable-line no-sparse-arrays
  const inherited = Object.create({ inheritedKey: 1 }) as { own?: number };
  inherited.own = 2;
  const hiddenKey = {};
  Object.defineProperty(hiddenKey, 'secret', { value: 1, enumerable: false });
  const table: unknown[] = [
    null,
    true,
    'text',
    0,
    -0,
    1.5,
    NaN,
    Infinity,
    -Infinity,
    [undefined, () => 1, Symbol('s'), 4],
    holey,
    { a: undefined, b: () => 1, c: Symbol('s'), d: 4, [Symbol('k')]: 5 },
    new Row(),
    new Answer([{ id: 'a', note: undefined }, new Row()]),
    { at: new Date(Date.UTC(2026, 0, 1)) },
    [new Date(0)],
    { map: new Map([['k', 1]]), set: new Set([1]) },
    new Map([['k', 1]]),
    { nested: withToJSON, list: [withToJSON] },
    // eslint-disable-next-line no-new-wrappers
    [new Number(3), new String('s'), new Boolean(false)],
    inherited,
    hiddenKey,
    { deep: { deeper: [{ gone: undefined, kept: null }] } },
  ];
  for (const [index, value] of table.entries()) {
    assert.deepStrictEqual(asJSON(value), JSON.parse(JSON.stringify(value)), `table row ${index}`);
  }
  // `undefined`, a function and a symbol at the top are no JSON at all.
  for (const value of [undefined, () => 1, Symbol('s')]) {
    assert.equal(asJSON(value), undefined);
    assert.equal(JSON.stringify(value), undefined);
  }
  // Exception two: JSON.stringify refuses a BigInt, and asJSON reads it as its exact digits, at any
  // depth, and through a toJSON that answers one.
  assert.throws(() => JSON.stringify({ n: 1n }), TypeError);
  assert.deepStrictEqual(asJSON({ n: 9007199254740993n, list: [-2n] }), {
    n: new JsonNumber('9007199254740993'),
    list: [new JsonNumber('-2')],
  });
  assert.deepStrictEqual(asJSON({ toJSON: () => 7n }), new JsonNumber('7'));
  // A cycle throws, as it does in JSON.stringify.
  const cycle: { self?: unknown } = {};
  cycle.self = cycle;
  assert.throws(() => JSON.stringify(cycle), TypeError);
  assert.throws(() => asJSON(cycle), TypeError);
  // A value reached twice, but not through itself, is no cycle.
  const shared = { id: 1 };
  assert.deepStrictEqual(asJSON([shared, shared]), [{ id: 1 }, { id: 1 }]);
});

// A JsonNumber a target forwards with JSON.stringify is written as its digits.
test('JsonNumber serializes as the number it spells', () => {
  const big = new JsonNumber('9007199254740993');
  const written = JSON.stringify({ n: big, m: new JsonNumber('1.5') });
  if (typeof (JSON as { rawJSON?: unknown }).rawJSON === 'function') {
    assert.equal(written, '{"n":9007199254740993,"m":1.5}');
  } else {
    assert.equal(written, '{"n":"9007199254740993","m":1.5}');
  }
});

// Exponent spellings are the decimals they denote, as `Number::exact_text` spells them, and a
// value past the `i128` units the Rust arithmetic keeps has no exact spelling.
test('exactDecimal expands exponent spellings and bounds the units', () => {
  assert.deepEqual(exactDecimal(0.0000001), [1n, 7]);
  assert.deepEqual(exactDecimal(new JsonNumber('1e21')), [10n ** 21n, 0]);
  assert.deepEqual(exactDecimal(new JsonNumber('1.50')), [15n, 1]);
  assert.deepEqual(exactDecimal(new JsonNumber('-2.5E-3')), [-25n, 4]);
  assert.equal(exactDecimal(new JsonNumber('1e40')), null);
  assert.equal(exactDecimal('1'), null);
});

// A literal stays exactly the number written: a JS number only where it is that number.
test('exactNumbers keeps an integer past 2^53 exact and plain numbers plain', () => {
  const big = new JsonNumber('9007199254740993');
  assert.equal(exactNumbers(big), big);
  assert.equal(exactNumbers(new JsonNumber('9007199254740992')), 9007199254740992);
  assert.equal(exactNumbers(new JsonNumber('1.666667')), 1.666667);
  assert.deepEqual(exactNumbers({ n: [new JsonNumber('3')] }), { n: [3] });
});

// ---- host execution provenance (beyond10x/ess#296) -----------------------------------------------

const build = `sha256:${'c'.repeat(64)}`;

test('an execution context is configured explicitly, with report/2 and a file of its own', () => {
  const unset = {
    ESS_IMPLEMENTATION_BUILD: undefined,
    ESS_EXECUTION_CONTEXT_OUT: undefined,
    ESS_REPORT_OUT: '/out/report.json',
  };
  withEnvironment(unset, () => {
    assert.equal(executionContextConfiguration('2'), undefined);
  });
  const both = {
    ESS_IMPLEMENTATION_BUILD: build,
    ESS_EXECUTION_CONTEXT_OUT: '/out/execution.json',
    ESS_REPORT_OUT: '/out/report.json',
  };
  withEnvironment(both, () => {
    assert.deepEqual(executionContextConfiguration('2'), { build, out: '/out/execution.json' });
    assert.throws(
      () => executionContextConfiguration('1'),
      /requires explicit ESS_REPORT_FORMAT=2/,
    );
  });
  for (const [change, reason] of [
    [{ ESS_IMPLEMENTATION_BUILD: undefined }, /together or not at all/],
    [{ ESS_EXECUTION_CONTEXT_OUT: undefined }, /together or not at all/],
    [{ ESS_REPORT_OUT: undefined }, /requires ESS_REPORT_OUT/],
    [{ ESS_IMPLEMENTATION_BUILD: 'v1.2.3' }, /64 lowercase hexadecimal digits/],
    [{ ESS_IMPLEMENTATION_BUILD: build.toUpperCase() }, /64 lowercase hexadecimal digits/],
    [{ ESS_EXECUTION_CONTEXT_OUT: '/out/./report.json' }, /other than ESS_REPORT_OUT/],
  ] as const) {
    withEnvironment({ ...both, ...change }, () => {
      assert.throws(() => executionContextConfiguration('2'), reason);
    });
  }
  assert.equal(publicBuild(build), true);
  assert.equal(publicBuild(`${build}0`), false);
});

test('the execution context binds the exact report and suite bytes and nothing else', async () => {
  const directory = mkdtempSync(join(tmpdir(), 'ess-execution-context-'));
  try {
    const report = join(directory, 'report.json');
    const context = join(directory, 'execution.json');
    const text = suiteText();
    const target = (): Target =>
      ({
        identity: () => ({ name: 'context-target', version: '7' }),
        beginScenario: () => {},
        endScenario: () => {},
        executeCommand: ({ command }: CommandRequest) => {
          if (command === 'billing.Unanswerable') throw ErrUnsupported;
          // Every command answers `created`, so the `refused` scenario fails.
          return { outcome: 'created' };
        },
      }) as unknown as Target;
    await withEnvironmentAsync(
      {
        ESS_REPORT_FORMAT: '2',
        ESS_REPORT_OUT: report,
        ESS_IMPLEMENTATION_BUILD: build,
        ESS_EXECUTION_CONTEXT_OUT: context,
        ESS_CONFORMANCE_STRICT: undefined,
        ESS_CONFORMANCE_ALLOW_INCOMPLETE: undefined,
      },
      () => runWith(new Recorder('context'), target, text),
    );
    const written = readFileSync(report, 'utf8');
    const reportDocument = JSON.parse(written);
    assert.deepEqual(reportDocument.outcomes.failed, ['billing.CreateInvoice/outcome/refused']);
    assert.deepEqual(reportDocument.outcomes.unsupported, [
      'billing.CreateInvoice/outcome/unanswered',
    ]);
    assert.equal(reportDocument.conformance_status, 'failed');
    const sha = (value: string): string =>
      `sha256:${createHash('sha256').update(value, 'utf8').digest('hex')}`;
    assert.equal(
      readFileSync(context, 'utf8'),
      countCanonical({
        format: 'ess-conformance-execution/1',
        implementation: 'context-target 7',
        implementation_build: build,
        report_digest: sha(written),
        suite_digest: sha(text),
      }),
    );
    assert.deepEqual(
      executionContextDocument(
        { original: text } as never,
        { name: 'context-target', version: '7' },
        written,
        build,
      ),
      JSON.parse(readFileSync(context, 'utf8')),
    );
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

test('a refused execution context configuration reaches no target', async () => {
  let made = 0;
  const target = (): Target => {
    made += 1;
    throw new Error('no target is made');
  };
  await assert.rejects(
    withEnvironmentAsync(
      {
        ESS_REPORT_FORMAT: '2',
        ESS_REPORT_OUT: '/nowhere/report.json',
        ESS_IMPLEMENTATION_BUILD: 'not-a-build',
        ESS_EXECUTION_CONTEXT_OUT: '/nowhere/execution.json',
      },
      () => runWith(new Recorder('refused'), target, suiteText()),
    ),
    /64 lowercase hexadecimal digits/,
  );
  assert.equal(made, 0);
});

// ---- an accessor through a unit variant (ess/22, beyond10x/ess#418) ------------------------------

// The observation the reference runner writes, in
// `crates/verify/ess-conformance/tests/fixtures/unit-variant-accessor.json`; the Rust and Go answers
// to the same payloads are `tests/union_unit_variants_accessor.rs`.
function unitVariantAccessor(): string {
  const relative = 'crates/verify/ess-conformance/tests/fixtures/unit-variant-accessor.json';
  let directory = import.meta.dirname;
  for (;;) {
    const candidate = join(directory, relative);
    if (existsSync(candidate)) return readFileSync(candidate, 'utf8');
    const parent = dirname(directory);
    if (parent === directory) throw new Error(`no ${relative} above ${import.meta.dirname}`);
    directory = parent;
  }
}

test('a unit variant is read as unavailable, and a payload beside its tag is malformed', () => {
  const accessor = admitAccessor(strictJSON(unitVariantAccessor()));
  assert.deepEqual(accessor.evaluate({ choice: { kind: 'gone' } }), [null, false]);
  assert.deepEqual(accessor.evaluate({ choice: { kind: 'ready', value: { status: 'yes' } } }), [
    'yes',
    true,
  ]);
  for (const malformed of [
    { kind: 'gone', value: 'gone' },
    { kind: 'gone', value: null },
    { kind: 'ready' },
  ]) {
    assert.throws(() => accessor.evaluate({ choice: malformed }), JSON.stringify(malformed));
  }
});
