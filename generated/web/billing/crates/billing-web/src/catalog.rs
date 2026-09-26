// generated from billing v3
// model digest 1e7906786567af32118eb2d0a8c3fcafa16c32c9649a80b60487fd2eeebc4c9c
// contract digest a21fd36f0055057629f4c235962163cdd34a3d178aa068925bcb53be623af301
// do not edit: regenerate with `ess synthesize --target web`

//! The model this page renders itself from.
//!
//! Pulled in from `catalog.json` beside the tree root rather than written here, so a reviewer reads the
//! catalogue as JSON and the module carries it without a second copy. The page asks the running
//! system for it — a page opened from `file://` can read its own WebAssembly module and cannot
//! always read its neighbours.

/// The model, as canonical JSON.
pub const CATALOG: &str = include_str!("../../../catalog.json");
