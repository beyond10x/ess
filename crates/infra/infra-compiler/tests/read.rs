//! The read-back guarantees: a persisted document round-trips, and every way a document can
//! lie — edited content, a hand-written `resolved` claim, a foreign format — is refused with
//! its own code.

use infra_domain::observation::Observation;
use infra_domain::raw::RawBundle;
use infra_domain::InfraCode;

/// A bundle whose compilation mints every handle kind: node, service, configmap, secret,
/// service account and claim all have at least one resolved reference site.
fn bundle() -> serde_json::Value {
    serde_json::json!({
        "format": "infra-observation/1",
        "context": "read-back",
        "scanned_at": "2026-08-21T08:00:00Z",
        "scout_version": "0.1.0",
        "kinds": {
            "namespaces": { "items": [ { "metadata": { "name": "app", "uid": "ns-1" } } ] },
            "nodes": { "items": [ { "metadata": { "name": "node-a", "uid": "no-1" } } ] },
            "deployments": { "items": [
                { "metadata": { "name": "web", "namespace": "app", "uid": "d-1" },
                  "spec": {
                    "replicas": 2,
                    "selector": { "matchLabels": { "app": "web" } },
                    "template": {
                      "metadata": { "labels": { "app": "web" } },
                      "spec": {
                        "serviceAccountName": "runner",
                        "containers": [{
                          "name": "main", "image": "web:1",
                          "env": [
                            { "name": "MODE", "valueFrom": { "configMapKeyRef": {
                                "name": "settings", "key": "mode" } } },
                            { "name": "TOKEN", "valueFrom": { "secretKeyRef": {
                                "name": "creds", "key": "token" } } }
                          ]
                        }],
                        "volumes": [
                          { "name": "state", "persistentVolumeClaim": { "claimName": "data" } }
                        ]
                      }
                    }
                  } }
            ] },
            "statefulsets": { "items": [
                { "metadata": { "name": "db", "namespace": "app", "uid": "s-1" },
                  "spec": {
                    "serviceName": "db-headless",
                    "selector": { "matchLabels": { "app": "db" } },
                    "template": { "metadata": { "labels": { "app": "db" } }, "spec": {
                      "containers": [ { "name": "db", "image": "db:2" } ]
                    } }
                  } }
            ] },
            "daemonsets": { "items": [] },
            "pods": { "items": [
                { "metadata": { "name": "web-1", "namespace": "app", "uid": "p-1",
                                "labels": { "app": "web" } },
                  "spec": { "nodeName": "node-a" },
                  "status": { "phase": "Running", "containerStatuses": [
                      { "name": "main", "ready": true, "restartCount": 0 } ] } }
            ] },
            "services": { "items": [
                { "metadata": { "name": "web", "namespace": "app", "uid": "sv-1" },
                  "spec": { "selector": { "app": "web" },
                            "ports": [ { "port": 80 } ] } },
                { "metadata": { "name": "db-headless", "namespace": "app", "uid": "sv-2" },
                  "spec": { "clusterIP": "None", "selector": { "app": "db" },
                            "ports": [ { "port": 5432 } ] } }
            ] },
            "ingresses": { "items": [
                { "metadata": { "name": "edge", "namespace": "app", "uid": "i-1" },
                  "spec": { "rules": [ { "host": "web.test", "http": { "paths": [
                      { "path": "/", "pathType": "Prefix",
                        "backend": { "service": { "name": "web",
                                                  "port": { "number": 80 } } } }
                  ] } } ] } }
            ] },
            "configmaps": { "items": [
                { "metadata": { "name": "settings", "namespace": "app", "uid": "c-1" },
                  "data": { "mode": "fast" } }
            ] },
            "secrets": { "items": [
                { "metadata": { "name": "creds", "namespace": "app", "uid": "se-1" },
                  "type": "Opaque",
                  "data": { "token": {
                      "sha256": "8a94462377096e0657f57b6e6bc0e29000464398727091d7863726ce50974968",
                      "length": 12 } } }
            ] },
            "serviceaccounts": { "items": [
                { "metadata": { "name": "runner", "namespace": "app", "uid": "sa-1" } },
                { "metadata": { "name": "default", "namespace": "app", "uid": "sa-2" } }
            ] },
            "persistentvolumeclaims": { "items": [
                { "metadata": { "name": "data", "namespace": "app", "uid": "pv-1" },
                  "spec": { "accessModes": ["ReadWriteOnce"] },
                  "status": { "phase": "Bound" } }
            ] }
        }
    })
}

