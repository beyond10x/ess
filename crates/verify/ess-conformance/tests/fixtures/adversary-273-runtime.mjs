// Adversary pass 1 for beyond10x/ess#273: the desk of `captured-identities.yaml`, with faults the
// unit's own fixture does not apply (see `adversary-273-runtime.go`).
import test from 'node:test';
import { run } from './dist/runtime.js';
class Desk {
  mode = process.env.ESS_IDENTITY_FAULT ?? 'good';
  minted = 0;
  firstQueue = null;
  tickets = new Map();
  closed = new Set();
  mint() { this.minted += 1; return `00000000-0000-4000-8000-${String(this.minted).padStart(12, '0')}`; }
  identity() { return {name:'desk',version:'1'}; }
  beginScenario() { this.tickets = new Map(); this.closed = new Set(); this.firstQueue = null; }
  endScenario() {}
  configureExternalOutcome() { throw Error('no external outcomes'); }
  executeCommand({command, input}) {
    switch (command) {
      case 'desk.tickets.OpenQueue': {
        const id = this.mint();
        if (this.firstQueue === null) this.firstQueue = id;
        return {outcome:'opened',directEvents:[{event:'desk.tickets.QueueOpened',payload:{queue_id:id,label:input.label}}]};
      }
      case 'desk.tickets.FileTicket': {
        const id = this.mint();
        this.tickets.set(id, input.queue_id);
        return {outcome:'filed',directEvents:[{event:'desk.tickets.TicketFiled',payload:{ticket_id:id,queue_id:input.queue_id}}]};
      }
      case 'desk.tickets.CloseTicket': {
        const ticket = input.ticket_id;
        if (!this.tickets.has(ticket) || this.closed.has(ticket)) return {outcome:'wrong-state',error:'desk.tickets.NotOpen'};
        this.closed.add(ticket);
        const queue = this.tickets.get(ticket);
        const payload = {ticket_id:ticket,closed:ticket,queue_id:queue};
        if (this.mode === 'swap-queue') payload.queue_id = this.firstQueue;
        if (this.mode === 'null-closed') payload.closed = null;
        if (this.mode === 'zero-ticket') payload.ticket_id = '00000000-0000-0000-0000-000000000000';
        if (this.mode === 'swap-pair') { payload.ticket_id = queue; payload.queue_id = ticket; }
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
await test('adversary 273', t => run(t, () => new Desk()));
