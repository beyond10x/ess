//! The single-crate layout (`--layout crate`): the workspace's code, as one crate at the root.
//!
//! For an adopter that never deploys a component alone, the workspace's crate boundaries buy
//! nothing and cost a directory level and a manifest per component. This module takes the
//! workspace the emitter already wrote — the same code, from the same plan, with the same
//! coverage accounting — and moves each crate to a module of one crate:
//!
//! | workspace | single crate |
//! | --- | --- |
//! | `crates/<system>-types/src/<file>` | `src/<file>`, its `lib.rs` the crate root |
//! | `crates/<component>/src/lib.rs` | `src/ports/<component>.rs`, declared by `src/ports.rs` |
//! | `crates/<system>-system/src/lib.rs` | `src/system.rs` |
//! | `crates/<system>-server/src/lib.rs` | `src/server.rs`, behind the `server` feature |
//! | `crates/<system>-server/src/<file>` | `src/server/<file>` |
//!
//! Nothing is deeper than `src/<module>/<file>`. The types crate's modules keep their paths — they
//! were already `crate::`-rooted — so only the three kinds of dependent crate are rewritten: a path
//! into another generated crate (`gatepass_types::visit::Visit`) becomes the module it now is
//! (`crate::visit::Visit`), and a crate's path to itself (`crate::entry`) gains the module it moved
//! into (`crate::server::entry`). The HTTP surface is the only code that reads a socket, and the
//! `server` feature gates the whole of it, so a build without the feature has no `std::net` in it.
//!
//! The rewrite is by path, not by text: an identifier is rewritten only where it begins a path
//! (`name::`, not preceded by `::`), which is the only position a crate name can take in the code
//! the emitter writes. The generated tests build every layout with warnings denied.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use ess_compiler::ir::EssIr;
use ess_gen::Artifact;

use crate::plan::{SynthesisPlan, REGENERATE};

use super::layout::Layout;
use super::EDITION;

/// The modules the single crate adds at its root, beside the bounded contexts; a bounded context
/// spelled like one of them is renamed by [`Layout::shaped`].
pub(crate) const ROOT_MODULES: [&str; 3] = ["ports", "server", "system"];

/// Where one workspace crate lands in the single crate.
enum Destination {
    /// The types crate: the root.
    Types,
    /// A component's port crate, by module identifier.
    Port(String),
    /// The system crate.
    System,
    /// The server crate.
    Server,
}

/// Every workspace crate, where it lands, and the path its name becomes.
struct Crates {
    /// Destination per package name.
    destinations: BTreeMap<String, Destination>,
    /// The path each crate identifier becomes, for [`rewrite`].
    paths: BTreeMap<String, String>,
    /// The component port modules, sorted.
    ports: Vec<String>,
}

impl Crates {
    fn of(ir: &EssIr, layout: &Layout) -> Self {
        let mut destinations = BTreeMap::new();
        let mut paths = BTreeMap::new();
        for (package, destination, path) in [
            (layout.package(), Destination::Types, "crate"),
            (
                layout.system_package(),
                Destination::System,
                "crate::system",
            ),
            (
                layout.server_package(),
                Destination::Server,
                "crate::server",
            ),
        ] {
            destinations.insert(package.to_owned(), destination);
            paths.insert(Layout::crate_ident(package), path.to_owned());
        }
        let mut ports = Vec::new();
        for component in ir.components().keys() {
            let package = layout.component_package(component);
            let module = Layout::crate_ident(package);
            paths.insert(module.clone(), format!("crate::ports::{module}"));
            destinations.insert(package.to_owned(), Destination::Port(module.clone()));
            ports.push(module);
        }
        ports.sort();
        Self {
            destinations,
            paths,
            ports,
        }
    }
}

