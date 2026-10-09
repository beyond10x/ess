//! The instant of the step a runner is executing, shared with a target's command clock
//! (<https://github.com/beyond10x/ess/issues/510>).
//!
//! The runner writes it once at the start of every step, from the one wall reading that step's
//! `now_offset` values resolve against; the target reads it once per command decision, through the
//! [`CommandClock`] [`Runner::command_clock`](super::Runner::command_clock) hands out. So a `now`
//! guard is decided against the instant the scenario's own values were sent relative to, and no
//! clock of the machine's is read unless the runner's wall is one.

use std::sync::{Arc, Mutex, PoisonError};

use ess_primitives::time::{Rfc3339Instant, Timestamp};

use crate::occurrence_clock::{CommandClock, DecisionInstant};

/// One shared cell: written by the runner, read by the target's command clock.
#[derive(Debug, Clone, Default)]
pub(super) struct StepInstant(Arc<Mutex<Option<DecisionInstant>>>);

impl StepInstant {
    /// Records the step's instant: the wall `reading`, rounded up to a whole second as a
    /// `now_offset` is ([`crate::now_offset`]). A reading with no spelling leaves no instant, and a
    /// decision that needs one is Unknown.
    pub(super) fn write(&self, reading: Timestamp) {
        let instant = i64::try_from(reading.epoch_millis())
            .ok()
            .and_then(Rfc3339Instant::from_epoch_millis)
            .map(Rfc3339Instant::ceil_to_second)
            .and_then(DecisionInstant::checked);
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = instant;
    }
}

impl CommandClock for StepInstant {
    fn read(&self) -> Option<DecisionInstant> {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Two runners are equal only when they write the same cell.
impl PartialEq for StepInstant {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for StepInstant {}
