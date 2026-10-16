//! `ess-runtime/2`: the scheduled workload and the pod settings every workload kind takes.
//!
//! `docs/design/runtime-workload-kinds.md` is the design. The old-reader module comes first: an
//! `ess-runtime/1` document keeps its compiled bytes and digest, and its chart keeps its bytes.

use std::collections::BTreeMap;
use std::path::PathBuf;

use ess_deployment::{compile_runtime, project_helm, Diagnostics, RuntimeIr, RuntimeSpec};

use super::{build, physical_realization, rendered, runtime, semantic};

/// The digest of the fixture's `ess-runtime/1` compiled IR before `ess-runtime/2` existed.
const RUNTIME_1_IR_DIGEST: &str =
    "sha256:56acf998ba36adab6a0abfc17b6edee13a913977eaefe45ffd064a21694bea77";
/// The digest of the fixture's `ess-runtime/1` chart (every file, path and bytes) before
/// `ess-runtime/2` existed.
const RUNTIME_1_CHART_DIGEST: &str =
    "sha256:f1fc47b5ec1c3f4143fb07e367ab9ae0dfb1730837f7545f9404c845454fe1d1";

fn chart(runtime: &RuntimeIr) -> BTreeMap<String, String> {
    project_helm(
        runtime,
        &"oracle".parse().unwrap(),
        &"1.0.0".parse().unwrap(),
    )
    .files()
    .clone()
}

fn chart_digest(files: &BTreeMap<String, String>) -> String {
    let mut joined = String::new();
    for (path, contents) in files {
        joined.push_str(path);
        joined.push('\0');
        joined.push_str(contents);
        joined.push('\0');
    }
    ess_deployment::Digest::of_bytes(joined.as_bytes()).to_string()
}

fn workloads_of(runtime: &RuntimeIr) -> String {
    chart(runtime)["templates/workloads.yaml"].clone()
}

const RUNNER_PROCESS: &str = "  - name: runner
    image: app
";

const RUNNER_CONTAINER: &str = "  - name: runner
    process: runner
    config:
      - name: log-level
        environment: LOG_LEVEL
        kind: optional
    secrets:
      - name: database-password
        environment: DATABASE_PASSWORD
        key: password
";

/// One scheduled workload `ingest-a` realizing both fixture components, with `schedule` as its
/// schedule body (indented six spaces) and `rest` appended to the workload (indented four).
fn scheduled(schedule: &str, rest: &str) -> String {
    format!(
        "  - name: ingest-a\n    components: [order-service, dispatch-service]\n    containers: [runner]\n    schedule:\n{schedule}{rest}"
    )
}

const BOUNDED: &str = "      cron: \"2-59/10 * * * *\"\n      run_deadline_seconds: 900\n";

/// Compiles a runtime document of `format` from its three lists, against the oracle fixture.
fn compile(
    format: &str,
    processes: &str,
    containers: &str,
    workloads: &str,
    tail: &str,
) -> Result<RuntimeIr, Diagnostics> {
    let semantic = semantic();
    let build = build();
    let physical = physical_realization(&semantic);
    let text = format!(
        "format: {format}\nruntime: oracle-runtime\nsemantic_digest: sha256:{}\nrealization_digest: {}\nbuild_digest: {}\nprocesses:\n{processes}containers:\n{containers}workloads:\n{workloads}{tail}",
        semantic.source_digest(),
        physical.realization_digest(),
        build.digest(),
    );
    let specification = RuntimeSpec::from_yaml(&text)
        .unwrap_or_else(|error| panic!("runtime parses: {error}\n{text}"));
    compile_runtime(&specification, &semantic, &physical, &build)
}

fn compile_v2(containers: &str, workloads: &str) -> Result<RuntimeIr, Diagnostics> {
    compile("ess-runtime/2", RUNNER_PROCESS, containers, workloads, "")
}

fn scheduled_ok(schedule: &str, rest: &str) -> RuntimeIr {
    compile_v2(RUNNER_CONTAINER, &scheduled(schedule, rest))
        .unwrap_or_else(|errors| panic!("scheduled runtime compiles:\n{}", rendered(&errors)))
}

