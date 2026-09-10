# Acceptance and evidence

| Area | Required independent evidence |
| --- | --- |
| ESS preservation | Complete existing task check, fixtures and canonical outputs; moved-capability mapping; every CLI parser/generator/process/protected-input/refusal case retained; unknowns explicit |
| Eventlog | File/SQLite/PostgreSQL group atomicity/idempotency/order/guards/projection rollback, concurrent writers, conflicting expectations, retries, crash/reopen, blob integrity, projection rebuild and snapshot invalidation |
| ER | Complete decision replay, zero-event records, ordered non-mutating observations, record-ID conflicts, multi-entity atomicity, tampering/unverified-history refusal and provider compatibility |
| AEP | New stores and all legacy configuration migrations, truthful history, deletion/rebuild and drift refusal, receipt-bearing post-commit projection failure, divergent history refusal |
| Rust realization | Generated billing/gatepass builds, starts, serves HTTP, persists through ER/Eventlog, restarts with auth/query/effect behavior intact |
| Connectors | Existing reads/federation, locked/MSRV gate, process start/stop, credential failure/uncertainty, restart reuse, repair/revoke races, protected inputs, schema validation and redaction |
| Babelconnect boundaries | Descriptor/wire compatibility, Go/Dart/TS cache agreement, client/server/downstream mapping/reduction, auth/reconnect/media behavior |
| Flutter | Existing tests plus generated widget/journeys; both-theme visual inspection; actual Linux, web and Android local execution |
| Infrastructure | Existing tests plus deterministic projections and desired-versus-observed checks for actual selected placements |
| Sensitivity | Wrong field mapping, dropped patch, incorrect transition, bypassed revocation and misbound UI action each fail their independent test |

Keep existing Babelconnect package gates, Linux integration and browser E2E. Add Android with a local
backend and generated journeys through real Go. A disposable local SIP/WebRTC smoke establishes
actual media evidence separately from simulated-media tests. Connectors uses local fake provider
endpoints and disposable SQL where needed; remote observations are separately reported.

Run affected tests while iterating and one full gate for each stable delivery slice. Retain exact
commands, exit statuses, source/dependency/configuration vectors and missing capabilities. A selected
lane that did not execute is missing evidence, never passing. A generated artifact is not a running
service. A local acceptance result is not a release, publication, deployment or Atlas delivery.

