// The retained-replay model (`tests/fixtures/retained-replay.yaml`), the backend
// `tests/retained_replay.rs` implements in Rust: the first `Seed` of a scenario seeds, every later
// one replays the retained result. Modes: `correct`, `next-result`, `new-stamp`, `null-optional`,
// `extra-field`, `reordered-list`, `error`, `missing-response`, `extra-event`,
// `ambiguous-original-event`, `mutated-subject`.
import { JsonNumber, unsupported } from './dist/runtime.js';

const mode = process.env.ESS_TARGET_MODE ?? 'correct';
const RECORD = '00000000-0000-4000-8000-000000000037';

function response() {
  return {
    revision_id: RECORD,
    stamp: '2026-09-22T01:02:03Z',
    number: new JsonNumber('9007199254740993'),
    values: [new JsonNumber('-9223372036854775808'), new JsonNumber('9223372036854775807')],
  };
}

class Backend {
  calls = 0;
  document = 'actual document';
  identity() {
    return { name: 'retained-fixture', version: '1' };
  }
  beginScenario() {}
  endScenario() {}
  executeCommand({ command, input }) {
    const retry = this.calls > 0;
    if (!retry && typeof input.document === 'string') {
      this.document = input.document;
    }
    this.calls += 1;
    const result = {
      outcome: retry ? 'replayed' : 'seeded',
      consistency: 'actual-write',
      directEvents: [],
    };
    const answer = response();
    if (retry) {
      switch (mode) {
        case 'next-result':
          answer.number = new JsonNumber('9007199254740992');
          break;
        case 'new-stamp':
          answer.stamp = '2026-09-22T01:02:04Z';
          break;
        case 'null-optional':
          answer.optional = null;
          break;
        case 'extra-field':
          answer.extra = null;
          break;
        case 'reordered-list':
          answer.values = [
            new JsonNumber('9223372036854775807'),
            new JsonNumber('-9223372036854775808'),
          ];
          break;
        case 'error':
          result.error = 'retained.core.Failed';
          break;
        default:
          break;
      }
    }
    if (!retry || mode !== 'missing-response') {
      result.response = answer;
    }
    if (!retry || mode === 'extra-event') {
      const event = {
        event: retry ? 'retained.core.Undeclared' : 'retained.core.Seeded',
        payload: { record_id: RECORD },
      };
      result.directEvents.push(event);
      if (!retry && mode === 'ambiguous-original-event') {
        result.directEvents.push({ ...event });
      }
    }
    if (command !== 'retained.core.Seed') {
      throw new Error(`unexpected command ${command}`);
    }
    return result;
  }
  queryView() {
    const row = {
      record_id: RECORD,
      value: this.document,
      stamp: '2026-09-22T01:02:03Z',
      state: 'Committed',
    };
    if (this.calls > 1 && mode === 'mutated-subject') {
      row.stamp = '2026-09-22T01:02:04Z';
    }
    return { rows: [row] };
  }
  configureExternalOutcome() {
    throw new Error('replay must never use fault injection');
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
  return new Backend();
}
