//! Implementations that are wrong in exactly one way, and where each one is caught.
//!
//! Design §25 and §26. A generated suite that passes against a correct implementation has shown one
//! thing only: that it asks for nothing a correct implementation cannot answer. It has shown nothing
//! at all about whether it would notice a wrong one, and a suite whose failures have never been
//! demonstrated is indistinguishable from a suite that checks nothing.
//!
//! So this module ships the other half of the evidence: a [`Fault`], a [`Faulty`] wrapper that
//! injects one, and — in `tests/faults.rs` — the matrix that asserts, per fault, **which named
//! scenario fails** and **how many unrelated ones still pass**. §25 is explicit that a generic panic
//! failing everything proves nothing, which is why both halves are asserted rather than the first.
//!
//! `aep_conformance::faulty` did this for AEP backends and is the precedent §25 names. What is
//! carried over unchanged is the shape: one `#[non_exhaustive]` enum, an `ALL` constant generated
//! from the same lines as the variants, one designated check per fault, and a blast-radius allowance
//! per fault that has to be updated *with a reason* rather than relaxed.
//!
//! # A fault caught by nothing is the finding, and every one recorded here has since been closed
//!
//! [`Caught::Nothing`] is not a gap in this module. A fault nobody catches is worth more than
//! another passing row: it says the specification cannot express the property, or that synthesis
//! does not ask for it. Three rows sat here; all three have since been closed and moved to the
//! other side of the matrix rather than being deleted — two by teaching synthesis to ask for more,
//! and the last by changing the **model**:
//!
//! | fault | what a client would see | what closed it |
//! |---|---|---|
//! | [`ExtraEvent`](Fault::ExtraEvent) | a cancelled invoice is announced as paid | every event the specification declares and this branch does not emit is now asserted absent |
//! | [`DropConsistencyToken`](Fault::DropConsistencyToken) | a `read_your_writes` view is never actually held to it | a read with no token to demand is `unsupported`, not a weaker read at `Current` |
//! | [`WrongEventPayload`](Fault::WrongEventPayload) | every consumer of `InvoicePaid` records an amount nobody paid | an outcome's `payload:` now says which input determines which event field, so the value stopped being a guess and became a reading |
//!
//! The last of the three sat here longest because it needed the **model** to change rather than
//! the synthesizer: `999` is a well-formed `Money` in a field the event declares, and until wave
//! 6.5 nothing in the model said where a payload field's *value* comes from — asserting
//! `InvoicePaid.amount == PayInvoice.amount` would have been a match on a shared field name, the
//! inference this crate refuses everywhere else. The `payload:` declaration on a command outcome
//! is the construct that licenses it, and [`PartialEventPayload`](Fault::PartialEventPayload)
//! remains the other half of the same coin: a *type* was always declared, so a declared field left
//! out was already caught before any value could be.
//!
//! An event field with **no** declared source stays undetermined — `InvoiceCreated.invoice_id` is
//! the implementation's to mint — and the suite shows that by asserting its presence and type and
//! never a value. A fault perturbing only an undetermined field would still be caught by nothing,
//! and that is the specification's decision rather than a gap here.
//!
//! # Boundary or implementation
//!
//! Fifteen of the seventeen are [`Injection::Boundary`]: [`Faulty`] perturbs what goes into the
//! target and what comes out of it, and never a broken internal, because that is the same position
//! a real client is in.
//!
//! Two cannot be. [`DropBinding`](Fault::DropBinding) and [`WrongMapping`](Fault::WrongMapping) are
//! defects *of a binding*, and the target interface deliberately does not attribute an observation
//! to the binding that caused it — an [`ObservedEvent`] carries no transport metadata (§41), so a
//! wrapper filtering `oracle.dispatch.HandedOff` out of `observe_events` would silence all three of
//! the oracle's bindings at once, not the one under test. Simulating them in the one observation
//! that *is* attributed — [`ObservedInvocation`] — would prove that the suite catches a **lie about**
//! a mapping while the system still mapped correctly, which is a different and much weaker claim. So
//! those two are injected in [`Oracle`], as
//! [`with_binding_dropped`](Oracle::with_binding_dropped) and
//! [`with_mapping_swapped`](Oracle::with_mapping_swapped), and [`Fault::injection`] says which of
//! the two mechanisms each fault uses so the split is a property of the type rather than a
//! paragraph.
//!
//! # A fault only an interleaving shows
//!
//! [`LostUpdate`](Fault::LostUpdate) is the row no single-client check can catch, and it is
//! [`Caught::ByHistory`] rather than [`Caught::By`] a scenario. A lifecycle command reads the
//! invoice when the call is invoked and writes when it returns, with no lock between the two
//! ([`record::Interleaved`](crate::record::Interleaved)). Run one call at a time — which is what
//! every suite does — nothing happens between the read and the write, and the answer is the
//! reference's own. Two clients paying one issued invoice at once both read `Issued`, and both
//! are told `settled`; the second payment is never applied. Only a recorded concurrent history,
//! checked by [`crate::linearize`], shows that no order of the two calls answers both.
//!
//! [`StaleReadUnderReadYourWrites`](Fault::StaleReadUnderReadYourWrites) is the second, and the
//! one a view's declared consistency decides. `OutstandingInvoices` is `read_your_writes`, and the
//! fault answers a demanding read from a copy refreshed only by a read that demands the newest
//! write. One client's demand is always the newest, so the suite reads fresh; with two clients, a
//! client whose read demands its own write after the other client wrote gets a copy from before
//! either. The history check judges the read per client session and names the client and the
//! read. [`StaleReadYourWrites`](Fault::StaleReadYourWrites) stays its own row: it is one read
//! behind for one client, and the suite catches it.
//!
//! # Faults only a declared fault shows
//!
//! Two rows are [`Caught::ByInjection`]: no scenario catches them, and neither does a concurrent
//! history recorded without injection, because the defect waits for a fault the specification
//! declares — and [`sessions::record_injected`] injects exactly those.
//!
//! [`DoubleApplyOnRedelivery`](Fault::DoubleApplyOnRedelivery) waits for a second delivery.
//! `notify-on-invoice-created` declares `delivery: at_least_once`, and this implementation, handed
//! `InvoiceCreated` a second time, runs the creation it announces again: a second invoice nobody
//! asked for. Its only witness is a read of `InvoiceById` listing an invoice no recorded call
//! created, which the history check names a future read.
//!
//! [`RetryCreatesSecondEntity`](Fault::RetryCreatesSecondEntity) waits for a client retry.
//! `retry.core.Seed` declares `replayed` with `replays: seeded`, and this implementation looks a
//! request up when it arrives but retains it only once answered. A retry after the answer is
//! replayed; a retry overlapping its original is applied as a new request, and one request answers
//! `seeded` twice ([`crate::linearize`]).
//!
//! # Nothing here is a new source of variation
//!
//! §37 gives the runner the clock and the id source, and a faulty target that reached for either
//! would make the matrix flaky and therefore worthless. Every fault below is a pure function of what
//! it was given, plus — for [`StaleReadYourWrites`](Fault::StaleReadYourWrites) — the previous
//! answer to the same view in the same scenario, which is reset by
//! [`begin_scenario`](ConformanceTarget::begin_scenario).

