// generated from billing v3
// model digest 096efa38ec46e97a32f81b72193e43114df1156464648a885134ffafc9ac9648
// contract digest c9ecfdf5bed1bcb88068dad060895f16ca73de361204ae488d6f0d39477f6f79
// do not edit: regenerate with `ess synthesize --target web`

//! The model this page renders itself from.
//!
//! Pulled in from `catalog.json` beside the tree root rather than written here, so a reviewer reads the
//! catalogue as JSON and the module carries it without a second copy. The page asks the running
//! system for it — a page opened from `file://` can read its own WebAssembly module and cannot
//! always read its neighbours.

/// The model, as canonical JSON.
pub const CATALOG: &str = include_str!("../../../catalog.json");
