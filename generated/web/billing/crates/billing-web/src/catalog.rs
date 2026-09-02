// generated from billing v3
// model digest aacdc2fe065d462cc4f9ba51e6740f88809b6b17ce006ef846b488f957005da3
// contract digest 6ba34a27496cc918b55c749b45599c03b3016fed36487b1763268b95e0c6ffc6
// do not edit: regenerate with `ess synthesize --target web`

//! The model this page renders itself from.
//!
//! Pulled in from `catalog.json` beside the tree root rather than written here, so a reviewer reads the
//! catalogue as JSON and the module carries it without a second copy. The page asks the running
//! system for it — a page opened from `file://` can read its own WebAssembly module and cannot
//! always read its neighbours.

/// The model, as canonical JSON.
pub const CATALOG: &str = include_str!("../../../catalog.json");
