//! The suite pair `ess-conformance/64` (ordinary) and `/65` (coverage), opened for the constructs
//! of one release (story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs).
//!
//! The pair is open and empty. Every reader admits it wherever it admits the member-path pair
//! below it, cumulative over every major below, and no synthesis writes it: a suite's major is
//! still chosen by the features its specification uses, so a suite keeps its earlier format and
//! bytes until a construct needs the pair. The stories that fill it add their vocabulary here, one
//! place a later unit can name, instead of each opening a pair of its own.

/// The ordinary suite major opened for the release's constructs.
pub const ORDINARY: u32 = 64;

/// Its coverage counterpart.
pub const COVERAGE: u32 = 65;

/// The suite majors this module opens.
pub const ADMITTED: [u32; 2] = [ORDINARY, COVERAGE];
