// generated from billing v3
// contract digest c9ecfdf5bed1bcb88068dad060895f16ca73de361204ae488d6f0d39477f6f79
// do not edit: regenerate with `ess synthesize`

//! What the specification fully determines, generated: the behaviour of every command the plan
//! lists as generated, written against ports the implementor supplies.
//!
//! Storage is a port: one trait per entity, get, put and delete of a snapshot by identity. ess
//! preserves the trait; generated network entries supply an ephemeral store. `Context` carries the caller's attributes,
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
/// Keyed by the identity `invoice_id`. Generated network entries supply an ephemeral implementation; durable storage remains a port.
pub trait InvoiceStorage {
    /// The instance with this identity, or `None` where none is stored.
    fn get(&self, identity: &crate::invoice::InvoiceId) -> Option<crate::invoice::InvoiceSnapshot>;

    /// Stores this instance under its identity, replacing what was held.
    fn put(&mut self, snapshot: crate::invoice::InvoiceSnapshot);

    /// Removes the instance with this identity.
    fn delete(&mut self, identity: &crate::invoice::InvoiceId);

    /// Every stored instance, in the order the store keeps them: the order a generated query
    /// answers an unordered view in.
    fn list(&self) -> Vec<crate::invoice::InvoiceSnapshot>;
}

/// The exact executing command input supplied to an external decision.
///
/// This supplies facts, not authority: the context must verify its request-bound proof.
#[derive(Debug, Clone, Copy)]
pub enum ExternalCommand<'a> {
    /// The executing `billing.email.SendEmail` input.
    BillingEmailSendEmail(&'a crate::email::SendEmail),
}

impl ExternalCommand<'_> {
    /// The canonical qualified identity of this command.
    pub fn name(&self) -> &'static str {
        match self {
            Self::BillingEmailSendEmail(_) => "billing.email.SendEmail",
        }
    }
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
    fn external(&mut self, command: ExternalCommand<'_>, outcome: &'static str) -> bool;
}

/// Context answers that may be unavailable, without fabricated values.
/// Existing `Context` implementations receive the blanket adapter.
pub trait TryContext {
/// Assigns the value, or names the unavailable answer.
fn try_generate_billing_email_message_id(&mut self) -> Result<crate::email::MessageId, UnmetObligation>;
/// Decides the named external branch, or names the unavailable answer.
fn try_external(&mut self, command: ExternalCommand<'_>, outcome: &'static str) -> Result<bool, UnmetObligation>;
}

impl<T: Context + ?Sized> TryContext for T {
fn try_generate_billing_email_message_id(&mut self) -> Result<crate::email::MessageId, UnmetObligation> { Ok(Context::generate_billing_email_message_id(self)) }
fn try_external(&mut self, command: ExternalCommand<'_>, outcome: &'static str) -> Result<bool, UnmetObligation> { Ok(Context::external(self, command, outcome)) }
}