fn compiled() -> infra_compiler::InfraIr {
    let raw: RawBundle = serde_json::from_value(bundle()).expect("the bundle parses");
    let observation = Observation::try_from(raw).expect("the fixture is valid");
    infra_compiler::compile(&observation)
}

/// The persisted document as a JSON value, exactly as `compile --out` writes it.
fn persisted() -> serde_json::Value {
    let ir = compiled();
    serde_json::to_value(ir.document()).expect("the document serializes")
}

/// Recomputes the document's digest over its (possibly edited) model — what a *dishonest*
/// producer would do, used here to prove the relational check does not hide behind the digest.
fn restamp_digest(document: &mut serde_json::Value) {
    let canonical = serde_json::to_vec(&document["model"]).expect("the model serializes");
    document["digest"] = serde_json::json!(infra_compiler::digest_of_canonical(&canonical));
}

#[test]
fn a_persisted_document_reads_back_into_the_identical_ir() {
    let original = compiled();
    let read = infra_compiler::read_document(&persisted())
        .expect("what the compiler wrote must read back");
    assert_eq!(
        read, original,
        "the round trip must lose nothing and invent nothing"
    );
    assert_eq!(
        read.digest(),
        original.digest(),
        "and the two must address the same content"
    );
}

#[test]
fn the_fixture_mints_every_handle_kind_or_the_round_trip_proves_too_little() {
    // The state where the re-minting is load-bearing: one resolved reference per handle kind.
    let text = serde_json::to_string(&persisted()).expect("serializes");
    for (kind, key) in [
        ("node", "node-a"),
        ("service", "app/db-headless"),
        ("configmap", "app/settings"),
        ("secret", "app/creds"),
        ("service account", "app/runner"),
        ("claim", "app/data"),
    ] {
        // Through `Value`, object keys are sorted, so `key` precedes `state` in the text.
        assert!(
            text.contains(&format!("\"key\":\"{key}\",\"state\":\"resolved\"")),
            "no resolved {kind} reference for `{key}` — the fixture stopped exercising it"
        );
    }
}

#[test]
fn an_edited_document_is_refused_for_its_digest() {
    let mut document = persisted();
    document["model"]["workloads"]["app/deployment/web"]["replicas"] = serde_json::json!(7);
    let errors = infra_compiler::read_document(&document)
        .expect_err("an edited document must not read back");
    assert!(
        errors.contains(InfraCode::IrDigestMismatch),
        "expected INFRA-IR-002, got: {errors}"
    );
}

#[test]
fn a_hand_written_resolved_claim_is_refused_even_when_its_digest_is_freshly_stamped() {
    // The attacker recomputes the digest, so INFRA-IR-002 cannot fire; only the relational
    // check stands between this document and a panicking total lookup.
    let mut document = persisted();
    document["model"]["workloads"]["app/deployment/web"]["containers"][0]["env"][0]["source"]
        ["config_map"] = serde_json::json!({ "state": "resolved", "key": "app/no-such-map" });
    restamp_digest(&mut document);
    let errors = infra_compiler::read_document(&document)
        .expect_err("a dangling resolved claim must not read back");
    assert!(
        errors.contains(InfraCode::IrDanglingHandle),
        "expected INFRA-IR-004, got: {errors}"
    );
    assert!(
        !errors.contains(InfraCode::IrDigestMismatch),
        "the digest was honestly restamped; the refusal must come from the relational check \
         alone: {errors}"
    );
}

#[test]
fn an_edited_document_with_a_dangling_claim_reports_both_defects_in_one_run() {
    let mut document = persisted();
    document["model"]["pods"]["app/web-1"]["node"] =
        serde_json::json!({ "state": "resolved", "key": "node-gone" });
    let errors = infra_compiler::read_document(&document)
        .expect_err("an edited document with a dangling claim must not read back");
    assert!(
        errors.contains(InfraCode::IrDigestMismatch)
            && errors.contains(InfraCode::IrDanglingHandle),
        "validation accumulates: both INFRA-IR-002 and INFRA-IR-004 arrive together, got: {errors}"
    );
}

