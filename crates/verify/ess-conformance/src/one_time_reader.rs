//! A one-time response's read cell names the reader (beyond10x/ess#614,
//! `docs/design/one-time-read-cells-for-other-actors.md`, option 2).
//!
//! The cell `<command>/disclosure/<outcome>/<field>/read/<view>/as/actor/<A>` issues the value as
//! an actor granted the issuing command (`A` where `A` is, else the command's first granted actor)
//! and then reads the view as `A` through a [`ReadAs`](crate::ScenarioStep::ReadAs) step, so an
//! actor who may read the view but may not issue the value has a cell of its own. Below
//! suite/[`ORDINARY`] the same id names the issuer and the read is sent as no actor; every reader
//! keeps that meaning for a suite labelled below it, so an existing suite keeps its bytes and what
//! they assert.
//!
//! A suite carrying a read cell for a named actor is ordinary suite/[`ORDINARY`] or coverage
//! suite/[`COVERAGE`], cumulative over every pair below. The identity spelling and the steps are
//! the ones older majors carry; only what the actor segment names moves, which is why the number
//! moves.

use crate::one_time_response::Aspect;
use crate::{ConformanceSuite, ScenarioId};

/// The ordinary suite major whose read cells read as their actor.
pub const ORDINARY: u32 = 56;

/// Its coverage counterpart.
pub const COVERAGE: u32 = 57;

/// The suite majors this module introduces.
pub const ADMITTED: [u32; 2] = [ORDINARY, COVERAGE];

/// Whether a suite files a read cell naming an actor.
pub fn used_by(suite: &ConformanceSuite) -> bool {
    suite.scenarios.keys().any(|id| {
        matches!(id, ScenarioId::Disclosure { cell }
            if matches!(cell.aspect, Aspect::Read(_)) && cell.actor.is_some())
    })
}

/// The ordinary major a fresh suite needs at least: [`ORDINARY`] when [`used_by`].
pub fn ordinary_floor(suite: &ConformanceSuite) -> Option<u32> {
    used_by(suite).then_some(ORDINARY)
}

/// The coverage major a fresh coverage suite needs at least: [`COVERAGE`] when [`used_by`].
pub fn coverage_floor(suite: &ConformanceSuite) -> Option<u32> {
    used_by(suite).then_some(COVERAGE)
}
