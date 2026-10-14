---
format: aep.planning-md/3
id: story:runtime-realizes-a-scheduled-job
kind: story
status: draft
title: A runtime workload can be realized as a scheduled job (Kubernetes CronJob)
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

An `ess-runtime` workload can be declared a scheduled job, and `ess generate project helm` renders
it as a Kubernetes `CronJob` (`batch/v1`) instead of a `Deployment` or `StatefulSet`: a cron
schedule, a concurrency policy that forbids overlapping runs, successful- and failed-run history
limits, secret references as the long-running workloads take them, and a declared state location
that survives between runs.

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

Failing tests first:

- `a_scheduled_workload_renders_a_cron_job`: a runtime with one workload declared
  `schedule: {cron: "*/15 * * * *"}` renders `apiVersion: batch/v1`, `kind: CronJob` with that
  schedule, `concurrencyPolicy: Forbid` by default, and the job template carrying the container,
  its environment, config and secret references exactly as a `Deployment` would.
- `a_scheduled_workload_declares_history_limits_and_overlap`: `successful_runs`, `failed_runs` and
  `overlap: forbid | replace | allow` render `successfulJobsHistoryLimit`, `failedJobsHistoryLimit`
  and `concurrencyPolicy`.
- `a_scheduled_workload_keeps_state_between_runs`: a declared state location renders a
  PersistentVolumeClaim the job mounts on every run (no `StatefulSet`).
- `a_malformed_or_conflicting_schedule_is_refused_by_name`: an invalid cron expression, a schedule
  beside replicas or a Service port, and a readiness probe on a scheduled workload are refused
  with a named diagnostic.
- `a_runtime_without_schedules_keeps_its_chart_bytes`.

## Design first

A section under `docs/design/` before code: where `schedule:` lives (`ess-runtime` workload,
format version consequence), how a scheduled workload realizes a semantic component, and what the
deployment-chain gaps already drafted (resources, annotations, probes) share with it.

## Out of scope

Running or triggering jobs; one-shot `Job` workloads (a later story if asked).
