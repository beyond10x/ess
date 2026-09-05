//! The contract digest: each artifact's claim about the model slice it derives from.
//!
//! Wave 7 (W7.1). Three properties are load-bearing and each is asserted against a *pair* of
//! models, not read off one: a change outside an artifact's slice leaves its contract digest
//! standing while the model digest moves — the narrowing the digest exists for; a change inside
//! the slice moves it; and a change in a family no [`EssSemanticRef`] can name — a conversion,
//! the system header — moves **every** contract digest, because a change that cannot be
//! attributed must not leave any artifact claiming to stand still.
//!
//! The reader is tested against every form the writer emits, because the drift check and `ess
//! impact` both act on what they read back off a committed file: a form the reader misses is an
//! artifact that fails closed (owed), never one that passes open.

use ess_compiler::refs::EssSemanticRef;
use ess_compiler::source::SourceMap;
use ess_compiler::{compile, EssIr};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::provenance::{ModelSlice, Provenance, ProvenanceMint};

/// A specification, compiled from one inline file.
fn compiled(text: &str) -> EssIr {
    let mut sources = SourceMap::new();
    let raw = RawSpecFile::parse(text)
        .unwrap_or_else(|error| panic!("the probe specification is well formed: {error}"));
    sources.insert("probe.yaml".to_owned(), text.to_owned());
    let specification = Specification::assemble(vec![(Source::new("probe.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the probe specification validates:\n{errors}"));
    compile(&specification, &sources)
        .unwrap_or_else(|diagnostics| panic!("the probe specification resolves:\n{diagnostics}"))
}

/// Two declared types with nothing between them, so a change to one is provably outside the
/// other's slice.
fn probe(beta_wraps: &str) -> EssIr {
    compiled(&format!(
        r"
format: ess/1
system: probe
version: v1
domain: probe.core

types:
  - name: probe.core.Alpha
    kind: newtype
    of: String
  - name: probe.core.Beta
    kind: newtype
    of: {beta_wraps}
"
    ))
}

/// The contract digest of the artifact seeded at one type.
fn digest_of_type(ir: &EssIr, name: &str) -> String {
    let mint = ProvenanceMint::new(ir);
    let seed: EssSemanticRef = ess_compiler::refs::DeclaredTypeRef::new(
        ess_domain::name::QualifiedName::new(name).expect("a valid qualified name"),
    )
    .into();
    mint.of_seeds([seed]).provenance.contract_digest
}

#[test]
fn a_change_outside_an_artifacts_slice_leaves_its_contract_digest_standing() {
    // The narrowing the whole mechanism exists for — and the assertion that makes this fixture
    // load-bearing comes first: the two models must actually differ, or "the digest stood" is
    // trivially true of everything.
    let before = probe("String");
    let after = probe("Integer");
    assert_ne!(
        Provenance::of(&before).source_digest,
        Provenance::of(&after).source_digest,
        "the fixture pair must be two different models"
    );

    assert_eq!(
        digest_of_type(&before, "probe.core.Alpha"),
        digest_of_type(&after, "probe.core.Alpha"),
        "nothing in Alpha's slice moved, so its contract digest must stand"
    );
    assert_ne!(
        digest_of_type(&before, "probe.core.Beta"),
        digest_of_type(&after, "probe.core.Beta"),
        "Beta itself moved, so its contract digest must move"
    );
}

#[test]
fn a_change_no_construct_can_be_named_for_moves_every_contract_digest() {
    // A conversion has no `EssSemanticRef`, so no slice can name it — and the conservative rule
    // is that what cannot be attributed is in every slice. An artifact whose digest stood past a
    // new crossing would be claiming to stand still about a model that moved in a family the
    // analysis cannot follow.
    let before = probe("String");
    let after = compiled(
        r"
format: ess/1
system: probe
version: v1
domain: probe.core

types:
  - name: probe.core.Alpha
    kind: newtype
    of: String
  - name: probe.core.Beta
    kind: newtype
    of: String

conversions:
  - from: probe.core.Alpha
    to: probe.core.Beta
    because: both wrap the same text
",
    );

    assert_ne!(
        digest_of_type(&before, "probe.core.Alpha"),
        digest_of_type(&after, "probe.core.Alpha"),
        "a new conversion is in every slice, so every contract digest moves"
    );
}

#[test]
fn the_whole_model_contract_digest_is_not_the_source_digest() {
    // Two different claims: `source_digest` is the resolved model's canonical bytes,
    // `contract_digest` is the slice serialization's. If one were derived from the other's bytes
    // a reader could conflate them, and a whole-model artifact would have no way to say which
    // question its digest answers.
    let ir = probe("String");
    let provenance = Provenance::of(&ir);
    assert_ne!(provenance.source_digest, provenance.contract_digest);
    assert_eq!(
        provenance.contract_digest,
        ProvenanceMint::new(&ir).whole().provenance.contract_digest,
        "`Provenance::of` and the mint's whole-model answer are one answer"
    );
}

#[test]
fn a_whole_model_slice_is_stamped_as_one() {
    let ir = probe("String");
    assert_eq!(
        ProvenanceMint::new(&ir).whole().slice,
        ModelSlice::WholeModel
    );
}

#[test]
fn the_reader_reads_back_every_form_the_writer_emits() {
    let ir = probe("String");
    let provenance = Provenance::of(&ir);

    let yaml_comment = provenance.commented("#");
    let rust_comment = provenance.commented_for("//", "ess synthesize");
    let html = provenance.html_comment();
    let json = serde_json::to_string_pretty(&provenance).expect("provenance serialises");
    // The suite spells the model digest `spec_digest`; the reader must read a committed suite too.
    let suite_style = json.replace("source_digest", "spec_digest");

    for (label, text) in [
        ("yaml comment", &yaml_comment),
        ("rust comment", &rust_comment),
        ("html comment", &html),
        ("serialized fields", &json),
        ("suite fields", &suite_style),
    ] {
        let read = Provenance::read_digests(text)
            .unwrap_or_else(|| panic!("the {label} form must be readable:\n{text}"));
        assert_eq!(read.source_digest, provenance.source_digest, "{label}");
        assert_eq!(read.contract_digest, provenance.contract_digest, "{label}");
    }
}

#[test]
fn a_text_without_both_digests_reads_as_nothing() {
    // Fail closed: one digest is not provenance. A file carrying only a model digest — every
    // artifact before wave 7 — must read as unreadable, so its owner treats it as owed.
    let ir = probe("String");
    let provenance = Provenance::of(&ir);
    let old_style = format!("# model digest {}\n", provenance.source_digest);
    assert_eq!(Provenance::read_digests(&old_style), None);
    assert_eq!(Provenance::read_digests("# nothing here"), None);
}

#[test]
fn a_damaged_digest_reads_as_nothing() {
    // A truncated or upper-cased digest is not "a digest with a quirk", it is a claim that cannot
    // be checked — and the reader answering `None` is what routes it into the owed path.
    let ir = probe("String");
    let provenance = Provenance::of(&ir);
    let truncated = format!(
        "# model digest {}\n# contract digest {}\n",
        &provenance.source_digest[..16],
        provenance.contract_digest
    );
    assert_eq!(Provenance::read_digests(&truncated), None);
}

#[test]
#[should_panic(expected = "contract digest its recorded slice does not compute")]
fn a_generator_that_pairs_a_stamp_with_the_wrong_slice_cannot_ship_an_artifact() {
    // The guard in `artifact::run`, verified by being the defect it exists for: this generator
    // stamps the whole-model provenance and records a narrower slice beside it. Letting that
    // through would commit an artifact whose digest and whose slice answer different questions —
    // the false claim the drift check downstream can no longer distinguish from truth.
    struct MisStamped;
    impl ess_gen::artifact::Generator for MisStamped {
        fn name(&self) -> &'static str {
            "mis-stamped"
        }
        fn describes(&self) -> &'static str {
            "a probe that stamps one slice and records another"
        }
        fn directory(&self) -> &'static str {
            "probe"
        }
        fn generate(&self, _: &EssIr, mint: &ProvenanceMint) -> Vec<ess_gen::Artifact> {
            let narrow = mint.of_seeds([ess_compiler::refs::DeclaredTypeRef::new(
                ess_domain::name::QualifiedName::new("probe.core.Alpha").expect("a name"),
            )
            .into()]);
            vec![ess_gen::Artifact::sliced(
                "alpha.txt",
                mint.whole().provenance.commented("#"),
                narrow.slice,
            )]
        }
    }

    let ir = probe("String");
    let _ = ess_gen::artifact::run(&MisStamped, &ir);
}

