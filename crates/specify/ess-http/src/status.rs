//! The status ESS derives for each declared outcome of a command, without a binding.
//!
//! Here rather than in the generator because an `ess-http/1` binding is checked against it: two
//! outcomes on one status must be told apart by their bodies, and an outcome the binding does not
//! list is counted at this status (`ESS-HTTP-025`). The generator re-exports every item
//! (`ess_gen::http`), so the document, the served Rust and Go applications and the binding still
//! read one answer.

use ess_compiler::ir::{EssIr, ResolvedCondition, ResolvedOutcome};

/// The branch was taken; its events reach consumers elsewhere.
pub const TAKEN: &str = "202";

/// The input was understood and refused on domain grounds.
///
/// A `422` and not a `400`: `400` is for a request the server could not parse, which is decided by
/// the schema and would be true of any endpoint. A refusal on domain grounds is a request the
/// server understood, and a client can act on the difference — one means fix the value, the other
/// means fix the serialiser.
pub const REFUSED: &str = "422";

/// Something outside the request refused; the input was acceptable.
///
/// A `5xx` because `external` names a branch the input cannot decide. Reporting it as a `4xx` sends
/// the caller to fix the one thing it cannot fix and tells every retry layer between that retrying
/// is pointless. `502` rather than `500`, which would claim a fault in this component, or `503`,
/// which would claim the whole component is unavailable when one provider refused one request.
pub const UPSTREAM: &str = "502";

/// The input was acceptable and the subject was in a state the command does not act from.
///
/// A `409` and not a `422`, and the difference is what a caller does next. `422` says the request
/// was wrong and resending it unchanged is pointless; `409` says the request was fine and the world
/// was not — the same visit, admitted before it was signed out, would have been accepted.
pub const CONFLICT: &str = "409";

/// The request named an identity no record carries, and the command declares that answer
/// (`unknown_instance:`, ess/15).
pub const NOT_FOUND: &str = "404";

/// The request carried no input at all, and the command declares that answer (`input_absent:`,
/// ess/16). A `400` and not a `422`: nothing was there to understand, which is the request the
/// server could not read rather than one it refused on domain grounds.
pub const NO_INPUT: &str = "400";

/// The caller is not one the branch admits: its guard compares the authenticated caller with the
/// input or the record (`caller.<attribute>`, ess/16). A `403` and not a `409` or a `422`: the
/// request and the record are fine, and the same request from the record's own agent is accepted.
///
/// It is also the status of the standard refusal for an actor no grant admits (beyond10x/ess#265,
/// `ess_gen::http::NOT_GRANTED`). One status for both, because both answer the same question —
/// *this caller may not do this* — and a client acts on both the same way: send it as someone else.
/// `401` would claim the credential is missing or wrong, and how a caller proves who it is belongs
/// to the realization, not to this contract. The two are told apart by the body: a declared branch
/// carries `outcome` and the declared `error`, the standard refusal carries `refused` and `actor`
/// and neither of those.
pub const FORBIDDEN: &str = "403";

/// Which status one declared outcome is.
///
/// The whole mapping, in one place, so that "which HTTP status does this refusal get" has exactly
/// one answer for the document that publishes it and the server that answers it. A server whose
/// statuses were computed separately would agree on the day it was written.
pub fn status(outcome: &ResolvedOutcome) -> &'static str {
    match (&outcome.condition, outcome.error.is_some()) {
        // First: who sent it decided the refusal, whatever else the guard reads.
        (_, true) if outcome.decided_by_caller => FORBIDDEN,
        // An external refusal narrowed to some records (ess/29) is still decided upstream.
        (
            ResolvedCondition::External { .. }
            | ResolvedCondition::ExternalWhen { .. }
            | ResolvedCondition::ExternalSubject { .. },
            true,
        ) => UPSTREAM,
        // All three are decided by the state the subject is resting in rather than by the
        // request, so a refusal from one of them is a conflict with that state and not a bad
        // request: `StateChange` is `SubjectState` with the states derived from the move.
        (
            ResolvedCondition::WrongState
            | ResolvedCondition::SubjectState { .. }
            | ResolvedCondition::SubjectField { .. }
            | ResolvedCondition::SubjectPredicate { .. }
            | ResolvedCondition::StateChange { .. }
            // And so is one decided by a stored row of another entity (ess/18, `when_related:`).
            | ResolvedCondition::Related { .. }
            // And by the rows a selector selects (ess/22, `when_related: {entity, where, …}`).
            | ResolvedCondition::RelatedSet { .. }
            // A duplicate of a record that exists conflicts with that record (ess/16).
            | ResolvedCondition::ExistingInstance,
            true,
        ) => CONFLICT,
        (ResolvedCondition::UnknownInstance, true) => NOT_FOUND,
        (ResolvedCondition::InputAbsent, true) => NO_INPUT,
        (ResolvedCondition::When { .. } | ResolvedCondition::Otherwise, true) => REFUSED,
        // An external branch that emits rather than errors is still a branch that was taken; what
        // decided it does not change what happened.
        (_, false) => TAKEN,
    }
}

/// The branch was taken and its answer is the command's response (`returns: true`, from
/// [`DIRECT_ANSWER_FORMAT`]).
///
/// A `200` and not a `202`: `202` tells a client the request was queued for later processing, and
/// a branch that returns its result in the same response was carried out, not queued
/// (beyond10x/ess#424).
pub const ANSWERED: &str = "200";

/// The first source format whose `returns: true` outcome is answered [`ANSWERED`], with the
/// command's declared response under `response` (beyond10x/ess#423, beyond10x/ess#424).
///
/// Below it the branch keeps the [`TAKEN`] status and the body it was published with, so a client
/// of an `ess/17` to `ess/21` contract keeps the answer it was told about.
pub const DIRECT_ANSWER_FORMAT: u32 = 22;

/// Whether `outcome` answers its caller with the command's response, under [`ANSWERED`].
///
/// The one answer for the document that declares the body and the servers that write it.
pub fn answers_with_response(ir: &EssIr, outcome: &ResolvedOutcome) -> bool {
    outcome.returns && outcome.error.is_none() && ir.format().major() >= DIRECT_ANSWER_FORMAT
}

/// Which status one declared outcome of `ir` is answered with: [`status`], except that a branch
/// that [answers with the response](answers_with_response) is [`ANSWERED`].
///
/// Every surface that writes or declares a command's status reads this, so the document and the
/// served Rust and Go applications cannot disagree about it.
pub fn outcome_status(ir: &EssIr, outcome: &ResolvedOutcome) -> &'static str {
    if answers_with_response(ir, outcome) {
        ANSWERED
    } else {
        status(outcome)
    }
}
