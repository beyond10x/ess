---
format: aep.planning-md/3
id: story:runtime-realizes-a-scheduled-job
kind: story
status: active
title: A runtime workload can be realized as a scheduled job (Kubernetes CronJob)
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- depends_on: story:runtime-workload-probes-resources-placement-annotations
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-15T03:05:53Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-10-15T03:05:53Z", actor: "human:timo", revision: 5}
---
## Outcome

An `ess-runtime` workload can be declared a scheduled job, and `ess generate project helm` renders
it as a Kubernetes `CronJob` (`batch/v1`) and the `Job` template it spawns, instead of a
`Deployment` or `StatefulSet`: a cron schedule, a concurrency policy (`forbid` by default,
`replace`, `allow`), successful- and failed-run history limits, a suspended flag, a run deadline, a
retry limit, secret references as the long-running workloads take them, and a declared state
location that survives between runs. A scheduled workload that could run unbounded is refused.

## Evidence

- The Helm projection renders every workload as `Deployment`, or `StatefulSet` when it has storage
  or volumes (`crates/generate/ess-deployment/src/environment.rs:741-745`, `render_workloads`);
  there is no `CronJob` or `Job` in the realization or deployment crates. `CronJob` and `Job`
  appear only on the cluster-observation side (`crates/infra/infra-compiler/src/read.rs:32`,
  `crates/infra/ess-kubernetes/src/topology.rs:8`), which reads clusters and emits nothing.
- An adopter runs a service as a periodic batch process (run, write state, exit) and wants its
  manifests from its ESS specification like its other components; today it has to hand-write the
  CronJob beside the generated chart.

## Acceptance

Failing tests first, each a module of an existing test binary of the crate (no new `tests/*.rs`):

- `a_scheduled_workload_renders_a_cron_job`: a runtime with one workload declared
  `schedule: {cron: "2-59/10 * * * *"}` and a run deadline renders `apiVersion: batch/v1`,
  `kind: CronJob` with that schedule, `concurrencyPolicy: Forbid` by default, and the
  `jobTemplate` carrying the container, its environment, config and secret references exactly as a
  `Deployment` would, with `restartPolicy: Never` by default (`OnFailure` when declared).
- `a_scheduled_workload_declares_its_time_zone`: `schedule: {cron: …, time_zone: Europe/Berlin}`
  renders `timeZone`; an unknown zone name is refused by name; absent renders no `timeZone`.
- `a_scheduled_workload_declares_history_limits_and_overlap`: `successful_runs`, `failed_runs` and
  `overlap: forbid | replace | allow` render `successfulJobsHistoryLimit`, `failedJobsHistoryLimit`
  and `concurrencyPolicy`.
- `a_scheduled_workload_declares_suspend_deadlines_and_retries`: `suspended: true` renders
  `suspend: true` (absent renders `false`); the start deadline renders `startingDeadlineSeconds`;
  the run deadline renders the job's `activeDeadlineSeconds`; the retry limit renders the job's
  `backoffLimit`.
- `an_unbounded_scheduled_workload_is_refused_by_name`: a scheduled workload with no run deadline
  is refused with a named diagnostic that names the workload and the missing deadline.
- `a_scheduled_workload_keeps_state_between_runs`: a declared state location renders a
  PersistentVolumeClaim the job mounts on every run (no `StatefulSet`).
- `a_malformed_or_conflicting_schedule_is_refused_by_name`: an invalid cron expression, a schedule
  beside replicas or a Service port, a readiness probe on a scheduled workload, and a zero or
  negative deadline or a negative retry limit are refused with a named diagnostic.
- `a_runtime_without_schedules_keeps_its_chart_bytes`.
- A consumer's minimal example renders exactly: workload `ingest-a`, image by digest
  (`example.invalid/runner@sha256:<64 hex>`), schedule `2-59/10 * * * *`, overlap forbid,
  suspended, with the pod settings of `story:runtime-workload-probes-resources-placement-annotations`
  (a Secret volume with one key-to-path item and mode 0400, `fsGroup: 10001`). Test:
  `the_scheduled_ingest_example_renders_exactly`, a golden chart.
- The format consequence of the new `ess-runtime` keys is decided under the repository's format
  rules, once for this story and the pod-settings story, with the old-reader compatibility test
  first.

## Design first

A section under `docs/design/` before code: where `schedule:` lives (`ess-runtime` workload,
format version consequence), how a scheduled workload realizes a semantic component, and what the
deployment-chain gaps already drafted (resources, annotations, probes) share with it.

## Out of scope

Running or triggering jobs; one-shot `Job` workloads (a later story if asked).
