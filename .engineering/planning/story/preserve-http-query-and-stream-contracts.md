---
format: aep.planning-md/1
id: story:preserve-http-query-and-stream-contracts
kind: story
status: draft
title: Project existing HTTP query and event-stream contracts faithfully
revision: 2
---
## Consumer contract gap
A read-only HTTP API needs a faithful OpenAPI contract for a JSON object snapshot response, a GET operation with a JSON-encoded query argument, server-sent events (text/event-stream), and plain-text HTTP error responses. The installed ESS0.20.0 native projector only derives its supported command/view route shapes; adapting the real API to those routes would change its consumer contract. The OpenAPI adapter's JSON-only supported subset cannot silently omit query locations or event-stream media types and still claim a faithful round trip.

## Requested outcome
Assess and extend concrete HTTP query serialization, response media-type and event-stream semantics in the owning typed model and adapter. Preserve existing route/method/status/schema meanings; do not coerce object snapshots into entity row arrays or claim completeness after dropping query or streaming behavior. Unsupported contracts must remain explicit refusals with coverage diagnostics and no misleading generated artifact. Use synthetic fixtures with no adopter data, credentials or private service addresses.

## Acceptance for future implementation
A synthetic existing GET snapshot, JSON query and event-stream contract can be imported/projected without changed routes or payload cardinality; each supported direction declares its coverage. Deterministic artifacts retain query schema/serialization and response media types/statuses. Unsupported cases preserve typed diagnostics. Existing adapter refusal and compatibility checks remain intact. A design identifying the required concrete types precedes implementation, without generic property bags or an unsolicited format fork.

## Scope and ownership
Draft consumer request only. ESS maintainers own assessment, design, implementation and release. This consumer session changes no ESS source and upgrades no installed dependency. The consumer can publish a reviewed native OpenAPI source meanwhile, explicitly without an ESS-generation claim. Exact synthetic reproduction evidence will be attached when retained by the consumer probe.

## Reproduction — ESS0.20.0
Synthetic OpenAPI3.1 operation GET /events declares query parameter query (string) and response200 text/event-stream (string). No private data or service address is involved.

`ess import openapi --path synthetic.openapi.json --out synthetic.import.json --format json` exits0 and explicitly reports coverage gaps:
- /paths/~1events/get/parameters: interface feature not preserved
- /paths/~1events/get/responses/200/content/text~1event-stream: interface feature not preserved

`ess generate project openapi --ir synthetic.import.json --out projected --format json` exits1:
```text
error: OpenAPI projection refused: /paths/~1events/get/parameters: unpreserved semantics prevent checked projection: interface feature not preserved; /paths/~1events/get/responses/200/content/text~1event-stream: unpreserved semantics prevent checked projection: interface feature not preserved
```
Import success therefore does not establish faithful projection. Original canonical contract SHA256 e7d182656631e1fb26ccc3465e97df390281d2d91851adbb04a666bb1d07b21c. This reproduces the query/event-stream seam, not every snapshot or error shape; broaden coverage during design.
