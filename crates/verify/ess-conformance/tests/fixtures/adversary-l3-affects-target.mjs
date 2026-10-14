// An in-memory implementation of `adversary-l3-affects.yaml`, as the interpreter runs it: `Note`
// writes its seat's reason and that of every other seat of the same team.
//
// `adversary_l3_affects_target.go` is the same target in Go.

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
    identity: () => ({ name: 'adversary-l3-affects-target', version: '1' }),
    beginScenario: () => reset(),
    endScenario: () => reset(),
    executeCommand({ command, input }) {
      switch (command) {
        case 'advaff.desk.Start': {
          created += 1;
          const id = `00000000-0000-4000-8000-${String(created).padStart(12, '0')}`;
          seats.set(id, { team: input.team, last_reason: 'none' });
          return answered('started', [{ event: 'advaff.desk.Started', payload: { seat_id: id } }]);
        }
        case 'advaff.desk.Note': {
          const seat = seats.get(input.seat_id);
          if (seat === undefined) throw new Error(`no seat ${input.seat_id}`);
          for (const other of seats.values()) {
            if (other.team === seat.team) other.last_reason = input.reason;
          }
          return answered('noted', [{ event: 'advaff.desk.Noted', payload: { seat_id: input.seat_id } }]);
        }
        default:
          throw unsupported(`${command} is not a command of adversary-l3-affects.yaml`);
      }
    },
    queryView({ view }) {
      if (view !== 'advaff.desk.Seats') throw unsupported(`${view} is not a view`);
      return { rows: [...seats].map(([id, seat]) => ({ seat_id: id, last_reason: seat.last_reason })) };
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
