// The #269 ledger (`tests/fixtures/refusal-policy.yaml`), the sender `tests/refusal_policy.rs`
// implements in Rust: a binding whose failure policy is selected per refusal of `Record` —
// `wrong-state` dropped, `unavailable`/`busy`/`rejected` retried up to three attempts in all with
// `rejected` final, and everything else, untyped failures included, escalated. Modes: `correct`,
// `fallback-everywhere`, `swapped-policy`, `extra-retry`, `duplicate-escalation`,
// `omits-attempt`, `retries-final`, `retries-drop`.

const mode = process.env.ESS_TARGET_MODE ?? 'correct';
const BINDING = 'notify-ledger';
const PLACE = 'demo.ledger.Place';
const RECORD = 'demo.ledger.Record';
const RECORDED = 'demo.ledger.Recorded';
const ESCALATED = 'demo.ledger.RecordEscalated';
const ERRORS = {
  unavailable: 'demo.ledger.Unavailable',
  busy: 'demo.ledger.Unavailable',
  rejected: 'demo.ledger.Unknown',
  'at-limit': 'demo.ledger.AtLimit',
  'wrong-state': 'demo.ledger.WrongState',
};

class Ledger {
  log = [];
  invocations = [];
  forced = undefined;
  identity() {
    return { name: 'refusal-policy', version: '1' };
  }
  beginScenario() {}
  endScenario() {}
  /** The next answer of `Record`: a forced one, or `recorded`. */
  answer() {
    if (this.forced !== undefined && this.forced.remaining > 0) {
      this.forced.remaining -= 1;
      return this.forced.outcome;
    }
    return 'recorded';
  }
  policy(answer) {
    if (mode === 'fallback-everywhere') {
      return 'escalate';
    }
    switch (answer) {
      case 'wrong-state':
        return mode === 'swapped-policy' ? 'escalate' : mode === 'retries-drop' ? 'retry' : 'drop';
      case 'unavailable':
      case 'busy':
      case 'rejected':
        return 'retry';
      case 'at-limit':
        return mode === 'swapped-policy' ? 'drop' : 'escalate';
      default:
        return 'escalate';
    }
  }
  /** The binding: invoke `Record`, and answer each failure with the policy its refusal selects. */
  notify(placed) {
    const orderId = placed.payload.order_id;
    const bound = mode === 'extra-retry' ? 4 : 3;
    for (let attempts = 1; attempts <= 64; attempts += 1) {
      if (mode !== 'omits-attempt' || attempts === 1) {
        this.invocations.push({ command: RECORD, input: { order_id: orderId } });
      }
      const answer = this.answer();
      if (answer === 'recorded') {
        this.log.push({ event: RECORDED, payload: { order_id: orderId } });
        return;
      }
      const policy = this.policy(answer);
      if (policy === 'drop') {
        return;
      }
      if (policy === 'escalate') {
        this.log.push({ event: ESCALATED, payload: { order_id: orderId } });
        if (mode !== 'duplicate-escalation') {
          return;
        }
        continue;
      }
      if (answer === 'rejected' && mode !== 'retries-final') {
        return;
      }
      if (attempts >= bound) {
        return;
      }
    }
  }
  executeCommand({ command, input }) {
    const orderId = input.order_id ?? null;
    if (command === PLACE) {
      const placed = { event: 'demo.ledger.OrderPlaced', payload: { order_id: orderId } };
      this.log.push(placed);
      this.notify(placed);
      return { outcome: 'placed', directEvents: [placed] };
    }
    if (command === RECORD) {
      const outcome = this.answer();
      if (outcome === 'recorded') {
        const event = { event: RECORDED, payload: { order_id: orderId } };
        this.log.push(event);
        return { outcome, directEvents: [event] };
      }
      return { outcome, directEvents: [], error: ERRORS[outcome] };
    }
    throw new Error(`unexpected command ${command}`);
  }
  queryView() {
    throw new Error('the model declares no view');
  }
  observeEvents({ event }) {
    return this.log.filter((held) => held.event === event);
  }
  configureExternalOutcome({ outcome }) {
    this.forced = { outcome, remaining: 1 };
  }
  configureExternalOutcomeRepeatedly({ outcome, times }) {
    this.forced = { outcome, remaining: times };
  }
  redeliverEvent({ event }) {
    const placed = [...this.log].reverse().find((held) => held.event === event);
    if (placed === undefined) {
      throw new Error('the event was not published');
    }
    this.notify(placed);
  }
  observeInvocations({ binding, command }) {
    return binding === BINDING
      ? this.invocations.filter((invocation) => invocation.command === command)
      : [];
  }
}

export function makeTarget() {
  return new Ledger();
}
