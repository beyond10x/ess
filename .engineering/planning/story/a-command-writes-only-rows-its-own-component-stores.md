---
format: aep.planning-md/3
id: story:a-command-writes-only-rows-its-own-component-stores
kind: story
status: draft
title: A command writes only rows its own component stores; a write into another component's domain is refused toward an event and a binding
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/generate/ess-synth/src/determined.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/declared_behaviour.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/set_effects.rs
- confidence: cited
  path: crates/specify/ess-domain/src/component.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/spec.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/adversary_w2_1_pass1.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/cross_component_writes.rs
- confidence: inferred
  path: release-notes/
- confidence: cited
  path: website/docs/guides/specify/bindings-and-components.md
- confidence: cited
  path: website/docs/guides/specify/selection-effects.md
revision: 12
---
## Outcome

`ess specify validate` refuses an outcome that creates, moves, updates or deletes rows of an entity
whose domain belongs to a different component than the command's domain. That covers the subject
(`creates:`, `moves:`, `updates:`, `deletes:` with `instance:`), the rows `instances:` selects, and
`affects:` entries. The hint names the idiom the language already gives for removal: emit an event,
and bind it to a command of the owning domain. Synthesis applies the same storage rule to a subject
write that it applies to `when_related:` reads and `affects:` writes, so a component never gets a
generated behaviour over rows it does not store. The refusal of a deleting `affects:` entry across
domains keeps its scope, and its message stops claiming more than it enforces.

## Need

Our own finding, made while assessing other reports. Nobody requested it, so there is no fit
review. Reproduced on the released 0.97.3 binary. `f2/base/` copies the earlier generic model:
`runs.job` is owned by `job-api`, `runs.work` by `job-worker`, and the commands of `runs.work` move
and update `runs.job.Job`. `f2/base2/` adds `unknown_instance:` and `wrong_state:` outcomes, so the
behaviours are generated. Every run is in `f2/run.txt`.

What validate admits (`f2/variants.sh`, one more write of `runs.job.Job` from `runs.work` per variant):

| variant | write from `runs.work` | 0.97.3 |
|---|---|---|
| `base` | subject `moves:` and `updates:` | valid |
| `v/v4-subject-delete` | subject `deletes:` | valid |
| `v/v7-creates` | subject `creates:` | valid |
| `v/v5-instances-update` | `updates:` with `instances: {where:}` | valid |
| `v/v6-instances-delete` | `deletes:` with `instances: {where:}` | valid |
| `v/v2-affects-update` | `affects:` entry with `sets:` | valid |
| `v/v3-affects-move` | `affects:` entry with `moves:` | valid |
| `v/v1-affects-delete` | `affects:` entry with `deletes:` | `ESS-COMMAND-009` "deletes rows of `runs.job.Job`, which domain `runs.job` owns, from domain `runs.work`; `affects:` stays inside its outcome's domain …" |
| `v/v8-one-owner-affects-delete` | the same, one component owning both domains | `ESS-COMMAND-009`, the same |

So the refusal covers only a deleting `affects:` entry, and it compares domains, not components.
A subject `deletes:` and a bulk `instances:` deletion across domains are admitted. That is removal
in another domain too, so the message's general claim "`affects:` stays inside its outcome's
domain" is false for `sets:` and `moves:` entries.

What the language says the rule is:

- `crates/specify/ess-domain/src/component.rs:1372-1375`, the hint of the refusal of two owners:
  "a domain has one owning component — §9 gives its state one component that modifies it; split
  the domain, or give it to one of them and connect the other with a binding".
- `component.rs:1404`: "§9 gives a command one handler, and it is the component owning its domain".
- `docs/design/ess-implementor-design-v0.1.md` § 9, the canonical graph: COMMAND "handled by"
  COMPONENT "modifies" DOMAIN STATE "emits" EVENT "triggers" COMMAND "handled by" OTHER COMPONENT.
- `website/docs/concepts/ess.md:48`: a logical component is "Domain ownership … A component can
  become a module or a process without changing which domain it owns."