/// Moves the workspace's artifacts into one crate.
///
/// # Panics
///
/// If the workspace holds a file this layout has no place for — a defect in this crate: a new
/// kind of generated crate needs a destination here before it can ship in this layout.
pub(crate) fn relayout(
    ir: &EssIr,
    plan: &SynthesisPlan,
    layout: &Layout,
    artifacts: Vec<Artifact>,
) -> Vec<Artifact> {
    let Crates {
        destinations,
        paths,
        ports,
    } = Crates::of(ir, layout);

    let regenerate = plan.regenerate();
    let header = |contents: &str| {
        contents.replacen(
            &format!("regenerate with `{REGENERATE}`"),
            &format!("regenerate with `{regenerate}`"),
            1,
        )
    };

    let mut out = Vec::new();
    let mut root = None;
    let mut system = false;
    let mut server = false;
    for artifact in artifacts {
        if artifact.path == "Cargo.toml" {
            continue;
        }
        let (package, file) = artifact
            .path
            .strip_prefix("crates/")
            .and_then(|rest| rest.split_once('/'))
            .unwrap_or_else(|| panic!("`{}` is not inside a workspace crate", artifact.path));
        if file == "Cargo.toml" {
            continue;
        }
        let file = file
            .strip_prefix("src/")
            .unwrap_or_else(|| panic!("`{}` is not a crate source", artifact.path));
        let destination = destinations
            .get(package)
            .unwrap_or_else(|| panic!("`{package}` has no place in the single crate"));
        let contents = header(&artifact.contents);
        let (path, own) = match destination {
            Destination::Types if file == "lib.rs" => {
                root = Some(contents);
                continue;
            }
            Destination::Types => {
                out.push(Artifact::new(format!("src/{file}"), contents));
                continue;
            }
            Destination::Port(module) => {
                assert_eq!(file, "lib.rs", "a port crate is one file");
                (
                    format!("src/ports/{module}.rs"),
                    format!("crate::ports::{module}"),
                )
            }
            Destination::System => {
                assert_eq!(file, "lib.rs", "the system crate is one file");
                system = true;
                ("src/system.rs".to_owned(), "crate::system".to_owned())
            }
            Destination::Server if file == "lib.rs" => {
                server = true;
                ("src/server.rs".to_owned(), "crate::server".to_owned())
            }
            Destination::Server if file.starts_with("bin/") => {
                // A binary is a separate crate even in the one-package layout.
                let library = Layout::crate_ident(&ir.system().segments().join("-"));
                let binary_paths = paths
                    .iter()
                    .map(|(name, path)| (name.clone(), path.replacen("crate", &library, 1)))
                    .collect();
                out.push(Artifact::new(
                    format!("src/{file}"),
                    rewrite(&contents, "crate", &binary_paths),
                ));
                continue;
            }
            Destination::Server => (format!("src/server/{file}"), "crate::server".to_owned()),
        };
        let contents = if std::path::Path::new(&path)
            .extension()
            .is_some_and(|extension| extension == "rs")
        {
            rewrite(&contents, &own, &paths)
        } else {
            contents
        };
        out.push(Artifact::new(path, contents));
    }

    let mut root = root.expect("the workspace has a types crate root");
    root.push('\n');
    if !ports.is_empty() {
        out.push(ports_module(ir, plan, &ports, &header));
        root.push_str("pub mod ports;\n");
    }
    if system {
        root.push_str("pub mod system;\n");
    }
    if server {
        root.push_str("#[cfg(feature = \"server\")]\npub mod server;\n");
    }
    out.push(Artifact::new("src/lib.rs", root));
    out.push(manifest(ir, plan, server));
    out
}

/// `src/ports.rs`: one module per component, each that component's port.
fn ports_module(
    ir: &EssIr,
    plan: &SynthesisPlan,
    ports: &[String],
    header: &impl Fn(&str) -> String,
) -> Artifact {
    let mut out = header(&plan.provenance.commented_for("//", REGENERATE));
    let _ = write!(
        out,
        "\n//! The component ports of `{}` {}: one module per component, each its accepted \
         commands,\n//! declared views and published events, typed against the bounded contexts \
         at the crate root.\n\n",
        ir.system(),
        ir.version()
    );
    for module in ports {
        let _ = writeln!(out, "pub mod {module};");
    }
    Artifact::new("src/ports.rs", out)
}

