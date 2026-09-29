// An in-memory implementation of `explore-adv-order.yaml` (adversary pass 1, beyond10x/ess#235),
// deciding each command in Entity Runtime's order. `explore_adv_order_target.go` is the same target
// in Go, and its header says what the order and the modes are.

import { unsupported } from './dist/index.js';

export function newTarget(mode = '') {
  let states = new Map();
  let created = 0;
  let version = 0;
  let forced = new Map();

  const reset = () => {
    states = new Map();
    created = 0;
    version = 0;
    forced = new Map();
  };
  const refused = (outcome, error) => ({ outcome, error, directEvents: [] });

  return {
    identity: () => ({ name: 'explore-adv-order-target', version: '1' }),
    beginScenario: () => reset(),
    endScenario: () => reset(),

    executeCommand({ command, input }) {
      switch (command) {
        case 'exploreadv.desk.Open': {
          created += 1;
          const id = `00000000-0000-4000-8000-${String(created).padStart(12, '0')}`;
          states.set(id, 'Open');
          version += 1;
          return {
            outcome: 'opened',
            consistency: `v${version}`,
            directEvents: [{ event: 'exploreadv.desk.Opened', payload: { ticket_id: id } }],
          };
        }
        case 'exploreadv.desk.Close': {
          if (mode === 'no-close') throw unsupported('Close is not exposed in mode no-close');
          const id = input.ticket_id;
          const state = states.get(id);
          if (state === undefined) throw new Error(`no ticket ${id}`);
          if (forced.get(command) === 'bounced') return refused('bounced', 'exploreadv.desk.Bounced');
          if (state !== 'Open') return refused('wrong-state', 'exploreadv.desk.StateConflict');
          states.set(id, 'Closed');
          version += 1;
          return {
            outcome: 'closed',
            consistency: `v${version}`,
            directEvents: [{ event: 'exploreadv.desk.Closed', payload: { ticket_id: id } }],
          };
        }
        case 'exploreadv.desk.Rate': {
          if (mode === 'no-rate') throw unsupported('Rate is not exposed in mode no-rate');
          if (input.score > 50) return refused('too-high', 'exploreadv.desk.TooHigh');
          if (forced.get(command) === 'throttled') {
            return refused('throttled', 'exploreadv.desk.Throttled');
          }
          version += 1;
          return {
            outcome: 'rated',
            consistency: `v${version}`,
            directEvents: [{ event: 'exploreadv.desk.Rated', payload: { score: input.score } }],
          };
        }
        case 'exploreadv.desk.Shut':
        case 'exploreadv.desk.Resolve': {
          // explore-adv-overlap.yaml: see the Go target.
          const id = input.ticket_id;
          const state = states.get(id);
          if (command === 'exploreadv.desk.Resolve' && input.level < 0) {
            return refused('unrated', 'exploreadv.desk.Unrated');
          }
          if (state === undefined) throw new Error(`no ticket ${id}`);
          if (state !== 'Open') return refused('wrong-state', 'exploreadv.desk.StateConflict');
          states.set(id, 'Closed');
          version += 1;
          const resolve = command === 'exploreadv.desk.Resolve';
          return {
            outcome: resolve ? 'quick' : 'shut',
            consistency: `v${version}`,
            directEvents: [
              { event: resolve ? 'exploreadv.desk.Resolved' : 'exploreadv.desk.ShutDown', payload: { ticket_id: id } },
            ],
          };
        }
        default:
          throw unsupported(`${command} is not a command of explore-adv-order.yaml`);
      }
    },

    queryView({ view }) {
      throw unsupported(`${view} is not a view of explore-adv-order.yaml`);
    },
    observeEvents: () => [],
    configureExternalOutcome({ command, outcome }) {
      forced.set(command, outcome);
    },
    redeliverEvent: () => {
      throw unsupported('no redelivery');
    },
    observeInvocations: () => {
      throw unsupported('no bindings');
    },
  };
}
