//! Where a generated seam needs a second spelling of a command's `wrong_state` branch.
//!
//! The rule and its one function live in [`ess_compiler::unknown_instance`], below both this crate
//! and `ess-http`, which asks the same question to decide which outcomes write their error's
//! fields. This module keeps the path every projection of a command surface already uses.

pub use ess_compiler::unknown_instance::unknown_instance_answer;