#[test]
fn a_foreign_format_is_refused_before_anything_else_is_believed() {
    let mut document = persisted();
    document["format"] = serde_json::json!("infra-ir/2");
    let errors = infra_compiler::read_document(&document).expect_err("a foreign format");
    assert!(
        errors.contains(InfraCode::IrUnsupportedFormat),
        "expected INFRA-IR-001, got: {errors}"
    );
}

#[test]
fn a_document_that_does_not_read_as_the_shape_is_refused_as_malformed() {
    let mut document = persisted();
    document["model"]["pods"] = serde_json::json!([1, 2, 3]);
    restamp_digest(&mut document);
    let errors = infra_compiler::read_document(&document).expect_err("not the shape");
    assert!(
        errors.contains(InfraCode::IrMalformed),
        "expected INFRA-IR-003, got: {errors}"
    );
}

struct Handles {
    node: infra_compiler::NodeHandle,
    service: infra_compiler::ServiceHandle,
    config_map: infra_compiler::ConfigMapHandle,
    secret: infra_compiler::SecretHandle,
    service_account: infra_compiler::ServiceAccountHandle,
    claim: infra_compiler::ClaimHandle,
}

fn resolved<H: Clone>(reference: &infra_compiler::Reference<H>) -> H {
    let infra_compiler::Reference::Resolved { key } = reference else {
        panic!("the fixture must actually resolve this handle");
    };
    key.clone()
}

impl Handles {
    fn from_ir(ir: &infra_compiler::InfraIr) -> Self {
        use infra_compiler::{ResolvedEnvSource, ResolvedVolumeSource};
        let model = ir.document().model;
        let web = &model.workloads["app/deployment/web"];
        let ResolvedEnvSource::ConfigMapKey { config_map, .. } = &web.containers[0].env[0].source
        else {
            panic!("MODE references the configmap");
        };
        let ResolvedEnvSource::SecretKey { secret, .. } = &web.containers[0].env[1].source else {
            panic!("TOKEN references the secret");
        };
        let ResolvedVolumeSource::Claim { claim } = &web.volumes[0].source else {
            panic!("the volume references the claim");
        };
        Self {
            node: resolved(
                model.pods["app/web-1"]
                    .node
                    .as_ref()
                    .expect("scheduled pod"),
            ),
            service: resolved(
                model.workloads["app/statefulset/db"]
                    .governing_service
                    .as_ref()
                    .expect("governing service"),
            ),
            config_map: resolved(config_map),
            secret: resolved(secret),
            service_account: resolved(&web.service_account),
            claim: resolved(claim),
        }
    }

    fn assert_usable(&self, ir: &infra_compiler::InfraIr) {
        assert_eq!(ir.node(&self.node).identity.name, "node-a");
        assert_eq!(ir.service(&self.service).identity.name, "db-headless");
        assert_eq!(ir.config_map(&self.config_map).identity.name, "settings");
        assert_eq!(ir.secret(&self.secret).identity.name, "creds");
        assert_eq!(
            ir.service_account(&self.service_account).identity.name,
            "runner"
        );
        assert_eq!(ir.claim(&self.claim).identity.name, "data");
    }
}

#[test]
fn every_handle_lookup_stays_total_after_compile_read_clone_and_checked_transform() {
    let original = compiled();
    let read =
        infra_compiler::read_document(&persisted()).expect("the persisted owner is admitted");
    let cloned = original.clone();
    let transformed = original
        .try_transform(|model| {
            model
                .workloads
                .get_mut("app/deployment/web")
                .expect("the workload exists")
                .replicas = Some(4);
        })
        .expect("replica changes preserve reference membership");
    for ir in [&original, &read, &cloned, &transformed] {
        Handles::from_ir(ir).assert_usable(ir);
    }
    assert_eq!(
        original.document().model.workloads["app/deployment/web"].replicas,
        Some(2)
    );
    assert_eq!(
        transformed.document().model.workloads["app/deployment/web"].replicas,
        Some(4)
    );
    assert_eq!(transformed.provenance, original.provenance);
    assert_ne!(transformed.digest(), original.digest());
    let reread =
        infra_compiler::read_document(&serde_json::to_value(transformed.document()).unwrap())
            .expect("transformed bytes read back");
    assert_eq!(reread, transformed);
    Handles::from_ir(&reread).assert_usable(&reread);
}

