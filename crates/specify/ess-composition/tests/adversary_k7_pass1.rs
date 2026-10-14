//! Adversary pass 1 on `story:a-component-plays-a-pinned-contract-role` (beyond10x/ess#525,
//! `docs/design/composition-contract-roles.md`).
//!
//! Cases named `control_*` hold the checker to what it must keep doing; the others probe where it
//! decides otherwise than the design page.

use ess_compiler::source::SourceMap;
use ess_compiler::{compile as compile_service, EssIr};
use ess_composition::{
    compile, CompiledService, CompositionDiagnostics, CompositionSpec, EssCompositionIr,
    ServiceKey, SourceDigest,
};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const LEFT_SYSTEM: &str = "format: ess/23
system: k7left
version: v1

domains:
  - k7left.cmd
  - k7left.other

components:
  - component: lc
    owns:
      domains: [k7left.cmd]
    accepts:
      commands:
        - k7left.cmd.Exact
        - k7left.cmd.ReqB
        - k7left.cmd.WireA
    reached_by: network
  - component: lo
    owns:
      domains: [k7left.other]
    accepts:
      commands:
        - k7left.other.Other
    reached_by: network
";

const LEFT_CMD: &str = "domain: k7left.cmd

actors:
  - name: k7left.cmd.Client
    may: [k7left.cmd.Exact, k7left.cmd.ReqB, k7left.cmd.WireA]

commands:
  # The contract's input exactly.
  - name: k7left.cmd.Exact
    input:
      - { name: a, type: String }
      - { name: b, type: Optional<String> }
    outcomes:
      - name: done
        emits: [k7left.cmd.Done]
        payload:
          k7left.cmd.Done:
            a: input.a

  # Requires `b`, which the contract's caller may leave out.
  - name: k7left.cmd.ReqB
    input:
      - { name: a, type: String }
      - { name: b, type: String }
    outcomes:
      - name: done
        emits: [k7left.cmd.Done]
        payload:
          k7left.cmd.Done:
            a: input.a

  # Reads `a` from another wire key than the contract's caller sends.
  - name: k7left.cmd.WireA
    input:
      - { name: a, type: String, wire: alpha }
      - { name: b, type: Optional<String> }
    outcomes:
      - name: done
        emits: [k7left.cmd.Done]
        payload:
          k7left.cmd.Done:
            a: input.a

events:
  - name: k7left.cmd.Done
    fields:
      - { name: a, type: String }
";

const LEFT_OTHER: &str = "domain: k7left.other

actors:
  - name: k7left.other.Client
    may: [k7left.other.Other]

commands:
  - name: k7left.other.Other
    input:
      - { name: a, type: String }
      - { name: b, type: Optional<String> }
    outcomes:
      - name: done
        emits: [k7left.other.Done]
        payload:
          k7left.other.Done:
            a: input.a

events:
  - name: k7left.other.Done
    fields:
      - { name: a, type: String }
";

const RIGHT_SYSTEM: &str = "format: ess/23
system: k7right
version: v1

domains:
  - k7right.api

components:
  - component: rc
    owns:
      domains: [k7right.api]
    accepts:
      commands:
        - k7right.api.Op
    reached_by: network
";

const RIGHT_API: &str = "domain: k7right.api

actors:
  - name: k7right.api.Client
    may: [k7right.api.Op]

commands:
  - name: k7right.api.Op
    input:
      - { name: a, type: String }
      - { name: b, type: Optional<String> }
    outcomes:
      - name: done
        emits: [k7right.api.Done]
        payload:
          k7right.api.Done:
            a: input.a

events:
  - name: k7right.api.Done
    fields:
      - { name: a, type: String }
";

fn compiled(files: &[(&str, &str)]) -> EssIr {
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for (label, text) in files {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}"));
        sources.insert((*label).to_owned(), (*text).to_owned());
        parsed.push((Source::new(*label), raw));
    }
    let specification = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("fixture validates:\n{errors}"));
    compile_service(&specification, &sources)
        .unwrap_or_else(|diagnostics| panic!("fixture resolves:\n{diagnostics}"))
}