/// The refusal text for a document that must not compile.
fn refused(result: Result<RuntimeIr, Diagnostics>) -> String {
    match result {
        Ok(runtime) => panic!("compiled:\n{}", runtime.to_canonical_json()),
        Err(errors) => rendered(&errors),
    }
}

/// Asserts the compiled-IR reader refuses `runtime` after `edit`, as the compiler does.
fn reader_refuses(runtime: &RuntimeIr, edit: impl FnOnce(&mut serde_json::Value)) {
    let mut value = serde_json::to_value(runtime).unwrap();
    edit(&mut value);
    assert!(
        RuntimeIr::from_json(&value.to_string()).is_err(),
        "the runtime IR reader admitted {value}"
    );
}

mod old_reader {
    use super::*;

    #[test]
    fn an_ess_runtime_1_document_keeps_its_bytes_and_digest() {
        let runtime = runtime(&semantic(), &build());
        let canonical = runtime.to_canonical_json();
        assert!(
            canonical.contains("\"format\": \"ess-runtime-ir/1\""),
            "{canonical}"
        );
        assert_eq!(runtime.digest().to_string(), RUNTIME_1_IR_DIGEST);
        assert_eq!(chart_digest(&chart(&runtime)), RUNTIME_1_CHART_DIGEST);
        let read_back = RuntimeIr::from_json(&canonical).expect("the /1 IR reads back");
        assert_eq!(read_back.to_canonical_json(), canonical);
    }

    #[test]
    fn an_ess_runtime_1_document_refuses_the_runtime_2_keys_by_name() {
        let text = refused(compile(
            "ess-runtime/1",
            RUNNER_PROCESS,
            RUNNER_CONTAINER,
            &scheduled(BOUNDED, ""),
            "",
        ));
        assert!(text.contains("ingest-a"), "{text}");
        assert!(text.contains("schedule"), "{text}");
        assert!(text.contains("ess-runtime/2"), "{text}");

        let compiled = scheduled_ok(BOUNDED, "");
        reader_refuses(&compiled, |value| {
            value["format"] = serde_json::json!("ess-runtime-ir/1");
        });
    }

    #[test]
    fn an_ess_runtime_1_document_refuses_probes_placement_and_annotations_by_name() {
        let workload = "  - name: oracle\n    components: [order-service, dispatch-service]\n    containers: [runner]\n    replicas: 1\n";
        for (containers, workloads, key) in [
            (
                format!("{RUNNER_CONTAINER}    readiness_tcp_port: 8080\n"),
                workload.to_owned(),
                "readiness_tcp_port",
            ),
            (
                format!("{RUNNER_CONTAINER}    liveness_tcp_port: 8080\n"),
                workload.to_owned(),
                "liveness_tcp_port",
            ),
            (
                RUNNER_CONTAINER.to_owned(),
                format!("{workload}    annotations:\n      team: ingest\n"),
                "annotations",
            ),
            (
                RUNNER_CONTAINER.to_owned(),
                format!("{workload}    affinity:\n      spread_across: kubernetes.io/hostname\n"),
                "affinity",
            ),
        ] {
            let text = refused(compile(
                "ess-runtime/1",
                RUNNER_PROCESS,
                &containers,
                &workloads,
                "",
            ));
            assert!(
                text.contains(key) && text.contains("ess-runtime/2"),
                "{key}: {text}"
            );
        }
    }

    #[test]
    fn an_ess_runtime_2_document_compiles_to_runtime_ir_2() {
        let compiled = scheduled_ok(BOUNDED, "");
        let canonical = compiled.to_canonical_json();
        assert!(
            canonical.contains("\"format\": \"ess-runtime-ir/2\""),
            "{canonical}"
        );
        let read_back = RuntimeIr::from_json(&canonical).expect("the /2 IR reads back");
        assert_eq!(read_back.to_canonical_json(), canonical);
    }
}

mod pod_settings {
    use super::*;

    const SERVICE_WORKLOAD: &str = "  - name: oracle
    components: [order-service, dispatch-service]
    containers: [runner]
    replicas: 1
";

    fn service(container_extra: &str, workload_extra: &str) -> Result<RuntimeIr, Diagnostics> {
        compile_v2(
            &format!("{RUNNER_CONTAINER}{container_extra}"),
            &format!("{SERVICE_WORKLOAD}{workload_extra}"),
        )
    }

