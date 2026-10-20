// The desk model of `tests/fixtures/navigate-a-reference.yaml` (ess/28, beyond10x/ess#564,
// beyond10x/ess#557): two views read the desk a ticket names, and a refusal names the desk's
// state. Modes: `correct`, `copies-at-write`, `latest-desk`, `wrong-current`.
const mode = process.env.ESS_TARGET_MODE ?? 'correct';
let minted = 0;

const uuid = (n) => `00000000-0000-4000-8000-${String(n).padStart(12, '0')}`;

class Desks {
  desks = [];
  tickets = [];
  identity() {
    return { name: 'desk-fixture', version: '1' };
  }
  beginScenario() {
    this.desks = [];
    this.tickets = [];
  }
  endScenario() {}
  /** The state and label this implementation reads for `ticket`'s desk. */
  read(ticket) {
    if (mode === 'copies-at-write') {
      return ticket.copied;
    }
    const desk =
      mode === 'latest-desk'
        ? this.desks[this.desks.length - 1]
        : this.desks.find((held) => held.id === ticket.desk);
    return desk === undefined ? undefined : { state: desk.state, label: desk.label };
  }
  executeCommand({ command, input }) {
    minted += 1;
    const consistency = `seq:${minted}`;
    switch (command) {
      case 'relstate.desk.OpenDesk': {
        const id = uuid(minted);
        this.desks.push({ id, label: input.label, state: 'Open' });
        return {
          outcome: 'opened',
          consistency,
          directEvents: [{ event: 'relstate.desk.DeskOpened', payload: { desk_id: id } }],
        };
      }
      case 'relstate.desk.CloseDesk': {
        const desk = this.desks.find((held) => held.id === input.desk_id && held.state === 'Open');
        if (desk === undefined) {
          return {
            outcome: 'wrong-state',
            consistency,
            error: 'relstate.desk.WrongState',
          };
        }
        desk.state = 'Closed';
        return {
          outcome: 'closed',
          consistency,
          directEvents: [{ event: 'relstate.desk.DeskClosed', payload: { desk_id: desk.id } }],
        };
      }
      case 'relstate.desk.OpenTicket': {
        const desk = this.desks.find((held) => held.id === input.desk_id);
        if (desk === undefined) {
          return {
            outcome: 'unknown-desk',
            consistency,
            error: 'relstate.desk.DeskNotFound',
          };
        }
        if (desk.state !== 'Open') {
          const current = mode === 'wrong-current' ? 'Open' : desk.state;
          return {
            outcome: 'desk-not-open',
            consistency,
            error: 'relstate.desk.DeskNotOpen',
            // The TypeScript runtime reads `errorPayload`, the Rust runner's proxy `errorFields`.
            errorPayload: { current },
            errorFields: { current },
          };
        }
        const id = uuid(minted);
        this.tickets.push({ id, desk: desk.id, copied: { state: desk.state, label: desk.label } });
        return {
          outcome: 'opened',
          consistency,
          directEvents: [{ event: 'relstate.desk.TicketOpened', payload: { ticket_id: id } }],
        };
      }
      default:
        throw new Error(`unexpected command ${command}`);
    }
  }
  queryView({ view }) {
    switch (view) {
      case 'relstate.desk.Desks':
        return { rows: this.desks.map((desk) => ({ desk_id: desk.id, state: desk.state })) };
      case 'relstate.desk.Tickets':
        return {
          rows: this.tickets.map((ticket) => {
            const read = this.read(ticket);
            return {
              ticket_id: ticket.id,
              desk_id: ticket.desk,
              desk_state: read === undefined ? null : read.state,
              desk_label: read === undefined ? null : read.label,
            };
          }),
        };
      case 'relstate.desk.TicketsAtOpenDesks':
        return {
          rows: this.tickets
            .filter((ticket) => this.read(ticket)?.state === 'Open')
            .map((ticket) => ({ ticket_id: ticket.id })),
        };
      default:
        throw new Error(`unexpected view ${view}`);
    }
  }
  observeEvents() {
    return [];
  }
  configureExternalOutcome() {
    throw new Error('nothing here is externally decided');
  }
  redeliverEvent() {
    throw new Error('no bindings');
  }
}

export function makeTarget() {
  return new Desks();
}
