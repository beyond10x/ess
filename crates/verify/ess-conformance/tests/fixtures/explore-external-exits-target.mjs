// An in-memory implementation of `explore-external-exits.yaml` for the second adversary pass of
// beyond10x/ess#156. An arrangement holds for its command until the scenario ends. `mode` is empty
// (arranges every external branch) or `cannot-arrange`. `explore_external_exits_target.go` is the
// same target in Go.

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
    identity: () => ({ name: 'explore-external-exits-target', version: '1' }),
    beginScenario: () => reset(),
    endScenario: () => reset(),

    executeCommand({ command, input }) {
      switch (command) {
        case 'exploreexit.desk.Open': {
          created += 1;
          const id = `00000000-0000-4000-8000-${String(created).padStart(12, '0')}`;
          states.set(id, 'Open');
          version += 1;
          return {
            outcome: 'opened',
            consistency: `v${version}`,
            directEvents: [{ event: 'exploreexit.desk.Opened', payload: { ticket_id: id } }],
          };
        }
        case 'exploreexit.desk.Close': {
          const id = input.ticket_id;
          if (forced.get(command) === 'bounced') return refused('bounced', 'exploreexit.desk.Bounced');
          if (states.get(id) === 'Open') {
            states.set(id, 'Closed');
            version += 1;
            return {
              outcome: 'closed',
              consistency: `v${version}`,
              directEvents: [{ event: 'exploreexit.desk.Closed', payload: { ticket_id: id } }],
            };
          }
          return refused('undeclared-state', 'exploreexit.desk.Bounced');
        }
        case 'exploreexit.desk.Rate': {
          if (forced.get(command) === 'throttled') {
            return refused('throttled', 'exploreexit.desk.Throttled');
          }
          if (input.score <= 3) return refused('low', 'exploreexit.desk.Low');
          return refused('high', 'exploreexit.desk.High');
        }
        default:
          throw unsupported(`${command} is not a command of explore-external-exits.yaml`);
      }
    },

    queryView({ view }) {
      throw unsupported(`${view} is not a view of explore-external-exits.yaml`);
    },

    observeEvents: () => [],
    configureExternalOutcome({ command, outcome }) {
      if (mode === 'cannot-arrange') throw unsupported('this double arranges no external outcome');
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