    fn service_ok(container_extra: &str, workload_extra: &str) -> RuntimeIr {
        service(container_extra, workload_extra)
            .unwrap_or_else(|errors| panic!("service compiles:\n{}", rendered(&errors)))
    }

    #[test]
    fn a_container_declares_requests_and_limits() {
        let runtime = service_ok(
            "    resources:\n      requests:\n        cpu: 100m\n        memory: 128Mi\n      limits:\n        memory: 256Mi\n",
            "",
        );
        let workloads = workloads_of(&runtime);
        assert!(
            workloads.contains(
                "          resources:\n            requests:\n              cpu: \"100m\"\n              memory: \"128Mi\"\n            limits:\n              memory: \"256Mi\"\n"
            ),
            "{workloads}"
        );
        let scheduled = scheduled_ok(BOUNDED, "");
        assert!(!workloads_of(&scheduled).contains("resources:"));

        for (bad, named) in [
            ("      requests:\n        cpu: lots\n", "lots"),
            ("      requests: {}\n", "requests"),
            ("      limits:\n        memory: \"\"\n", "memory"),
        ] {
            let text = refused(service(&format!("    resources:\n{bad}"), ""));
            assert!(text.contains("runner"), "{text}");
            assert!(text.contains(named), "{named}: {text}");
        }
    }

    #[test]
    fn a_workload_declares_its_pod_security() {
        let runtime = service_ok(
            "    read_only_root_filesystem: true\n",
            "    security:\n      run_as_user: 10001\n      run_as_group: 10002\n      fs_group: 10001\n      run_as_non_root: true\n",
        );
        let workloads = workloads_of(&runtime);
        assert!(
            workloads.contains(
                "      securityContext:\n        runAsUser: 10001\n        runAsGroup: 10002\n        fsGroup: 10001\n        runAsNonRoot: true\n"
            ),
            "{workloads}"
        );
        assert!(
            workloads
                .contains("          securityContext:\n            readOnlyRootFilesystem: true\n"),
            "{workloads}"
        );
        let text = refused(service("", "    security: {}\n"));
        assert!(
            text.contains("oracle") && text.contains("security"),
            "{text}"
        );
    }

    #[test]
    fn a_workload_opts_out_of_the_service_account_token() {
        let runtime = service_ok("", "    service_account_token: false\n");
        let workloads = workloads_of(&runtime);
        assert!(
            workloads.contains(
                "      serviceAccountName: {{ .Values.serviceAccount.name }}\n      automountServiceAccountToken: false\n"
            ),
            "{workloads}"
        );
        let unset = service_ok("", "");
        assert!(!workloads_of(&unset).contains("automountServiceAccountToken"));
    }

    const MOUNTED: &str = "  - name: runner
    process: runner
    secrets:
      - name: credentials
        mount:
          path: /etc/ingest
          mode: \"0400\"
          items:
            - key: token.json
              path: token.json
";

    #[test]
    fn a_secret_slot_is_mounted_as_a_volume() {
        let runtime = compile_v2(MOUNTED, SERVICE_WORKLOAD)
            .unwrap_or_else(|errors| panic!("{}", rendered(&errors)));
        let files = chart(&runtime);
        let workloads = &files["templates/workloads.yaml"];
        assert!(
            workloads.contains(
                "          volumeMounts:\n            - name: secret-credentials\n              mountPath: \"/etc/ingest\"\n              readOnly: true\n"
            ),
            "{workloads}"
        );
        assert!(
            workloads.contains(
                "      volumes:\n        - name: secret-credentials\n          secret:\n            secretName: {{ index .Values.secrets \"credentials\" \"name\" }}\n            defaultMode: 256\n            items:\n              - key: \"token.json\"\n                path: \"token.json\"\n"
            ),
            "{workloads}"
        );
        assert!(!workloads.contains("secretKeyRef"), "{workloads}");
        assert!(
            files["values.yaml"].contains("secrets:\n  credentials:\n    name: \"\"\n"),
            "{}",
            files["values.yaml"]
        );
        let schema: serde_json::Value = serde_json::from_str(&files["values.schema.json"]).unwrap();
        assert_eq!(
            schema["properties"]["secrets"]["properties"]["credentials"]["required"],
            serde_json::json!(["name"])
        );

        for (from, to, named) in [
            ("path: token.json", "path: ../token.json", "../token.json"),
            ("path: token.json", "path: /token.json", "/token.json"),
            ("mode: \"0400\"", "mode: \"0800\"", "0800"),
            ("mode: \"0400\"", "mode: \"1777\"", "1777"),
            ("path: /etc/ingest", "path: etc/ingest", "etc/ingest"),
            (
                "              path: token.json\n",
                "              path: token.json\n            - key: other\n              path: token.json\n",
                "token.json",
            ),
        ] {
            let text = refused(compile_v2(&MOUNTED.replace(from, to), SERVICE_WORKLOAD));
            assert!(text.contains("credentials"), "{to}: {text}");
            assert!(text.contains(named), "{to}: {text}");
        }
        reader_refuses(&runtime, |value| {
            value["containers"]["runner"]["secrets"][0]["mount"]["mode"] =
                serde_json::json!("0999");
        });
    }

