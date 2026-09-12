//! Adversary cases for `story:literal-representation-walk-exhaustion`.
//!
//! The unit's own cases build a `TypeRegistry` in code and hand it straight to
//! `validate_bindings`. These drive the same claims through the public document surface —
//! `RawSpecFile::parse` + `Specification::assemble` — which is the only surface an author
//! reaches, and they check the two workspace gates the unit's package-scoped gate cannot see.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use ess_domain::Specification;

/// The `ess-domain` crate root.
fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every `.rs` file under a directory, concatenated.
fn rust_sources(dir: &Path) -> String {
    let mut out = String::new();
    let mut entries: Vec<_> = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("read {}: {error}", dir.display()))
        .map(|entry| entry.expect("dir entry").path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            out.push_str(&rust_sources(&path));
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push_str(&fs::read_to_string(&path).expect("read source"));
            out.push('\n');
        }
    }
    out
}

/// Assemble one document and return whatever it refused with.
fn assemble(document: &str) -> Result<(), String> {
    let raw = RawSpecFile::parse(document).map_err(|error| error.to_string())?;
    Specification::assemble(vec![(Source::new("adversary.yaml"), raw)])
        .map(|_| ())
        .map_err(|errors| errors.to_string())
}

/// The ess-gen literal fixture's shape: one enum, one command taking `reason`, one binding whose
/// mapping writes a literal into it. `extra_types` is spliced into `types:` as written.
fn document(extra_types: &str, target: &str, literal: &str) -> String {
    format!(
        r"
format: ess/1
system: notifications
version: v1
domain: notifications.core
types:
  - name: notifications.core.Reason
    kind: enum
    variants: [declined, unavailable]
{extra_types}commands:
  - name: notifications.core.Reject
    input:
      - name: reason
        type: {target}
    outcomes:
      - name: rejected
        emits: [notifications.core.Rejected]
events:
  - name: notifications.core.Refused
  - name: notifications.core.Rejected
bindings:
  - id: reject-on-refusal
    when:
      event: notifications.core.Refused
    invoke:
      command: notifications.core.Reject
    mapping:
      reason: {literal}
    delivery: at_most_once
    on_failure: drop
"
    )
}

/// `Link0` … `Link{length - 1}`, each a newtype of the next, the last a newtype of the enum.
fn chain_types(length: usize) -> String {
    let mut out = String::new();
    for index in 0..length {
        let of = if index + 1 == length {
            "notifications.core.Reason".to_owned()
        } else {
            format!("notifications.core.Link{}", index + 1)
        };
        let _ = write!(
            out,
            "  - name: notifications.core.Link{index}\n    kind: newtype\n    of: {of}\n"
        );
    }
    out
}

/// The acceptance statement, asserted through the surface an author actually writes.
///
/// "Every admitted binding … literal must undergo its declared representation check, including
/// finite Optional/newtype mixtures at and beyond the current internal walk boundary." The unit
/// measured this against a registry it built in code; a document is what has to be refused.
#[test]
fn a_document_writing_a_bad_variant_past_the_old_walk_bound_is_refused() {
    for length in [
        ess_domain::binding::WRAPPER_LIMIT - 1,
        ess_domain::binding::WRAPPER_LIMIT,
        ess_domain::binding::WRAPPER_LIMIT + 1,
    ] {
        let types = chain_types(length);
        let good = assemble(&document(&types, "notifications.core.Link0", "declined"));
        assert!(
            good.is_ok(),
            "{length}: a real variant through a long chain is a good document: {:?}",
            good.unwrap_err()
        );

        let refused = assemble(&document(
            &types,
            "notifications.core.Link0",
            "not_a_variant",
        ))
        .expect_err("an undeclared variant must not be admitted at any chain length");
        assert!(refused.contains("not a variant"), "{length}: {refused}");
    }
}

/// The `Cyclic` branch, reached from a document rather than from a hand-built registry.
///
/// `Cycle = newtype of Optional<Cycle>` is inhabited, so `check_inhabitation` admits it and the
/// refusal has to come from the literal check. If a document cannot carry this type at all, the
/// branch the unit added is unreachable from the public surface.
#[test]
fn a_document_whose_input_type_resolves_through_itself_is_refused_at_the_literal() {
    let types = "  - name: notifications.core.Cycle\n    kind: newtype\n    of: \
                 Optional<notifications.core.Cycle>\n";
    let refused = assemble(&document(types, "notifications.core.Cycle", "anything"))
        .expect_err("a literal filling a self-resolving type is checked by nothing else");
    assert!(
        refused.contains("resolves through"),
        "the refusal names the cycle: {refused}"
    );
    assert!(
        refused.contains("notifications.core.Cycle"),
        "the refusal is source-addressed to the type: {refused}"
    );
}