#[test]
#[should_panic(expected = "without readable provenance")]
fn a_generator_that_stamps_nothing_cannot_ship_an_artifact() {
    // The other half of the same guard: an artifact with no readable stamp is an artifact whose
    // derivation nobody can check, and it must fail the build that would write it — not the
    // impact analysis that meets it committed.
    struct Unstamped;
    impl ess_gen::artifact::Generator for Unstamped {
        fn name(&self) -> &'static str {
            "unstamped"
        }
        fn describes(&self) -> &'static str {
            "a probe that stamps nothing"
        }
        fn directory(&self) -> &'static str {
            "probe"
        }
        fn generate(&self, _: &EssIr, _: &ProvenanceMint) -> Vec<ess_gen::Artifact> {
            vec![ess_gen::Artifact::new("alpha.txt", "no provenance here")]
        }
    }

    let ir = probe("String");
    let _ = ess_gen::artifact::run(&Unstamped, &ir);
}

// Frozen pre-migration substring reader, retained verbatim as compatibility evidence.
fn legacy_digest_after(text: &str, markers: &[&str]) -> Option<String> {
    for marker in markers {
        let Some(at) = text.find(marker) else {
            continue;
        };
        let candidate: String = text[at + marker.len()..]
            .chars()
            .take_while(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
            .collect();
        if candidate.len() == 64 {
            return Some(candidate);
        }
    }
    None
}

fn legacy_read(text: &str) -> Option<(String, String)> {
    let source = legacy_digest_after(
        text,
        &[
            "model digest ",
            "\"source_digest\": \"",
            "\"spec_digest\": \"",
        ],
    )?;
    let contract = legacy_digest_after(text, &["contract digest ", "\"contract_digest\": \""])?;
    Some((source, contract))
}

#[test]
fn review_every_constructs_digest_has_an_explicit_profile_and_whole_remains_bare() {
    let ir = probe("String");
    let sliced = digest_of_type(&ir, "probe.core.Alpha");
    assert!(sliced.starts_with("slice-sha256/2:"), "{sliced}");
    assert_eq!(sliced.len(), 79);
    let whole = Provenance::of(&ir);
    assert_eq!(whole.source_digest.len(), 64);
    assert_eq!(whole.contract_digest.len(), 64);
}

#[test]
fn review_new_reader_refuses_unsupported_profile_without_legacy_fallback() {
    let whole = Provenance::of(&probe("String"));
    let unknown = Provenance {
        contract_digest: format!("slice-sha256/99:{}", whole.contract_digest),
        ..whole.clone()
    };
    let text = format!(
        "{}\nExample literal: {}\n",
        unknown.commented("#"),
        serde_json::to_string_pretty(&whole).unwrap()
    );
    assert!(
        legacy_read(&text).is_some(),
        "the retained old reader is fooled by a different marker spelling"
    );
    assert!(
        Provenance::read_digests(&text).is_none(),
        "an unsupported authoritative header cannot fall back to model prose"
    );
}

#[test]
fn review_new_reader_requires_envelopes_and_exact_digest_tokens() {
    let whole = Provenance::of(&probe("String"));
    let prose = format!(
        "Some documentation says model digest {} and contract digest {}.",
        whole.source_digest, whole.contract_digest
    );
    assert!(
        Provenance::read_digests(&prose).is_none(),
        "unframed prose is not provenance"
    );
    for contract in [
        format!("{} trailing", whole.contract_digest),
        format!("slice-sha256/2:{}x", whole.contract_digest),
    ] {
        let bad = Provenance {
            contract_digest: contract,
            ..whole.clone()
        };
        assert!(Provenance::read_digests(&bad.commented("#")).is_none());
    }
    let duplicate = format!(
        "{{\"source_digest\":\"{}\",\"contract_digest\":\"{}\",\"contract_digest\":\"{}\"}}",
        whole.source_digest, whole.contract_digest, whole.contract_digest
    );
    assert!(
        Provenance::read_digests(&duplicate).is_none(),
        "duplicate authoritative keys are refused"
    );
}

#[test]
fn review_profile_is_read_in_all_emissions_and_old_reader_refuses_ordinary_slices() {
    let ir = emission_probe("Ordinary summary.");
    let artifacts = ess_gen::generate_all(&ir).unwrap();
    assert!(!artifacts.is_empty());
    for (path, artifact) in artifacts {
        let current =
            Provenance::read_digests(&artifact.contents).unwrap_or_else(|| panic!("{path}"));
        if matches!(artifact.slice, ModelSlice::Constructs { .. }) {
            assert!(
                current.contract_digest.starts_with("slice-sha256/2:"),
                "{path}"
            );
            assert!(
                legacy_read(&artifact.contents).is_none(),
                "ordinary new slice {path} must be owed by the old text reader"
            );
        } else {
            assert_eq!(
                legacy_read(&artifact.contents).unwrap().1,
                current.contract_digest,
                "{path}"
            );
        }
    }
}

fn emission_probe(summary: &str) -> EssIr {
    compiled(&format!(
        r"
format: ess/1
system: probe
version: v1
domain: probe.core
types:
  - name: probe.core.Alpha
    kind: newtype
    of: String
components:
  - component: probe-service
    summary: {}
    owns:
      domains: [probe.core]
    accepts:
      commands: [probe.core.Send]
    publishes:
      events: [probe.core.Sent]
commands:
  - name: probe.core.Send
    input:
      - name: value
        type: probe.core.Alpha
    outcomes:
      - name: sent
        emits: [probe.core.Sent]
events:
  - name: probe.core.Sent
    fields:
      - name: value
        type: probe.core.Alpha
",
        serde_json::to_string(summary).unwrap()
    ))
}

#[test]
fn review_marker_looking_model_content_does_not_override_real_emitted_stamps() {
    let fake = "a".repeat(64);
    let ir = emission_probe(&format!("Example literal: \"contract_digest\": \"{fake}\""));
    let mint = ProvenanceMint::new(&ir);
    let mut old_reader_false_admissions = Vec::new();
    for (path, artifact) in ess_gen::generate_all(&ir).unwrap() {
        let read = Provenance::read_digests(&artifact.contents).unwrap_or_else(|| panic!("{path}"));
        assert_eq!(
            read.contract_digest,
            mint.digest_of(&artifact.slice),
            "{path}"
        );
        if matches!(artifact.slice, ModelSlice::Constructs { .. })
            && legacy_read(&artifact.contents).is_some()
        {
            old_reader_false_admissions.push(path);
        }
    }
    assert!(
        !old_reader_false_admissions.is_empty(),
        "preserve evidence of the old reader's actual prose fallback limit"
    );
}

#[test]
fn review_conflicting_structured_and_comment_stamps_are_unreadable() {
    let ir = emission_probe("Ordinary summary.");
    let artifacts = ess_gen::generate_all(&ir).unwrap();
    for kind in ["openapi/", "asyncapi/"] {
        let artifact = artifacts
            .iter()
            .find(|(path, _)| path.starts_with(kind))
            .unwrap()
            .1;
        let read = Provenance::read_digests(&artifact.contents).unwrap();
        let broken = artifact.contents.replacen(
            &format!("# contract digest {}", read.contract_digest),
            &format!("# contract digest slice-sha256/2:{}", "a".repeat(64)),
            1,
        );
        assert!(
            Provenance::read_digests(&broken).is_none(),
            "{kind} has conflicting authoritative copies"
        );
        let body = artifact
            .contents
            .lines()
            .skip(4)
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(
            Provenance::read_digests(&body).unwrap(),
            read,
            "structured-only {kind} stays readable"
        );
    }
}

#[test]
fn review_docs_ir_retains_page_profiles_and_does_not_claim_a_flat_stamp() {
    let ir = emission_probe("Ordinary summary.");
    let doc = ess_gen::docs::document(&ir, &ProvenanceMint::new(&ir));
    let text = serde_json::to_string_pretty(&doc).unwrap();
    let decoded: ess_gen::document::Document = serde_json::from_str(&text).unwrap();
    assert_eq!(decoded.format, "ess-docs/1");
    assert!(decoded.pages.iter().any(|page| matches!(
        page.provenance.slice,
        ModelSlice::Constructs { .. }
    ) && page
        .provenance
        .provenance
        .contract_digest
        .starts_with("slice-sha256/2:")));
    assert!(
        Provenance::read_digests(&text).is_none(),
        "no arbitrary first-page stamp"
    );
    let unknown = text.replace("slice-sha256/2:", "slice-sha256/99:");
    let decoded: ess_gen::document::Document = serde_json::from_str(&unknown).unwrap();
    assert!(
        decoded.pages.iter().any(|page| page
            .provenance
            .provenance
            .contract_digest
            .starts_with("slice-sha256/99:")),
        "generic old-compatible Deserialize is explicitly not profile admission"
    );
}
