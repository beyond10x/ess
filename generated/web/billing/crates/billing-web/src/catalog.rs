// generated from billing v3
// model digest 62706dc8de60f859f9fa11d363bae20825e7c74e71435e2fd28691488d787af1
// contract digest d0791c480f462a0bd205e4eda077f60c22bedf0f83756f7ff35687682ce8e3dd
// do not edit: regenerate with `ess synthesize --target web`

//! The model this page renders itself from.
//!
//! Pulled in from `catalog.json` beside the tree root rather than written here, so a reviewer reads the
//! catalogue as JSON and the module carries it without a second copy. The page asks the running
//! system for it — a page opened from `file://` can read its own WebAssembly module and cannot
//! always read its neighbours.

/// The model, as canonical JSON.
pub const CATALOG: &str = include_str!("../../../catalog.json");