    #[test]
    fn an_image_is_pinned_by_digest() {
        let pinned = format!("example.invalid/runner@sha256:{}", "ab".repeat(32));
        let runtime = compile(
            "ess-runtime/2",
            &format!("  - name: runner\n    image: {pinned}\n"),
            RUNNER_CONTAINER,
            SERVICE_WORKLOAD,
            "",
        )
        .unwrap_or_else(|errors| panic!("{}", rendered(&errors)));
        let files = chart(&runtime);
        assert!(
            files["templates/workloads.yaml"].contains(&format!("          image: \"{pinned}\"\n")),
            "{}",
            files["templates/workloads.yaml"]
        );
        assert!(
            files["values.yaml"].contains("images: {}\n"),
            "{}",
            files["values.yaml"]
        );

        for bad in [
            "example.invalid/runner:latest".to_owned(),
            format!("example.invalid/runner@sha256:{}", "AB".repeat(32)),
            format!("example.invalid/runner@sha256:{}", "a".repeat(63)),
            format!("example.invalid/runner@sha512:{}", "a".repeat(64)),
        ] {
            let text = refused(compile(
                "ess-runtime/2",
                &format!("  - name: runner\n    image: \"{bad}\"\n"),
                RUNNER_CONTAINER,
                SERVICE_WORKLOAD,
                "",
            ));
            assert!(
                text.contains("runner") && text.contains(&bad),
                "{bad}: {text}"
            );
            assert!(text.contains("digest"), "{bad}: {text}");
        }
        reader_refuses(&runtime, |value| {
            value["processes"]["runner"]["image"] = serde_json::json!("example.invalid/runner");
        });
    }

    #[test]
    fn a_workload_declares_its_pod_annotations() {
        let runtime = service_ok(
            "",
            "    annotations:\n      example.invalid/team: ingest\n      prometheus.io/scrape: \"true\"\n",
        );
        let workloads = workloads_of(&runtime);
        assert!(
            workloads.contains(
                "        app.kubernetes.io/component: oracle\n      annotations:\n        \"example.invalid/team\": \"ingest\"\n        \"prometheus.io/scrape\": \"true\"\n    spec:\n"
            ),
            "{workloads}"
        );
        let scheduled = scheduled_ok(BOUNDED, "    annotations:\n      team: ingest\n");
        assert!(
            workloads_of(&scheduled).contains(
                "          annotations:\n            \"team\": \"ingest\"\n        spec:\n"
            ),
            "{}",
            workloads_of(&scheduled)
        );
        for (bad, named) in [
            ("    annotations: {}\n", "annotations"),
            ("    annotations:\n      \"-team\": ingest\n", "-team"),
            (
                "    annotations:\n      \"Example/team\": ingest\n",
                "Example/team",
            ),
            (
                "    annotations:\n      team: \"{{ .Release.Name }}\"\n",
                "annotation",
            ),
        ] {
            let text = refused(service("", bad));
            assert!(
                text.contains("oracle") && text.contains(named),
                "{bad}: {text}"
            );
        }
        reader_refuses(&runtime, |value| {
            value["workloads"]["oracle"]["annotations"] = serde_json::json!({"a b": "c"});
        });
    }

