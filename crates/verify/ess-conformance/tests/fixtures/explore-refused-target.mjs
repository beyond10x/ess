// An in-memory implementation of `explore-refused.yaml` (beyond10x/ess#537).
//
// `mode` selects the double:
//
//   (empty)   the specification
//   ringing   `Answer` is unsupported while the call rings, and driven in every other state
//   answer    `Answer` is unsupported in every state
//
// Every mode refuses the view `RingingCalls`. `explore_refused_target.go` is the same target in Go,
// line for line.

import { unsupported } from './dist/index.js';

export function newTarget(mode = '') {
  let calls = new Map();
  let created = 0;
  let version = 0;

  const reset = () => {
    calls = new Map();
    created = 0;
    version = 0;
  };
  const answered = (outcome, event, id) => {
    version += 1;
    return {
      outcome,
      consistency: `v${version}`,
      directEvents: [{ event, payload: { call_id: id } }],
    };
  };
  const refused = (outcome, error) => ({ outcome, error, directEvents: [] });

  return {
    identity: () => ({ name: 'explore-refused-target', version: '1' }),
    beginScenario: () => reset(),
    endScenario: () => reset(),

    executeCommand({ command, input }) {
      const call = calls.get(input.call_id);
      switch (command) {
        case 'explorerefused.desk.Dial': {
          created += 1;
          const id = `00000000-0000-4000-8000-${String(created).padStart(12, '0')}`;
          calls.set(id, { state: 'Ringing', line: input.line });
          return answered('dialled', 'explorerefused.desk.Dialled', id);
        }
        case 'explorerefused.desk.Hold': {
          if (call === undefined) throw new Error(`no call ${input.call_id}`);
          if (call.state !== 'Ringing') return refused('not-ringing', 'explorerefused.desk.NotRinging');
          call.state = 'Waiting';
          return answered('held', 'explorerefused.desk.Held', input.call_id);
        }
        case 'explorerefused.desk.Answer': {
          if (call === undefined) throw new Error(`no call ${input.call_id}`);
          if (mode === 'answer') throw unsupported('Answer has no callback');
          if (mode === 'ringing' && call.state === 'Ringing') {
            throw unsupported('Answer has no callback while the call rings');
          }
          switch (call.state) {
            case 'Ringing':
              call.state = 'Connected';
              return answered('answered', 'explorerefused.desk.Connected', input.call_id);
            case 'Waiting':
              call.state = 'Connected';
              return answered('picked-up', 'explorerefused.desk.Connected', input.call_id);
            default:
              return refused('not-open', 'explorerefused.desk.NotOpen');
          }
        }
        default:
          throw unsupported(`${command} is not a command of explore-refused.yaml`);
      }
    },

    queryView({ view }) {
      switch (view) {
        case 'explorerefused.desk.Calls':
          return {
            rows: [...calls].map(([id, call]) => ({ call_id: id, state: call.state, line: call.line })),
          };
        default:
          throw unsupported(`${view} is not served`);
      }
    },

    observeEvents: () => [],
    configureExternalOutcome: () => {
      throw unsupported('no external outcomes');
    },
    redeliverEvent: () => {
      throw unsupported('no redelivery');
    },
    observeInvocations: () => {
      throw unsupported('no bindings');
    },
  };
}