struct Models {
    left: EssIr,
    right: EssIr,
}

impl Models {
    fn new() -> Self {
        Self {
            left: compiled(&[
                ("system.yaml", LEFT_SYSTEM),
                ("domains/cmd.yaml", LEFT_CMD),
                ("domains/other.yaml", LEFT_OTHER),
            ]),
            right: compiled(&[
                ("system.yaml", RIGHT_SYSTEM),
                ("domains/api.yaml", RIGHT_API),
            ]),
        }
    }

    /// `l` is `k7left`/`lc`, `o` is `k7left`/`lo` (the same system, another component) and `r` is
    /// `k7right`/`rc`, all at their real digests unless `right_digest` overrides `r`'s.
    fn document(&self, format: &str, right_digest: Option<&str>, rest: &str) -> String {
        let left = SourceDigest::of(&self.left).to_string();
        let right =
            right_digest.map_or_else(|| SourceDigest::of(&self.right).to_string(), str::to_owned);
        format!(
            "format: {format}\n\
             composition: k7\n\
             services:\n  \
               - {{key: l, system: k7left, version: v1, source_digest: {left}, component: lc}}\n  \
               - {{key: o, system: k7left, version: v1, source_digest: {left}, component: lo}}\n  \
               - {{key: r, system: k7right, version: v1, source_digest: {right}, component: rc}}\n\
             {rest}"
        )
    }

    fn compose(&self, text: &str) -> Result<EssCompositionIr, CompositionDiagnostics> {
        let specification = CompositionSpec::from_yaml(text)
            .unwrap_or_else(|error| panic!("the document reads: {error}\n{text}"));
        let (l, o, r) = (key("l"), key("o"), key("r"));
        compile(
            &specification,
            [
                CompiledService::new(&l, &self.left),
                CompiledService::new(&o, &self.left),
                CompiledService::new(&r, &self.right),
            ],
        )
    }
}

fn key(value: &str) -> ServiceKey {
    ServiceKey::new(value).expect("valid service key")
}

fn findings(diagnostics: &CompositionDiagnostics) -> Vec<(String, String)> {
    diagnostics
        .as_slice()
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.code().to_string(),
                diagnostic.detail().to_owned(),
            )
        })
        .collect()
}

fn role(local: &str, stands: &str, contract: &str) -> String {
    format!(
        "roles:\n  \
           - service: l\n    \
             plays: r\n    \
             commands:\n      \
               - {{local: {local}, stands: {stands}, contract: {contract}}}\n"
    )
}

/// The design page, Compatibility: "A `/1`, `/2` or `/3` document carrying `roles`, even `[]`, is
/// refused when read as an unknown field, as before `/4` existed." Before `/4`, the closed reader
/// refused the key whatever its value; `roles: null` (YAML `roles:` or `roles: ~`, JSON `null`)
/// is a document carrying `roles`.
#[test]
fn an_earlier_document_carrying_a_null_roles_key_is_refused_when_read() {
    let models = Models::new();
    for format in [
        "ess-composition/1",
        "ess-composition/2",
        "ess-composition/3",
    ] {
        for roles in ["roles:\n", "roles: ~\n", "roles: null\n"] {
            let text = models.document(format, None, roles);
            let read = CompositionSpec::from_yaml(&text);
            assert!(
                read.as_ref()
                    .is_err_and(|error| error.to_string().contains("unknown field `roles`")),
                "{format} with {roles:?} must not read, as before /4: {read:?}"
            );
        }
        let json = format!(
            "{{\"format\":\"{format}\",\"composition\":\"k7\",\"services\":[],\"roles\":null}}"
        );
        let read = CompositionSpec::from_json(&json);
        assert!(
            read.as_ref()
                .is_err_and(|error| error.to_string().contains("unknown field `roles`")),
            "{format} JSON with `roles: null` must not read: {read:?}"
        );
    }
}

