//! Adversarial cases for `ess-runtime/2`: inputs the runtime admits and the Kubernetes API server,
//! Helm or the stories' acceptance do not.

use super::{
    chart, compile, compile_v2, scheduled, workloads_of, BOUNDED, RUNNER_CONTAINER, RUNNER_PROCESS,
};
use crate::rendered;

const SERVICE_WORKLOAD: &str = "  - name: oracle
    components: [order-service, dispatch-service]
    containers: [runner]
    replicas: 1
";

/// `cron` with `run_deadline_seconds: 900`, as a schedule body.
fn bounded(cron: &str) -> String {
    format!("      cron: \"{cron}\"\n      run_deadline_seconds: 900\n")
}

/// Kubernetes parses a schedule with `robfig/cron` v3 `ParseStandard`, whose day-of-week bounds
/// are 0-6: `0 0 * * 7` is refused by the API server ("end of range (7) above maximum (6)").
///
/// Finding (red today): `cron_error` bounds day of week at 0-7 (`src/workload_kind.rs:332`).
#[test]
fn adversary_day_of_week_seven_is_refused_as_kubernetes_does() {
    for cron in ["0 0 * * 7", "0 0 * * 5-7"] {
        let result = compile_v2(RUNNER_CONTAINER, &scheduled(&bounded(cron), ""));
        assert!(
            result.is_err(),
            "cron {cron:?} compiled, and the API server refuses day of week 7:\n{}",
            workloads_of(&result.unwrap())
        );
        let text = rendered(&result.unwrap_err());
        assert!(
            text.contains("ingest-a") && text.contains("day of week"),
            "{cron}: {text}"
        );
    }
}

/// `backoffLimit`, `successfulJobsHistoryLimit` and `failedJobsHistoryLimit` are `int32` in
/// `batch/v1`; a value past `i32::MAX` renders a manifest the API server cannot decode.
///
/// Finding (red today): `Schedule::successful_runs`/`failed_runs` are `u32` and `retry_limit`
/// an unbounded `i64` (`src/workload_kind.rs:202,205,217`).
#[test]
fn adversary_job_counts_beyond_int32_are_refused_by_name() {
    let mut admitted = Vec::new();
    for (key, value) in [
        ("retry_limit", "2147483648"),
        ("successful_runs", "2147483648"),
        ("failed_runs", "4294967295"),
    ] {
        let schedule = format!("{BOUNDED}      {key}: {value}\n");
        match compile_v2(RUNNER_CONTAINER, &scheduled(&schedule, "")) {
            Ok(runtime) => admitted.push(format!(
                "{key}: {value} rendered {:?}",
                workloads_of(&runtime)
                    .lines()
                    .find(|line| line.contains(value))
                    .unwrap_or_default()
            )),
            Err(errors) => {
                let text = rendered(&errors);
                assert!(text.contains("ingest-a") && text.contains(key), "{text}");
            }
        }
    }
    assert!(
        admitted.is_empty(),
        "int32 Kubernetes fields admitted out of range:\n{}",
        admitted.join("\n")
    );
}

/// The API server validates `runAsUser`, `runAsGroup` and `fsGroup` as 0..=2147483647
/// (`IsValidUserID`, `IsValidGroupID`); `PodSecurity` takes any `u32`.
///
/// Finding (red today): `PodSecurity` ids are `u32` with no range check (`src/workload_kind.rs:100,103,106`).
#[test]
fn adversary_pod_security_ids_beyond_int32_are_refused_by_name() {
    let mut admitted = Vec::new();
    for key in ["run_as_user", "run_as_group", "fs_group"] {
        let workload = format!("{SERVICE_WORKLOAD}    security:\n      {key}: 4294967295\n");
        if let Ok(runtime) = compile_v2(RUNNER_CONTAINER, &workload) {
            admitted.push(format!(
                "{key}: {:?}",
                workloads_of(&runtime)
                    .lines()
                    .find(|line| line.contains("4294967295"))
                    .unwrap_or_default()
            ));
        }
    }
    assert!(
        admitted.is_empty(),
        "pod security ids admitted past the API server's range:\n{}",
        admitted.join("\n")
    );
}

