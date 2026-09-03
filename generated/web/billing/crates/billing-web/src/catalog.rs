// generated from billing v3
// model digest 56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942
// contract digest cb634bd5e6f1afa6ebc8e9dca752e9901a9a68a2e51fc5009d099f155680606c
// do not edit: regenerate with `ess synthesize --target web`

//! The model this page renders itself from.
//!
//! Pulled in from `catalog.json` beside the tree root rather than written here, so a reviewer reads the
//! catalogue as JSON and the module carries it without a second copy. The page asks the running
//! system for it — a page opened from `file://` can read its own WebAssembly module and cannot
//! always read its neighbours.

/// The model, as canonical JSON.
pub const CATALOG: &str = include_str!("../../../catalog.json");