/// An unavailable runtime context answer, rather than a new planned capability.
pub fn unmet_context(source: &'static str) -> UnmetObligation { UnmetObligation { capability: "context answer", source } }

/// Every generated behaviour of this workspace, over the ports `P` supplies.
///
/// `P` implements the storage trait of each entity a generated behaviour reads or writes,
/// `TryContext` (or its legacy `Context` blanket adapter) where one asks it anything, and every `…Behavior` and `…Query` trait the plan still
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
    P: TryContext,
{
    fn send_email(&mut self, input: crate::email::SendEmail) -> Result<crate::email::SendEmailOutcome, UnmetObligation> {
        let _ = &input;
        // `failed`: an external branch, where the context takes it.
        if self.ports.try_external(ExternalCommand::BillingEmailSendEmail(&input), "failed")? {
            return Ok(crate::email::SendEmailOutcome::Failed { error: crate::email::Undeliverable });
        }
        // `sent`: the default.
        return Ok(crate::email::SendEmailOutcome::Sent { email_sent: crate::email::EmailSent { message_id: self.ports.try_generate_billing_email_message_id()?, recipient: input.recipient.clone() } });
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
        let answer = crate::invoice::CancelInvoiceOutcome::Cancelled { invoice_cancelled: crate::invoice::InvoiceCancelled { invoice_id: input.invoice_id.clone() } };
        InvoiceStorage::put(&mut self.ports, next);
        return Ok(answer);
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
        let mut next = moved.snapshot();
        next.data.issued_at = Some(input.issued_at.clone());
        if let Some(broken) = next.data.broken_invariant() {
            let capability = "entity invariant";
            return Err(UnmetObligation { capability, source: broken });
        }
        let answer = crate::invoice::IssueInvoiceOutcome::Issued { invoice_issued: crate::invoice::InvoiceIssued { invoice_id: input.invoice_id.clone() } };
        InvoiceStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

impl<P: crate::invoice::obligations::PayInvoiceBehavior> crate::invoice::obligations::PayInvoiceBehavior for Generated<P> {
    fn pay_invoice(&mut self, input: crate::invoice::PayInvoice) -> Result<crate::invoice::PayInvoiceOutcome, UnmetObligation> {
        crate::invoice::obligations::PayInvoiceBehavior::pay_invoice(&mut self.ports, input)
    }
}

/// `billing.invoice.InvoiceById`, generated: every row is one the specification fully determines from the stored `billing.invoice.Invoice`s.
impl<P> crate::invoice::obligations::InvoiceByIdQuery for Generated<P>
where
    P: InvoiceStorage,
{
    fn invoice_by_id(&self) -> Result<Vec<crate::invoice::InvoiceById>, UnmetObligation> {
        let admitted = InvoiceStorage::list(&self.ports);
        Ok(admitted
            .into_iter()
            .map(|held| crate::invoice::InvoiceById {
                invoice_id: held.data.invoice_id,
                total: held.data.total,
                reminder_count: held.data.reminder_count,
            })
            .collect())
    }
}

/// `billing.invoice.OutstandingInvoices`, generated: every row is one the specification fully determines from the stored `billing.invoice.Invoice`s.
impl<P> crate::invoice::obligations::OutstandingInvoicesQuery for Generated<P>
where
    P: InvoiceStorage,
{
    fn outstanding_invoices(&self) -> Result<Vec<crate::invoice::OutstandingInvoices>, UnmetObligation> {
        let mut admitted = InvoiceStorage::list(&self.ports);
        // `filter:` shows a row where it holds; false or unknown hides it.
        admitted.retain(|held| equal(Some(&held.state).map(|value| match value { crate::invoice::InvoiceState::Cancelled => "Cancelled", crate::invoice::InvoiceState::Draft => "Draft", crate::invoice::InvoiceState::Issued => "Issued", crate::invoice::InvoiceState::Paid => "Paid" }.to_owned()), Some("Issued".to_owned())) == Some(true));
        // `order_by:`, stably: rows equal on every key keep the port's order.
        admitted.sort_by(|left, right| (match (Some(&left.data.issued_at).and_then(|value| value.as_ref()).map(|value| value.0.clone()), Some(&right.data.issued_at).and_then(|value| value.as_ref()).map(|value| value.0.clone())) { (Some(left), Some(right)) => instant_order(&left, &right), (left, right) => left.is_some().cmp(&right.is_some()) }).reverse());
        Ok(admitted
            .into_iter()
            .map(|held| crate::invoice::OutstandingInvoices {
                invoice_id: held.data.invoice_id,
                total: held.data.total,
                issued_at: held.data.issued_at,
            })
            .collect())
    }
}

/// Equality of two read values; an unread one is Unknown.
fn equal<T: PartialEq>(left: Option<T>, right: Option<T>) -> Option<bool> {
    Some(left? == right?)
}

/// The instant an RFC 3339 `date-time` names, as seconds from the epoch and nanoseconds.
fn instant(text: &str) -> Option<(i64, u32)> {
    let bytes = text.as_bytes();
    let digits = |from: usize, to: usize| -> Option<u32> {
        let slice = bytes.get(from..to)?;
        if slice.is_empty() || !slice.iter().all(u8::is_ascii_digit) {
            return None;
        }
        slice
            .iter()
            .try_fold(0u32, |total, digit| Some(total * 10 + u32::from(digit - b'0')))
    };
    let at = |index: usize, expected: &[u8]| bytes.get(index).is_some_and(|b| expected.contains(b));
    if !(at(4, b"-") && at(7, b"-") && at(10, b"Tt") && at(13, b":") && at(16, b":")) {
        return None;
    }
    let (year, month, day) = (i64::from(digits(0, 4)?), digits(5, 7)?, digits(8, 10)?);
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let length = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return None,
    };
    if day < 1 || day > length {
        return None;
    }
    let (hour, minute, second) = (digits(11, 13)?, digits(14, 16)?, digits(17, 19)?);
    if hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    let mut position = 19;
    let mut nanos = 0u32;
    if at(position, b".") {
        let start = position + 1;
        let mut end = start;
        while bytes.get(end).is_some_and(u8::is_ascii_digit) {
            end += 1;
        }
        let width = end - start;
        if width == 0 || width > 9 {
            return None;
        }
        nanos = digits(start, end)? * 10u32.pow(u32::try_from(9 - width).ok()?);
        position = end;
    }
    let offset = match bytes.get(position..)? {
        b"Z" | b"z" => 0i64,
        [sign @ (b'+' | b'-'), _, _, b':', _, _] => {
            let (hours, minutes) = (digits(position + 1, position + 3)?, digits(position + 4, position + 6)?);
            if hours > 23 || minutes > 59 {
                return None;
            }
            let magnitude = i64::from(hours * 3600 + minutes * 60);
            if *sign == b'-' {
                -magnitude
            } else {
                magnitude
            }
        }
        _ => return None,
    };
    let shifted = year - i64::from(month <= 2);
    let era = if shifted >= 0 { shifted } else { shifted - 399 } / 400;
    let year_of_era = shifted - era * 400;
    let month = i64::from(month);
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    let days = era * 146_097 + day_of_era - 719_468;
    Some((
        days * 86_400 + i64::from(hour * 3600 + minute * 60 + second) - offset,
        nanos,
    ))
}

/// Two RFC 3339 renderings by the instant each names; text that names none by its bytes.
fn instant_order(left: &str, right: &str) -> core::cmp::Ordering {
    match (instant(left), instant(right)) {
        (Some(left), Some(right)) => left.cmp(&right),
        _ => left.as_bytes().cmp(right.as_bytes()),
    }
}
