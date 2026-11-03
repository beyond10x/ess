// An in-memory implementation of `explore-held-state-external.yaml`, a test double that arranges
// external outcomes the way the synthesized scenarios ask it to. It is copied into the package as
// `explore-external-target.mjs`, which `explore-external-driver.mjs` runs.
//
// An arrangement holds for its command until the scenario ends. `mode` selects the double:
//
//   (empty)         answers a parked ticket's held state before it asks the arranged gateway, as
//                   the interpreter does
//   external-first  asks the arranged gateway before it reads the ticket's held state
//
// Each `Nudge` it answers is appended to `<ESS_EXPLORE_ANSWERS>/<mode>.jsonl` (`held-first` for the
// empty mode) as the state the ticket rested in (empty for an identity no ticket carries), the
// branch arranged for the command and the branch it answered.
//
// `explore_held_state_external_target.go` is the same target in Go, line for line.

import { appendFileSync } from 'node:fs';
import { join } from 'node:path';

import { unsupported } from './dist/index.js';

export function newTarget(mode = '') {
  let tickets = new Map();
  let created = 0;
  let version = 0;
  let forced = new Map();

  const reset = () => {
    tickets = new Map();
    created = 0;
    version = 0;
    forced = new Map();
  };
  const answered = (outcome, event, payload) => {
    version += 1;
    return { outcome, consistency: `v${version}`, directEvents: [{ event, payload }] };
  };
  const refused = (outcome, error) => ({ outcome, error, directEvents: [] });
  const record = (state, outcome) => {
    const dir = process.env.ESS_EXPLORE_ANSWERS;
    if (dir === undefined || dir === '') return;
    const name = mode === '' ? 'held-first' : mode;
    const arranged = forced.get('explorehx.desk.Nudge') ?? '';
    appendFileSync(
      join(dir, `${name}.jsonl`),
      `${JSON.stringify({ answered: outcome, arranged, state })}\n`,
    );
  };
  const nudge = (id) => {
    const ticket = tickets.get(id);
    if (ticket === undefined) return refused('no-ticket', 'explorehx.desk.NoTicket');
    const bounced = forced.get('explorehx.desk.Nudge') === 'bounced';
    if (mode === 'external-first' && bounced) {
      return refused('bounced', 'explorehx.desk.Bounced');
    }
    if (ticket.state === 'Parked') return refused('parked', 'explorehx.desk.NotOpen');
    if (bounced) return refused('bounced', 'explorehx.desk.Bounced');
    return answered('nudged', 'explorehx.desk.Nudged', { ticket_id: ticket.ticket_id });
  };

  return {
    identity: () => ({ name: 'explore-held-state-external-target', version: '1' }),
    beginScenario: () => reset(),
    endScenario: () => reset(),

    executeCommand({ command, input }) {
      switch (command) {
        case 'explorehx.desk.OpenTicket': {
          created += 1;
          const id = `00000000-0000-4000-8000-${String(created).padStart(12, '0')}`;
          tickets.set(id, { ticket_id: id, label: input.label, state: 'Open' });
          return answered('opened', 'explorehx.desk.Opened', { ticket_id: id });
        }
        case 'explorehx.desk.Park': {
          const ticket = tickets.get(input.ticket_id);
          if (ticket === undefined) throw new Error(`no ticket ${input.ticket_id}`);
          if (ticket.state !== 'Open') return refused('not-open', 'explorehx.desk.NotOpen');
          ticket.state = 'Parked';
          return answered('parked-now', 'explorehx.desk.Parked', { ticket_id: ticket.ticket_id });
        }
        case 'explorehx.desk.Unpark': {
          const ticket = tickets.get(input.ticket_id);
          if (ticket === undefined) throw new Error(`no ticket ${input.ticket_id}`);
          if (ticket.state !== 'Parked') return refused('not-parked', 'explorehx.desk.NotParked');
          ticket.state = 'Open';
          return answered('unparked', 'explorehx.desk.Unparked', { ticket_id: ticket.ticket_id });
        }
        case 'explorehx.desk.Nudge': {
          const state = tickets.get(input.ticket_id)?.state ?? '';
          const result = nudge(input.ticket_id);
          record(state, result.outcome);
          return result;
        }
        default:
          throw unsupported(`${command} is not a command of explore-held-state-external.yaml`);
      }
    },

    queryView({ view }) {
      if (view === 'explorehx.desk.Tickets') {
        return { rows: [...tickets.values()].map((ticket) => ({ ...ticket })) };
      }
      throw unsupported(`${view} is not a view of explore-held-state-external.yaml`);
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
