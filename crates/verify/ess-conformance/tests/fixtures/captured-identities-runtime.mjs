// A desk that files tickets into queues (`captured-identities.yaml`). ESS_IDENTITY_FAULT selects a
// faulty implementation: one that drops the Optional identity a closed ticket names, or one that
// publishes another identity in place of the ticket's or its queue's.
import test from 'node:test';
import { run } from './dist/runtime.js';
class Desk {
  mode = process.env.ESS_IDENTITY_FAULT ?? 'good';
  minted = 0;
  tickets = new Map();
  closed = new Set();
  mint() { this.minted += 1; return `00000000-0000-4000-8000-${String(this.minted).padStart(12, '0')}`; }
  identity() { return {name:'desk',version:'1'}; }
  beginScenario() { this.tickets = new Map(); this.closed = new Set(); }
  endScenario() {}
  configureExternalOutcome() { throw Error('no external outcomes'); }
  executeCommand({command, input}) {
    switch (command) {
      case 'desk.tickets.OpenQueue':
        return {outcome:'opened',directEvents:[{event:'desk.tickets.QueueOpened',payload:{queue_id:this.mint(),label:input.label}}]};
      case 'desk.tickets.FileTicket': {
        const id = this.mint();
        this.tickets.set(id, input.queue_id);
        const queue = this.mode === 'swap-filed' ? this.mint() : input.queue_id;
        return {outcome:'filed',directEvents:[{event:'desk.tickets.TicketFiled',payload:{ticket_id:id,queue_id:queue}}]};
      }
      case 'desk.tickets.CloseTicket': {
        const ticket = input.ticket_id;
        if (!this.tickets.has(ticket) || this.closed.has(ticket)) return {outcome:'wrong-state',error:'desk.tickets.NotOpen'};
        this.closed.add(ticket);
        const payload = {ticket_id:ticket,closed:ticket,queue_id:this.tickets.get(ticket)};
        if (this.mode === 'drop-subject') delete payload.closed;
        if (this.mode === 'swap-input') payload.ticket_id = this.mint();
        if (this.mode === 'swap-related') payload.queue_id = this.mint();
        return {outcome:'closed',directEvents:[{event:'desk.tickets.TicketClosed',payload}]};
      }
    }
    throw Error(`undeclared command ${command}`);
  }
  queryView() { throw Error('no views'); }
  observeEvents() { return []; }
  redeliverEvent() { throw Error('no bindings'); }
  observeInvocations() { throw Error('no bindings'); }
}
await test('captured identities', t => run(t, () => new Desk()));