use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_primitives::consistency::ConsistencyToken;
use ess_primitives::facts::Number;
use ess_primitives::ids::CorrelationId;
use ess_primitives::node::Node;

use crate::record::{Call, Interleaved, Subject, Workload};
use crate::reference::{
    Billing, Oracle, Retained, CANCEL_INVOICE, CREATE_INVOICE, INVALID_AMOUNT, INVOICE_BY_ID,
    INVOICE_CANCELLED, INVOICE_CREATED, INVOICE_ISSUED, INVOICE_PAID, ISSUE_INVOICE, OUTSTANDING,
    PAY_INVOICE, SEED,
};
use crate::scenario::{ErrorRef, EventRef, OutcomeRef, ViewRef};
use crate::sessions::{self, Act};
use crate::target::{
    ConformanceTarget, DeclaredErrorValue, EventObservationRequest, ExternalOutcomeControl,
    ImplementationIdentity, InvocationObservationRequest, ObservedEvent, ObservedInvocation,
    RedeliveryRequest, ScenarioContext, SemanticCommandRequest, SemanticCommandResult,
    SemanticViewRequest, SemanticViewResult, TargetError,
};

// ---- the vocabulary --------------------------------------------------------------------------

/// Which specification a fault is injected into, and therefore which suite catches it.
///
/// Two, because one of them cannot make §26's second claim. `examples/billing/` declares a single
/// binding, so a binding that stops running fails every binding scenario there is; the oracle
/// fixture declares three, on three events, and its `README.md` says in as many words that this is
/// what it is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum System {
    /// `examples/billing/`, the normative example.
    Billing,
    /// `examples/oracle-fixture/`, the fixture built for the checks billing cannot make fail.
    Oracle,
    /// `crates/verify/ess-conformance/tests/fixtures/explore-retry/`: one command declaring
    /// `replays:`, which neither example declares and which a client retry is owed only by.
    Retry,
}

impl System {
    /// The directory that holds it: under `examples/` for the two examples, and the fixture's own
    /// directory name for [`System::Retry`] — [`path`](Self::path) says where each is.
    pub fn directory(self) -> &'static str {
        match self {
            Self::Billing => "billing",
            Self::Oracle => "oracle-fixture",
            Self::Retry => "explore-retry",
        }
    }

    /// The directory that holds it, from the workspace root.
    pub fn path(self) -> &'static str {
        match self {
            Self::Billing => "examples/billing",
            Self::Oracle => "examples/oracle-fixture",
            Self::Retry => "crates/verify/ess-conformance/tests/fixtures/explore-retry",
        }
    }
}

/// Where a fault is injected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Injection {
    /// In [`Faulty`], which perturbs only what goes in and what comes out.
    Boundary,
    /// In the reference implementation, because the boundary cannot express it.
    ///
    /// See the [module documentation](self): the two that need this are defects of a *binding*, and
    /// the target interface does not attribute an observation to the binding that produced it.
    Implementation,
}

