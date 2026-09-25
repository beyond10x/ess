# Acceptance and evidence

Authority: approved ESS evolution plan ess-evolution-20260915 revision 1.

| Area | Required independent evidence |
| --- | --- |
| ESS preservation | Complete task check with consumers enabled, task site-build, canonical fixtures/outputs, capability mapping and every existing CLI behavior; unknown/refused coverage remains explicit |
| Eventlog | File/SQLite/PostgreSQL atomicity, ordered repeated-stream expectations, retries/conflicts, guard/projector rollback, concurrency, crash/reopen, blobs, snapshot invalidation and corruption/divergence refusal |
| ER | Complete replay, zero-event decisions, ordered non-mutating observations, global record-ID conflicts, atomic multi-entity execution, tamper refusal, compatible provider/session behavior and explicit legacy imports |
| AEP migration | All legacy inputs/configurations, truthful history/boundaries, source fencing, staged verification, interruption before/after cutover, drift/deleted projections, rebuild, durable projection-failure receipts and safe retries |
| Actual planning stores | Eventlog, ER, Service SDK, ESS, Connectors v2 and AEP cut over in that order, with retained recovery copies and verified history/query/mutation/restart/rebuild |
| Services | Generated billing/gatepass build, boot, serve HTTP, persist through ER/Eventlog, restart and preserve auth/query/effect behavior |
| Connectors | Locked/MSRV gate, exact metadata migration and dependency pins, real disposable provider/keyring fixtures, supervision, restart, repair/revoke races, protected input, uncertainty and redaction |
| Infrastructure | Deterministic service/local-process projections plus independently supplied observations and credential-redaction mutation checks |
| Fault sensitivity | Wrong fields, dropped state/event updates, incorrect transitions and bypassed revocation each fail the corresponding independent test |

Run affected tests during development and each repository's complete gate on a stable integration
slice. Required commands are ESS task check with consumer coverage and task site-build;
ER/AEP/Service SDK task check; Eventlog bash scripts/gate.sh; and Connectors
cargo run --locked -p connectors-build -- gate --msrv. Exercise affected Rust minimum boundaries.

PostgreSQL requirements run against disposable databases. Record every selected lane's own output,
exit status and actual execution. A selected-zero, ignored or skipped required case is missing
evidence even when a wrapper exits zero. Preserve Eventlog's required production proof and separate
restart evidence; a local generic conformance run does not substitute for them.

Keep exact source, dependency, configuration and fixture vectors with logs. Expected behavior comes
from upstream scenarios and independent fixtures; tests derived only from target plans are
insufficient. A generated artifact does not prove a running service. Desired declarations do not
prove observed infrastructure.

Generic protobuf/UI/Flutter acceptance belongs to task:deferred-protocol-ui-bindings. Local acceptance
does not imply release, publication, deployment or documentation delivery.
