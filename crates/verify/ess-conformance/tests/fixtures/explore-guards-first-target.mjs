// An in-memory implementation of `explore-guards-first.yaml` (beyond10x/ess#235), deciding each
// command in Entity Runtime's order: the input-guarded refusals, the first declared whose guard
// holds, in every state; then the accepting branch or the default; then, for a move from a state no
// move of the command starts from, the wrong-state answer.
//
// `mode` selects the double:
//
//   (empty)       Entity Runtime's order
//   state-first   answers the wrong-state branch before any guard, the order #235 reported
//   last-refusal  answers `negative` where `tiny` and `negative` both hold
//   accept-first  answers `held` where `held` and `capped` both hold
//
// `explore_guards_first_target.go` is the same target in Go, line for line.

import { unsupported } from './dist/index.js';

export function newTarget(mode = '') {
  let tickets = new Map();
  let created = 0;
  let version = 0;

  const reset = () => {
    tickets = new Map();
    created = 0;
    version = 0;
  };
  const refused = (outcome, error) => ({ outcome, error, directEvents: [] });
  const answered = (outcome, event, id) => {
    version += 1;
    return {
      outcome,
      consistency: `v${version}`,
      directEvents: [{ event, payload: { ticket_id: id } }],
    };
  };
  const conflict = () => refused('wrong-state', 'exploreorder.desk.TicketStateConflict');

  return {
    identity: () => ({ name: 'explore-guards-first-target', version: '1' }),
    beginScenario: () => reset(),
    endScenario: () => reset(),

    executeCommand({ command, input }) {
      switch (command) {
        case 'exploreorder.desk.Open': {
          created += 1;
          const id = `00000000-0000-4000-8000-${String(created).padStart(12, '0')}`;
          tickets.set(id, { state: 'Open', priority: 0 });
          return answered('opened', 'exploreorder.desk.Opened', id);
        }
        case 'exploreorder.desk.Hold': {
          const ticket = tickets.get(input.ticket_id);
          const priority = input.priority;
          if (mode === 'state-first' && ticket?.state === 'Held') return conflict();
          if (priority < -5 && mode !== 'last-refusal') {
            return refused('tiny', 'exploreorder.desk.PriorityTiny');
          }
          if (priority < 0) return refused('negative', 'exploreorder.desk.PriorityNegative');
          if (priority > 100 && mode !== 'accept-first') {
            return refused('capped', 'exploreorder.desk.PriorityCapped');
          }
          if (priority === 0) return refused('unrated', 'exploreorder.desk.Unrated');
          if (ticket === undefined) throw new Error(`no ticket ${input.ticket_id}`);
          if (ticket.state !== 'Open') return conflict();
          ticket.state = 'Held';
          ticket.priority = priority;
          return answered('held', 'exploreorder.desk.Held', input.ticket_id);
        }
        case 'exploreorder.desk.Release': {
          const ticket = tickets.get(input.ticket_id);
          if (ticket === undefined) throw new Error(`no ticket ${input.ticket_id}`);
          if (mode === 'state-first' && ticket.state === 'Open') return conflict();
          if (input.note === 7) {
            ticket.priority = 7;
            return answered('noted', 'exploreorder.desk.Noted', input.ticket_id);
          }
          if (ticket.state !== 'Held') return conflict();
          ticket.state = 'Open';
          return answered('released', 'exploreorder.desk.Released', input.ticket_id);
        }
        default:
          throw unsupported(`${command} is not a command of explore-guards-first.yaml`);
      }
    },

    queryView({ view }) {
      if (view !== 'exploreorder.desk.Tickets') {
        throw unsupported(`${view} is not a view of explore-guards-first.yaml`);
      }
      return {
        rows: [...tickets].map(([id, ticket]) => ({
          ticket_id: id,
          priority: ticket.priority,
          state: ticket.state,
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
