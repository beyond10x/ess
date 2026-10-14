// An in-memory implementation of `adversary-l3-selected-rows.yaml`, as the interpreter runs it.
//
// `CloseTagged` closes every `Open` seat whose tag is the team sent; an untagged seat is not
// selected. `Finish` finishes every `Active` desk whose zone is the zone sent; an `Idle` desk,
// which holds no zone, is outside the move and never read.
//
// Mode `closes-untagged` also closes every untagged `Open` seat: a defect the explorer can see only
// by drawing `CloseTagged` while an untagged seat exists.
//
// `adversary_l3_selected_rows_target.go` is the same target in Go.

import { unsupported } from './dist/index.js';

export function newTarget(mode = '') {
  let seats = new Map();
  let desks = new Map();
  let created = 0;
  let version = 0;

  const reset = () => {
    seats = new Map();
    desks = new Map();
    created = 0;
    version = 0;
  };
  const answered = (outcome, directEvents) => {
    version += 1;
    return { outcome, consistency: `v${version}`, directEvents };
  };
  const mint = () => {
    created += 1;
    return `00000000-0000-4000-8000-${String(created).padStart(12, '0')}`;
  };

  return {
    identity: () => ({ name: 'adversary-l3-selected-rows-target', version: '1' }),
    beginScenario: () => reset(),
    endScenario: () => reset(),

    executeCommand({ command, input }) {
      switch (command) {
        case 'advrows.desk.Start': {
          const id = mint();
          seats.set(id, { state: 'Open', team: input.team, tag: null, last_reason: 'none' });
          return answered('started', [{ event: 'advrows.desk.Started', payload: { seat_id: id } }]);
        }
        case 'advrows.desk.Tag': {
          const seat = seats.get(input.seat_id);
          if (seat === undefined) throw new Error(`no seat ${input.seat_id}`);
          seat.tag = input.tag;
          return answered('tagged', [{ event: 'advrows.desk.Tagged', payload: { seat_id: input.seat_id } }]);
        }
        case 'advrows.desk.CloseTagged': {
          for (const seat of seats.values()) {
            const selected = seat.tag === input.team || (mode === 'closes-untagged' && seat.tag === null);
            if (!selected || seat.state !== 'Open') continue;
            seat.state = 'Closed';
            seat.last_reason = 'hungup';
          }
          return answered('closed', [{ event: 'advrows.desk.TagClosed', payload: { team: input.team } }]);
        }
        case 'advrows.desk.OpenDesk': {
          const id = mint();
          desks.set(id, { state: 'Idle', owner: input.owner, zone: null });
          return answered('opened', [{ event: 'advrows.desk.DeskOpened', payload: { desk_id: id } }]);
        }
        case 'advrows.desk.Activate': {
          const desk = desks.get(input.desk_id);
          if (desk === undefined) throw new Error(`no desk ${input.desk_id}`);
          if (desk.state !== 'Idle') throw new Error('only an idle desk activates');
          desk.state = 'Active';
          desk.zone = input.zone;
          return answered('activated', [{ event: 'advrows.desk.Activated', payload: { desk_id: input.desk_id } }]);
        }
        case 'advrows.desk.Finish': {
          for (const desk of desks.values()) {
            if (desk.state !== 'Active' || desk.zone !== input.zone) continue;
            desk.state = 'Done';
          }
          return answered('finished', [{ event: 'advrows.desk.Finished', payload: { zone: input.zone } }]);
        }
        default:
          throw unsupported(`${command} is not a command of adversary-l3-selected-rows.yaml`);
      }
    },

    queryView({ view }) {
      if (view === 'advrows.desk.Seats') {
        return {
          rows: [...seats].map(([id, seat]) => ({ seat_id: id, state: seat.state, last_reason: seat.last_reason })),
        };
      }
      if (view === 'advrows.desk.Desks') {
        return { rows: [...desks].map(([id, desk]) => ({ desk_id: id, state: desk.state })) };
      }
      throw unsupported(`${view} is not a view of adversary-l3-selected-rows.yaml`);
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
