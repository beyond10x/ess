// An in-memory implementation of `explore-input-beside-unknown.yaml`. It is copied into the package
// as `explore-external-target.mjs`, which `explore-external-driver.mjs` runs.
//
// The mode is the order `Nudge` and `Route` ask their questions in: '' (the declaration order: the
// amount, the ticket, then the desk) or 'ticket-first' (the ticket before the amount).
//
// `explore_input_beside_unknown_target.go` is the same target in Go, line for line.

import { unsupported } from './dist/index.js';

export function newTarget(mode) {
  let tickets = new Set();
  let desks = new Set();
  let created = 0;
  let version = 0;

  const reset = () => {
    tickets = new Set();
    desks = new Set();
    created = 0;
    version = 0;
  };
  const answered = (outcome, event, payload) => {
    version += 1;
    return { outcome, consistency: `v${version}`, directEvents: [{ event, payload }] };
  };
  const refused = (outcome, error) => ({ outcome, error, directEvents: [] });
  const identity = () => {
    created += 1;
    return `00000000-0000-4000-8000-${String(created).padStart(12, '0')}`;
  };

  // `Nudge`, or `Route` where `routed` is set, which reads the desk as well.
  const answer = (input, routed) => {
    const ticket = input.ticket_id;
    const desk = input.desk_id;
    if (mode === 'ticket-first' && !tickets.has(ticket)) {
      return refused('no-ticket', 'exploreiu.desk.NoTicket');
    }
    if (Number(input.amount) > 100) return refused('too-much', 'exploreiu.desk.TooMuch');
    if (!tickets.has(ticket)) return refused('no-ticket', 'exploreiu.desk.NoTicket');
    if (!routed) return answered('nudged', 'exploreiu.desk.Nudged', { ticket_id: ticket });
    if (!desks.has(desk)) return refused('no-desk', 'exploreiu.desk.NoDesk');
    return answered('routed', 'exploreiu.desk.Routed', { ticket_id: ticket });
  };

  return {
    identity: () => ({ name: 'explore-input-beside-unknown-target', version: '1' }),
    beginScenario: () => reset(),
    endScenario: () => reset(),

    executeCommand({ command, input }) {
      switch (command) {
        case 'exploreiu.desk.MakeDesk': {
          const id = identity();
          desks.add(id);
          return answered('made', 'exploreiu.desk.DeskMade', { desk_id: id });
        }
        case 'exploreiu.desk.OpenTicket': {
          const id = identity();
          tickets.add(id);
          return answered('opened', 'exploreiu.desk.Opened', { ticket_id: id });
        }
        case 'exploreiu.desk.Nudge':
          return answer(input, false);
        case 'exploreiu.desk.Route':
          return answer(input, true);
        default:
          throw unsupported(`${command} is not a command of explore-input-beside-unknown.yaml`);
      }
    },

    queryView({ view }) {
      throw unsupported(`${view} is not a view of explore-input-beside-unknown.yaml`);
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