    #[test]
    fn a_container_declares_tcp_probes() {
        let runtime = service_ok(
            "    readiness_tcp_port: 5432\n    liveness_tcp_port: 5433\n",
            "",
        );
        let workloads = workloads_of(&runtime);
        assert!(
            workloads.contains(
                "          readinessProbe:\n            tcpSocket:\n              port: 5432\n          livenessProbe:\n            tcpSocket:\n              port: 5433\n"
            ),
            "{workloads}"
        );
        assert!(!workloads.contains("httpGet"), "{workloads}");
        for (bad, named) in [
            ("    readiness_tcp_port: 0\n", "readiness_tcp_port"),
            (
                "    http_port: 8080\n    liveness_path: /live\n    liveness_tcp_port: 8080\n",
                "liveness_tcp_port",
            ),
        ] {
            let text = refused(service(bad, ""));
            assert!(
                text.contains("runner") && text.contains(named),
                "{bad}: {text}"
            );
        }
        let text = refused(compile_v2(
            &format!("{RUNNER_CONTAINER}    readiness_tcp_port: 8080\n"),
            &scheduled(BOUNDED, ""),
        ));
        assert!(
            text.contains("ingest-a") && text.contains("readiness"),
            "{text}"
        );
    }

    #[test]
    fn a_workload_declares_its_affinity() {
        let runtime = service_ok(
            "",
            "    affinity:\n      spread_across: kubernetes.io/hostname\n      node_labels:\n        example.invalid/pool: batch\n",
        );
        let workloads = workloads_of(&runtime);
        assert!(
            workloads.contains(
                "      affinity:\n        nodeAffinity:\n          requiredDuringSchedulingIgnoredDuringExecution:\n            nodeSelectorTerms:\n              - matchExpressions:\n                  - key: \"example.invalid/pool\"\n                    operator: In\n                    values: [\"batch\"]\n        podAntiAffinity:\n          preferredDuringSchedulingIgnoredDuringExecution:\n            - weight: 100\n              podAffinityTerm:\n                labelSelector:\n                  matchLabels:\n                    app.kubernetes.io/instance: {{ .Release.Name }}\n                    app.kubernetes.io/component: oracle\n                topologyKey: \"kubernetes.io/hostname\"\n      containers:\n"
            ),
            "{workloads}"
        );
        let required = service_ok(
            "",
            "    affinity:\n      spread_across: topology.kubernetes.io/zone\n      spread: required\n",
        );
        let workloads = workloads_of(&required);
        assert!(
            workloads.contains(
                "        podAntiAffinity:\n          requiredDuringSchedulingIgnoredDuringExecution:\n            - labelSelector:\n                matchLabels:\n                  app.kubernetes.io/instance: {{ .Release.Name }}\n                  app.kubernetes.io/component: oracle\n              topologyKey: \"topology.kubernetes.io/zone\"\n      containers:\n"
            ),
            "{workloads}"
        );
        assert!(!workloads.contains("nodeAffinity"), "{workloads}");
        for (bad, named) in [
            ("    affinity: {}\n", "affinity"),
            ("    affinity:\n      spread: required\n", "spread"),
            (
                "    affinity:\n      spread_across: \"not a key\"\n",
                "not a key",
            ),
            (
                "    affinity:\n      node_labels:\n        pool: \"-batch\"\n",
                "-batch",
            ),
        ] {
            let text = refused(service("", bad));
            assert!(
                text.contains("oracle") && text.contains(named),
                "{bad}: {text}"
            );
        }
    }

    #[test]
    fn a_runtime_without_pod_settings_keeps_its_chart_bytes() {
        let first = runtime(&semantic(), &build());
        let semantic = semantic();
        let build = build();
        let physical = physical_realization(&semantic);
        let mut second = super::super::runtime_spec(&semantic, &physical, &build);
        let text = ess_yaml::to_string(&second)
            .unwrap()
            .replace("ess-runtime/1", "ess-runtime/2");
        second = RuntimeSpec::from_yaml(&text).unwrap();
        let second = compile_runtime(&second, &semantic, &physical, &build).unwrap();
        assert_eq!(chart(&first), chart(&second));
    }
}

mod scheduled_jobs {
    use super::*;

