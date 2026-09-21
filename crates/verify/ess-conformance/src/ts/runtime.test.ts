// Cases for the parts of the runtime that are pure logic, plus one end-to-end run.
//
// Not emitted: `mod.rs` names the files the package carries, and this is not one of them. It is
// the evidence that the port answers what the Go runtime answers, held beside the source it
// checks so a change to one is a change in the same directory as the other.

import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

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
  equal,
  exactInteger,
  goMarshal,
  holds,
  integral,
  isUnsupported,
  lookup,
  matches,
  newHarness,
  paddedBase64,
  primitive,
  ranked,
  reduce,
  render,
  reportConfiguration,
  runWith,
  scenarioIdentity,
  strictJSON,
  unsupported,
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
      unsupported: 0,
      skipped: 1,
    });
    assert.equal(document.execution_status, 'failed');
    assert.equal(document.conformance_status, 'failed');
    assert.equal(document.producer_profile, 'go-scenario-status/1');
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
