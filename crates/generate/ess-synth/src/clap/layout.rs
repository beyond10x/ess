//! Where the emitted command-line realization lands.
//!
//! One decision taken once, for the reason [`crate::rust::layout`] and [`crate::web::layout`]
//! exist: the manifest writes a package name, a module writes a `use`, and a test greps for a
//! path. A convention re-derived per renderer is a convention four renderers spell four ways.

use ess_compiler::ir::EssIr;

/// The shape of the emitted binary: one crate, whatever the specification declares.
///
/// One crate rather than one per command-line component, and the same call the web target made for
/// the same reason: a *binary* is not the specification's unit of ownership. A tree that emitted
/// one crate per component would emit several binaries where the declaration says one word is
/// typed, which is the thing the `cli:` block exists to decide.
pub(crate) struct Layout {
    /// The package name of the emitted crate — `billing-cli`.
    package: String,
}

impl Layout {
    /// Derives the layout of a resolved specification.
    pub fn of(ir: &EssIr) -> Self {
        Self {
            package: format!("{}-cli", ir.system().segments().join("-")),
        }
    }

    /// The package name of the emitted crate.
    pub fn package(&self) -> &str {
        &self.package
    }

    /// One source file of the emitted crate, by module name.
    pub fn source(&self, module: &str) -> String {
        format!("crates/{}/src/{module}.rs", self.package)
    }

    /// The emitted crate's manifest.
    pub fn manifest(&self) -> String {
        format!("crates/{}/Cargo.toml", self.package)
    }
}
