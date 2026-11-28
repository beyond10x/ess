//! The suite pair `ess-conformance/66` (ordinary) and `/67` (coverage), opened for the suite
//! vocabulary of the stories that close the open issues
//! (story:the-suite-pair-ess-conformance-66-and-67-is-open-read-wherever-64-and-65-are-and-written-by-nothing-yet).
//!
//! The pair `/64` and `/65` is written since 0.95.0 by the coverage suite that holds a covered
//! refusal, so a reader of that release would read new vocabulary there as a suite nobody wrote.
//! This pair is open and empty: every reader admits it wherever it admits `/64` and `/65`,
//! cumulative over every major below, and no synthesis writes it. The stories that fill it add
//! their vocabulary here.

/// The ordinary suite major opened for the new vocabulary.
pub const ORDINARY: u32 = 66;

/// Its coverage counterpart.
pub const COVERAGE: u32 = 67;

/// The suite majors this module opens.
pub const ADMITTED: [u32; 2] = [ORDINARY, COVERAGE];
