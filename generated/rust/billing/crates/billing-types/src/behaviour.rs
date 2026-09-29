// generated from billing v3
// model digest 1e7906786567af32118eb2d0a8c3fcafa16c32c9649a80b60487fd2eeebc4c9c
// contract digest a21fd36f0055057629f4c235962163cdd34a3d178aa068925bcb53be623af301
// do not edit: regenerate with `ess synthesize`

//! What the specification fully determines, generated: the behaviour of every command the plan
//! lists as generated, written against ports the implementor supplies.
//!
//! Storage is a port: one trait per entity, get, put and delete of a snapshot by identity. ess
//! generates the trait and never a store. `Context` is the other port: the caller's attributes,
//! every identity and value the model says the implementation assigns, and the answer to each
//! `external:` branch. [`Generated`] implements every generated `…Behavior` trait over those ports
//! and forwards every behaviour and query the plan still owes to them, so it is a complete bundle
//! for every component port. To replace one generated behaviour, write a bundle of your own that
//! implements that trait and delegates the rest to a `Generated`.
//!
//! An `Err` from a generated behaviour is the typed refusal naming the command: the model declares
//! no outcome for the request (a guard is undecidable over it, or no declared branch answers it),
//! or — as `entity invariant` — the declared outcome would leave an entity breaking an invariant.

use crate::obligation::UnmetObligation;

/// Where `billing.invoice.Invoice` is stored — a port the implementor provides.
///
/// Keyed by the identity `invoice_id`. ess generates this trait and never an implementation of it.
pub trait InvoiceStorage {
    /// The instance with this identity, or `None` where none is stored.
    fn get(&self, identity: &crate::invoice::InvoiceId) -> Option<crate::invoice::InvoiceSnapshot>;

    /// Stores this instance under its identity, replacing what was held.
    fn put(&mut self, snapshot: crate::invoice::InvoiceSnapshot);

    /// Removes the instance with this identity.
    fn delete(&mut self, identity: &crate::invoice::InvoiceId);
}

/// What the specification leaves to the implementor's context — a port the implementor provides.
///
/// The caller's attributes, the values the model says the implementation assigns, and the answer
/// to each `external:` branch.
pub trait Context {
    /// A new `billing.email.MessageId`, which the model says the implementation assigns — a created identity, a
    /// `{generated: true}` value, or an event field the model leaves undetermined.
    fn generate_billing_email_message_id(&mut self) -> crate::email::MessageId;

    /// Whether the external branch `outcome` of `command` is taken on this invocation.
    ///
    /// Asked in declaration order, before the branch's input guard is read; the first branch
    /// answered `true` whose guard holds is taken. A test forces a branch by answering `true`
    /// for it alone; a deployment asks whatever decides it.
    fn external(&mut self, command: &'static str, outcome: &'static str) -> bool;
}

/// Every generated behaviour of this workspace, over the ports `P` supplies.
///
/// `P` implements the storage trait of each entity a generated behaviour reads or writes,
/// `Context` where one asks it anything, and every `…Behavior` and `…Query` trait the plan still
/// owes; `Generated<P>` forwards those to it.
pub struct Generated<P> {
    /// The storage and context ports, and every behaviour or query still owed.
    pub ports: P,
}

impl<P> Generated<P> {
    /// The generated behaviours, over `ports`.
    pub fn new(ports: P) -> Self {
        Self { ports }
    }
}

/// `billing.email.SendEmail`, generated: every outcome is one the specification fully determines.
impl<P> crate::email::obligations::SendEmailBehavior for Generated<P>
where
    P: Context,
{
    fn send_email(&mut self, input: crate::email::SendEmail) -> Result<crate::email::SendEmailOutcome, UnmetObligation> {
        let _ = &input;
        // `failed`: an external branch, where the context takes it.
        if self.ports.external("billing.email.SendEmail", "failed") {
            return Ok(crate::email::SendEmailOutcome::Failed { error: crate::email::Undeliverable });
        }
        // `sent`: the default.
        return Ok(crate::email::SendEmailOutcome::Sent { email_sent: crate::email::EmailSent { message_id: self.ports.generate_billing_email_message_id(), recipient: input.recipient.clone() } });
    }
}