/// What a suite catches a fault with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Caught {
    /// The one scenario that exists to catch it, by id.
    By(&'static str),
    /// Nothing catches it, and why — see the [module documentation](self).
    Nothing(&'static str),
    /// No scenario catches it, and a recorded concurrent history checked by
    /// [`crate::linearize`] does; the text says why only an interleaving shows it.
    ByHistory(&'static str),
    /// No scenario catches it, and no concurrent history recorded without injection does: only a
    /// history recorded with the faults the specification declares injected
    /// ([`sessions::record_injected`]), checked by [`crate::linearize`]. The text says which
    /// declared fault it takes.
    ByInjection(&'static str),
}

impl Caught {
    /// The scenario that catches it, where one does.
    pub fn scenario(self) -> Option<&'static str> {
        match self {
            Self::By(scenario) => Some(scenario),
            Self::Nothing(_) | Self::ByHistory(_) | Self::ByInjection(_) => None,
        }
    }
}

/// Declares one fault per line, and generates the list a matrix walks from the same lines.
///
/// Hand-maintaining `ALL` beside the enum is what `ess_primitives`'s `validation_codes!` macro exists to
/// stop, after five codes had fallen out of such a list. The same argument applies with more force
/// here: a fault missing from `ALL` is a row the matrix silently does not run, which is exactly the
/// silent omission this whole slice is about.
macro_rules! faults {
    ($(
        $(#[$attribute:meta])*
        $variant:ident => $written:literal, $system:expr, $injection:expr, $caught:expr, $describe:literal;
    )*) => {
        /// One way an implementation of a specification can be wrong.
        ///
        /// Seven are design §25's fault table. Five more were gone looking for, three of which
        /// nothing caught when they were written down — see the [module documentation](self) for
        /// which of those are closed and which one still needs the model to change.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[non_exhaustive]
        pub enum Fault {
            $( $(#[$attribute])* $variant, )*
        }

        impl Fault {
            /// Every fault the matrix injects, in declaration order.
            ///
            /// Generated, so it cannot fall behind the enum — which is what makes the matrix a
            /// matrix rather than a list somebody has to remember to extend (§25).
            pub const ALL: &'static [Self] = &[ $( Self::$variant, )* ];

            /// How it is written, in a report and in the identity a faulty target answers with.
            pub fn written(self) -> &'static str {
                match self { $( Self::$variant => $written, )* }
            }

            /// Which specification it is injected into.
            pub fn system(self) -> System {
                match self { $( Self::$variant => $system, )* }
            }

            /// Whether it is injected at the boundary or in the implementation, and it is the
            /// [module documentation](self) that argues why for the two that are not at the boundary.
            pub fn injection(self) -> Injection {
                match self { $( Self::$variant => $injection, )* }
            }

            /// The scenario that exists to catch it, or why nothing does.
            pub fn caught(self) -> Caught {
                match self { $( Self::$variant => $caught, )* }
            }

            /// What goes wrong, in one line, for a report.
            pub fn describe(self) -> &'static str {
                match self { $( Self::$variant => $describe, )* }
            }
        }
    };
}

faults! {
    /// `F-WRONG-EVENT`: a created invoice is announced as an issued one.
    WrongEvent => "wrong-event", System::Billing, Injection::Boundary,
        Caught::By("billing.invoice.CreateInvoice/outcome/accepted"),
        "a branch reports an event it does not declare it emits in place of the one it does";

    /// `F-REJECTION`: an amount the guard refuses is accepted anyway.
    AcceptInvalidAmount => "accept-invalid-amount", System::Billing, Injection::Boundary,
        Caught::By("billing.invoice.CreateInvoice/outcome/rejected"),
        "an input that satisfies no branch's guard is accepted by the guarded one";

    /// `F-ILLEGAL-TRANSITION`: a move the lifecycle does not declare is honoured.
    AllowIllegalTransition => "allow-illegal-transition", System::Billing, Injection::Boundary,
        Caught::By("billing.invoice.Invoice/state/Paid/refuses/billing.invoice.CancelInvoice"),
        "a command moves an entity from a state its transition does not run from";

    /// A wrong-state refusal reports the wrong declared error.
    ///
    /// Not one of §25's seven, and it was uncatchable until the model could say what a command
    /// answers in a state its moves do not start from: nothing moved, nothing was published, and the
    /// only thing wrong is the name of the error — so every assertion the suite could previously
    /// make passed. It is the fault that says the `wrong_state:` branch bought a real check rather
    /// than a tidier document.
    WrongRefusalError => "wrong-refusal-error", System::Billing, Injection::Boundary,
        Caught::By("billing.invoice.Invoice/state/Paid/refuses/billing.invoice.IssueInvoice"),
        "a command refused for the right reason names the wrong declared error";

    /// `F-DROPPED-BINDING`: an event reaches nothing.
    DropBinding => "drop-binding", System::Oracle, Injection::Implementation,
        Caught::By("handoff-on-placed/binding/flow"),
        "a declared binding never runs, so the consequence it promises never happens";

    /// `F-WRONG-MAPPING`: a binding fills an input from the wrong field of the event.
    WrongMapping => "wrong-mapping", System::Oracle, Injection::Implementation,
        Caught::By("handoff-on-placed/binding/mapping"),
        "a binding maps `alternate_contact` where the specification writes `contact`";

    /// `F-VIEW-RACE`: a read that demands the write it was told about is answered from before it.
    StaleReadYourWrites => "stale-read-your-writes", System::Billing, Injection::Boundary,
        Caught::By("billing.invoice.IssueInvoice/outcome/issued"),
        "a read_your_writes view answers from before the command that just returned";

    /// `F-EXTERNAL-OUTCOME`: a forced failure is reported as a success.
    IgnoreExternalOutcome => "ignore-external-outcome", System::Billing, Injection::Boundary,
        Caught::By("billing.email.SendEmail/outcome/failed"),
        "an externally decided failure is ignored and the command reports success";

    /// A published event carries a value the command was never given.
    ///
    /// Not one of §25's seven, and the row that was caught by nothing the longest — until wave 6.5
    /// gave the model a `payload:` declaration on a command outcome. Every field
    /// `billing.invoice.InvoicePaid` declares is present and every one is of its declared type —
    /// `999` is as well-formed a `Money` as the amount that was submitted — so the only check that
    /// can see it is `InvoicePaid.amount == PayInvoice.amount`, and that check is licensed exactly
    /// where the specification declares `amount: input.amount`. `examples/billing/` declares it on
    /// `settled`, so both scenarios that exercise the branch now assert the value; the matrix
    /// designates the *transition* scenario because §10's outcome scenario already designates
    /// [`PartialEventPayload`](Fault::PartialEventPayload), and one scenario names one fault.
    WrongEventPayload => "wrong-event-payload", System::Billing, Injection::Boundary,
        Caught::By("billing.invoice.Invoice/transition/settle/by/billing.invoice.PayInvoice/settled"),
        "a published event carries an amount the command was never given";

    /// A published event leaves out a field its declaration says it carries.
    ///
    /// The other half of [`WrongEventPayload`](Fault::WrongEventPayload), and the half the model
    /// does license: `billing.invoice.InvoicePaid` declares `invoice_id` and `amount`, so an
    /// occurrence carrying only the first contradicts the specification without any claim about
    /// where a value comes from. It is here so that "presence and type are asserted" is a row in the
    /// matrix rather than a sentence in a doc comment.
    PartialEventPayload => "partial-event-payload", System::Billing, Injection::Boundary,
        Caught::By("billing.invoice.PayInvoice/outcome/settled"),
        "a published event leaves out a field its declaration says it carries";

    /// A branch publishes an event it does not declare it emits, beside the ones it does.
    ///
    /// Not one of §25's seven. It was uncaught while `ExpectNoEvent` was synthesised only for the
    /// events a *sibling* branch emits; the rule `ESS-CF-NO-EVENT` names is wider than that, and
    /// synthesis now asks it of every event the specification declares and this branch does not.
    ExtraEvent => "extra-event", System::Billing, Injection::Boundary,
        Caught::By("billing.invoice.CancelInvoice/outcome/cancelled"),
        "cancelling an invoice also announces that it was paid";

    /// A command answers without the token a later read would demand.
    ///
    /// Not one of §25's seven. It was uncaught while the runner fell back to `Current` with no token
    /// in hand — a weaker read that passes, which is the "skip that looks like a pass". A
    /// read-your-writes read that cannot be demanded is now `unsupported`, which fails the run.
    DropConsistencyToken => "drop-consistency-token", System::Billing, Injection::Boundary,
        Caught::By("billing.invoice.Invoice/transition/issue/by/billing.invoice.IssueInvoice/issued"),
        "a command returns no consistency token, so no read is ever held to it";

    /// A projection publishes a value the type it holds forbids.
    ///
    /// The wave 6.5 value-object family's row. `OutstandingInvoices.total` is a `Money` and `Money`
    /// declares `amount >= 0` of every value; a projection that corrupts what it publishes breaks
    /// that claim with **no command having done anything wrong** — the write side is untouched, so
    /// every outcome, transition and refusal scenario passes, and the only checks positioned to see
    /// it are the two that read a value object's own invariants off the field positions that hold
    /// one. The designated scenario is the position the fault corrupts; the check at
    /// `InvoiceById.total` stays green, which is exactly why two positions are two scenarios.
    NegativeProjectedTotal => "negative-projected-total", System::Billing, Injection::Boundary,
        Caught::By("billing.invoice.Money/invariant/at/billing.invoice.OutstandingInvoices/total"),
        "the outstanding list reports a total below zero, which no Money admits";

    /// A lifecycle command reads the invoice at invoke and writes at return, with no lock.
    ///
    /// The row the concurrent history check exists for. One call at a time, the read and the write
    /// are adjacent and the answer is the reference's; two calls on one invoice at once both
    /// decide from the state before either wrote, and the one whose write finds the invoice moved
    /// is still told it succeeded. See the [module documentation](self).
    LostUpdate => "lost-update", System::Billing, Injection::Boundary,
        Caught::ByHistory(
            "every suite scenario runs one call at a time, so nothing ever happens between the \
             read and the write; only two clients' calls on one invoice, overlapping, show two \
             successes no order of the calls allows"
        ),
        "a lifecycle command decides from the state it read at invoke and is told success after \
         a concurrent write made that decision stale";

    /// `OutstandingInvoices` answers a read-your-writes read from a lagged copy.
    ///
    /// Not [`StaleReadYourWrites`](Fault::StaleReadYourWrites), which answers every demanding
    /// read one read behind and which one client already sees. This copy is refreshed by a read
    /// demanding the **newest** write anybody made, and a read demanding an older one is answered
    /// from the copy as it stands — taking "older than the newest" for "already in the copy". One
    /// client's reads always demand its own last write, which is the newest there is, so every
    /// suite scenario reads fresh. Two clients are needed: one issues an invoice, the other
    /// issues one after it, and the first client's read is answered from a copy that has neither.
    /// See the [module documentation](self).
    StaleReadUnderReadYourWrites => "stale-read-under-read-your-writes", System::Billing,
        Injection::Boundary,
        Caught::ByHistory(
            "every suite scenario is one client, so the token its read demands is always the \
             newest write and the copy is refreshed; only a second client writing between one \
             client's write and its read leaves that read answered from before its own write"
        ),
        "a read_your_writes view answers a client's read from a copy that predates the client's \
         own write, whenever another client wrote since";

    /// A second delivery of `InvoiceCreated` is applied as a second creation.
    ///
    /// `notify-on-invoice-created` declares `delivery: at_least_once`, so the transport may deliver
    /// one `InvoiceCreated` twice and the binding's `SendEmail` must survive it — which it does.
    /// This implementation also re-applies the creation the event announces, as a handler that is
    /// not idempotent does: a second invoice, with an identity no client was ever told about. The
    /// suite's one second delivery (`notify-on-invoice-created/binding/delivery`) asserts only that
    /// the mail still goes out, and no concurrent history without injection delivers anything
    /// twice. See the [module documentation](self).
    DoubleApplyOnRedelivery => "double-apply-on-redelivery", System::Billing, Injection::Boundary,
        Caught::ByInjection(
            "only a second delivery of `InvoiceCreated`, which `delivery: at_least_once` declares, \
             creates the second invoice, and only a later read of `InvoiceById` shows a row no \
             client's call created; the suite's second delivery reads nothing after it"
        ),
        "a redelivered InvoiceCreated is applied again, creating an invoice nobody asked for";

    /// A retry of `retry.core.Seed` that arrives while the first is in flight creates a second
    /// record.
    ///
    /// `replayed` declares `replays: seeded`: the same request answered again is answered from what
    /// was retained. This implementation looks the request up when it arrives, and retains it only
    /// when it has been answered, so a retry arriving after its original answered is replayed —
    /// every suite scenario, one call at a time, sees that — and one arriving while the original is
    /// still in flight is applied as a new request. See the [module documentation](self).
    RetryCreatesSecondEntity => "retry-creates-second-entity", System::Retry, Injection::Boundary,
        Caught::ByInjection(
            "only a client retry, which `replays:` declares, sends one request twice, and only a \
             retry overlapping its original finds nothing retained yet; the suite retries after \
             the original answered"
        ),
        "a request sent again while the first is in flight creates a second record instead of \
         replaying the first";
}

// ---- the wrapper -----------------------------------------------------------------------------

/// A target wrapped so that exactly one property fails to hold.
///
/// It perturbs what goes in and what comes out, and never an internal — see the
/// [module documentation](self) for why, and for the two faults that cannot be injected this way.
#[derive(Debug)]
pub struct Faulty<T> {
    inner: T,
    fault: Fault,
    memory: RefCell<Vec<(ViewRef, SemanticViewResult)>>,
    /// Every command the target underneath has executed in this scenario, in order: what a read
    /// at an earlier instant replays ([`Fault::LostUpdate`]).
    applied: RefCell<Vec<SemanticCommandRequest>>,
    /// The newest consistency token any command answered with in this scenario
    /// ([`Fault::StaleReadUnderReadYourWrites`]).
    newest: RefCell<Option<ConsistencyToken>>,
    /// The lagged copy of `OutstandingInvoices` ([`Fault::StaleReadUnderReadYourWrites`]).
    copy: RefCell<SemanticViewResult>,
}

impl<T> Faulty<T> {
    /// Wraps `inner` so that `fault` is injected.
    pub fn new(inner: T, fault: Fault) -> Self {
        Self {
            inner,
            fault,
            memory: RefCell::new(Vec::new()),
            applied: RefCell::new(Vec::new()),
            newest: RefCell::new(None),
            copy: RefCell::new(SemanticViewResult::default()),
        }
    }

    /// Which fault this target carries.
    pub fn fault(&self) -> Fault {
        self.fault
    }

    /// The target underneath.
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Records the fresh answer to `view` and returns the one before it.
    ///
    /// A projection exactly one read behind, which is what "stale" means here: not empty forever,
    /// but always answering the question before the one that was asked. Deterministic, and reset per
    /// scenario, so two runs of the matrix agree.
    fn one_read_behind(&self, view: &ViewRef, fresh: SemanticViewResult) -> SemanticViewResult {
        let mut memory = self.memory.borrow_mut();
        if let Some((_, previous)) = memory.iter_mut().find(|(known, _)| known == view) {
            return std::mem::replace(previous, fresh);
        }
        // The first read of a scenario has nothing behind it, and an empty view is what a projection
        // that has not caught up holds.
        memory.push((view.clone(), fresh));
        SemanticViewResult::default()
    }
}

/// The billing reference, wrong in exactly one way.
///
/// # Panics
///
/// If `fault` is not one of `System::Billing`'s, which is a defect in the caller rather than a
/// condition: a fault names the specification it belongs to, and injecting an oracle fault into
/// billing would produce a green run that proves nothing.
pub fn billing(fault: Fault) -> Faulty<Billing> {
    assert_eq!(
        fault.system(),
        System::Billing,
        "{fault:?} is a fault of `{}`, not of billing",
        fault.system().directory()
    );
    Faulty::new(Billing::new(), fault)
}

/// The oracle reference, wrong in exactly one way — including the two the boundary cannot express.
///
/// # Panics
///
/// If `fault` is not one of `System::Oracle`'s. See [`billing`].
pub fn oracle(fault: Fault) -> Faulty<Oracle> {
    assert_eq!(
        fault.system(),
        System::Oracle,
        "{fault:?} is a fault of `{}`, not of the oracle fixture",
        fault.system().directory()
    );
    let reference = match fault {
        Fault::DropBinding => Oracle::new().with_binding_dropped(Oracle::HANDOFF_ON_PLACED),
        Fault::WrongMapping => Oracle::new().with_mapping_swapped(Oracle::HANDOFF_ON_PLACED),
        _ => Oracle::new(),
    };
    Faulty::new(reference, fault)
}

/// The retry fixture's reference, wrong in exactly one way.
///
/// # Panics
///
/// If `fault` is not one of `System::Retry`'s. See [`billing`].
pub fn retry(fault: Fault) -> Faulty<Retained> {
    assert_eq!(
        fault.system(),
        System::Retry,
        "{fault:?} is a fault of `{}`, not of the retry fixture",
        fault.system().directory()
    );
    Faulty::new(Retained::new(), fault)
}

impl<T: ConformanceTarget> ConformanceTarget for Faulty<T> {
    /// The implementation underneath, named for the defect it carries.
    ///
    /// A report that said `billing-reference` for a build that is deliberately wrong would attest
    /// the opposite of what happened (§30).
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        let inner = self.inner.identity()?;
        Ok(ImplementationIdentity::new(
            format!("{}-{}", inner.name, self.fault.written()),
            inner.version,
        ))
    }

    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        // Isolation covers the fault's own memory too, or a stale answer from the previous scenario
        // would be a second, undeclared defect (§8).
        self.memory.borrow_mut().clear();
        self.applied.borrow_mut().clear();
        *self.newest.borrow_mut() = None;
        *self.copy.borrow_mut() = SemanticViewResult::default();
        self.inner.begin_scenario(scenario)
    }

    fn execute_command(
        &self,
        mut request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command = request.command.to_string();
        if self.fault == Fault::AcceptInvalidAmount && command == CREATE_INVOICE {
            // On the way in, as `aep_conformance`'s `ReplayApplies` rewrites an idempotency key: the
            // client submitted an amount the specification refuses and got an invoice.
            if let Some(amount) = request.input.get_mut("amount") {
                *amount = money(1.0);
            }
        }
        let input = request.input.clone();
        let logged = request.clone();
        let mut result = self.inner.execute_command(request)?;
        self.applied.borrow_mut().push(logged);
        if let Some(token) = &result.consistency {
            *self.newest.borrow_mut() = Some(token.clone());
        }

        match self.fault {
            Fault::WrongEvent if command == CREATE_INVOICE => {
                for occurrence in &mut result.direct_events {
                    if occurrence.event == event(INVOICE_CREATED) {
                        occurrence.event = event(INVOICE_ISSUED);
                    }
                }
            }
            Fault::WrongEventPayload if command == PAY_INVOICE => {
                for occurrence in &mut result.direct_events {
                    if occurrence.event == event(INVOICE_PAID) {
                        occurrence.payload.insert("amount".to_owned(), money(999.0));
                    }
                }
            }
            Fault::PartialEventPayload if command == PAY_INVOICE => {
                for occurrence in &mut result.direct_events {
                    if occurrence.event == event(INVOICE_PAID) {
                        occurrence.payload.remove("amount");
                    }
                }
            }
            Fault::ExtraEvent if command == CANCEL_INVOICE => {
                // Named after the invoice that was just cancelled, so it is the plausible version of
                // this defect rather than an obviously stray message — and it is the dangerous one:
                // a consumer that hears `InvoicePaid` stops chasing an invoice nobody paid.
                let identity = input.get("invoice_id").cloned().unwrap_or_default();
                result
                    .direct_events
                    .push(ObservedEvent::new(event(INVOICE_PAID)).with("invoice_id", identity));
            }
            Fault::AllowIllegalTransition
                if command == CANCEL_INVOICE && refused_for_state(&result, CANCEL_INVOICE) =>
            {
                // The reference answers the declared `wrong-state` branch for a move its lifecycle
                // does not run; this reports that refusal as the success it was not, drops the error
                // that named it, and publishes what the move would have published.
                result.outcome = Some(cancelled());
                result.error = None;
                result
                    .direct_events
                    .push(ObservedEvent::new(event(INVOICE_CANCELLED)).with(
                        "invoice_id",
                        input.get("invoice_id").cloned().unwrap_or_default(),
                    ));
            }
            Fault::WrongRefusalError
                if command == ISSUE_INVOICE && refused_for_state(&result, ISSUE_INVOICE) =>
            {
                // The branch is right and the error is not. Nothing observable changed — no event
                // was published and no state moved — so the only thing that distinguishes this from
                // a correct implementation is the error the refusal names, which is exactly what
                // `wrong_state:` put into the model.
                result.error = Some(DeclaredErrorValue::new(declared_error(INVALID_AMOUNT)));
            }
            Fault::DropConsistencyToken => result.consistency = None,
            _ => {}
        }
        Ok(result)
    }

    /// Forwarded unchanged: no fault this wrapper carries is about a request with no input.
    fn execute_command_without_input(
        &self,
        request: crate::target::AbsentInputRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inner.execute_command_without_input(request)
    }

    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let view = request.view.clone();
        // Only a read that *demanded* the write is perturbed: an eventual read asks at `Current` and
        // is allowed to be behind, so answering it late would be conformant rather than faulty.
        let demanded = request.consistency.token().cloned();
        let mut fresh = self.inner.query_view(request)?;
        if self.fault == Fault::StaleReadYourWrites && demanded.is_some() {
            return Ok(self.one_read_behind(&view, fresh));
        }
        if self.fault == Fault::StaleReadUnderReadYourWrites && view.to_string() == OUTSTANDING {
            if let Some(token) = demanded {
                // Refreshed only by a read demanding the newest write; any other demand is taken
                // to be covered by the copy, which is the defect.
                if self.newest.borrow().as_ref() == Some(&token) {
                    self.copy.borrow_mut().clone_from(&fresh);
                    return Ok(fresh);
                }
                return Ok(self.copy.borrow().clone());
            }
        }
        if self.fault == Fault::NegativeProjectedTotal && view.to_string() == OUTSTANDING {
            // One view, every row, one field: the projection's own corruption, not the write's.
            // `InvoiceById` answers untouched, which is what pins the two positions apart.
            for row in &mut fresh.rows {
                if let Some(Node::Map(fields)) = row.get_mut("total") {
                    fields.insert(
                        "amount".to_owned(),
                        Node::Number(
                            Number::new(-1.0)
                                .unwrap_or_else(|error| panic!("-1 is finite: {error}")),
                        ),
                    );
                }
            }
        }
        Ok(fresh)
    }

    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }

    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        if self.fault == Fault::IgnoreExternalOutcome {
            // Accepted and ignored, which is the defect: a target that *refused* the control would
            // be reported as `error` and would be telling the truth about itself.
            return Ok(());
        }
        self.inner.configure_external_outcome(request)
    }

    fn configure_external_outcome_repeatedly(
        &self,
        request: ExternalOutcomeControl,
        times: std::num::NonZeroU32,
    ) -> Result<(), TargetError> {
        if self.fault == Fault::IgnoreExternalOutcome {
            // The same defect as above, for the repeated control.
            return Ok(());
        }
        self.inner
            .configure_external_outcome_repeatedly(request, times)
    }

    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        let redelivered = request.event.clone();
        self.inner.redeliver_event(request)?;
        if self.fault == Fault::DoubleApplyOnRedelivery && redelivered == event(INVOICE_CREATED) {
            // The binding ran again, as `at_least_once` allows; the defect is that the creation
            // the event announces is applied again too.
            let creation = self
                .applied
                .borrow()
                .iter()
                .rev()
                .find(|earlier| earlier.command.to_string() == CREATE_INVOICE)
                .cloned();
            if let Some(creation) = creation {
                self.inner.execute_command(creation)?;
            }
        }
        Ok(())
    }

    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        self.inner.observe_invocations(request)
    }

    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
}

