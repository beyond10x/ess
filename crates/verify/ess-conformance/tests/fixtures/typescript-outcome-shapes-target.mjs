// The ess/15 outcome-shapes model (`crates/specify/ess-compiler/tests/fixtures/outcome-shapes.yaml`),
// the target `tests/outcome_shapes.rs` implements in Rust. Modes: `correct`,
// `folds-unknown-into-wrong-state` (#145), `keeps-ended-rows` (#151), `offers-into-initial` (#150),
// `touch-writes` (#144).
import { unsupported } from './dist/runtime.js';

const mode = process.env.ESS_TARGET_MODE ?? 'correct';

class Calls {
  users = new Map();
  rows = new Map();
  identity() {
    return { name: 'outcome-shapes', version: '1' };
  }
  beginScenario() {}
  endScenario() {}
  executeCommand({ command, input }) {
    const id = (field) => (typeof input[field] === 'string' ? input[field] : '');
    const took = (outcome, event, field) => {
      const result = { outcome, consistency: 'write', directEvents: [] };
      if (event !== undefined) {
        result.directEvents.push({ event, payload: { [field]: id(field) } });
      }
      return result;
    };
    const notFound = () => ({ ...took('no-such-call'), error: 'example.call.CallNotFound' });
    // #152: nothing but opening a session is taken outside an open session.
    if (command !== 'example.call.OpenSession' && this.users.size === 0) {
      return {};
    }
    switch (command) {
      case 'example.call.OpenSession':
        this.users.set(id('user_id'), 'Active');
        return took('opened', 'example.call.SessionOpened', 'user_id');
      case 'example.call.PlaceCall':
        this.rows.set(id('call_id'), 'Dialing');
        return took('placed', 'example.call.CallPlaced', 'call_id');
      case 'example.call.OfferCall':
        this.rows.set(id('call_id'), mode === 'offers-into-initial' ? 'Dialing' : 'Ringing');
        return took('offered', 'example.call.CallOffered', 'call_id');
      case 'example.call.RingCall':
        if (this.rows.get(id('call_id')) === 'Dialing') {
          this.rows.set(id('call_id'), 'Ringing');
          return took('rang', 'example.call.CallRang', 'call_id');
        }
        return took('not-dialing');
      case 'example.call.AnswerCall': {
        const state = this.rows.get(id('call_id'));
        if (state === undefined) {
          return mode === 'folds-unknown-into-wrong-state' ? took('not-ringing') : notFound();
        }
        if (state === 'Ringing') {
          this.rows.set(id('call_id'), 'Connected');
          return took('answered', 'example.call.CallAnswered', 'call_id');
        }
        return took('not-ringing');
      }
      case 'example.call.EndCall':
        if (this.rows.has(id('call_id'))) {
          if (mode !== 'keeps-ended-rows') {
            this.rows.delete(id('call_id'));
          }
          return took('ended', 'example.call.CallEnded', 'call_id');
        }
        return notFound();
      case 'example.call.Touch':
        if (mode === 'touch-writes') {
          this.rows.set('00000000-0000-4000-8000-000000000144', 'Dialing');
        }
        return took('accepted');
      default:
        throw new Error(`unexpected command ${command}`);
    }
  }
  queryView({ view }) {
    const [rows, identity] =
      view === 'example.call.Calls'
        ? [this.rows, 'call_id']
        : view === 'example.call.Users'
          ? [this.users, 'user_id']
          : [undefined, ''];
    if (rows === undefined) {
      throw new Error(`unexpected view ${view}`);
    }
    // In identity order, as the Rust target's map yields them.
    return {
      rows: [...rows.entries()]
        .sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0))
        .map(([key, state]) => ({ [identity]: key, state })),
    };
  }
  configureExternalOutcome() {
    throw unsupported('the model declares no external outcome');
  }
  redeliverEvent() {
    throw unsupported('unused');
  }
  observeEvents() {
    throw unsupported('unused');
  }
  observeInvocations() {
    throw unsupported('unused');
  }
}

export function makeTarget() {
  return new Calls();
}