#[test]
fn deleting_any_referenced_target_is_refused_without_changing_the_source_owner() {
    let original = compiled();
    let handles = Handles::from_ir(&original);
    let bytes = serde_json::to_vec(&original.document()).unwrap();
    let digest = original.digest();
    let provenance = original.provenance.clone();
    let deletions: [fn(&mut infra_compiler::InfraModel); 6] = [
        |model| model.nodes.clear(),
        |model| model.services.clear(),
        |model| model.config_maps.clear(),
        |model| model.secrets.clear(),
        |model| model.service_accounts.clear(),
        |model| model.claims.clear(),
    ];
    for delete in deletions {
        let errors = original
            .try_transform(delete)
            .expect_err("a live reference cannot lose its target");
        assert!(errors.contains(InfraCode::IrDanglingHandle), "{errors}");
        assert!(
            !errors.contains(InfraCode::IrDigestMismatch),
            "the candidate stamps its actual bytes: {errors}"
        );
        assert_eq!(serde_json::to_vec(&original.document()).unwrap(), bytes);
        assert_eq!(original.digest(), digest);
        assert_eq!(original.provenance, provenance);
        handles.assert_usable(&original);
    }
}

#[test]
fn privacy_and_noop_transform_preserve_the_frozen_base_writer_document() {
    // Captured with the unmodified base CLI at 28e97095d9e06c8b4585876a681a5eda5278c1ab;
    // it exactly matched this committed example, including the final newline.
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let raw: RawBundle = serde_json::from_str(
        &std::fs::read_to_string(root.join("examples/k3d-dev-cluster/observation.json")).unwrap(),
    )
    .unwrap();
    let original = infra_compiler::compile(&Observation::try_from(raw).unwrap());
    let frozen =
        std::fs::read_to_string(root.join("examples/k3d-dev-cluster/cluster.ir.json")).unwrap();
    let transformed = original
        .try_transform(|_| {})
        .expect("a no-op preserves validity");
    let read = infra_compiler::read_document(&serde_json::from_str(&frozen).unwrap()).unwrap();
    for ir in [&original, &transformed, &read] {
        assert_eq!(
            format!(
                "{}\n",
                serde_json::to_string_pretty(&ir.document()).unwrap()
            ),
            frozen
        );
        assert_eq!(ir.digest(), original.digest());
    }
}

#[test]
fn checked_retargeting_remints_a_new_owner_without_invalidating_the_source_handles() {
    let source = compiled();
    let old_handles = Handles::from_ir(&source);
    let before = serde_json::to_vec(&source.document()).unwrap();
    let replacement = source.model().ingresses["app/edge"].rules[0].paths[0]
        .backend
        .service
        .clone();
    let transformed = source
        .try_transform(|model| {
            model
                .workloads
                .get_mut("app/statefulset/db")
                .unwrap()
                .governing_service = Some(replacement);
            model.services.remove("app/db-headless").unwrap();
        })
        .expect("the removed service has no remaining reference");
    let new_handle = resolved(
        transformed.model().workloads["app/statefulset/db"]
            .governing_service
            .as_ref()
            .unwrap(),
    );
    assert_eq!(transformed.service(&new_handle).identity.name, "web");
    let rejected = transformed
        .try_transform(|model| {
            model.services.remove("app/web").unwrap();
        })
        .expect_err("the reminted replacement remains checked on a second transform");
    assert!(rejected.contains(InfraCode::IrDanglingHandle), "{rejected}");
    assert_eq!(transformed.service(&new_handle).identity.name, "web");
    assert_eq!(serde_json::to_vec(&source.document()).unwrap(), before);
    old_handles.assert_usable(&source);
}