// ---- invoke and return -----------------------------------------------------------------------

/// A [`Faulty`] billing call between its invoke and its return.
#[derive(Debug)]
pub struct InFlight {
    request: SemanticCommandRequest,
    /// What the call decided from the state it read at invoke, where the fault reads early.
    read: Option<Result<SemanticCommandResult, TargetError>>,
    /// Whether nothing had been answered for this request when it arrived, where the fault looks
    /// retained requests up early ([`Fault::RetryCreatesSecondEntity`]).
    unretained: bool,
}

impl Interleaved for Faulty<Billing> {
    type Pending = InFlight;

    /// Every fault but [`Fault::LostUpdate`] does all of its work at the return instant.
    /// `LostUpdate` reads at the invoke instant too: it decides a lifecycle command's answer from
    /// the invoice as it stands now.
    fn invoke(&self, request: SemanticCommandRequest) -> InFlight {
        let read = (self.fault == Fault::LostUpdate && reads_then_writes(&request))
            .then(|| self.read_now(&request));
        InFlight {
            request,
            read,
            unretained: false,
        }
    }

    /// The write, at the return instant, against the invoice as it stands then. Where the answer
    /// decided at invoke differs from the one the write came to, the client is told the one
    /// decided at invoke — the check it was based on is the one no lock kept true.
    ///
    /// With nothing between the invoke and the return, the two answers are the same, so a call
    /// run on its own is answered exactly as the reference answers it.
    fn complete(&self, pending: InFlight) -> Result<SemanticCommandResult, TargetError> {
        let written = self.execute_command(pending.request)?;
        match pending.read {
            Some(Ok(read)) if read.outcome != written.outcome => Ok(read),
            _ => Ok(written),
        }
    }
}

