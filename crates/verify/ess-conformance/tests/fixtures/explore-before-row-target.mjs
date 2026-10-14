// An in-memory implementation of `explore-before-row.yaml`, a test double that arranges external
// outcomes the way the synthesized scenarios ask it to. It is copied into the package as
// `explore-external-target.mjs`, which `explore-external-driver.mjs` runs.
//
// An arrangement holds for its command until the scenario ends. `mode` selects the double:
//
//   (empty)    asks the arranged credential check before it reads the seat, as the model says
//   row-first  reads the seat first, and asks the credential check only of a seat it holds
//
// `explore_before_row_target.go` is the same target in Go, line for line.

import { unsupported } from './dist/index.js';

export function newTarget(mode = '') {
  let seats = new Map();
  let created = 0;
  let version = 0;
  let forced = new Map();

  const reset = () => {
    seats = new Map();
    created = 0;
    version = 0;
    forced = new Map();
  };
  const answered = (outcome, event, payload) => {
    version += 1;
    return { outcome, consistency: `v${version}`, directEvents: [{ event, payload }] };
  };
  const refused = (outcome, error) => ({ outcome, error, directEvents: [] });
  const arranged = (command, outcome) => forced.get(command) === outcome;

  return {
    identity: () => ({ name: 'explore-before-row-target', version: '1' }),
    beginScenario: () => reset(),
    endScenario: () => reset(),

    executeCommand({ command, input }) {
      switch (command) {
        case 'exploreext.desk.OpenSeat': {
          created += 1;
          const id = `00000000-0000-4000-8000-${String(created).padStart(12, '0')}`;
          seats.set(id, { seat_id: id, label: input.label, state: 'Open' });
          return answered('opened', 'exploreext.desk.SeatOpened', { seat_id: id });
        }
        case 'exploreext.desk.SignIn': {
          if (mode !== 'row-first' && arranged(command, 'bad-credential')) {
            return refused('bad-credential', 'exploreext.desk.BadCredential');
          }
          const seat = seats.get(input.seat_id);
          if (seat === undefined) return refused('no-seat', 'exploreext.desk.NoSeat');
          if (arranged(command, 'bad-credential')) {
            return refused('bad-credential', 'exploreext.desk.BadCredential');
          }
          if (seat.state === 'Open') {
            seat.state = 'Busy';
            return answered('taken', 'exploreext.desk.Taken', { seat_id: seat.seat_id });
          }
          seat.state = 'Open';
          return answered('released', 'exploreext.desk.Released', { seat_id: seat.seat_id });
        }
        default:
          throw unsupported(`${command} is not a command of explore-before-row.yaml`);
      }
    },

    queryView({ view }) {
      if (view === 'exploreext.desk.Seats') {
        return { rows: [...seats.values()].map((seat) => ({ ...seat })) };
      }
      throw unsupported(`${view} is not a view of explore-before-row.yaml`);
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
