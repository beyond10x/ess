// generated from billing v3
// model digest 8b52fe739078f96a006d7bee5e9b9530c3a30221f7bc003f291dcfe17cdcfea3
// contract digest 0c2f2067136aea0bc0a45ca5b01bf70f551fc6199956699c5d5b939c350688f8
// do not edit: regenerate with `ess synthesize --target web`

//! The model this page renders itself from.
//!
//! Pulled in from `catalog.json` beside the tree root rather than written here, so a reviewer reads the
//! catalogue as JSON and the module carries it without a second copy. The page asks the running
//! system for it — a page opened from `file://` can read its own WebAssembly module and cannot
//! always read its neighbours.

/// The model, as canonical JSON.
pub const CATALOG: &str = include_str!("../../../catalog.json");
