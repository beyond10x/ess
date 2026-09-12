//! The compilation options every validator built from this crate is built from.
//!
//! `uniqueItems` is decided by schema equality, under which `0`, `0.0` and `-0.0` are one number;
//! the dependency decides it by array length instead. The corrected keyword lives in
//! `realize::normalize::unique_items`, because the Rust normalization runtimes this crate emits
//! carry that same file and must not decide it differently from the reference they were
//! generated from.

use jsonschema::ValidationOptions;

use crate::realize::normalize::unique_items;

/// Validator options that decide `uniqueItems` the same way at every array length.
///
/// Every validator this crate compiles, emits or checks itself against starts here — including
/// the crate's own test oracles, which would otherwise hold the code to the behaviour it exists
/// to replace. This is the crate's only call to the dependency's own constructor.
pub fn options<'i>() -> ValidationOptions<'i> {
    jsonschema::options().with_keyword("uniqueItems", unique_items::compile)
}