    #[test]
    fn a_scheduled_workload_renders_a_cron_job() {
        let runtime = scheduled_ok(BOUNDED, "");
        let workloads = workloads_of(&runtime);
        for expected in [
            "apiVersion: batch/v1\nkind: CronJob\nmetadata:\n  name: {{ .Release.Name }}-ingest-a\n",
            "spec:\n  schedule: \"2-59/10 * * * *\"\n  concurrencyPolicy: Forbid\n  suspend: false\n",
            "  jobTemplate:\n    spec:\n      activeDeadlineSeconds: 900\n      template:\n",
            "        spec:\n          restartPolicy: Never\n          serviceAccountName: {{ .Values.serviceAccount.name }}\n          containers:\n            - name: runner\n",
            "                - name: LOG_LEVEL\n                  value: {{ index .Values.config \"log-level\" | quote }}\n",
            "                - name: DATABASE_PASSWORD\n                  valueFrom:\n                    secretKeyRef:\n",
        ] {
            assert!(workloads.contains(expected), "missing {expected:?} in\n{workloads}");
        }
        for absent in [
            "Deployment",
            "StatefulSet",
            "replicas",
            "timeZone",
            "backoffLimit",
        ] {
            assert!(!workloads.contains(absent), "{absent} in\n{workloads}");
        }
        let values = &chart(&runtime)["values.yaml"];
        assert!(values.ends_with("workloads:\n  ingest-a: {}\n"), "{values}");

        let on_failure = scheduled_ok(&format!("{BOUNDED}      restart: on_failure\n"), "");
        assert!(workloads_of(&on_failure).contains("          restartPolicy: OnFailure\n"));
    }

    #[test]
    fn a_scheduled_workload_declares_its_time_zone() {
        let runtime = scheduled_ok(&format!("{BOUNDED}      time_zone: Europe/Berlin\n"), "");
        assert!(
            workloads_of(&runtime)
                .contains("  schedule: \"2-59/10 * * * *\"\n  timeZone: \"Europe/Berlin\"\n"),
            "{}",
            workloads_of(&runtime)
        );
        for unknown in ["Europe/Atlantis", "europe/berlin", "CET+1"] {
            let text = refused(compile_v2(
                RUNNER_CONTAINER,
                &scheduled(&format!("{BOUNDED}      time_zone: \"{unknown}\"\n"), ""),
            ));
            assert!(
                text.contains("ingest-a") && text.contains(unknown),
                "{text}"
            );
        }
        assert!(!workloads_of(&scheduled_ok(BOUNDED, "")).contains("timeZone"));
    }

    #[test]
    fn a_scheduled_workload_declares_history_limits_and_overlap() {
        for (overlap, policy) in [
            ("forbid", "Forbid"),
            ("replace", "Replace"),
            ("allow", "Allow"),
        ] {
            let runtime = scheduled_ok(
                &format!(
                    "{BOUNDED}      overlap: {overlap}\n      successful_runs: 3\n      failed_runs: 1\n"
                ),
                "",
            );
            let workloads = workloads_of(&runtime);
            assert!(
                workloads.contains(&format!("  concurrencyPolicy: {policy}\n")),
                "{workloads}"
            );
            assert!(
                workloads
                    .contains("  successfulJobsHistoryLimit: 3\n  failedJobsHistoryLimit: 1\n"),
                "{workloads}"
            );
        }
        let unset = workloads_of(&scheduled_ok(BOUNDED, ""));
        assert!(!unset.contains("HistoryLimit"), "{unset}");
    }

    #[test]
    fn a_scheduled_workload_declares_suspend_deadlines_and_retries() {
        let runtime = scheduled_ok(
            &format!(
                "{BOUNDED}      suspended: true\n      start_deadline_seconds: 120\n      retry_limit: 2\n"
            ),
            "",
        );
        let workloads = workloads_of(&runtime);
        for expected in [
            "  suspend: true\n",
            "  startingDeadlineSeconds: 120\n",
            "      activeDeadlineSeconds: 900\n      backoffLimit: 2\n",
        ] {
            assert!(
                workloads.contains(expected),
                "missing {expected:?} in\n{workloads}"
            );
        }
        let unset = workloads_of(&scheduled_ok(BOUNDED, ""));
        assert!(unset.contains("  suspend: false\n"), "{unset}");
        assert!(!unset.contains("startingDeadlineSeconds"), "{unset}");
    }

