// A small in-memory implementation of `examples/billing`, for the concurrent explorer lanes in
// `crates/edge/ess-cli/tests/explore_concurrent.rs`.
//
// `explore_concurrent_billing_target.go` is the same target in Go, line for line: the two lanes'
// histories are compared byte for byte, so both mint the same identities in the same order. It has
// no `invokeCommand`, so every call takes effect at its return instant. Both views answer from the
// current invoices, which is what `read_your_writes` and `eventual` both allow.
//
// The mutant `double-apply` (the argument, or `ESS_EXPLORE_MUTANT`) applies the creation an
// `InvoiceCreated` announces again when the event is delivered a second time: an invoice nobody
// created, which only a read of a view shows.

import { unsupported } from './dist/index.js';

const id = (n) => `00000000-0000-4000-8000-${String(n).padStart(12, '0')}`;
const positive = (input) => typeof input.amount?.amount === 'number' && input.amount.amount > 0;

export function newBillingTarget(mutant = '') {
  if (mutant === '') mutant = process.env.ESS_EXPLORE_MUTANT ?? '';
  // Every invoice's state, in the order it was created.
  let invoices = new Map();
  let created = 0;
  let sent = 0;
  // The input of the last `CreateInvoice` that created an invoice.
  let creation = null;
  const reset = () => {
    invoices = new Map();
    created = 0;
    sent = 0;
    creation = null;
  };
  // Mints an invoice in `Draft` from `input`.
  const create = (input) => {
    created += 1;
    const minted = id(created);
    invoices.set(minted, 'Draft');
    creation = input;
    return minted;
  };

  return {
    identity: () => ({ name: 'explore-billing-target', version: '1' }),
    beginScenario: () => reset(),
    endScenario: () => reset(),

    executeCommand({ command, input }) {
      const invoice = input.invoice_id;
      const state = invoices.get(invoice) ?? '';
      const conflict = {
        outcome: 'wrong-state',
        error: 'billing.invoice.InvoiceStateConflict',
        directEvents: [],
      };
      const moved = (outcome, event, to, payload) => {
        invoices.set(invoice, to);
        return { outcome, directEvents: [{ event, payload }] };
      };
      switch (command) {
        case 'billing.invoice.CreateInvoice': {
          if (!positive(input)) return { outcome: 'rejected', error: 'billing.invoice.InvalidAmount' };
          const minted = create(input);
          return {
            outcome: 'accepted',
            directEvents: [
              {
                event: 'billing.invoice.InvoiceCreated',
                payload: {
                  invoice_id: minted,
                  account_id: input.account_id,
                  customer_email: input.customer_email,
                  amount: input.amount,
                },
              },
            ],
          };
        }
        case 'billing.invoice.IssueInvoice':
          if (state !== 'Draft') return conflict;
          return moved('issued', 'billing.invoice.InvoiceIssued', 'Issued', { invoice_id: invoice });
        case 'billing.invoice.PayInvoice':
          if (!positive(input)) return { outcome: 'rejected', error: 'billing.invoice.InvalidAmount' };
          if (state !== 'Issued') return conflict;
          return moved('settled', 'billing.invoice.InvoicePaid', 'Paid', {
            invoice_id: invoice,
            amount: input.amount,
          });
        case 'billing.invoice.CancelInvoice':
          if (state !== 'Draft' && state !== 'Issued') return conflict;
          return moved('cancelled', 'billing.invoice.InvoiceCancelled', 'Cancelled', {
            invoice_id: invoice,
          });
        case 'billing.email.SendEmail':
          sent += 1;
          return {
            outcome: 'sent',
            directEvents: [
              {
                event: 'billing.email.EmailSent',
                payload: { message_id: id(sent), recipient: input.recipient },
              },
            ],
          };
        default:
          throw unsupported(`${command} is not a command of examples/billing`);
      }
    },

    // `InvoiceById` is every invoice and `OutstandingInvoices` the issued ones, in the order they
    // were created.
    queryView({ view }) {
      const outstanding = view === 'billing.invoice.OutstandingInvoices';
      if (!outstanding && view !== 'billing.invoice.InvoiceById') {
        throw unsupported(`${view} is not a view of examples/billing`);
      }
      const rows = [];
      for (const [invoice, state] of invoices) {
        if (!outstanding || state === 'Issued') rows.push({ invoice_id: invoice });
      }
      return { rows };
    },
    observeEvents: () => [],
    configureExternalOutcome: () => {
      throw unsupported('no external outcome');
    },
    // `InvoiceCreated` again reaches `notify-on-invoice-created`, whose mail this target does not
    // model, so nothing changes; any other event it has no binding for. Under `double-apply` the
    // creation it announces is applied again.
    redeliverEvent({ event }) {
      if (event !== 'billing.invoice.InvoiceCreated') throw unsupported(`no binding reacts to ${event}`);
      if (mutant === 'double-apply' && creation !== null) create(creation);
    },
    observeInvocations: () => {
      throw unsupported('no bindings');
    },
  };
}
