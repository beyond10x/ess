// An in-memory implementation of `explore-selected-rows.yaml` (beyond10x/ess#538).
//
// `mode` selects the double:
//
//   (empty)           the specification
//   close-skips-one   `Close` moves every selected seat but leaves the first one's `last_reason`
//   close-every-team  `Close` moves and sets every `Open` seat, whatever its team
//   dial-adds-one     `Dial` stores one more than the level it was sent
//
// `explore_selected_rows_target.go` is the same target in Go, line for line.

import { unsupported } from './dist/index.js';

export function newTarget(mode = '') {
  let seats = new Map();
  let created = 0;
  let version = 0;

  const reset = () => {
    seats = new Map();
    created = 0;
    version = 0;
  };
  const refused = (outcome, error) => ({ outcome, error, directEvents: [] });
  const answered = (outcome, directEvents) => {
    version += 1;
    return { outcome, consistency: `v${version}`, directEvents };
  };

  return {
    identity: () => ({ name: 'explore-selected-rows-target', version: '1' }),
    beginScenario: () => reset(),
    endScenario: () => reset(),

    executeCommand({ command, input }) {
      const seat = seats.get(input.seat_id);
      switch (command) {
        case 'explorerows.desk.Start': {
          created += 1;
          const id = `00000000-0000-4000-8000-${String(created).padStart(12, '0')}`;
          seats.set(id, { state: 'Open', team: input.team, last_reason: 'none', level: input.level });
          return answered('started', [{ event: 'explorerows.desk.Started', payload: { seat_id: id } }]);
        }
        case 'explorerows.desk.Dial': {
          if (seat === undefined) throw new Error(`no seat ${input.seat_id}`);
          seat.level = mode === 'dial-adds-one' ? input.dial_level + 1 : input.dial_level;
          return answered('dialled', [
            { event: 'explorerows.desk.Dialled', payload: { seat_id: input.seat_id, level: input.dial_level } },
          ]);
        }
        case 'explorerows.desk.Note': {
          if (seat === undefined) throw new Error(`no seat ${input.seat_id}`);
          seat.last_reason = input.reason;
          return answered('noted', [{ event: 'explorerows.desk.Noted', payload: { seat_id: input.seat_id } }]);
        }
        case 'explorerows.desk.Close': {
          let closed = 0;
          for (const each of seats.values()) {
            const selected = each.team === input.team || mode === 'close-every-team';
            if (!selected || each.state !== 'Open') continue;
            each.state = 'Closed';
            if (!(mode === 'close-skips-one' && closed === 0)) each.last_reason = 'hungup';
            closed += 1;
          }
          return answered('closed', [
            { event: 'explorerows.desk.TeamClosed', payload: { team: input.team, closed } },
          ]);
        }
        case 'explorerows.desk.Reopen': {
          if (seat === undefined) throw new Error(`no seat ${input.seat_id}`);
          if (seat.state !== 'Closed') return refused('not-closed', 'explorerows.desk.NotClosed');
          seat.state = 'Open';
          return answered('reopened', [
            { event: 'explorerows.desk.Reopened', payload: { seat_id: input.seat_id } },
          ]);
        }
        default:
          throw unsupported(`${command} is not a command of explore-selected-rows.yaml`);
      }
    },

    queryView({ view }) {
      if (view !== 'explorerows.desk.Seats') {
        throw unsupported(`${view} is not a view of explore-selected-rows.yaml`);
      }
      return {
        rows: [...seats].map(([id, seat]) => ({
          seat_id: id,
          state: seat.state,
          team: seat.team,
          last_reason: seat.last_reason,
          level: seat.level,
        })),
      };
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