    #[test]
    fn an_unbounded_scheduled_workload_is_refused_by_name() {
        let text = refused(compile_v2(
            RUNNER_CONTAINER,
            &scheduled("      cron: \"*/5 * * * *\"\n", ""),
        ));
        assert!(text.contains("InvalidValue ingest-a"), "{text}");
        assert!(text.contains("run_deadline_seconds"), "{text}");
        let compiled = scheduled_ok(BOUNDED, "");
        reader_refuses(&compiled, |value| {
            value["workloads"]["ingest-a"]["schedule"]
                .as_object_mut()
                .unwrap()
                .remove("run_deadline_seconds");
        });
    }

    #[test]
    fn a_scheduled_workload_keeps_state_between_runs() {
        let container = format!(
            "{RUNNER_CONTAINER}    volume_mounts:\n      - volume: state\n        mount_path: /var/lib/ingest\n"
        );
        let runtime = compile_v2(
            &container,
            &scheduled(
                BOUNDED,
                "    volumes:\n      - name: state\n        size: 1Gi\n",
            ),
        )
        .unwrap_or_else(|errors| panic!("{}", rendered(&errors)));
        let files = chart(&runtime);
        let workloads = &files["templates/workloads.yaml"];
        for expected in [
            "              volumeMounts:\n                - name: state\n                  mountPath: \"/var/lib/ingest\"\n",
            "          volumes:\n            - name: state\n              persistentVolumeClaim:\n                claimName: {{ .Release.Name }}-ingest-a-state\n",
            "---\napiVersion: v1\nkind: PersistentVolumeClaim\nmetadata:\n  name: {{ .Release.Name }}-ingest-a-state\nspec:\n  accessModes: [\"ReadWriteOnce\"]\n",
            "      storage: {{ index $root.Values.workloads \"ingest-a\" \"volumes\" \"state\" \"size\" | quote }}\n",
        ] {
            assert!(workloads.contains(expected), "missing {expected:?} in\n{workloads}");
        }
        assert!(!workloads.contains("StatefulSet"), "{workloads}");
        assert!(!workloads.contains("volumeClaimTemplates"), "{workloads}");
        assert_eq!(files["templates/services.yaml"], "", "no headless Service");
        assert!(
            files["values.yaml"].ends_with(
                "workloads:\n  ingest-a:\n    volumes:\n      state:\n        class: \"\"\n        size: \"1Gi\"\n"
            ),
            "{}",
            files["values.yaml"]
        );
    }

    #[test]
    fn a_malformed_or_conflicting_schedule_is_refused_by_name() {
        let cases: Vec<(String, String, &str, &str)> = vec![
            (
                "      cron: \"61 * * * *\"\n      run_deadline_seconds: 900\n".into(),
                String::new(),
                RUNNER_CONTAINER,
                "61 * * * *",
            ),
            (
                "      cron: \"* * * *\"\n      run_deadline_seconds: 900\n".into(),
                String::new(),
                RUNNER_CONTAINER,
                "* * * *",
            ),
            (
                "      cron: \"CRON_TZ=UTC * * * * *\"\n      run_deadline_seconds: 900\n".into(),
                String::new(),
                RUNNER_CONTAINER,
                "time_zone",
            ),
            (
                "      cron: \"*/0 * * * *\"\n      run_deadline_seconds: 900\n".into(),
                String::new(),
                RUNNER_CONTAINER,
                "*/0 * * * *",
            ),
            (BOUNDED.into(), "    replicas: 1\n".into(), RUNNER_CONTAINER, "replicas"),
            (BOUNDED.into(), "    storage: 1Gi\n".into(), RUNNER_CONTAINER, "storage"),
            (
                BOUNDED.into(),
                String::new(),
                "  - name: runner\n    process: runner\n    http_port: 8080\n    readiness_path: /ready\n",
                "readiness",
            ),
            (
                "      cron: \"* * * * *\"\n      run_deadline_seconds: 0\n".into(),
                String::new(),
                RUNNER_CONTAINER,
                "run_deadline_seconds",
            ),
            (
                "      cron: \"* * * * *\"\n      run_deadline_seconds: -5\n".into(),
                String::new(),
                RUNNER_CONTAINER,
                "run_deadline_seconds",
            ),
            (
                format!("{BOUNDED}      start_deadline_seconds: 0\n"),
                String::new(),
                RUNNER_CONTAINER,
                "start_deadline_seconds",
            ),
            (
                format!("{BOUNDED}      retry_limit: -1\n"),
                String::new(),
                RUNNER_CONTAINER,
                "retry_limit",
            ),
        ];
        for (schedule, rest, container, named) in cases {
            let text = refused(compile_v2(container, &scheduled(&schedule, &rest)));
            assert!(text.contains("ingest-a"), "{named}: {text}");
            assert!(text.contains(named), "{named}: {text}");
        }

        // A Service port: a provided endpoint selecting the scheduled workload.
        let text = refused(compile(
            "ess-runtime/2",
            RUNNER_PROCESS,
            "  - name: runner\n    process: runner\n    http_port: 8080\n",
            &scheduled(BOUNDED, ""),
            "provided_endpoints:\n  - name: api\n    workload: ingest-a\n    container: runner\n    scheme: http\n",
        ));
        assert!(
            text.contains("ingest-a") && text.contains("Service"),
            "{text}"
        );

        let compiled = scheduled_ok(BOUNDED, "");
        reader_refuses(&compiled, |value| {
            value["workloads"]["ingest-a"]["schedule"]["cron"] = serde_json::json!("* * *");
        });
        reader_refuses(&compiled, |value| {
            value["workloads"]["ingest-a"]["replicas"] = serde_json::json!(1);
        });
    }

