// An in-memory implementation of `explore-external-adversary.yaml` for the adversary lane of
// beyond10x/ess#156. An arrangement holds for its command until the scenario ends.
// `explore_external_adversary_target.go` is the same target in Go.

import { unsupported } from './dist/index.js';

export function newTarget() {
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
  const answered = (outcome, event, payload) => {
    version += 1;
    return { outcome, consistency: `v${version}`, directEvents: [{ event, payload }] };
  };
  const conflict = () => ({
    outcome: 'wrong-state',
    error: 'exploreadv.desk.TicketStateConflict',
    directEvents: [],
  });

  return {
    identity: () => ({ name: 'explore-external-adversary-target', version: '1' }),
    beginScenario: () => reset(),
    endScenario: () => reset(),

    executeCommand({ command, input }) {
      switch (command) {
        case 'exploreadv.desk.Open': {
          created += 1;
          const id = `00000000-0000-4000-8000-${String(created).padStart(12, '0')}`;
          states.set(id, 'Open');
          return answered('opened', 'exploreadv.desk.Opened', { ticket_id: id });
        }
        case 'exploreadv.desk.Hold': {
          const id = input.ticket_id;
          if (states.get(id) !== 'Open') return conflict();
          states.set(id, 'Held');
          return answered('held', 'exploreadv.desk.Held', { ticket_id: id });
        }
        case 'exploreadv.desk.Close': {
          const id = input.ticket_id;
          const state = states.get(id);
          if (state === 'Open') {
            states.set(id, 'Closed');
            return answered('closed', 'exploreadv.desk.Closed', { ticket_id: id });
          }
          if (state === 'Held' && forced.get(command) === 'overridden') {
            states.set(id, 'Closed');
            return answered('overridden', 'exploreadv.desk.Overridden', { ticket_id: id });
          }
          return conflict();
        }
        default:
          throw unsupported(`${command} is not a command of explore-external-adversary.yaml`);
      }
    },

    queryView({ view }) {
      throw unsupported(`${view} is not a view of explore-external-adversary.yaml`);
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
