// An in-memory implementation of `explore.yaml`, and the engine mutants the explorer must catch.
//
// `ESS_EXPLORE_MUTANT` (or the argument to `newTarget`) selects one:
//
//   view-cap-5                  `TicketsByItems` returns at most 5 rows
//   wrong-state-applies-sets    a `wrong_state` answer to `HoldTicket` still writes `priority`
//   fourth-create-reuses-first  the 4th `OpenTicket` answers with the first ticket's identity
//   third-return-refused        the 3rd time a ticket re-enters a state, the move is refused
//   close-unsupported           `CloseTicket` is not exposed
//
// `ESS_EXPLORE_GRADE` says which branch a score from 50 to 60 takes, where the specification's
// two guards overlap: `low` (the default) or `high`. Either is correct.
//
// `explore_target.go` is the same target in Go, line for line.

import { unsupported } from './dist/index.js';

const OPEN = 'Open';
const HELD = 'Held';
const CLOSED = 'Closed';
const CONFLICT = 'explore.desk.TicketStateConflict';

export function newTarget(
  mutant = process.env.ESS_EXPLORE_MUTANT ?? '',
  grade = process.env.ESS_EXPLORE_GRADE ?? 'low',
) {
  let tickets = new Map();
  let created = 0;
  let first = '';
  let version = 0;
  // What an `eventual` read shows: the rows as of the read before this one.
  let projected = [];

  const reset = () => {
    tickets = new Map();
    created = 0;
    first = '';
    version = 0;
    projected = [];
  };
  const row = (ticket) => ({
    ticket_id: ticket.ticket_id,
    items: ticket.items,
    priority: ticket.priority,
  });
  const refused = (error) => ({ outcome: 'wrong-state', error, directEvents: [] });
  const answered = (outcome, event, payload) => {
    version += 1;
    return {
      outcome,
      consistency: `v${version}`,
      directEvents: [{ event, payload }],
    };
  };

  // A move into `to`, unless the third-return mutant refuses it.
  const move = (ticket, to) => {
    if (ticket.visited.has(to)) {
      ticket.returns += 1;
      if (mutant === 'third-return-refused' && ticket.returns === 3) return false;
    }
    ticket.visited.add(to);
    ticket.state = to;
    return true;
  };

  return {
    identity: () => ({ name: 'explore-target', version: '1' }),
    beginScenario: () => reset(),
    endScenario: () => reset(),

    executeCommand({ command, input }) {
      const ticket = tickets.get(input.ticket_id);
      switch (command) {
        case 'explore.desk.OpenTicket': {
          if (!(input.items >= 0))
            return { outcome: 'rejected', error: 'explore.desk.InvalidItems' };
          created += 1;
          let id = `00000000-0000-4000-8000-${String(created).padStart(12, '0')}`;
          if (created === 1) first = id;
          if (mutant === 'fourth-create-reuses-first' && created === 4) id = first;
          tickets.set(id, {
            ticket_id: id,
            items: input.items,
            priority: 0,
            state: OPEN,
            visited: new Set([OPEN]),
            returns: 0,
          });
          return answered('opened', 'explore.desk.TicketOpened', {
            ticket_id: id,
            items: input.items,
          });
        }
        case 'explore.desk.HoldTicket': {
          if (ticket.state !== OPEN) {
            if (mutant === 'wrong-state-applies-sets') ticket.priority = input.priority;
            return refused(CONFLICT);
          }
          if (!move(ticket, HELD)) return refused(CONFLICT);
          ticket.priority = input.priority;
          return answered('held', 'explore.desk.TicketHeld', { ticket_id: ticket.ticket_id });
        }
        case 'explore.desk.ReleaseTicket': {
          if (ticket.state !== HELD || !move(ticket, OPEN)) return refused(CONFLICT);
          return answered('released', 'explore.desk.TicketReleased', {
            ticket_id: ticket.ticket_id,
          });
        }
        case 'explore.desk.CloseTicket': {
          if (mutant === 'close-unsupported') throw unsupported('CloseTicket is not exposed');
          if (ticket.state === CLOSED || !move(ticket, CLOSED)) return refused(CONFLICT);
          return answered('closed', 'explore.desk.TicketClosed', { ticket_id: ticket.ticket_id });
        }
        case 'explore.desk.Restock': {
          if (!(input.items >= 0))
            return { outcome: 'refused', error: 'explore.desk.InvalidItems' };
          ticket.items = input.items;
          return answered('restocked', 'explore.desk.TicketRestocked', {
            ticket_id: ticket.ticket_id,
            items: input.items,
          });
        }
        case 'explore.desk.Grade': {
          const score = input.score;
          if (score > 60 || (score >= 50 && grade === 'high')) {
            return answered('high', 'explore.desk.GradedHigh', { score });
          }
          if (score >= 10) return answered('low', 'explore.desk.GradedLow', { score });
          return { outcome: 'ungraded', error: 'explore.desk.Ungradable' };
        }
        default:
          throw unsupported(`${command} is not a command of explore.yaml`);
      }
    },

    queryView({ view }) {
      const all = [...tickets.values()];
      if (view === 'explore.desk.OpenTickets') {
        return { rows: all.filter((ticket) => ticket.state === OPEN).map(row) };
      }
      if (view === 'explore.desk.TicketsByItems') {
        const shown = projected;
        projected = all.map(row).sort((left, right) => right.items - left.items);
        return { rows: mutant === 'view-cap-5' ? shown.slice(0, 5) : shown };
      }
      throw unsupported(`${view} is not a view of explore.yaml`);
    },

    observeEvents: () => [],
    configureExternalOutcome: () => {
      throw unsupported('no external outcome');
    },
    redeliverEvent: () => {
      throw unsupported('no redelivery');
    },
    observeInvocations: () => {
      throw unsupported('no bindings');
    },
  };
}
