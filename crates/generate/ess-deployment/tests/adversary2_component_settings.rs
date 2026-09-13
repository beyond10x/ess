//! Adversary pass 2 against `story:component-declares-its-settings`, at the projection.
//!
//! Two states the derivation reaches that `tests/deployment.rs` does not build: a setting the model
//! says may be absent and also says is a secret, and two components of one workload declaring one
//! setting name. Both are `review-result:adversary-wave25-unit3-pass-2`, F1 and F2, and each is
//! followed here by the enumeration of its class rather than by its one instance.
//!
//! The helpers below are copies of `tests/deployment.rs`'s, because an integration test cannot see
//! another integration test's items. Nothing in that file is edited by this one.

use std::path::{Path, PathBuf};

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_deployment::{compile_build, compile_runtime, BuildIr, BuildSpec, RuntimeSpec};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples/oracle-fixture")
        .canonicalize()
        .expect("fixture exists")
}

fn build_spec_yaml() -> String {
    format!(
        r#"
format: ess-build/1
build: oracle-runtime
platforms:
  - os: linux
    architecture: amd64
secrets: [registry-token]
nodes:
  - id: base
    kind: oci_base
    reference: docker.io/library/alpine
    digest: sha256:{zeros}
  - id: source
    kind: source
    path: .
    destination: /src
  - id: compile
    kind: run
    base: base
    argv: [cp, /src/oracle, /usr/local/bin/oracle]
    mounts:
      - kind: input
        from: source
        target: /src
  - id: runtime-image
    kind: image
    rootfs: compile
    config:
      entrypoint: [/usr/local/bin/oracle]
      user: "10001"
  - id: chart-file
    kind: artifact
    from: compile
    path: /src/chart.tgz
outputs:
  - name: app
    release_unit: oracle-runtime
    node: runtime-image
    kind: oci_image
    repository: registry.example/oracle
  - name: chart
    release_unit: oracle-chart
    node: chart-file
    kind: helm_chart
"#,
        zeros = "0".repeat(64)
    )
}

fn build() -> BuildIr {
    compile_build(&BuildSpec::from_yaml(&build_spec_yaml()).expect("build fixture parses"))
        .expect("build compiles")
}

fn physical_realization(semantic: &ess_compiler::EssIr) -> ess_realization::RealizationIr {
    let source = format!(
        r"type: ess-realization/1
id: oracle-implementation
specification:
  system: oracle
  version: v1
  source_digest: sha256:{semantic_digest}
synthesis:
  target: rust-linux-amd64/1
  generator: ess/0.8.0
components: [order-service, dispatch-service]
actors: []
implementations:
  - id: oracle-binary
    components: [order-service, dispatch-service]
    artifact:
      kind: source
      locator: https://example.invalid/oracle.git
      identity: git:{commit}
entrypoints:
  - id: http-api
    title: HTTP API
    summary: Invoke the Oracle API.
    primary: true
    interaction: invoke
    attachment: network
    availability: internal
    support: preview
    implementation: oracle-binary
    actors: []
    surfaces:
      - kind: command
        name: oracle.order.PlaceOrder
    invocation:
      kind: url
      url: http://127.0.0.1:8080
",
        semantic_digest = semantic.source_digest(),
        commit = "a".repeat(40),
    );
    let specification =
        ess_realization::RealizationSpec::from_yaml(&source).expect("realization fixture parses");
    ess_realization::compile(&specification, semantic).expect("physical realization compiles")
}

/// The fixture's semantic model with a `settings:` block spliced under each named component.
fn semantic_with_settings_on(settings: &[(&str, &str)]) -> ess_compiler::EssIr {
    try_semantic_with_settings_on(settings)
        .unwrap_or_else(|refusal| panic!("the fixture is accepted: {refusal}"))
}