impl Faulty<Billing> {
    /// The answer `request` gets from the invoices as they stand now, without writing anything:
    /// the commands executed so far, replayed on a fresh reference, and then this one.
    fn read_now(
        &self,
        request: &SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let snapshot = Billing::new();
        for earlier in self.applied.borrow().iter() {
            // Replayed for the state it leaves; what each answered was already answered.
            let _ = snapshot.execute_command(earlier.clone());
        }
        snapshot.execute_command(request.clone())
    }
}

impl Interleaved for Faulty<Retained> {
    type Pending = InFlight;

    /// [`Fault::RetryCreatesSecondEntity`] looks the request up when it arrives: whether this
    /// target has answered it before is decided now, not at the return instant.
    fn invoke(&self, request: SemanticCommandRequest) -> InFlight {
        let unretained = self.fault == Fault::RetryCreatesSecondEntity
            && !self.applied.borrow().contains(&request);
        InFlight {
            request,
            read: None,
            unretained,
        }
    }

    /// The call, at the return instant. A request found unretained at invoke and answered since —
    /// its twin was in flight beside it — is applied as a new one: its correlation, which is what
    /// the reference retains a request under, is replaced by one nothing has used. Run on its own,
    /// a call finds at invoke exactly what it finds at return, and is answered as the reference
    /// answers it.
    fn complete(&self, pending: InFlight) -> Result<SemanticCommandResult, TargetError> {
        let answered_since = pending.unretained && self.applied.borrow().contains(&pending.request);
        if !answered_since {
            return self.execute_command(pending.request);
        }
        let mut fresh = pending.request.clone();
        fresh.correlation =
            CorrelationId::new(format!("{}-unretained", pending.request.correlation))
                .unwrap_or_else(|error| panic!("a suffixed correlation is valid: {error}"));
        let result = self.inner.execute_command(fresh)?;
        self.applied.borrow_mut().push(pending.request);
        Ok(result)
    }
}