/// The `Cyclic` arm's own claim: "nothing else reports this one".
///
/// `Ring = newtype of Ring` — no `Optional` to give it a base case — is the other shape the walk
/// returns `Cyclic` for, and `check_inhabitation` in `system.rs` already refuses it as
/// `SelfReference`. The module's stated policy for a fact another pass owns is silence, "which
/// reporting here would report twice and repair neither time".
#[test]
fn a_self_newtype_another_pass_already_refuses_is_not_reported_twice() {
    let types = "  - name: notifications.core.Ring\n    kind: newtype\n    of: \
                 notifications.core.Ring\n";
    let refused = assemble(&document(types, "notifications.core.Ring", "anything"))
        .expect_err("a newtype of itself cannot be built");
    assert!(
        refused.contains("no value of `notifications.core.Ring` can exist"),
        "the pass that owns this says so: {refused}"
    );
    assert!(
        !refused.contains("resolves through"),
        "the literal check stays quiet about a refusal another pass already makes: {refused}"
    );
}

/// The workspace consumer-coverage gate classifies every concrete source entry.
///
/// `crates/edge/ess-xtask/src/consumer_coverage/consumer.rs` mints one entry per non-test `enum`
/// and one per variant; `proposal.rs::classify` bails with "unclassified concrete consumer entry"
/// when an entry has no row here. The pre-existing `Representation` enum and all four of its
/// variants are classified — the new `Resolution` enum and its three are not, and the unit's
/// package-scoped gate cannot see it.
#[test]
fn the_new_resolution_enum_is_classified_for_consumer_coverage() {
    let classifications =
        crate_root().join("../../edge/ess-xtask/src/consumer_coverage/entry-classifications.json");
    let text = fs::read_to_string(&classifications)
        .unwrap_or_else(|error| panic!("read {}: {error}", classifications.display()));

    let control = "ess_domain::lib(ess_domain)::binding::enum::Representation";
    assert!(
        text.contains(&format!("\"{control}\"")),
        "control: the sibling enum in the same module is classified"
    );

    let missing: Vec<&str> = [
        "ess_domain::lib(ess_domain)::binding::enum::Resolution",
        "ess_domain::lib(ess_domain)::binding::enum::Resolution/variant/Established",
        "ess_domain::lib(ess_domain)::binding::enum::Resolution/variant/Undeclared",
        "ess_domain::lib(ess_domain)::binding::enum::Resolution/variant/Cyclic",
    ]
    .into_iter()
    .filter(|id| !text.contains(&format!("\"{id}\"")))
    .collect();
    assert!(
        missing.is_empty(),
        "unclassified concrete consumer entries, which `task check` refuses: {missing:#?}"
    );
}

/// Every intra-doc link in `binding.rs` names something that exists.
///
/// `Taskfile.yml`'s `doc-check` runs `cargo doc --workspace --no-deps` under
/// `RUSTDOCFLAGS: -D warnings`, so a link whose target does not resolve fails the integration
/// gate. `cargo test`, `cargo clippy` and `cargo fmt` all stay green on one.
#[test]
fn every_intra_doc_link_target_in_binding_names_a_type_the_crate_can_see() {
    let binding = fs::read_to_string(crate_root().join("src/binding.rs")).expect("read binding.rs");
    let sources = rust_sources(&crate_root().join("src"));
    let imported: Vec<&str> = binding
        .lines()
        .map(str::trim_start)
        .filter(|line| line.starts_with("use ") || line.starts_with("pub use "))
        .collect();

    let mut unresolved: Vec<String> = Vec::new();
    for (_, rest) in binding
        .match_indices("](")
        .map(|(at, _)| (at, &binding[at + 2..]))
    {
        let Some(target) = rest.split(')').next() else {
            continue;
        };
        let Some((head, _)) = target.split_once("::") else {
            continue;
        };
        if head == "Self" || !head.starts_with(|c: char| c.is_ascii_uppercase()) {
            continue;
        }
        let declared = ["enum ", "struct ", "trait ", "type ", "union "]
            .iter()
            .any(|kind| sources.contains(&format!("{kind}{head}")));
        let in_scope = imported.iter().any(|line| line.contains(head));
        if !declared && !in_scope {
            unresolved.push(target.to_owned());
        }
    }
    unresolved.sort();
    unresolved.dedup();
    assert!(
        unresolved.is_empty(),
        "broken intra-doc link targets in binding.rs, which `-D warnings` rustdoc refuses: \
         {unresolved:#?}"
    );
}