/// A pod volume name is a DNS-1123 label: lowercase alphanumerics and `-`, at most 63 characters.
/// A secret slot identifier may carry `_` or `.` and has no length bound, and the mounted volume
/// is named `secret-<slot>`.
///
/// Finding (red today): `secret_volume_name` prefixes the slot identifier unchecked (`src/workload_kind.rs:225`).
#[test]
fn adversary_a_mounted_secret_volume_name_is_a_dns_label() {
    let long = format!("s{}", "a".repeat(59));
    for slot in ["db_password", "db.password", long.as_str()] {
        let container = format!(
            "  - name: runner\n    process: runner\n    secrets:\n      - name: {slot}\n        mount:\n          path: /etc/ingest\n"
        );
        let Ok(runtime) = compile_v2(&container, SERVICE_WORKLOAD) else {
            continue;
        };
        let workloads = workloads_of(&runtime);
        let volume = format!("secret-{slot}");
        let label = volume.len() <= 63
            && volume
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
        assert!(
            label,
            "slot {slot} compiled and renders the pod volume name {volume:?}, which is not a \
             DNS-1123 label:\n{workloads}"
        );
    }
}

/// The API server refuses a secret item path that starts with `..` (`..data` is the atomic
/// writer's own directory), and a secret key outside `[-._a-zA-Z0-9]+`.
///
/// Finding (red today): `validate_secret_slot` checks only an empty key and `is_contained` (`src/workload_kind.rs:282,549`).
#[test]
fn adversary_secret_items_kubernetes_refuses_are_refused_by_name() {
    let mut admitted = Vec::new();
    for (key, path) in [
        ("token", "..data"),
        ("token", "..token"),
        ("a/b", "token"),
        ("a b", "token"),
    ] {
        let container = format!(
            "  - name: runner\n    process: runner\n    secrets:\n      - name: credentials\n        mount:\n          path: /etc/ingest\n          items:\n            - key: \"{key}\"\n              path: \"{path}\"\n"
        );
        match compile_v2(&container, SERVICE_WORKLOAD) {
            Ok(_) => admitted.push(format!("key {key:?} path {path:?}")),
            Err(errors) => {
                let text = rendered(&errors);
                assert!(text.contains("credentials"), "{text}");
            }
        }
    }
    assert!(
        admitted.is_empty(),
        "secret items the API server refuses were admitted:\n{}",
        admitted.join("\n")
    );
}

/// Every chart file is a Helm template. A runtime string rendered verbatim that carries `{{` is
/// evaluated by Helm at install time, so the chart no longer says what the runtime says: an image
/// reference or a mount path can call `lookup` and read a cluster object into the manifest.
///
/// Finding (red today): image references and mount strings reach the chart through `quoted`,
/// unescaped for Helm (`src/environment.rs:760`, `src/workload_kind.rs:238,291`).
#[test]
fn adversary_template_actions_in_runtime_strings_are_refused() {
    let hex = "ab".repeat(32);
    let mut admitted = Vec::new();

    let reference = format!("{{{{.Release.Name}}}}/runner@sha256:{hex}");
    if let Ok(runtime) = compile(
        "ess-runtime/2",
        &format!("  - name: runner\n    image: \"{reference}\"\n"),
        RUNNER_CONTAINER,
        SERVICE_WORKLOAD,
        "",
    ) {
        admitted.push(format!(
            "image {reference:?}: {:?}",
            workloads_of(&runtime)
                .lines()
                .find(|line| line.contains("runner@sha256"))
                .unwrap_or_default()
        ));
    }

    for (mount_path, key, path) in [
        ("/etc/{{.Release.Name}}", "token", "token"),
        ("/etc/ingest", "{{.Release.Name}}", "token"),
        ("/etc/ingest", "token", "{{.Release.Name}}"),
    ] {
        let container = format!(
            "  - name: runner\n    process: runner\n    secrets:\n      - name: credentials\n        mount:\n          path: \"{mount_path}\"\n          items:\n            - key: \"{key}\"\n              path: \"{path}\"\n"
        );
        if let Ok(runtime) = compile_v2(&container, SERVICE_WORKLOAD) {
            let workloads = chart(&runtime)["templates/workloads.yaml"].clone();
            admitted.push(format!(
                "mount {mount_path:?} key {key:?} path {path:?}: {:?}",
                workloads
                    .lines()
                    .filter(|line| line.contains("{{.Release.Name}}"))
                    .collect::<Vec<_>>()
            ));
        }
    }
    assert!(
        admitted.is_empty(),
        "runtime strings carrying a Helm template action were admitted and rendered verbatim:\n{}",
        admitted.join("\n")
    );
}