#[test]
fn indirect_environment_and_volume_sites_cannot_lose_their_resolved_targets() {
    for via_environment in [true, false] {
        let mut value = bundle();
        let pod = &mut value["kinds"]["deployments"]["items"][0]["spec"]["template"]["spec"];
        pod["containers"][0]["env"] = serde_json::json!([]);
        if via_environment {
            pod["containers"][0]["envFrom"] = serde_json::json!([
                {"configMapRef": {"name": "settings"}},
                {"secretRef": {"name": "creds", "optional": true}}
            ]);
        } else {
            pod["volumes"] = serde_json::json!([
                {"name": "config", "configMap": {"name": "settings"}},
                {"name": "credentials", "secret": {"secretName": "creds", "optional": true}}
            ]);
        }
        let raw: RawBundle = serde_json::from_value(value).unwrap();
        let source = infra_compiler::compile(&Observation::try_from(raw).unwrap());
        let admitted = source.try_transform(|_| {}).unwrap();
        let container = &admitted.model().workloads["app/deployment/web"].containers[0];
        assert!(container.env.is_empty());
        let (config, secret) = if via_environment {
            let infra_compiler::ResolvedEnvFromSource::ConfigMap { config_map, .. } =
                &container.env_from[0].source
            else {
                panic!("the first envFrom must carry the resolved configmap");
            };
            let infra_compiler::ResolvedEnvFromSource::Secret { secret, .. } =
                &container.env_from[1].source
            else {
                panic!("the second envFrom must carry the resolved secret");
            };
            (resolved(config_map), resolved(secret))
        } else {
            let volumes = &admitted.model().workloads["app/deployment/web"].volumes;
            let infra_compiler::ResolvedVolumeSource::ConfigMap { config_map, .. } =
                &volumes[0].source
            else {
                panic!("the first volume must carry the resolved configmap");
            };
            let infra_compiler::ResolvedVolumeSource::Secret { secret, .. } = &volumes[1].source
            else {
                panic!("the second volume must carry the resolved secret");
            };
            (resolved(config_map), resolved(secret))
        };
        assert_eq!(admitted.config_map(&config).identity.name, "settings");
        assert_eq!(admitted.secret(&secret).identity.name, "creds");
        let before = serde_json::to_vec(&admitted.document()).unwrap();
        for delete_config in [true, false] {
            let errors = admitted
                .try_transform(|model| {
                    if delete_config {
                        model.config_maps.clear();
                    } else {
                        model.secrets.clear();
                    }
                })
                .expect_err("every resolved indirect site must retain target membership");
            assert!(errors.contains(InfraCode::IrDanglingHandle), "{errors}");
            assert!(!errors.contains(InfraCode::IrDigestMismatch), "{errors}");
            assert_eq!(serde_json::to_vec(&admitted.document()).unwrap(), before);
            assert_eq!(admitted.config_map(&config).identity.name, "settings");
            assert_eq!(admitted.secret(&secret).identity.name, "creds");
        }
    }
}

#[test]
fn captured_detached_edits_and_panics_cannot_mutate_existing_owners() {
    let source = compiled();
    let handles = Handles::from_ir(&source);
    let before = serde_json::to_vec(&source.document()).unwrap();
    let mut captured = None;
    let admitted = source
        .try_transform(|model| {
            captured = Some(model.clone());
        })
        .unwrap();
    let admitted_handles = Handles::from_ir(&admitted);
    let mut detached = captured.unwrap();
    detached.nodes.clear();
    detached.services.clear();
    detached.config_maps.clear();
    detached.secrets.clear();
    detached.service_accounts.clear();
    detached.claims.clear();
    assert_eq!(serde_json::to_vec(&admitted.document()).unwrap(), before);
    admitted_handles.assert_usable(&admitted);

    let panic = std::panic::catch_unwind(|| {
        let _ = source.try_transform(|model| {
            model.nodes.clear();
            panic!("synthetic callback abort after detached mutation");
        });
    });
    assert!(panic.is_err(), "the callback must actually abort");
    assert_eq!(serde_json::to_vec(&source.document()).unwrap(), before);
    assert_eq!(source.digest(), admitted.digest());
    handles.assert_usable(&source);
}
