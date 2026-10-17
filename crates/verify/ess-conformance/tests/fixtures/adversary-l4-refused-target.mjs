// Adversary pass 1 on beyond10x/ess#537: an in-memory implementation of `explore-refused.yaml`
// whose modes refuse a command in some held states and drive it in the others.
//
// `mode` selects the double:
//
//   (empty)                         the specification
//   hold-waiting                    `Hold` is unsupported while the call waits, driven elsewhere
//   ringing-once                    `Answer` is unsupported the first time a ringing call is
//                                   answered; a second send in `Ringing` is a plain error
//   ringing-connected               `Answer` is unsupported while ringing and once connected
//   hold-everywhere-answer-ringing  `Hold` is unsupported everywhere, `Answer` while ringing
//
// Unlike `explore-refused-target.mjs` it serves the view `RingingCalls`.
// `adversary_l4_refused_target.go` is the same target in Go, line for line.

import { unsupported } from './dist/index.js';

export function newTarget(mode = '') {
  let calls = new Map();
  let created = 0;
  let version = 0;
  // The `Answer` steps sent for a ringing call to this target, which serves one explored sequence.
  let sent = 0;

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
    identity: () => ({ name: 'adversary-l4-refused-target', version: '1' }),
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
          if (mode === 'hold-waiting' && call.state === 'Waiting') {
            throw unsupported('Hold has no callback while the call waits');
          }
          if (mode === 'hold-everywhere-answer-ringing') throw unsupported('Hold has no callback');
          if (call.state !== 'Ringing') return refused('not-ringing', 'explorerefused.desk.NotRinging');
          call.state = 'Waiting';
          return answered('held', 'explorerefused.desk.Held', input.call_id);
        }
        case 'explorerefused.desk.Answer': {
          if (call === undefined) throw new Error(`no call ${input.call_id}`);
          if (call.state === 'Ringing') {
            if (mode === 'ringing-once') {
              sent += 1;
              const count = sent;
              if (count === 1) throw unsupported('Answer has no callback while the call rings');
              throw new Error(
                `Answer was sent again in Ringing after the target refused it there (send ${count})`,
              );
            }
            if (mode === 'ringing-connected' || mode === 'hold-everywhere-answer-ringing') {
              throw unsupported('Answer has no callback while the call rings');
            }
          }
          if (mode === 'ringing-connected' && call.state === 'Connected') {
            throw unsupported('Answer has no callback once connected');
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
        case 'explorerefused.desk.RingingCalls':
          return {
            rows: [...calls]
              .filter(([, call]) => call.state === 'Ringing')
              .map(([id]) => ({ call_id: id })),
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
