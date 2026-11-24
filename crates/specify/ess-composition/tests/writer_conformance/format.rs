//! `ess-composition/6`: what reads, what is refused, and the bytes of every earlier format.

use crate::common::{key, Pair};
use crate::presence::{pair, plain_entry};
use ess_compiler::refs::{ComponentRef, DeclaredTypeRef};
use ess_composition::{
    CompositionCode, CompositionSpec, ServiceImportSpec, TypeBinding, TypeConformance,
    WRITER_COMPOSITION_FORMAT,
};
use ess_domain::component::ComponentName;
use ess_domain::name::QualifiedName;

#[test]
fn writer_together_with_reader_is_refused_when_read_naming_both_keys() {
    let models = pair();
    let text = models.document(
        "ess-composition/6",
        &Pair::entry(
            "type: relay.session.Required",
            "type: wire.session.Required",
            "    writer: true\n    reader: true\n",
        ),
    );
    let error = CompositionSpec::from_yaml(&text)
        .expect_err("both keys are refused when read")
        .to_string();
    for wanted in ["writer", "reader", "a conformance entry carries"] {
        assert!(error.contains(wanted), "{wanted}: {error}");
    }
    for extra in [
        "    writer: true\n    reader: false\n",
        "    writer: false\n    reader: true\n",
    ] {
        let text = models.document(
            "ess-composition/6",
            &Pair::entry(
                "type: relay.session.Required",
                "type: wire.session.Required",
                extra,
            ),
        );
        models.composes(&text);
    }
}

#[test]
fn writer_under_an_earlier_format_is_refused_as_unsupported_format_naming_the_new_one() {
    let models = pair();
    for format in [
        "ess-composition/2",
        "ess-composition/3",
        "ess-composition/4",
        "ess-composition/5",
    ] {
        for line in [
            "    writer: true\n",
            "    writer: false\n",
            "    writer: null\n",
            "    writer:\n",
        ] {
            let text = models.document(
                format,
                &Pair::entry(
                    "type: relay.session.Required",
                    "type: wire.session.Required",
                    line,
                ),
            );
            let diagnostics = models
                .compose(&text)
                .expect_err(&format!("{format} with {line:?} is refused"));
            let all = diagnostics.as_slice();
            assert_eq!(all.len(), 1, "{format} {line:?}: {diagnostics}");
            assert_eq!(all[0].code(), CompositionCode::UnsupportedFormat);
            assert!(
                all[0].detail().contains("ess-composition/6"),
                "{format} {line:?}: {}",
                all[0].detail()
            );
        }
    }
}

#[test]
fn a_document_without_writer_keeps_its_authored_and_compiled_bytes() {
    let models = pair();
    let text = models.document("ess-composition/5", &plain_entry("Required", "Required"));
    let specification = CompositionSpec::from_yaml(&text).expect("reads");
    assert!(!specification.to_canonical_json().contains("writer"));
    let compiled = models.composes(&text).to_canonical_json();
    assert_eq!(compiled, BYTES_V5);
}

#[test]
fn the_writer_key_is_kept_as_authored_in_the_compiled_composition() {
    let models = pair();
    for (written, kept) in [("true", "true"), ("false", "false"), ("null", "null")] {
        let text = models.document(
            "ess-composition/6",
            &Pair::entry(
                "type: relay.session.Required",
                "type: wire.session.Required",
                &format!("    writer: {written}\n"),
            ),
        );
        let json = models.composes(&text).to_canonical_json();
        let entry = json
            .split("\"conformances\"")
            .nth(1)
            .expect("the compiled document has conformances");
        let local = entry.find("\"local\"").expect("local");
        let imported = entry.find("\"conforms_to\"").expect("conforms_to");
        let writer = entry
            .find(&format!("\"writer\": {kept}"))
            .unwrap_or_else(|| panic!("`writer: {kept}` is kept:\n{json}"));
        assert!(local < imported && imported < writer, "{json}");
        let read = CompositionSpec::from_yaml(&text).expect("reads");
        assert_eq!(read.conformances()[0].writer(), written == "true");
        assert!(json.contains("\"format\": \"ess-composition/6\""));
    }

    let imports = vec![
        ServiceImportSpec::of(
            key("relay"),
            ComponentRef::new(ComponentName::new("relay-component").expect("component")),
            &models.relay,
        ),
        ServiceImportSpec::of(
            key("wire"),
            ComponentRef::new(ComponentName::new("wire-server").expect("component")),
            &models.wire,
        ),
    ];
    let declared = |name: &str| DeclaredTypeRef::new(QualifiedName::new(name).expect("name"));
    let asserted = TypeConformance::for_writer(
        TypeBinding::new(key("relay"), declared("relay.session.Required")),
        TypeBinding::new(key("wire"), declared("wire.session.Required")),
    );
    assert!(asserted.writer());
    let built = CompositionSpec::with_writer_conformances(
        key("relay-wire"),
        imports,
        Vec::new(),
        vec![asserted],
        Vec::new(),
    );
    assert_eq!(built.format(), WRITER_COMPOSITION_FORMAT);
    let json = built.to_canonical_json();
    assert!(json.contains("\"format\": \"ess-composition/6\""), "{json}");
    assert!(json.contains("\"writer\": true"), "{json}");
    assert_eq!(
        CompositionSpec::from_json(&json).expect("reads back"),
        built
    );
}