/// The workload [`Fault::DoubleApplyOnRedelivery`] is recorded under: client 0 creates one invoice,
/// then two clients each read `InvoiceById` three times.
///
/// Recorded with injection, the `InvoiceCreated` the creation published is delivered a second time
/// at a tick the seed picks; a read after the second delivery, once the projection has caught up,
/// is what shows the invoice nobody created. Without injection it is delivered once.
pub fn double_apply_workload() -> sessions::Workload {
    let reads = || vec![Act::Read(INVOICE_BY_ID.to_owned()); 3];
    sessions::Workload {
        prefix: vec![Call::new(CREATE_INVOICE, invoice(), Subject::Creates)],
        clients: vec![reads(), reads()],
    }
}

/// The workload [`Fault::RetryCreatesSecondEntity`] is recorded under: each of two clients seeds
/// one record.
///
/// Recorded with injection, each `Seed` is sent a second time, at a tick the seed picks; where the
/// retry is sent before the original answered, both find nothing retained. Without injection no
/// request is sent twice.
pub fn retry_workload() -> sessions::Workload {
    let seed = |document: &str| {
        vec![Act::Call(Call::new(
            SEED,
            BTreeMap::from([("document".to_owned(), Node::Text(document.to_owned()))]),
            Subject::Creates,
        ))]
    };
    sessions::Workload {
        prefix: Vec::new(),
        clients: vec![seed("first"), seed("second")],
    }
}

