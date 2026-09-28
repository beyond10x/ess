// The #165 ledger (`crates/specify/ess-compiler/tests/fixtures/bounded-retry.yaml`), the target
// `tests/bounded_retry.rs` implements in Rust: a binding that retries `Record` at most three times
// and stops at once on the final refusal. Modes: `correct`, `unbounded` (ten attempts),
// `too-few` (two), `retries-final`, `cannot-repeat` (no repeated forcing).
import { unsupported } from './dist/runtime.js';

const mode = process.env.ESS_TARGET_MODE ?? 'correct';
const BINDING = 'notify-ledger';
const RECORD = 'demo.ledger.Record';

class Ledger {
  log = [];
  invocations = [];
  forced = undefined;
  identity() {
    return { name: 'bounded-retry', version: '1' };
  }
  beginScenario() {}
  endScenario() {}
  /** One answer of `Record`, honouring a forced outcome. */
  record(orderId) {
    let outcome = 'recorded';
    if (this.forced !== undefined && this.forced.remaining > 0) {
      this.forced.remaining -= 1;
      outcome = this.forced.outcome;
    }
    const result = { outcome, directEvents: [] };
    if (outcome === 'recorded') {
      const event = { event: 'demo.ledger.Recorded', payload: { order_id: orderId } };
      this.log.push(event);
      result.directEvents.push(event);
    } else {
      result.error = outcome === 'unavailable' ? 'demo.ledger.Unavailable' : 'demo.ledger.Unknown';
    }
    return [outcome, result];
  }
  /** The binding: invoke `Record` until recorded, a final refusal, or the bound is spent. */
  notify(placed) {
    const orderId = placed.payload.order_id;
    const bound = mode === 'unbounded' ? 10 : mode === 'too-few' ? 2 : 3;
    for (let attempt = 0; attempt < bound; attempt += 1) {
      this.invocations.push({ command: RECORD, input: { order_id: orderId } });
      const [outcome] = this.record(orderId);
      if (outcome === 'recorded' || (outcome === 'rejected' && mode !== 'retries-final')) {
        return;
      }
    }
  }
  executeCommand({ command, input }) {
    const orderId = input.order_id ?? null;
    if (command === 'demo.ledger.Place') {
      const placed = { event: 'demo.ledger.OrderPlaced', payload: { order_id: orderId } };
      this.log.push(placed);
      this.notify(placed);
      return { outcome: 'placed', directEvents: [placed] };
    }
    if (command === RECORD) {
      return this.record(orderId)[1];
    }
    throw new Error(`unexpected command ${command}`);
  }
  queryView() {
    throw unsupported('the model declares no view');
  }
  observeEvents({ event }) {
    return this.log.filter((held) => held.event === event);
  }
  configureExternalOutcome({ outcome }) {
    this.forced = { outcome, remaining: 1 };
  }
  configureExternalOutcomeRepeatedly({ outcome, times }) {
    if (mode === 'cannot-repeat') {
      throw unsupported('this adapter forces the next answer only');
    }
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