- Synthesis already holds a storage rule. A `when_related:` read and an `affects:` write of a row
  "which no component accepting the command stores" stay owed
  (`crates/generate/ess-synth/src/determined.rs:1745-1757`, `:1948-1959`;
  `tests/declared_behaviour.rs` `a_related_row_no_accepting_component_stores_stays_owed`: "A related
  row of a domain no component accepting the command owns has no storage port in that component").
  A subject write has no such check.

The delete refusal (`set_effects.rs:2637-2666`, `selection-effects.md:142-152`, beyond10x/ess#452)
is therefore the special case. It declines cascade removal by `affects:` and answers it with one
event and one binding per receiving domain. The general rule, a component modifies only the state
of the domains it owns, is enforced for acceptance and for synthesis's reads and `affects:` writes,
and missed for the subject and for `instances:`.

What follows downstream today (the suites from `f2/base`, the generated code from `f2/base2`):

- The system suite (`ess verify conform synthesize --path base`, 10 scenarios) runs `SubmitJob`
  (job-api), then `StartJob` (job-worker), then reads `runs.job.JobById` (job-api) and requires
  the moved row there. It passes only where both components share one store of `runs.job.Job`.
- `--component job-worker`: "0 scenario(s) for `job-worker` (0 authored), 9 outside it", each
  "needs `command runs.job.SubmitJob`, `event runs.job.JobChanged`, `view runs.job.JobById`". The
  component that accepts the three commands can never be held to them on its own.
  `--component job-api` has 1 scenario, and none of them moves the `Job` lifecycle that job-api
  owns.
- `ess generate synthesize --target rust` reports "19 capabilities: 19 generated, 0 obligation(s)".
  The `StartJob` behaviour is generated over a `JobStorage` port in job-worker. The served binary
  hands both components clones of one in-memory store
  (`gen-rust2/crates/runs-server/src/bin/job-api-server.rs:20-21`, `MemoryPorts` "Ports shared by
  generated components"). Split into two processes, as the concept page allows, job-worker holds
  no `Job` rows.
- The OpenAPI projection puts `/work/commands/StartJob` on job-worker's document and
  `/job/views/JobById` on job-api's (`http-surface/run.txt`), with nothing saying they share rows.

Also observed, and not part of this story: `StartJob` emits `runs.job.JobChanged`, which job-api
publishes. The generated job-worker publishes nothing for it (`PublishedEvent` is empty, and
`start_job` matches `Started { .. } => {}`). Emitting another component's event is a sibling
question for a story of its own.

## Design

One refusal, written at the outcome where the write is declared. A command's domain and the
written entity's domain are each owned by a component (`components.yaml` `owns:`). Where both are
owned and by different components, each write is refused as `unsupported_construct`
(`ESS-COMMAND-009`, the code of the delete refusal), at the key that names it: the subject's
`creates:`/`moves:`/`updates:`/`deletes:`, `instances:`, or `affects[i]`. The message is:

"outcome `<o>` of `<C>` <verb>s rows of `<E>`, whose domain `<D>` component `<K>` owns, and
`<C>` is handled by `<K'>`; a component modifies only the state of the domains it owns"

The hint: "emit an event from this branch and bind it to a command of `<D>` that `<verb>`s its own
rows, with that binding's own `delivery:` and `on_failure:`; or give `<D>` and `<C>`'s domain to one
component".

- Not refused: a model without `components.yaml`, a domain no component owns (it is already
  `ess generate --strict`'s refusal), and two domains one component owns (`v8`'s shape, apart from
  the existing cascade refusal). This matches synthesis's storage rule, which is per component.
- The existing domain-level refusal of a deleting `affects:` entry stays (D4). Its message changes
  from "`affects:` stays inside its outcome's domain" to "a deleting `affects:` entry stays inside
  its outcome's domain". `selection-effects.md:144` gets the same narrowing. The page also says
  that a deleting subject or a bulk `instances:` deletion across domains is governed by the
  component rule.
- Synthesis: the `unstored` check of `determined.rs` also covers the subject and `instances:`. A
  command accepted by a component that does not own the written entity's domain stays an
  obligation, "a subject row of `<E>`, which no component accepting the command stores". After the
  refusal, this is reachable only where the entity's domain has no owner. It closes the shared-store
  generation for that case too.
- `bindings-and-components.md` § Three layers states the rule in one sentence beside "duplicating
  domain ownership is refused".

Breaking-change analysis:

- Committed models newly refused: none. `census/census.sh` compiled all 365 model roots under
  `examples/` and `crates/**/tests/fixtures/` at tag `0.97.3` (353 compile; 19 of them declare two
  or more domains). It listed every outcome that writes an entity of a domain other than its
  command's, through the subject, `instances:` or `affects:`. It found 0, so none crosses
  components either. Inline YAML in Rust test sources was not counted.
- Adopter models: a model whose command writes another component's domain is refused. Its
  conformance suite already needed both components in one store, and its component suites
  already listed those scenarios as outside.
- Format: no admitted document changes meaning, identity or bytes. The refusal removes models
  whose two components the generated runtime could not separate. By AGENTS.md § Determinism and
  formats no new `ess/N` is required. The precedent is refusal in every format with a
  `**Breaking**` changelog entry (#93, #438). D3 offers gating it on the next `ess/N` instead. No
  IR, suite or plan format changes. The synthesis plan of an affected unowned-domain model moves
  one capability from generated to obligation.

## Acceptance

1. On `f2/base`, `ess specify validate` refuses `StartJob`, `RecordProgress` and `FinishJob`, each
   with `ESS-COMMAND-009` at its `moves:`/`updates:` line, naming `runs.job`, `job-api` and
   `job-worker`, with the event-and-binding hint. The same holds for the subject `creates:` and
   `deletes:`, `instances:` updates and deletions, and `affects:` `sets:` and `moves:` entries
   (variants v2-v7): `cargo test -p ess-domain --test cross_component_writes` (new).
2. The same commands are admitted when one component owns both domains, and when no
   `components.yaml` exists: same test.
3. The deleting `affects:` entry across domains is still refused, with the narrowed message:
   `ess23_deleting_entry_over_another_domains_entity_is_refused` in
   `crates/specify/ess-domain/tests/adversary_w2_1_pass1.rs`, updated to the new wording.
4. A command of an owned domain that moves an entity of an unowned domain is planned as an
   obligation naming the storage rule, never generated:
   `cargo test -p ess-synth --test declared_behaviour`, a case beside
   `a_related_row_no_accepting_component_stores_stays_owed`.
5. The committed example trees do not move: `ess-synth`'s `feasibility`,
   `adversary_related_guard_behaviour_pass1` and `http_binding_rust` pass unchanged.
6. `selection-effects.md` and `bindings-and-components.md` state the rule. `CHANGELOG.md` and
   `release-notes/<version>.md` carry a `**Breaking**` bullet: `cargo xtask release notes --check`,
   `task test-xtask`.

## Decisions

- **D1. Which boundary.** (A) Components: refuse a write where the command's and the entity's
  domains have different owning components. That is §9, the hint of the two-owner refusal, and the
  storage rule synthesis already applies. (B) Domains: refuse every cross-domain write, the scope of
  the cascade refusal, even inside one component and without components. (C) Intended: admit
  cross-component writes, document that their components share one store, and drop the storage
  rule in synthesis. **Recommended: A.** It is the rule the language already states in three
  places. B would refuse a pattern synthesis deliberately supports for reads inside one component,
  and C contradicts "a component can become a module or a process without changing which domain it
  owns".
- **D2. Code and place.** (A) `ESS-COMMAND-009` `unsupported_construct` at the outcome's write key,
  as the cascade refusal. (B) `conflicting_declaration` at the component's `accepts.commands`,
  as the refusal of a command accepted by a non-owner. **Recommended: A.** The write is what
  conflicts, and the line a modeller must change is the outcome.
- **D3. Which formats.** (A) Every format, Breaking changelog entry (precedent #93, #438). (B) From
  the next open `ess/N`; below it, a warning on standard error. **Recommended: A.** No committed
  model is affected, and the generated runtime does not separate such components in any format.
- **D4. The cascade refusal's scope.** (A) Keep it domain-level and narrow only its message. (B)
  Lift it to the component rule, which admits a deleting `affects:` entry across two domains of one
  component. **Recommended: A.** Its reason is the cascade's semantics (one binding per receiving
  domain, no order between them), not storage, and relaxing it is a separate design question.
