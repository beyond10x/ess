//! The instance names one outcome scenario's earlier runs captured (beyond10x/ess#679).
//!
//! An outcome scenario is the full invocation followed by further runs of the same branch — an
//! Optional left out, a union variant, an expression point — each appended to the same steps.
//! Every run arranges its view companions beside its own subject, and a companion is numbered from
//! the first further distinction its run has not captured. A run sees only its own captures, so
//! the second run numbered its companion where the first had: a second row under the instance
//! name, and for a creation that takes its identity from input, under the identity, a row the
//! first run made still holds. The scenario was then withdrawn as creating one identity twice
//! ([`super::repeated_identity`]), and with it every observation of the branch.
//!
//! [`scenario`] opens the scope for one scenario's runs; [`record`] adds what a run captured, and
//! [`captured`] answers every name the runs before it captured. Outside a scope nothing is
//! recorded and nothing is answered, so a run built on its own numbers its companions as before.

use std::cell::RefCell;
use std::collections::BTreeSet;

use crate::scenario::{InstanceName, ScenarioStep};

thread_local! {
    static EARLIER: RefCell<Option<BTreeSet<InstanceName>>> = const { RefCell::new(None) };
}

/// Holds the scope [`scenario`] opened until dropped, then restores the one before it.
#[must_use = "the scope closes when this is dropped"]
pub(super) struct Scenario {
    previous: Option<BTreeSet<InstanceName>>,
}

impl Drop for Scenario {
    fn drop(&mut self) {
        let previous = self.previous.take();
        EARLIER.with(|scope| *scope.borrow_mut() = previous);
    }
}

/// Opens the scope of one scenario's runs, holding no name yet.
pub(super) fn scenario() -> Scenario {
    let previous = EARLIER.with(|scope| scope.replace(Some(BTreeSet::new())));
    Scenario { previous }
}

/// Every instance name the steps of a run capture, recorded for the runs after it.
pub(super) fn record(steps: &[ScenarioStep]) {
    EARLIER.with(|scope| {
        if let Some(names) = scope.borrow_mut().as_mut() {
            names.extend(steps.iter().filter_map(|step| match step {
                ScenarioStep::CaptureInstance { instance, .. } => Some(instance.clone()),
                _ => None,
            }));
        }
    });
}

/// Every instance name the runs recorded before this one captured; empty outside a scope.
pub(super) fn captured() -> BTreeSet<InstanceName> {
    EARLIER.with(|scope| scope.borrow().clone().unwrap_or_default())
}