const CODE_TYPES: &str = "
types:
  - name: SYSTEM.session.Code
    kind: newtype
    of: String
    alphabet: \"ALPHABET\"
";

fn newtype_domain(system: &str, alphabet: &str) -> String {
    format!(
        "domain: {system}.session\n{}\nevents:\n  - name: {system}.session.Ping\n    fields:\n      - {{ name: code, type: String }}\n",
        CODE_TYPES
            .replace("SYSTEM", system)
            .replace("ALPHABET", alphabet)
    )
}

#[test]
fn a_v6_document_reads_roles_and_payload_ends_and_compares_value_constraints_as_v5_does() {
    let models = Pair::with_surface(
        &newtype_domain("relay", "abc"),
        (&[], &["relay.session.Ping"]),
        &newtype_domain("wire", "ab"),
        (&[], &["wire.session.Ping"]),
    );
    let with_roles = models.document("ess-composition/6", "roles: []\n");
    CompositionSpec::from_yaml(&with_roles).expect("`roles` reads under /6");
    let under_3 = models.document("ess-composition/3", "roles: []\n");
    let refused = CompositionSpec::from_yaml(&under_3).expect_err("`roles` is unknown under /3");
    assert!(
        refused.to_string().contains("unknown field `roles`"),
        "{refused}"
    );

    let event = Pair::entry("event: relay.session.Ping", "event: wire.session.Ping", "");
    models.composes(&models.document("ess-composition/6", &event));

    let newtype = Pair::entry("type: relay.session.Code", "type: wire.session.Code", "");
    for format in ["ess-composition/6", "ess-composition/5"] {
        let detail = models.drift(&models.document(format, &newtype));
        assert!(detail.contains("alphabet"), "{format}: {detail}");
    }
    models.composes(&models.document("ess-composition/4", &newtype));
}

/// The `/5` document above, compiled by the tree before `/6` existed.
const BYTES_V5: &str = "{\n  \"format\": \"ess-composition/5\",\n  \"composition\": \"relay-wire\",\n  \"services\": {\n    \"relay\": {\n      \"system\": \"relay\",\n      \"version\": 1,\n      \"source_digest\": \"a3af5e4b76739d132cd12a550f4d852958dd77134a10e13bebd060f117939e8f\",\n      \"component\": \"relay-component\",\n      \"commands\": [],\n      \"queries\": [],\n      \"events\": [],\n      \"errors\": [],\n      \"types\": []\n    },\n    \"wire\": {\n      \"system\": \"wire\",\n      \"version\": 1,\n      \"source_digest\": \"aeca6c7c34c5b9b61b7cb2bac38cf4d559c9abf79ad81eeb86659c0f44ff9920\",\n      \"component\": \"wire-server\",\n      \"commands\": [],\n      \"queries\": [],\n      \"events\": [],\n      \"errors\": [],\n      \"types\": []\n    }\n  },\n  \"conformances\": [\n    {\n      \"local\": {\n        \"service\": \"relay\",\n        \"type\": \"relay.session.Required\"\n      },\n      \"conforms_to\": {\n        \"service\": \"wire\",\n        \"type\": \"wire.session.Required\"\n      }\n    }\n  ]\n}\n";