/// The input of a `CreateInvoice` the reference accepts.
fn invoice() -> BTreeMap<String, Node> {
    let mut create = BTreeMap::from([("amount".to_owned(), money(5.0))]);
    create.insert(
        "account_id".to_owned(),
        Node::Text("00000000-0000-4000-8000-0000000000aa".to_owned()),
    );
    create.insert(
        "customer_email".to_owned(),
        Node::Text("payer@example.com".to_owned()),
    );
    create
}

/// The workload [`Fault::LostUpdate`] is recorded under: client 0 creates and issues one invoice,
/// then two clients each pay it once.
///
/// Exactly one of the two payments may settle it. Whether the two are in flight at once is the
/// seed's to decide ([`record::record`](crate::record::record)).
pub fn lost_update_workload() -> Workload {
    let amount = || BTreeMap::from([("amount".to_owned(), money(5.0))]);
    let pay = || Call::new(PAY_INVOICE, amount(), Subject::Created(0));
    let mut create = amount();
    create.insert(
        "account_id".to_owned(),
        Node::Text("00000000-0000-4000-8000-0000000000aa".to_owned()),
    );
    create.insert(
        "customer_email".to_owned(),
        Node::Text("payer@example.com".to_owned()),
    );
    Workload {
        prefix: vec![
            Call::new(CREATE_INVOICE, create, Subject::Creates),
            Call::new(ISSUE_INVOICE, BTreeMap::new(), Subject::Created(0)),
        ],
        clients: vec![vec![pay()], vec![pay()]],
    }
}

/// The workload [`Fault::StaleReadUnderReadYourWrites`] is recorded under: client 0 creates two
/// invoices, then each of two clients issues one of them and reads `OutstandingInvoices`.
///
/// A client's read demands its own last write ([`sessions`]). Where the other
/// client issued its invoice between the two, the read is answered from the copy, which does not
/// hold the reader's own invoice. Whether that happens is the seed's to decide.
pub fn stale_read_workload() -> sessions::Workload {
    let mut create = BTreeMap::from([("amount".to_owned(), money(5.0))]);
    create.insert(
        "account_id".to_owned(),
        Node::Text("00000000-0000-4000-8000-0000000000aa".to_owned()),
    );
    create.insert(
        "customer_email".to_owned(),
        Node::Text("payer@example.com".to_owned()),
    );
    let session = |prefix: usize| {
        vec![
            Act::Call(Call::new(
                ISSUE_INVOICE,
                BTreeMap::new(),
                Subject::Created(prefix),
            )),
            Act::Read(OUTSTANDING.to_owned()),
        ]
    };
    sessions::Workload {
        prefix: vec![
            Call::new(CREATE_INVOICE, create.clone(), Subject::Creates),
            Call::new(CREATE_INVOICE, create, Subject::Creates),
        ],
        clients: vec![session(0), session(1)],
    }
}

/// `true` for the commands that read an invoice's state and then move it.
fn reads_then_writes(request: &SemanticCommandRequest) -> bool {
    matches!(
        request.command.to_string().as_str(),
        ISSUE_INVOICE | PAY_INVOICE | CANCEL_INVOICE
    )
}