/// `billing.invoice.CancelInvoice`, generated: every outcome is one the specification fully determines.
impl<P> crate::invoice::obligations::CancelInvoiceBehavior for Generated<P>
where
    P: InvoiceStorage,
{
    fn cancel_invoice(&mut self, input: crate::invoice::CancelInvoice) -> Result<crate::invoice::CancelInvoiceOutcome, UnmetObligation> {
        let _ = &input;
        // `cancelled`: the default.
        let Some(held) = InvoiceStorage::get(&self.ports, &input.invoice_id) else {
            return Ok(crate::invoice::CancelInvoiceOutcome::WrongStateUnknownInstance);
        };
        let _ = &held;
        let held_state = held.state;
        let moved = match held.refine() {
            crate::invoice::AnyInvoice::Draft(instance) => crate::invoice::AnyInvoice::Cancelled(instance.cancel()),
            crate::invoice::AnyInvoice::Issued(instance) => crate::invoice::AnyInvoice::Cancelled(instance.cancel()),
            _ => return Ok(crate::invoice::CancelInvoiceOutcome::WrongState { error: crate::invoice::InvoiceStateConflict { state: held_state } }),
        };
        let next = moved.snapshot();
        if let Some(broken) = next.data.broken_invariant() {
            let capability = "entity invariant";
            return Err(UnmetObligation { capability, source: broken });
        }
        InvoiceStorage::put(&mut self.ports, next);
        return Ok(crate::invoice::CancelInvoiceOutcome::Cancelled { invoice_cancelled: crate::invoice::InvoiceCancelled { invoice_id: input.invoice_id.clone() } });
    }
}

impl<P: crate::invoice::obligations::CreateInvoiceBehavior> crate::invoice::obligations::CreateInvoiceBehavior for Generated<P> {
    fn create_invoice(&mut self, input: crate::invoice::CreateInvoice) -> Result<crate::invoice::CreateInvoiceOutcome, UnmetObligation> {
        crate::invoice::obligations::CreateInvoiceBehavior::create_invoice(&mut self.ports, input)
    }
}

/// `billing.invoice.IssueInvoice`, generated: every outcome is one the specification fully determines.
impl<P> crate::invoice::obligations::IssueInvoiceBehavior for Generated<P>
where
    P: InvoiceStorage,
{
    fn issue_invoice(&mut self, input: crate::invoice::IssueInvoice) -> Result<crate::invoice::IssueInvoiceOutcome, UnmetObligation> {
        let _ = &input;
        // `issued`: the default.
        let Some(held) = InvoiceStorage::get(&self.ports, &input.invoice_id) else {
            return Ok(crate::invoice::IssueInvoiceOutcome::WrongStateUnknownInstance);
        };
        let _ = &held;
        let held_state = held.state;
        let moved = match held.refine() {
            crate::invoice::AnyInvoice::Draft(instance) => crate::invoice::AnyInvoice::Issued(instance.issue()),
            _ => return Ok(crate::invoice::IssueInvoiceOutcome::WrongState { error: crate::invoice::InvoiceStateConflict { state: held_state } }),
        };
        let next = moved.snapshot();
        if let Some(broken) = next.data.broken_invariant() {
            let capability = "entity invariant";
            return Err(UnmetObligation { capability, source: broken });
        }
        InvoiceStorage::put(&mut self.ports, next);
        return Ok(crate::invoice::IssueInvoiceOutcome::Issued { invoice_issued: crate::invoice::InvoiceIssued { invoice_id: input.invoice_id.clone() } });
    }
}

impl<P: crate::invoice::obligations::PayInvoiceBehavior> crate::invoice::obligations::PayInvoiceBehavior for Generated<P> {
    fn pay_invoice(&mut self, input: crate::invoice::PayInvoice) -> Result<crate::invoice::PayInvoiceOutcome, UnmetObligation> {
        crate::invoice::obligations::PayInvoiceBehavior::pay_invoice(&mut self.ports, input)
    }
}

impl<P: crate::invoice::obligations::InvoiceByIdQuery> crate::invoice::obligations::InvoiceByIdQuery for Generated<P> {
    fn invoice_by_id(&self) -> Result<Vec<crate::invoice::InvoiceById>, UnmetObligation> {
        crate::invoice::obligations::InvoiceByIdQuery::invoice_by_id(&self.ports)
    }
}

impl<P: crate::invoice::obligations::OutstandingInvoicesQuery> crate::invoice::obligations::OutstandingInvoicesQuery for Generated<P> {
    fn outstanding_invoices(&self) -> Result<Vec<crate::invoice::OutstandingInvoices>, UnmetObligation> {
        crate::invoice::obligations::OutstandingInvoicesQuery::outstanding_invoices(&self.ports)
    }
}