/// The contract's caller may leave `b` out; a local command that requires it refuses that call.
#[test]
fn an_implements_entry_requiring_an_input_the_contract_makes_optional_is_refused() {
    let models = Models::new();
    let diagnostics = models
        .compose(&models.document(
            "ess-composition/4",
            None,
            &role("k7left.cmd.ReqB", "implements", "k7right.api.Op"),
        ))
        .expect_err("a required local `b` does not answer an optional contract `b`");
    let all = findings(&diagnostics);
    assert_eq!(all.len(), 1, "{all:#?}");
    assert_eq!(all[0].0, "role_input_drift", "{all:#?}");
    assert!(all[0].1.starts_with("roles[0].commands[0]"), "{all:#?}");
    assert!(all[0].1.contains("field `b`"), "{all:#?}");
}

/// The design page, check 4: "the same name and wire name".
#[test]
fn an_implements_entry_reading_another_wire_name_is_refused() {
    let models = Models::new();
    let diagnostics = models
        .compose(&models.document(
            "ess-composition/4",
            None,
            &role("k7left.cmd.WireA", "implements", "k7right.api.Op"),
        ))
        .expect_err("a local `a` read from `alpha` does not answer a contract `a` sent as `a`");
    let all = findings(&diagnostics);
    assert_eq!(all.len(), 1, "{all:#?}");
    assert_eq!(all[0].0, "role_input_drift", "{all:#?}");
    assert!(all[0].1.contains("alpha"), "{all:#?}");
}

#[test]
fn control_an_exact_implements_entry_and_a_same_system_role_compile_in_document_order() {
    let models = Models::new();
    let text = models.document(
        "ess-composition/4",
        None,
        "roles:\n  \
           - service: r\n    \
             plays: l\n  \
           - service: l\n    \
             plays: r\n    \
             commands:\n      \
               - {local: k7left.cmd.Exact, stands: implements, contract: k7right.api.Op}\n  \
           - service: l\n    \
             plays: o\n    \
             commands:\n      \
               - {local: k7left.cmd.Exact, stands: implements, contract: k7left.other.Other}\n",
    );
    let first = models
        .compose(&text)
        .unwrap_or_else(|diagnostics| panic!("composes:\n{diagnostics}"));
    let second = models.compose(&text).expect("composes again");
    assert_eq!(first.to_canonical_json(), second.to_canonical_json());
    let order: Vec<_> = first
        .roles()
        .iter()
        .map(|role| {
            (
                role.service().key().to_string(),
                role.plays().key().to_string(),
                role.plays().component().to_string(),
            )
        })
        .collect();
    assert_eq!(
        order,
        [
            ("r".to_owned(), "l".to_owned(), "lc".to_owned()),
            ("l".to_owned(), "r".to_owned(), "rc".to_owned()),
            ("l".to_owned(), "o".to_owned(), "lo".to_owned()),
        ]
    );
}

/// A role naming an import whose pinned digest is not its compiled model's: the pin is refused,
/// and the role adds no refusal of its own measured against a model the pin does not name.
#[test]
fn control_a_role_naming_a_digest_mismatched_import_is_refused_at_the_pin() {
    let models = Models::new();
    let wrong = SourceDigest::of(&models.left).to_string();
    let diagnostics = models
        .compose(&models.document(
            "ess-composition/4",
            Some(&wrong),
            &role("k7left.cmd.Exact", "implements", "k7right.api.Op"),
        ))
        .expect_err("a digest mismatch is refused");
    let all = findings(&diagnostics);
    assert!(
        all.iter().any(|(code, _)| code == "digest_mismatch"),
        "{all:#?}"
    );
    assert!(
        all.iter().all(|(_, detail)| !detail.starts_with("roles[")),
        "{all:#?}"
    );
}

/// A `/4` document read and written back reads as the same document.
#[test]
fn control_a_composition_4_document_round_trips_through_canonical_json() {
    let models = Models::new();
    for rest in [
        "roles: []\n".to_owned(),
        role("k7left.cmd.Exact", "gates", "k7right.api.Op"),
    ] {
        let text = models.document("ess-composition/4", None, &rest);
        let spec = CompositionSpec::from_yaml(&text).expect("reads");
        let again = CompositionSpec::from_json(&spec.to_canonical_json()).expect("reads back");
        assert_eq!(spec, again);
    }
}