// ---- the names this module writes --------------------------------------------------------------

/// Parses an event name this module names as a literal.
///
/// # Panics
///
/// It does not: the names come from [`crate::reference`], which checks them against
/// `examples/billing/`.
fn event(value: &str) -> EventRef {
    value
        .parse()
        .unwrap_or_else(|error| panic!("`{value}` is a well-formed event: {error}"))
}

/// `billing.invoice.CancelInvoice/cancelled`, the branch the illegal move reports.
///
/// # Panics
///
/// It does not, for the reason [`event`] does not.
fn cancelled() -> OutcomeRef {
    branch_of(CANCEL_INVOICE, "cancelled")
}

/// `true` when the reference answered the `wrong_state:` branch of `command_name`.
///
/// Read off the branch the result names rather than off "no outcome and no events": a target that
/// declares the branch reports it, and a fault that keyed on the absence of an outcome would stop
/// firing the moment a specification adopted the construct.
fn refused_for_state(result: &SemanticCommandResult, command_name: &str) -> bool {
    result
        .outcome
        .as_ref()
        .is_some_and(|taken| taken == &branch_of(command_name, WRONG_STATE))
}

/// The name every `wrong_state:` branch in `examples/billing/` is declared under.
const WRONG_STATE: &str = "wrong-state";

/// One declared error, by name.
///
/// # Panics
///
/// It does not, for the reason [`event`] does not.
fn declared_error(value: &str) -> ErrorRef {
    value
        .parse()
        .unwrap_or_else(|error| panic!("`{value}` is a well-formed error: {error}"))
}

/// One declared branch of one command.
///
/// # Panics
///
/// It does not, for the reason [`event`] does not.
fn branch_of(command_name: &str, name: &str) -> OutcomeRef {
    OutcomeRef::new(
        command_name
            .parse()
            .unwrap_or_else(|error| panic!("`{command_name}` is a well-formed command: {error}")),
        name.parse()
            .unwrap_or_else(|error| panic!("`{name}` is a well-formed outcome name: {error}")),
    )
}

/// A `billing.invoice.Money` of `amount`.
///
/// # Panics
///
/// It does not: the literals here are finite.
fn money(amount: f64) -> Node {
    let mut fields = BTreeMap::new();
    fields.insert(
        "amount".to_owned(),
        Node::Number(
            Number::new(amount).unwrap_or_else(|error| panic!("{amount} is finite: {error}")),
        ),
    );
    fields.insert(
        "currency".to_owned(),
        Node::Text("amount.currency".to_owned()),
    );
    Node::Map(fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_two_faults_claim_the_same_scenario() {
        // Two faults caught by one scenario makes it impossible to say which property that scenario
        // actually protects — `aep_conformance`'s argument, one level down: there the unit is a
        // suite, here it is a single named check.
        let mut claimed: Vec<&str> = Fault::ALL
            .iter()
            .filter_map(|fault| fault.caught().scenario())
            .collect();
        claimed.sort_unstable();
        let mut unique = claimed.clone();
        unique.dedup();
        assert_eq!(
            claimed, unique,
            "two faults designating one scenario means neither of them shows what that scenario \
             protects"
        );
    }

    #[test]
    fn every_fault_says_what_it_is_and_where_it_goes() {
        // The list is generated from the same lines as the variants, so what is left to check is
        // that no row was declared empty — a fault with no description is a row in a matrix nobody
        // can read.
        assert_eq!(Fault::ALL.len(), 17);
        for fault in Fault::ALL {
            assert!(!fault.written().is_empty(), "{fault:?} has no written form");
            assert!(!fault.describe().is_empty(), "{fault:?} describes nothing");
            match fault.caught() {
                Caught::By(scenario) => assert!(
                    scenario.contains('/'),
                    "{fault:?} names `{scenario}`, which is not a scenario id"
                ),
                Caught::Nothing(why) => assert!(
                    why.len() > 20,
                    "{fault:?} is uncaught and says only `{why}`; an uncaught fault is a finding \
                     about the model or the synthesizer, and the finding is the reason"
                ),
                Caught::ByHistory(why) => assert!(
                    why.len() > 20,
                    "{fault:?} is caught only by a concurrent history and says only `{why}`; the \
                     reason no single-client scenario sees it is the row's claim"
                ),
                Caught::ByInjection(why) => assert!(
                    why.len() > 20,
                    "{fault:?} is caught only by a history with declared faults injected and says \
                     only `{why}`; which declared fault it waits for is the row's claim"
                ),
            }
        }
    }

    #[test]
    fn only_the_two_faults_the_boundary_cannot_express_are_injected_in_the_implementation() {
        // The split is a property worth pinning rather than a habit: every fault that *can* be a
        // perturbation of what goes in and what comes out must be one, because that is the position
        // a real client is in. The two exceptions are argued in the module documentation.
        let implementation: Vec<Fault> = Fault::ALL
            .iter()
            .copied()
            .filter(|fault| fault.injection() == Injection::Implementation)
            .collect();
        assert_eq!(
            implementation,
            vec![Fault::DropBinding, Fault::WrongMapping],
            "a fault injected in the implementation needs the argument in this module's \
             documentation, not just a line in the table"
        );
    }

    #[test]
    fn a_fault_is_injected_into_the_system_that_declares_what_it_breaks() {
        // `oracle` and `billing` refuse a fault from the other system rather than running it, which
        // would otherwise be a green row in the matrix that proved nothing at all.
        for fault in Fault::ALL {
            match fault.system() {
                System::Billing => {
                    let _ = billing(*fault);
                }
                System::Oracle => {
                    let _ = oracle(*fault);
                }
                System::Retry => {
                    let _ = retry(*fault);
                }
            }
        }
        assert_eq!(System::Billing.directory(), "billing");
        assert_eq!(System::Oracle.directory(), "oracle-fixture");
        assert_eq!(System::Retry.directory(), "explore-retry");
        for system in [System::Billing, System::Oracle, System::Retry] {
            assert!(system.path().ends_with(system.directory()), "{system:?}");
        }
    }
}