/// The same, for a settings block the specification may refuse.
///
/// The refusal is rendered to a string rather than typed, because the two layers that can raise
/// one here — `Specification::assemble` and `ess_compiler::resolve::compile` — carry different
/// error types and the cases below ask only what the refusal *names*.
fn try_semantic_with_settings_on(settings: &[(&str, &str)]) -> Result<ess_compiler::EssIr, String> {
    let root = fixture();
    let mut sources = SourceMap::new();
    let parsed: Vec<_> = [
        "system.yaml",
        "components.yaml",
        "domains/order.yaml",
        "domains/dispatch.yaml",
    ]
    .into_iter()
    .map(|label| {
        let mut text = std::fs::read_to_string(root.join(label)).expect("read fixture");
        if label == "components.yaml" {
            for (component, block) in settings {
                let anchor = format!("  - component: {component}\n");
                assert!(
                    text.contains(&anchor),
                    "the fixture declares the component {component}"
                );
                text = text.replace(&anchor, &format!("{anchor}    settings:\n{block}"));
            }
        }
        let raw = RawSpecFile::parse(&text).expect("parse fixture");
        sources.insert(label.to_owned(), text);
        (Source::new(label), raw)
    })
    .collect();
    let specification = Specification::assemble(parsed).map_err(|errors| errors.to_string())?;
    compile(&specification, &sources).map_err(|diagnostics| {
        diagnostics
            .as_slice()
            .iter()
            .map(|diagnostic| {
                format!(
                    "{} {} {}",
                    diagnostic.code,
                    diagnostic.message,
                    diagnostic.hint.as_deref().unwrap_or_default()
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    })
}

/// The fixture runtime document with one workload and no slot of its own.
fn runtime_spec_with_workloads(
    semantic: &ess_compiler::EssIr,
    physical: &ess_realization::RealizationIr,
    build: &BuildIr,
    workloads: &str,
) -> RuntimeSpec {
    RuntimeSpec::from_yaml(&format!(
        r"
format: ess-runtime/1
runtime: oracle-runtime
semantic_digest: sha256:{semantic_digest}
realization_digest: {realization_digest}
build_digest: {build_digest}
processes:
  - name: server
    image: app
containers:
  - name: server
    process: server
    http_port: 8080
{workloads}provided_endpoints:
  - name: api
    workload: oracle
    container: server
    scheme: http
",
        semantic_digest = semantic.source_digest(),
        realization_digest = physical.realization_digest(),
        build_digest = build.digest(),
    ))
    .expect("runtime fixture parses")
}

const ONE_WORKLOAD: &str = "\
workloads:
  - name: oracle
    components: [order-service, dispatch-service]
    containers: [server]
    replicas: 1
";

fn rendered(diagnostics: &ess_deployment::Diagnostics) -> String {
    diagnostics
        .as_slice()
        .iter()
        .map(|diagnostic| {
            format!(
                "{:?} {} {}",
                diagnostic.code(),
                diagnostic
                    .subject()
                    .map_or_else(String::new, ToString::to_string),
                diagnostic.detail()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// A secret the model says may be absent must not become a secret the deployment gate demands.
///
/// `ComponentSpec::validate_settings` accepted `secret: true` beside `Optional<…>` — silence over
/// an `Optional` type is not a contradiction, and `requires_a_value` answers `false`. The
/// derivation then built a [`SecretSlot`], which carries `name`, `environment` and `key` and
/// nothing that says the value may be absent, and `environment.rs` requires **every** secret slot
/// to be bound: "required secret slot {slot} is unbound". So the one statement a settings list
/// exists to make about this value was accepted, carried into the IR, and dropped at the
/// projection with no refusal and no note.
///
/// The resolution taken is to refuse the declaration where it is made rather than to give
/// [`SecretSlot`] a kind. A kind is a new statement in a persisted format — `ess-runtime/1`'s slot,
/// the composition lock's secret set, and the environment gate that reads it — and the model has
/// no way to say "optional secret" at any of the three. Refusing says the same thing at the
/// declaration, where the author can act on it, and keeps the projection total.
#[test]
fn an_optional_secret_setting_does_not_lose_the_one_thing_it_declared() {
    let refusal = try_semantic_with_settings_on(&[(
        "order-service",
        "      - name: slack-bot-token\n        type: Optional<oracle.order.Email>\n        \
         secret: true\n",
    )])
    .err()
    .unwrap_or_else(|| {
        panic!(
            "`secret: true` over `Optional<oracle.order.Email>` is accepted, and the derived \
             `SecretSlot` carries `name`, `environment` and `key` and nothing that says the value \
             may be absent, while `environment.rs` refuses every unbound secret slot with \
             \"required secret slot … is unbound\": the declaration is accepted and then discarded"
        )
    });
    assert!(
        refusal.contains("slack-bot-token"),
        "the refusal must name the setting it is about: {refusal}"
    );
    assert!(
        refusal.contains("secret") && refusal.contains("Optional<oracle.order.Email>"),
        "and must name both halves of the contradiction, because either one alone is legal: \
         {refusal}"
    );
}

/// The class F1 belongs to, enumerated: no accepted setting may state something its slot drops.
///
/// F1 is one cell of a cube, and the cube is what a settings list can say about one value:
/// `secret`, a literal `value:`, and whether the type admits absence. Eight cells. For each, the
/// declaration is either refused — in which case nothing is dropped — or it derives a slot, and
/// then every statement the setting made must be recoverable from that slot.
///
/// The recoverability is checked field by field, so the enumeration closes the class rather than
/// the instance: a ninth statement added to `ComponentSetting`, or a cell that changes from
/// refused to accepted, fails here unless the derived slot carries it.
///
/// [`SecretSlot`] is the whole difficulty: it has three fields and none of them is a kind, so the
/// only way an accepted secret can be free of a dropped statement is if it always requires a
/// value. That is what the sixth refusal buys, and this says so as an assertion rather than as a
/// paragraph.
#[test]
fn every_accepted_setting_derives_a_slot_that_carries_every_statement_it_made() {
    for secret in [false, true] {
        for value in [None, Some("fixed")] {
            for optional in [false, true] {
                let type_ref = if optional {
                    "Optional<oracle.order.Email>"
                } else {
                    "oracle.order.Email"
                };
                let block = format!(
                    "      - name: log-level\n        type: {type_ref}\n{}{}",
                    if secret { "        secret: true\n" } else { "" },
                    value.map_or_else(String::new, |literal| format!(
                        "        value: \"{literal}\"\n"
                    )),
                );
                let cell = format!("secret={secret} value={value:?} type={type_ref}");

                let Ok(semantic) = try_semantic_with_settings_on(&[("order-service", &block)])
                else {
                    // Refused where it is declared: nothing reaches a slot, so nothing is dropped.
                    continue;
                };
                let build = build();
                let physical = physical_realization(&semantic);
                let specification =
                    runtime_spec_with_workloads(&semantic, &physical, &build, ONE_WORKLOAD);
                let compiled = compile_runtime(&specification, &semantic, &physical, &build)
                    .unwrap_or_else(|diagnostics| {
                        panic!(
                            "{cell} is accepted, so it compiles: {}",
                            rendered(&diagnostics)
                        )
                    });
                let container = &compiled.containers()[&"server".parse().unwrap()];

                if secret {
                    assert_eq!(container.secrets.len(), 1, "{cell} derived no secret slot");
                    assert!(
                        !optional,
                        "{cell} is accepted and derives a `SecretSlot`, which carries `name`, \
                         `environment` and `key` and no field for absence, while `environment.rs` \
                         requires every secret slot in a locked runtime to be bound. An accepted \
                         secret setting whose type admits absence therefore states something the \
                         projection drops"
                    );
                    let slot = &container.secrets[0];
                    assert_eq!(slot.name.to_string(), "log-level", "{cell} kept the name");
                    assert_eq!(slot.environment, "LOG_LEVEL", "{cell} kept the variable");
                    assert_eq!(slot.key, "log-level", "{cell} kept the key");
                    assert!(
                        value.is_none(),
                        "{cell} is accepted with a literal beside a secret, and `SecretSlot` \
                         carries no value"
                    );
                    continue;
                }

                assert_eq!(container.config.len(), 1, "{cell} derived no config slot");
                let slot = &container.config[0];
                assert_eq!(slot.name.to_string(), "log-level", "{cell} kept the name");
                assert_eq!(slot.environment, "LOG_LEVEL", "{cell} kept the variable");
                assert_eq!(
                    slot.value.as_deref(),
                    value,
                    "{cell} must carry the literal it fixed, and only that"
                );
                let expected = match (value, optional) {
                    (Some(_), _) => ess_deployment::ConfigKind::Literal,
                    (None, false) => ess_deployment::ConfigKind::Required,
                    (None, true) => ess_deployment::ConfigKind::Optional,
                };
                assert_eq!(
                    slot.kind, expected,
                    "{cell} must derive the kind its statements determine; the kind is the only \
                     field a `ConfigSlot` has to carry them in"
                );
            }
        }
    }
}

/// A collision between two derived slots must name what collided.
///
/// The correction's own reason for moving the derivation above `validate_container` is that "a
/// derived slot is held to the same uniqueness rule a hand-authored one is". It is — and the
/// refusal that resulted was `validate_container`'s, which names the container role and neither the
/// setting nor the component that declared it. The runtime document contains no `config:` block at
/// all, so the author was told that a document they can read hand-authored a duplicate slot that is
/// not in it. That is the defect F2 of `review-result:adversary-wave25-unit3-pass-1` named, reached
/// through the other door, and this door is the fixture's own shape: `oracle` realizes
/// `order-service` and `dispatch-service` in the one container role `server`.
#[test]
fn two_components_of_one_workload_colliding_on_a_setting_name_are_refused_naming_the_setting() {
    let declaration =
        "      - name: log-level\n        type: oracle.order.Email\n        required: true\n";
    let semantic = semantic_with_settings_on(&[
        ("order-service", declaration),
        ("dispatch-service", declaration),
    ]);
    let build = build();
    let physical = physical_realization(&semantic);
    let specification = runtime_spec_with_workloads(&semantic, &physical, &build, ONE_WORKLOAD);

    let diagnostics = compile_runtime(&specification, &semantic, &physical, &build).expect_err(
        "two components declaring one setting name cannot both bind LOG_LEVEL in one container",
    );
    let text = rendered(&diagnostics);
    assert!(
        text.contains("log-level") || text.contains("LOG_LEVEL"),
        "the refusal must name the setting that collided; it names only the container role, and \
         the runtime document under test declares no configuration slot at all, so an author is \
         sent to look for a duplicate in a document that has none:\n{text}"
    );
    assert!(
        text.contains("order-service") && text.contains("dispatch-service"),
        "and it must name both components whose declarations collided, as the hand-authored \
         refusal beside it does:\n{text}"
    );
}

/// The class F2 belongs to, enumerated: every route into `validate_container`'s anonymous duplicate.
///
/// The defect is not "two components collide"; it is that a derived slot can reach
/// `validate_container`, which holds a list of slots and neither a setting name nor a component
/// name, and reports "container `server` has a duplicate configuration slot or environment
/// variable" about a document that may declare no slot at all. There are four routes a derived
/// slot can take to a duplicate, and each is either refused before `validate_container` sees it or
/// cannot happen:
///
/// | route | where it is answered |
/// |---|---|
/// | two settings of one component | `ComponentSpec::validate_settings`, at the declaration |
/// | two components of one container | `derive_component_settings`, named below |
/// | a derived slot against a hand-authored `config:`/`secrets:` slot | the two-authors refusal |
/// | a derived slot against a hand-authored `endpoints:` slot | `derive_component_settings`, named below |
///
/// The fourth is the one the named fix did not cover and the one this case exists for: an endpoint
/// slot is *never* derived, so it raises no two-authors refusal, and it occupies a name and an
/// environment variable in the same container that `validate_container` checks against one set.
#[test]
fn a_derived_slot_colliding_with_a_hand_authored_endpoint_is_refused_naming_both() {
    let semantic = semantic_with_settings_on(&[(
        "order-service",
        "      - name: carrier-api\n        type: oracle.order.Email\n        required: true\n",
    )]);
    let build = build();
    let physical = physical_realization(&semantic);
    let specification = RuntimeSpec::from_yaml(&format!(
        r"
format: ess-runtime/1
runtime: oracle-runtime
semantic_digest: sha256:{semantic_digest}
realization_digest: {realization_digest}
build_digest: {build_digest}
processes:
  - name: server
    image: app
containers:
  - name: server
    process: server
    http_port: 8080
    endpoints:
      - name: carrier-api
        environment: CARRIER_API
        system: carrier
{ONE_WORKLOAD}provided_endpoints:
  - name: api
    workload: oracle
    container: server
    scheme: http
",
        semantic_digest = semantic.source_digest(),
        realization_digest = physical.realization_digest(),
        build_digest = build.digest(),
    ))
    .expect("runtime fixture parses");

    let diagnostics = compile_runtime(&specification, &semantic, &physical, &build)
        .expect_err("one container cannot bind CARRIER_API twice");
    let text = rendered(&diagnostics);
    assert!(
        text.contains("carrier-api") || text.contains("CARRIER_API"),
        "the refusal must name what collided; `validate_container`'s does not, and an endpoint \
         slot raises no two-authors refusal because an endpoint slot is never derived:\n{text}"
    );
    assert!(
        text.contains("order-service"),
        "and it must name the component whose declaration derived the other side:\n{text}"
    );
    assert!(
        text.contains("endpoint"),
        "and it must say which hand-authored slot it collided with, because the author is looking \
         at a document with no `config:` block in it:\n{text}"
    );
}
