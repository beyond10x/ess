# ADR index

Current scope authority is approval-record:ess-evolution-20260915, approved plan revision 1.
The 2026-09-10 records retain their history; their current bodies are reconciled where the revised
scope changes acceptance. Lifecycle status and actual implementation evidence remain separate.

| ADR | Owner | Governed record / state |
| --- | --- | --- |
| 01 — Evolve ESS without feature loss | ESS | [Record](../../../.engineering/planning/architecture-decision-record/ess-evolution-01-evolve-without-feature-loss.md) |
| 02 — Semantic models precede implementation bindings | ESS | [Record](../../../.engineering/planning/architecture-decision-record/ess-evolution-02-semantics-before-bindings.md) |
| 03 — Bindings and language ownership | ESS | [Record](../../../.engineering/planning/architecture-decision-record/ess-evolution-03-binding-language-ownership.md) |
| 08 — Source imports produce reviewable candidates | ESS | [Record](../../../.engineering/planning/architecture-decision-record/ess-evolution-08-reviewable-source-candidates.md) |
| 09 — Conformance observes implementations independently | ESS | [Record](../../../.engineering/planning/architecture-decision-record/ess-evolution-09-independent-conformance.md) |
| 10 — Infrastructure intention and observation remain distinct | ESS | [Record](../../../.engineering/planning/architecture-decision-record/ess-evolution-10-infra-intention-observation.md) |
| 04 — Recorded entity execution over Eventlog | ER | Owner-local async/adapter design and decision remain pending |
| 05 — File Eventlog and atomic append groups | Eventlog | architecture-decision-record:ess-evolution-05-file-and-atomic-groups; current-vector qualification pending |
| 06 — Markdown is an AEP projection | AEP | Owner-local record pending; exactly one owning migration story |
| 07 — Service execution delegates to ER | Service SDK | Owner-local record pending |
| 12 — Connectors local runtime profile | Connectors v2 | Existing CLI specification/reviews retained; metadata-adoption ADR pending |
| Coordinated format and six-store cutover | Atlas | Required cross-repository ADR pending before new contract implementation |

Generic protocol/UI/Flutter work is preserved under task:deferred-protocol-ui-bindings. No excluded
adopter ADR is required for this initiative. The record list does not claim outstanding designs or
runtime verification already exist.
