// An in-memory implementation of `explore-selected-rows-unknown.yaml`, as the interpreter runs it:
// no outcome writes `zone` or `tag`, so `SweepTagged` closes no seat, and `Sweep` is answered only
// where there is no seat to read (the model redraws it beside one).
//
// `explore_selected_rows_unknown_target.go` is the same target in Go.

import { unsupported } from './dist/index.js';

export function newTarget() {
  let seats = new Map();
  let created = 0;
  let version = 0;
  const reset = () => {
    seats = new Map();
    created = 0;
    version = 0;
  };
  const answered = (outcome, directEvents) => {
    version += 1;
    return { outcome, consistency: `v${version}`, directEvents };
  };
  return {
    identity: () => ({ name: 'explore-selected-rows-unknown-target', version: '1' }),
    beginScenario: () => reset(),
    endScenario: () => reset(),
    executeCommand({ command, input }) {
      switch (command) {
        case 'exploreunk.desk.Start': {
          created += 1;
          const id = `00000000-0000-4000-8000-${String(created).padStart(12, '0')}`;
          seats.set(id, { team: input.team, state: 'Open' });
          return answered('started', [{ event: 'exploreunk.desk.Started', payload: { seat_id: id } }]);
        }
        case 'exploreunk.desk.Sweep': {
          if (seats.size > 0) throw new Error('sweep reads a zone no outcome wrote');
          return answered('swept', [{ event: 'exploreunk.desk.Swept', payload: { team: input.team } }]);
        }
        case 'exploreunk.desk.SweepTagged':
          return answered('swept-tagged', [
            { event: 'exploreunk.desk.Swept', payload: { team: input.team } },
          ]);
        default:
          throw unsupported(`${command} is not a command of explore-selected-rows-unknown.yaml`);
      }
    },
    queryView({ view }) {
      if (view !== 'exploreunk.desk.Seats') throw unsupported(`${view} is not a view`);
      return { rows: [...seats].map(([id, seat]) => ({ seat_id: id, state: seat.state })) };
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
