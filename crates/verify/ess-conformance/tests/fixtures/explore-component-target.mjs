// An in-memory implementation of the `front` component of `explore-component.yaml`. It is copied
// into the package as `explore-external-target.mjs`, which `explore-external-driver.mjs` runs.
// `WriteNote` belongs to `back`, so it is answered as unsupported.
//
// `explore_component_target.go` is the same target in Go, line for line.

import { unsupported } from './dist/index.js';

export function newTarget() {
  let states = new Map();
  let created = 0;
  let version = 0;

  const reset = () => {
    states = new Map();
    created = 0;
    version = 0;
  };
  const answered = (outcome, event, payload) => {
    version += 1;
    return { outcome, consistency: `v${version}`, directEvents: [{ event, payload }] };
  };

  return {
    identity: () => ({ name: 'explore-component-target', version: '1' }),
    beginScenario: () => reset(),
    endScenario: () => reset(),

    executeCommand({ command, input }) {
      switch (command) {
        case 'explorecmp.desk.OpenTicket': {
          created += 1;
          const id = `00000000-0000-4000-8000-${String(created).padStart(12, '0')}`;
          states.set(id, 'Open');
          return answered('opened', 'explorecmp.desk.Opened', { ticket_id: id });
        }
        case 'explorecmp.desk.CloseTicket': {
          const state = states.get(input.ticket_id);
          if (state === undefined) throw new Error(`no ticket ${input.ticket_id}`);
          if (state !== 'Open') {
            return { outcome: 'not-open', error: 'explorecmp.desk.NotOpen', directEvents: [] };
          }
          states.set(input.ticket_id, 'Closed');
          return answered('closed', 'explorecmp.desk.Closed', { ticket_id: input.ticket_id });
        }
        default:
          throw unsupported(`${command} is not a command of the front component`);
      }
    },

    queryView({ view }) {
      throw unsupported(`${view} is not a view of the front component`);
    },

    observeEvents: () => [],
    configureExternalOutcome: () => {
      throw unsupported('no external branches');
    },
    redeliverEvent: () => {
      throw unsupported('no redelivery');
    },
    observeInvocations: () => {
      throw unsupported('no bindings');
    },
  };
}