/// The crate's manifest: its own workspace root, with executable dependencies gated behind
/// `server` when there is an HTTP surface.
fn manifest(ir: &EssIr, plan: &SynthesisPlan, server: bool) -> Artifact {
    let mut out = plan.provenance.commented_for("#", &plan.regenerate());
    let _ = write!(
        out,
        "\n[package]\nname = \"{}\"\ndescription = \"The `{}` specification, {}, synthesised as \
         one crate{}.\"\nversion = \"{}.0.0\"\nedition = \"{EDITION}\"\n",
        ir.system().segments().join("-"),
        ir.system(),
        ir.version(),
        if server {
            "; its HTTP surface is the `server` feature"
        } else {
            ""
        },
        ir.version().get()
    );
    if server {
        out.push_str("\n[features]\nserver = [\"dep:clap\", \"dep:uuid\", \"dep:time\"]\n");
    }
    out.push_str("\n[dependencies]\n");
    if server {
        out.push_str("clap = { version = \"4.6.7\", features = [\"derive\"], optional = true }\nuuid = { version = \"1.26.1\", features = [\"v4\"], optional = true }\ntime = { version = \"0.3.55\", features = [\"formatting\"], optional = true }\n");
        for component in super::http::served(ir) {
            let _ = writeln!(out, "\n[[bin]]\nname = \"{}-server\"\npath = \"src/bin/{}-server.rs\"\nrequired-features = [\"server\"]", component.name, component.name);
        }
    }
    out.push_str("\n[workspace]\n");
    Artifact::new("Cargo.toml", out)
}

/// Rewrites every path that begins with a generated crate's name, or with `crate`, to where that
/// code now lives: `crate` to `own`, another crate's name to its entry in `paths`.
///
/// A path begins where an identifier is followed by `::` and not preceded by `::`, an identifier
/// character or `$`; nothing else is touched, so a field, a function or a module *inside* a path
/// that happens to share a crate's name keeps its spelling.
fn rewrite(source: &str, own: &str, paths: &BTreeMap<String, String>) -> String {
    let mut out = String::with_capacity(source.len() + source.len() / 16);
    let mut rest = source;
    let mut previous: Option<char> = None;
    while let Some(first) = rest.chars().next() {
        let starts = (first.is_alphabetic() || first == '_')
            && !previous.is_some_and(|before| {
                before.is_alphanumeric() || before == '_' || before == ':' || before == '$'
            });
        if !starts {
            out.push(first);
            previous = Some(first);
            rest = &rest[first.len_utf8()..];
            continue;
        }
        let end = rest
            .find(|character: char| !(character.is_alphanumeric() || character == '_'))
            .unwrap_or(rest.len());
        let ident = &rest[..end];
        let replacement = if rest[end..].starts_with("::") {
            if ident == "crate" {
                Some(own)
            } else {
                paths.get(ident).map(String::as_str)
            }
        } else {
            None
        };
        out.push_str(replacement.unwrap_or(ident));
        previous = ident.chars().last();
        rest = &rest[end..];
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_path_that_begins_with_a_crate_name_is_rewritten() {
        let paths: BTreeMap<String, String> = [
            ("demo_types".to_owned(), "crate".to_owned()),
            (
                "pay_service".to_owned(),
                "crate::ports::pay_service".to_owned(),
            ),
        ]
        .into();
        let source = "use crate::{entry, http};\n\
                      fn f(x: demo_types::pay::Invoice) -> pay_service::Port {}\n\
                      let pay_service = crate::pay_service::serve(pub(crate) x);\n\
                      my_demo_types::x; $crate::y; <demo_types::A as pay_service::B>::c\n";
        assert_eq!(
            rewrite(source, "crate::server", &paths),
            "use crate::server::{entry, http};\n\
             fn f(x: crate::pay::Invoice) -> crate::ports::pay_service::Port {}\n\
             let pay_service = crate::server::pay_service::serve(pub(crate) x);\n\
             my_demo_types::x; $crate::y; <crate::A as crate::ports::pay_service::B>::c\n"
        );
    }
}