/// `story:runtime-workload-probes-resources-placement-annotations`, Outcome and first acceptance
/// bullet: annotations validate in the runtime document and render into the chart. The change
/// adds none (nor TCP probes or affinity); the key spelling here is the story title's noun.
///
/// Finding (red today): `Workload` has no annotations key (`src/runtime.rs:197`).
#[test]
fn adversary_a_workload_declares_annotations() {
    let zero = "0".repeat(64);
    let text = format!(
        "format: ess-runtime/2\nruntime: oracle-runtime\nsemantic_digest: sha256:{zero}\nrealization_digest: sha256:{zero}\nbuild_digest: sha256:{zero}\nprocesses:\n{RUNNER_PROCESS}containers:\n{RUNNER_CONTAINER}workloads:\n{SERVICE_WORKLOAD}    annotations:\n      example.invalid/team: ingest\n"
    );
    let parsed = ess_deployment::RuntimeSpec::from_yaml(&text);
    assert!(
        parsed.is_ok(),
        "ess-runtime/2 has no workload annotations: {}",
        parsed
            .err()
            .map(|error| error.to_string())
            .unwrap_or_default()
    );
}

/// The API server refuses a container whose request exceeds its limit ("must be less than or
/// equal to cpu limit"); a chart rendering one installs nothing.
///
/// Finding (red today): `validate_container_settings` checks each quantity's spelling and never
/// compares a request with its limit (`src/workload_kind.rs:438`).
#[test]
fn adversary_a_request_above_its_limit_is_refused_by_name() {
    let mut admitted = Vec::new();
    for (resource, request, limit) in [
        ("cpu", "2", "1"),
        ("cpu", "1500m", "1"),
        ("memory", "1Gi", "512Mi"),
    ] {
        let container = format!(
            "  - name: runner\n    process: runner\n    resources:\n      requests:\n        {resource}: \"{request}\"\n      limits:\n        {resource}: \"{limit}\"\n"
        );
        match compile_v2(&container, SERVICE_WORKLOAD) {
            Ok(_) => admitted.push(format!("{resource} request {request} above limit {limit}")),
            Err(errors) => {
                let text = rendered(&errors);
                assert!(text.contains("runner") && text.contains(resource), "{text}");
            }
        }
    }
    assert!(
        admitted.is_empty(),
        "requests above their limits were admitted:\n{}",
        admitted.join("\n")
    );
}

/// `story:runtime-workload-probes-resources-placement-annotations`, Outcome and first acceptance
/// bullet: a container states a TCP readiness and liveness probe and a workload its affinity or
/// anti-affinity, each validated in the runtime document. Key spellings are the story's nouns.
///
/// Finding (red today): `ContainerRole` has only `readiness_path`/`liveness_path`
/// (`src/runtime.rs:158,161`) and `Workload` no placement (`src/runtime.rs:197`).
#[test]
fn adversary_tcp_probes_and_placement_are_declared() {
    let zero = "0".repeat(64);
    let document = |containers: &str, workloads: &str| {
        format!(
            "format: ess-runtime/2\nruntime: oracle-runtime\nsemantic_digest: sha256:{zero}\nrealization_digest: sha256:{zero}\nbuild_digest: sha256:{zero}\nprocesses:\n{RUNNER_PROCESS}containers:\n{containers}workloads:\n{workloads}"
        )
    };
    let mut missing = Vec::new();
    for (setting, text) in [
        (
            "tcp probes",
            document(
                "  - name: runner\n    process: runner\n    http_port: 8080\n    readiness_tcp_port: 8080\n    liveness_tcp_port: 8080\n",
                SERVICE_WORKLOAD,
            ),
        ),
        (
            "affinity",
            document(
                RUNNER_CONTAINER,
                &format!("{SERVICE_WORKLOAD}    affinity:\n      spread_across: kubernetes.io/hostname\n"),
            ),
        ),
    ] {
        if let Err(error) = ess_deployment::RuntimeSpec::from_yaml(&text) {
            missing.push(format!("{setting}: {error}"));
        }
    }
    assert!(
        missing.is_empty(),
        "ess-runtime/2 cannot state the story's settings:\n{}",
        missing.join("\n")
    );
}