    #[test]
    fn a_runtime_without_schedules_keeps_its_chart_bytes() {
        let runtime = runtime(&semantic(), &build());
        assert_eq!(chart_digest(&chart(&runtime)), RUNTIME_1_CHART_DIGEST);
        assert!(!workloads_of(&runtime).contains("CronJob"));
    }

    fn golden_directory() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/workload_kinds/scheduled-ingest")
    }

    /// A consumer's minimal scheduled job, every file of its chart compared byte for byte.
    ///
    /// `ESS_DEPLOYMENT_BLESS=1` rewrites the golden files; the diff is then reviewed by hand.
    #[test]
    fn the_scheduled_ingest_example_renders_exactly() {
        let pinned = format!(
            "example.invalid/runner@sha256:{}",
            "0123456789abcdef".repeat(4)
        );
        let runtime = compile(
            "ess-runtime/2",
            &format!("  - name: runner\n    image: {pinned}\n    arguments: [ingest, --once]\n"),
            "  - name: runner
    process: runner
    read_only_root_filesystem: true
    resources:
      requests:
        cpu: 50m
        memory: 64Mi
      limits:
        memory: 128Mi
    secrets:
      - name: credentials
        mount:
          path: /etc/ingest
          mode: \"0400\"
          items:
            - key: credentials.json
              path: credentials.json
",
            "  - name: ingest-a
    components: [order-service, dispatch-service]
    containers: [runner]
    service_account_token: false
    security:
      run_as_user: 10001
      run_as_group: 10001
      fs_group: 10001
      run_as_non_root: true
    schedule:
      cron: \"2-59/10 * * * *\"
      overlap: forbid
      suspended: true
      run_deadline_seconds: 540
",
            "",
        )
        .unwrap_or_else(|errors| panic!("{}", rendered(&errors)));
        let files = project_helm(
            &runtime,
            &"ingest".parse().unwrap(),
            &"0.1.0".parse().unwrap(),
        );
        let directory = golden_directory();
        if std::env::var_os("ESS_DEPLOYMENT_BLESS").is_some() {
            for (path, contents) in files.files() {
                let target = directory.join(path);
                std::fs::create_dir_all(target.parent().unwrap()).unwrap();
                std::fs::write(target, contents).unwrap();
            }
        }
        for (path, contents) in files.files() {
            let golden = std::fs::read_to_string(directory.join(path))
                .unwrap_or_else(|error| panic!("golden {path}: {error}"));
            assert_eq!(contents, &golden, "{path} differs from its golden file");
        }
        assert_eq!(
            files.files().len(),
            5,
            "the chart is the five files project_helm emits"
        );
    }
}

mod adversary;
