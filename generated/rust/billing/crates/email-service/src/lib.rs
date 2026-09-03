// generated from billing v3
// model digest 8b52fe739078f96a006d7bee5e9b9530c3a30221f7bc003f291dcfe17cdcfea3
// contract digest 0c2f2067136aea0bc0a45ca5b01bf70f551fc6199956699c5d5b939c350688f8
// do not edit: regenerate with `ess synthesize`

//! email-service — the `email-service` component of `billing` v3.
//!
//! Sends what other contexts ask it to send.
//!
//! The component's outer surface exactly as the specification declares it: accepted commands as
//! handlers, declared views as queries, published events as a typed outbox. The behaviour behind
//! every handler is an implementation obligation — see the `PLAN.md` beside this workspace — and
//! until one is satisfied, its stub answers with a typed refusal naming what is owed.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// An event this component declares it publishes, on its way to the system's transport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublishedEvent {
    /// `billing.email.DeliveryEscalated`.
    DeliveryEscalated(billing_types::email::DeliveryEscalated),
    /// `billing.email.EmailSent`.
    EmailSent(billing_types::email::EmailSent),
}

/// email-service — the port over the component's obligations.
///
/// `B` bundles every behaviour and query this component owes; constructing it over the domain's
/// `obligations::Unimplemented` yields a component that compiles and refuses, in the type system,
/// everything not yet implemented.
pub struct EmailService<B> {
    behaviors: B,
    outbox: Vec<PublishedEvent>,
}

impl<B> EmailService<B> {
    /// A new port over the given obligation implementations.
    pub fn new(behaviors: B) -> Self {
        Self {
            behaviors,
            outbox: Vec::new(),
        }
    }

    /// Hands over everything published since the last drain, in publication order.
    ///
    /// The system's transport calls this; anything else reading it is taking events the transport
    /// will then never deliver.
    pub fn drain_outbox(&mut self) -> Vec<PublishedEvent> {
        core::mem::take(&mut self.outbox)
    }
}

impl<B> EmailService<B>
where
    B: billing_types::email::obligations::SendEmailBehavior,
{
    /// Accepts `billing.email.SendEmail`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn send_email(&mut self, input: billing_types::email::SendEmail) -> Result<billing_types::email::SendEmailOutcome, billing_types::obligation::UnmetObligation> {
        let outcome = self.behaviors.send_email(input)?;
        match &outcome {
            billing_types::email::SendEmailOutcome::Sent { email_sent, .. } => {
                self.outbox.push(PublishedEvent::EmailSent(email_sent.clone()));
            }
            billing_types::email::SendEmailOutcome::Failed { .. } => {}
        }
        Ok(outcome)
    }
}
