---
title: Format version history
sidebar_position: 3
description: What each ESS format version number means, which release introduced it, and what an older reader does with a newer document.
---

# Format version history

An ESS document declares its own format in its bytes — `ess/14`, `ess-diff/7`,
`ess-conformance/19`. That number is the format's major version and nothing else. It is not the
release that produced the document, and not the specification version the document describes;
[Formats and digests](./formats.md) separates those three. This page says what each number changed,
which release introduced it, and what happens when an older reader meets a newer document.

## When the number moves

A format version is required when meaning, identity, references, canonicalization, names, or the
persisted envelope changes. A new internal capability that leaves the persisted shape alone does
not move it.

Every family is read by a build that states which versions it implements and refuses the rest. The
refusal is the point: a reader that accepts a shape it does not understand returns a wrong answer
about somebody's system, and blames the document for the age of the tool.

A version number is per family. `ess/14` and `ess-conformance/19` count
separately and always have.

## `ess/` — the authored specification

The format of the system a person writes. Read by `ess specify validate` and everything downstream
of it.

Declare the lowest version that admits every construct the specification uses. A build refuses a
header newer than it implements, and refuses each construct under a header older than the one that
introduced it with `unsupported_format_version`. A specification that uses none of a version's
constructs keeps its bytes and its compiled digest under the older header.

| Version | Released in | What it admits |
|---|---|---|
| `ess/1` | [0.1.0][r1] | types, entities, commands, events, errors, views, actors, components, bindings, topology |
| `ess/2` | [0.20.0][r20] | `Binary64` |
| `ess/3` | [0.23.0][r23] | `when_subject_state`; binding accessors into an event envelope |
| `ess/4` | [0.23.0][r23] | error wire names; command response fields mapped into event payloads |
| `ess/5` | [0.27.0][r27] | an enum variant's own `wire`, `display`, `summary` and `code` |
| `ess/6` | [0.28.0][r28] | an input guard beside an external cause; `when_subject` over an enum field; `preserves` |
| `ess/7` | [0.29.0][r29] | `replays`; an effect-free named error as the default of subject-state branches |
| `ess/8` | [0.34.0][r34] | `starts_with`, `ends_with`, `contains` |
| `ess/9` | [0.34.0][r34] | `when_subject: {predicate: …}` over stored fields |
| `ess/10` | [0.34.0][r34] | aggregate views: `aggregate:` and `group_by:` |
| `ess/11` | [0.34.0][r34] | `alphabet:`, input `example:`, `.count` on a `String` |
| `ess/12` | [0.34.0][r34] | `outcome_groups:` |
| `ess/13` | [0.35.0][r35] | `fixture_inputs:` |
| `ess/14` | [0.36.0][r36] | value expressions in `payload:` and `sets:` |
| `ess/15` | [0.37.0][r37] | `unknown_instance:`, `deletes:`, `into:`, `accepts: nothing`, `preconditions:`; `input.` in subject guards; `equals_ignore_case`, `in_ignore_case`; `prefix:`, `Json`, `presence:`; aggregates over `Optional` fields |
| `ess/16` | [0.38.0][r38] | `input_absent:`, `existing_instance:`, actor `attributes:`, view `paging:`, bounded retry, `instances:` and `affects:`, `{related: …}`, literal `else:` |
| `ess/17` | [0.39.0][r39] | `returns: true` |
| `ess/18` | [0.41.0][r41] | a list in `when_subject_state:`, and `when_subject_state:` on a refusal naming no subject; `state` in a `when_subject` predicate; `when_related:`; `when.context_fields`, `when.context_authority` and `context.<field>` on a binding |

The sections below give each version's rules. The first five are a table:

| Version | Released in | What changed | An older reader |
|---|---|---|---|
| `ess/1` | [0.1.0][r1] | The first format: types, commands, events, errors, views, entities. | — |
| `ess/2` | [0.20.0][r20] | Finite `Binary64` fields, distinct from integer and decimal values. Signed zero, subnormals and nearest-even rounding are preserved end to end. | Refuses `Binary64` at every declared type position, including as a map key. |
| `ess/3` | [0.23.0][r23] | `when_subject_state`, which combines a declared held lifecycle state with input guards. Opt-in binding accessors read two or three declared field segments from an event envelope. | Refuses the document. |
| `ess/4` | [0.23.0][r23] | Error wire names declared without merging semantic error identities. Typed command response fields fill emitted event payloads through explicit response mappings, with complete payload ownership. | Refuses the document. |
| `ess/5` | [0.27.0][r27] | An enum variant carries its own `wire`, `display`, `summary` and `code`. A variant is authored as a bare name or as a mapping. | Refuses with `unsupported_format_version` at `types.<type>.variants.<variant>`. |

`ess/6`, introduced in [0.28.0][r28], admits an input eligibility predicate beside an external
cause. The fault remains independently arranged; the guard does not select the observed outcome.
Earlier source formats refuse this combination. It also admits `when_subject` over a
declared enum field of an existing subject and `preserves` for a successful silent no-op.
History is observed independently from lifecycle. The bounded arrangement search preserves
different histories that reach the same state; unknown or unobservable history refuses synthesis.
A preserving outcome cannot assign fields, emit events, or declare an error.

`ess/7`, introduced in [0.29.0][r29], adds command-local `replays`, which returns the originating
success's retained typed result without repeating its effects. The origin supplies
the observable subject identity; the retry cannot invent another selector. It also
admits an effect-free named error as the finite default complement of explicit
subject-state branches sharing one existing subject. Earlier formats refuse these
constructs. A replay response containing Decimal or Binary64, including through
nested declarations, is outside the exact-result observation profile and refuses
synthesis. This does not change existing response-to-event comparisons.

`ess/8`, introduced in [0.34.0][r34], admits the string operators `starts_with`, `ends_with` and `contains` in
every predicate position: command guards, invariants, view filters and binding selections. They
are map form only and apply to `String` and newtypes of it. Earlier source formats refuse them with
`unsupported_format_version` at the position that uses one. A model that uses none keeps its bytes
and its compiled digest. See [string operators](./predicates.md#string-operators).

`ess/9`, introduced in [0.34.0][r34], admits `when_subject: {predicate: …}`: a predicate over the declared
stored fields of the subject a command addresses, read immediately before selection and
conjunctive with an ordinary `when:`. It reads the entity's fields and nothing else — not the
input, not `state`. A refusal may carry it without naming a subject of its own; it reads the one its
sibling branches name. Closed stored-field domains enter the branch partition beside the input, and
an open one, such as `weight_kg > 20`, needs a genuine default. Conformance arranges a row to the
guard through the arranging commands' `sets:` mappings and observes it before the command runs. An
older build refuses the header; this build refuses the predicate form under an earlier header with
`unsupported_format_version`. `{field, equals}` keeps `ess/6` and its bytes.

`ess/10`, introduced in [0.34.0][r34], admits aggregate views: a view field may declare `aggregate:` — one of
`count`, `count_distinct`, `sum`, `min`, `max` and `avg` — and a view may declare `group_by:` over
its other fields. The filter runs per source row, the admitted rows are grouped, and a group with no
admitted row is absent; a view without `group_by` returns exactly one row. Each aggregate field
declares its result type exactly: `Integer` for the counts and for `sum` of an `Integer`, `Optional<T>`
for `min` and `max`, and `Optional<Decimal>` for `avg`, rounded to 6 places half-even. An older build
refuses the header, and this build refuses the construct under an earlier header with
`unsupported_format_version`. A model without it keeps its bytes and its compiled digest. See
[aggregate views](../guides/write-a-specification.md#aggregate-views).

`ess/11`, introduced in [0.34.0][r34], admits three things. A newtype of `String` may declare `alphabet:`, the
characters every value is drawn from. A command input may declare `example:`, the value synthesis
builds it from. And `.count` on a `String` is its length in Unicode scalar values, in every
predicate position. An older build refuses the header, and this build refuses each construct under
an earlier header with `unsupported_format_version`. A model without them keeps its bytes and its
compiled digest.

`ess/12`, introduced in [0.34.0][r34], admits `outcome_groups:`, a top-level list that declares one external
refusal once for many commands. A group selects its members by an explicit `commands:` list, by
`actor:` (every command that actor `may:` invoke) or by `domain:` (every command that domain's
files declare), with `except:` beside a selector. Each member gains the group's outcomes after its
own, in ascending group-name order, before anything is validated, so a group and the same outcomes
copied by hand compile to the same IR and synthesize the same suite. An outcome of a group is
`external:` plus `error:` and nothing else. A member that already declares an outcome of the same
name, and two groups giving one command outcomes of the same name, are refused rather than
overridden. An older build refuses the header, and this build refuses a group under an earlier
header with `unsupported_format_version`. A model without groups keeps its bytes and its compiled
digest. See [one outcome for many commands](../guides/write-a-specification.md#one-outcome-for-many-commands).

`ess/13`, introduced in [0.35.0][r35], admits `fixture_inputs:` on a command: a map from a declared input
field to a lower-kebab fixture name, typed by that input. A deployed resource's identity or
canonical input, which a deterministic generator cannot invent, is then resolved from an
independent provider before the scenario starts, and the command request and its event-value
assertions use the same copied value. A fixture input that an outcome predicate reads, that
names an undeclared input or a scenario-owned subject, or that reuses one fixture name under a
different type is refused. An older build refuses the header, and this build refuses the key
under an earlier header with `unsupported_format_version`. A model without it keeps its bytes and
its compiled digest.

`ess/14`, introduced in [0.36.0][r36], adds value expressions to `payload:` and `sets:`: `{subject: <field>}` reads the addressed entity as it was before the outcome, `{increment: <number>}` adds to a stored `Integer` or `Decimal`, `{input: <field>, else: {generated: true}}` takes an optional input or a minted value, a nested mapping gives each field of a struct-typed target its own source, and `{generated: true}` is admitted in `sets:`. Synthesis asserts each value where the arrangement determined what it reads, and makes no claim otherwise. An older build refuses the header, and this build refuses each construct under an earlier header with `unsupported_format_version`. A model without them keeps its bytes and its compiled digest. See [value expressions](../guides/write-a-specification.md#value-expressions).

`ess/15`, introduced in [0.37.0][r37], admits these constructs: outcome shapes (`docs/design/outcome-shapes.md`), `input.` operands in subject guards and case-insensitive text comparison (`docs/design/value-expressions.md` E6, E7), new value types, and aggregates over optional fields. This build admits the header; each construct states its own refusal under an earlier header.

`ess/16`, introduced in [0.38.0][r38], admits the constructs described in the paragraphs below. This build admits the header; each construct states its own refusal under an earlier header.

Under `ess/16` a command may declare `input_absent: true` with an `error:`: the answer for a request that carries no input at all (beyond10x/ess#170). It is refused below `ess/16` with `unsupported_format_version`. Under `ess/16`, a guard that cannot hold because every way it could hold needs an input `f` that is not `Optional` to be absent (`not defined(f)`, `missing(f)`) is refused as a type mismatch; below `ess/16` it validates as before.

Under `ess/16` an outcome may be selected by whether the addressed record exists (beyond10x/ess#164): a `creates:` branch marked `unknown_instance: true` beside the branch that updates the record the same input names (create or update), or an `existing_instance: true` branch with an `error:` beside a creation whose identity the caller supplies (create or refuse). Each is refused below `ess/16` with `unsupported_format_version`. See [selection by existence](../guides/write-a-specification.md#an-outcome-can-be-selected-by-whether-the-record-exists).

Under `ess/16` an actor may declare `attributes:` its credential carries, which a command reads as `{caller: <attribute>}` in `payload:` and `sets:` and as `caller.<attribute>` in a `when:` or `when_subject:` comparison (beyond10x/ess#168). Below `ess/16` the attributes and a `caller.` operand are refused with `unsupported_format_version`, and `{caller: …}` stays a nested mapping.

Under `ess/16` a view with `order_by:` may declare `paging: {page: <param>, size: <param>, first_page: 0|1, total: true|false}` (beyond10x/ess#174): two declared `Integer` parameters that slice the declared order, `size` rows starting at `(page - first_page) * size`, with the number of rows the filter admits beside them where `total: true`; a read that sends neither answers every row. The parameters `paging:` names are exempt from the refusal of a parameter no filter reads. `paging:` is refused below `ess/16` with `unsupported_format_version`. A caller-supplied filter expression is not part of it. See [paging a view](../guides/write-a-specification.md#a-view-can-be-paged).

Under `ess/16` a binding may bound its retry: `on_failure: {retry: {attempts: 3, final: [<refusal>]}}` (beyond10x/ess#165). `attempts` counts invocations including the first and is at least 2; `final` names refusals of the invoked command, by outcome or by error, that end the retry at once. The block is refused below `ess/16` with `unsupported_format_version`; `on_failure: retry` written bare keeps its meaning.

Under `ess/16` a `moves:` or `updates:` outcome may declare `instances: {where: <predicate>}` instead of `instance:`, changing every stored record the predicate selects over the entity's fields and `input.<field>` (beyond10x/ess#167), with `{count: changed}` as the number it changed; and an outcome with one existing subject may declare `affects:`, a list of `{entity, where, sets}` changing the records each filter selects, which may also read `subject.<field>` (beyond10x/ess#175). Each is refused below `ess/16` with `unsupported_format_version`, and `{count: changed}` stays a nested mapping there. See [set effects](../guides/write-a-specification.md#an-outcome-can-change-every-record-a-filter-selects).

`ess/3` and `ess/4` both arrived in 0.23.0. There was never a release that implemented `3` and not
`4`, and there is no missing release between them.

A bare variant list is admitted by every version, and a variant that declares no naming serializes
back as a bare name, so a specification written before `ess/5` keeps its exact bytes.

`ess/17`, introduced in [0.39.0][r39], adds `returns: true` to an outcome whose command declares a nonempty
typed `response`. The outcome promises a successful return matching that schema; it makes no
claim about persistence or side effects. Those require their own declarations and real API
observations. It cannot also declare an error, `accepts: nothing`, or `replays`. Older readers
refuse the header, and this build refuses `returns` under an earlier header. A model without it
keeps its bytes and compiled digest.

`ess/18`, introduced in [0.41.0][r41]. It collects the new authored
constructs of that release; each is refused under an earlier header with
`unsupported_format_version`, and a model without them keeps its bytes and compiled digest.
`when_subject_state:` may list several held states, `[Delivered, Cancelled]`, and a refusal may
carry it without naming a subject: it reads the subject its siblings name, so a command can accept
a re-send in one state no move starts from and refuse in others, where `wrong_state:` gives them
all one answer (beyond10x/ess#201). `when_state_changes:` still needs the branch's own move. A
`when_subject` predicate may read `state`, the lifecycle state the addressed row holds before
selection, beside its stored fields: `{all: [state == Ready, hold_note != ""]}`
(beyond10x/ess#204). A guarded branch selects before `wrong_state:` applies, and one taking a
move must be able to take it in every state its predicate may select it in. An outcome may carry
`when_related: {via: input.<field>, exists: false}` or `when_related: {via: input.<field>,
predicate: …}`, a guard over the row of another entity whose identity the input carries
(beyond10x/ess#211). A missing row is answered by the `exists: false` branch before any other.

## `ess-diff/` — what moved between two revisions

| Version | Released in | What changed | An older reader |
|---|---|---|---|
| `ess-diff/1` | [0.1.0][r1] | The first delta format. | — |
| `ess-diff/2` | [0.19.0][r19] | Previously omitted constructs, served views and reusable row shapes are accounted for. Sliced provenance moves to the explicit `slice-sha256/2:` digest profile. | Regenerate sliced artifacts; a bare legacy slice digest is refused. |
| `ess-diff/3` | [0.23.0][r23] | Cause, selection-plan and reading-contract deltas. | Refuses the delta. |
| `ess-diff/4` | [0.23.0][r23] | Error and response deltas. | Refuses the delta. |
| `ess-diff/5` | [0.27.0][r27] | `VariantWireNameChanged`, `VariantDisplayNameChanged` and `VariantSummaryChanged` on `TypeChange`. | Refuses a delta carrying any of the three. |
| `ess-diff/6` | [0.29.0][r29] | Typed deltas retain the before/after originating replay relation and complete refusal-observation requirement. | Refuses the new vocabulary; existing changes retain their earlier format. |
| `ess-diff/7` | [0.34.0][r34] | `GroupingChanged` and `FieldAggregateChanged` on `ViewChange`: an aggregate view's group keys and what one field computes. | Refuses a delta carrying either. |
| `ess-diff/8` | [0.34.0][r34] | `AlphabetChanged` on `TypeChange`, related by set membership, and `InputExampleChanged` on `CommandChange`. | Refuses a delta carrying either. |
| `ess-diff/9` | [0.38.0][r38] | `PagingChanged` on `ViewChange`: a view's `paging:` (ess/16) declared, dropped or changed, carrying the parameters, the first page and whether a total is answered on each side. `OutcomeSetEffectChanged` on `CommandChange`: an outcome's `instances:` or `affects:` (ess/16) declared, dropped or changed, one line per construct on each side. | Refuses a delta carrying it. |
| `ess-diff/10` | [0.41.0][r41] | `CauseChanged` on `BindingChange` whose before or after is an `external` cause: an event binding's `ess/18` delivery context (beyond10x/ess#195), carrying the event, the channel (`authority`) and the typed `context_fields`, each with any `wire` name, on each side. `ContextFieldDisplayChanged` and `ContextFieldSummaryChanged` on `BindingChange`: a context field's `display` or `summary` moved (documentation only). A cause change without an `external` side keeps its earlier format. | Refuses a delta carrying it. |
| `ess-diff/11` | Unreleased | `PrefixAdded`, `PrefixRemoved` and `PrefixChanged` on `TypeChange`: a newtype's `prefix:` (beyond10x/ess#219) declared, dropped or replaced, which the residual reported as `unclassified-changed` before. | Refuses a delta carrying it. |

`ess-diff/5` exists because a variant's own name does not move when its wire spelling does. Before
it, the variant set and the variant order both said nothing, and the comparison returned an empty
delta for a change that breaks a deployed consumer.

## `ess-conformance/` — the suite

The family that has moved most, because its step and expectation vocabulary is exactly what the
persisted document's shape is.

| Version | Released in | What changed |
|---|---|---|
| `ess-conformance/1` | [0.1.0][r1] | The canonical scenario IR. |
| `ess-conformance/2` | [0.7.0][r7] | The expectation vocabulary grew. |
| `ess-conformance/3` | [0.16.0][r16] | The step vocabulary grew. |
| `ess-conformance/4` | [0.18.0][r18] | The step vocabulary grew again. Written as 0.17.0 and released in 0.18.0; the CHANGELOG section says so. |
| `ess-conformance/5` | [0.21.0][r21] | Selected generated and authored coverage, omitted scenarios, source identities and every refusal occurrence. |
| `ess-conformance/6` | [0.23.0][r23] | Authored `ess-scenario/2` entity setup, as an ordinary suite. |
| `ess-conformance/7` | [0.23.0][r23] | The same, with declared coverage. |
| `ess-conformance/8` | [0.23.0][r23] | Corrected structured text comparison; response-mapping value comparison. |
| `ess-conformance/9` | [0.23.0][r23] | The same, with declared coverage. |

`ess-conformance/10` and `ess-conformance/11` arrive in [0.28.0][r28]. Version 10 adds
`expect_no_error`, `snapshot_subject`, and `expect_subject_unchanged`; version 11
carries the same steps with coverage. Snapshots capture exactly one actual row by
identity before a command and compare every returned field afterward. Synthesis
requires immediate views covering all subject fields, including generated values.
Missing or duplicate rows fail. Legacy suite formats refuse these steps.

`ess-conformance/12` and `/13`, introduced in [0.29.0][r29], add exact retained-result capture and
comparison, plus an explicit empty-direct-event assertion. Version 12 is ordinary;
13 carries the same declared coverage and exact-parent rules as earlier coverage
formats. Rust and Go execute these steps with report/2; TypeScript/browser readers
refuse these envelopes before target callbacks. Older envelopes refuse the new
steps even if the rest of their document is well shaped.

A write-once snapshot binds the actual original response, command/outcome, subject
identity, input and actor. Retry comparison requires the admitted original values,
no error and no direct events, including unknown event names. Paired subject
snapshots compare the complete original subject independently. Integers compare
exactly, text retains its admitted spelling, collections compare recursively, and
an absent optional field differs from explicit null. Decimal and Binary64 are not
admitted in this response profile. This immediate witness cannot distinguish a
current-head result while it still equals the original; adopters must separately
test later-head and restart retries through their real handlers.

`ess-conformance/14` and `/15`, introduced in [0.34.0][r34], carry a string operator where a suite carries a
predicate: a `satisfies` expectation or an observed selection plan. Version 14 is ordinary and 15
carries declared coverage; each implies every major below it. Rust and Go admit and evaluate them,
and refuse an operand that is not a JSON string. Older envelopes refuse the operators, and the
TypeScript and browser readers refuse these envelopes by their version. A string guard over
command input is decided at synthesis and never reaches the suite, so such a suite keeps its
earlier format.

`ess-conformance/16` and `/17`, introduced in [0.34.0][r34], carry the `<view>/aggregate` scenario: rows created
through the declared creating outcome with values only that scenario uses, and one read asserting
every group's exact aggregates and the absence of every group whose rows the filter refuses.
Version 16 is ordinary and 17 carries declared coverage; each implies every major below it.
Coverage 17 also carries the refusals `ESS-SYNTH-016` (no group key or parameter scopes the
view's rows) and `ESS-SYNTH-017` (the rows cannot be arranged). Rust and Go admit and run them;
older envelopes refuse an aggregate scenario or refusal, and the TypeScript and browser readers
refuse these envelopes by their version.

`ess-conformance/18` and `/19`, introduced in [0.35.0][r35], carry fixture values: a leading `resolve_fixtures`
prelude naming each fixture and its source-owned type declarations, `{kind: fixture}` scenario
values, and `expect_event_values`, which compares the first direct occurrence of an event, chosen by
name, with literal and fixture values, so a later correct occurrence cannot hide an earlier wrong
one. Version 18 is ordinary and 19 carries declared coverage; each implies every major below it.
Rust, Go and TypeScript resolve and validate the values before `BeginScenario` with report/2: a
malformed, incomplete or wrongly typed value stops before any target activity, and a missing
provider is an explicit skip. TypeScript admits these envelopes and still refuses the
retained-result steps, string operators and aggregate scenarios of 12–17 by name. Browser replay
refuses fixtures. A suite without fixtures keeps its earlier format, and older envelopes refuse
the new steps.

`ess-conformance/20` and `ess-conformance/21`, introduced in [0.37.0][r37], They carry the case-insensitive text
operators `equals_ignore_case` and `in_ignore_case` (ASCII folding only) where a suite carries a
predicate: a view's `satisfies` expectation and an observed selection plan. Version 20 is ordinary
and 21 carries declared coverage; each implies every major below it. Rust, Go and TypeScript admit
and evaluate them identically and refuse an operand that is not a JSON string; older envelopes
refuse the operators, and browser replay refuses these envelopes by their version. A
case-insensitive guard over command input is decided at synthesis and never reaches the suite, so a
suite without one in a view expectation keeps its earlier format.

`ess-conformance/22` and `ess-conformance/23`, introduced in [0.37.0][r37], add three steps for the outcome
shapes of `ess/15`: `expect_subject_absent` after a `deletes:` outcome, and `snapshot_view` /
`expect_view_unchanged` around an `accepts: nothing` outcome. Version 22 is ordinary and 23
carries declared coverage; each implies every major below it. The Rust runner evaluates them; Go and
TypeScript refuse these envelopes by their version.

`ess-conformance/24` and `/25`, introduced in [0.37.0][r37], carry a field's presence policy (`ess/15`, beyond10x/ess#139) as `presence: null_when_absent` or `omitted_when_absent` on a payload leaf, and a runner holding the suite fails an implementation that leaves a `null_when_absent` field out or sends an `omitted_when_absent` field as `null`. Version 24 is ordinary and 25 carries declared coverage; each implies every major below it. The Go and TypeScript runtimes execute both from 0.40.0 (beyond10x/ess#188); earlier runtimes refuse them by version. A suite without a policy keeps its earlier format.

`ess-conformance/26` and `/27`, introduced in [0.38.0][r38], carry the vocabulary below for the `ess/16` constructs; `/27` also carries declared coverage. Per-leaf struct values (beyond10x/ess#179): a nested mapping whose struct has an undetermined leaf, such as `rank: {generated: true}`, is asserted leaf by leaf, each determined leaf under its dotted path (`lead.number`) in the event payload or the view row, and the undetermined leaf by the payload shape for presence and type. Presence of an `Optional` aggregate (beyond10x/ess#176): a view `satisfies` predicate reading `defined(x)` or `missing(x)` where `x` is an `Optional` struct, list, map or `Json` in the view's fields, which a 0.37.0 runner would read as absent for a present value; over an `Optional` scalar the predicate selects nothing, and a view filter or `when_subject` guard is decided at synthesis and never reaches the suite. Version 26 is ordinary and 27 carries declared coverage; each implies every major below it. The Rust runner compares them, and from 0.40.0 so do the Go and TypeScript runtimes (beyond10x/ess#188); earlier runtimes refuse both by version. A suite with none of them keeps its earlier format and bytes. The pair also carries the `changed_by` view expectation (beyond10x/ess#148): an ungrouped aggregate view with no parameter is read and snapshotted before its scenario creates rows, and read again after them, holding exactly one row whose every named `count` and `sum` moved by exactly the stated amount. Only a skipping `sum`, listed in `absent_is_zero`, reads as zero where absent; a `count` or a required `sum` absent on either read fails. The pair also carries the `now_offset` scenario value (beyond10x/ess#171): an instant that many seconds from the moment the runner first sends it, which a guard over the current time is witnessed with.

The `execute_command_without_input` step (beyond10x/ess#170) belongs to this pair: it invokes a command with no input document, which is not `execute_command` with `input: {}`, and a suite carrying it takes `/26` or `/27`. A target that cannot send a request without input reports the scenario `unsupported`.

The `page` view expectation (beyond10x/ess#174) belongs to this pair too: after a read of a paged view that sent its page and size parameters, the page holds exactly `rows` rows (at least `rows`, and at most `size`, with `at_least: true`), the answer carries a total of at least `total_at_least` where one is named, and with `follows` the page continues the one the run snapshotted before it — its rows in `follows.order_by`, none ranked before that page's last row, and none carrying that page's values in all of `follows.distinct_by`. Each claim holds on a target other users share. A suite carrying it takes `ess-conformance/26` or `/27`; the Go and TypeScript runtimes execute it from 0.40.0 (beyond10x/ess#188).

A bounded retry (beyond10x/ess#165) belongs to this pair too: `configure_external_outcome` may carry `times` (force the outcome on the next `times` invocations), `expect_invocation` may carry `count` (exactly that many matching invocations), and a binding scenario may be filed under the `final-failure` aspect. A suite carrying any of them takes `/26` or `/27`. A target that cannot force an outcome more than once reports the scenario `unsupported`.

`ess-conformance/28` and `ess-conformance/29`, introduced in [0.39.0][r39], add
`expect_direct_response`, which checks the immediately preceding invocation's actual return
against its complete typed response schema and any authored literals. Version 28 is ordinary;
29 carries declared coverage and exact-parent lineage. The Rust runner requires report/2.
Go and TypeScript generation refuse the observation, and older readers refuse these envelopes
before target callbacks. Released suites 26 and 27 retain their meaning and bytes.
Direct responses preserve exact integers, nested presence policies, collection order and
duplicate multiplicity; Binary64 remains outside the admitted profile. Responses are bounded
to 1 MiB, depth 128 and 65,536 members per collection, without truncation.

`ess-conformance/30` and `ess-conformance/31`, introduced in [0.41.0][r41], carry the `ess/18` delivery context
(beyond10x/ess#195): `deliver_event` delivers one occurrence of an event from its external
channel with the context that channel binds, and `expect_every_invocation` requires every
invocation for one occurrence to carry what it was delivered with. Version 30 is ordinary; 31
carries declared coverage. The Rust runner executes both; Go and TypeScript generation refuse
them, and older readers refuse these envelopes before target callbacks. A suite without them
keeps its earlier format.

For `ess/7`, generated held-state refusals include ordinary `wrong_state` outcomes:
they compare the complete subject before and after the call and refuse every
direct event, including undeclared names. Incomplete subject views cause a named
synthesis refusal. Earlier source formats retain their existing witness bytes.

Integer exactness starts at the actual target adapter: it must preserve each
handler value without a floating-point conversion before supplying the typed
response. Native adapters can construct numbers directly from integers. A JSON
adapter must decode declared Integer values losslessly or report the observation
Unsupported; generic JSON-to-Node conversion does not provide this guarantee.
Typed response parity does not claim equivalent parsing of arbitrary JSON number
spellings. Existing numeric serialization remains unchanged.

Versions `6` through `9` all arrived in 0.23.0. They are two capabilities crossed with the
ordinary/coverage distinction, not four separate releases.

Two readers fail differently at the same document, which is why the number exists at all. An old
Rust reader parses a closed tagged enum and fails with `unknown variant` — a message that blames
the document. An old Go runner abandons a scenario whose first word it does not know and reports it
skipped, which is the right answer reached by accident. Neither is a verdict anyone should act on.

## `ess-scenario/` — authored conformance scenarios

`ess-scenario/4`, introduced in [0.39.0][r39], lets an act declare `response: {field: literal}` to assert
selected fields of its command's return. Each literal must match its complete declared type;
unknown fields and invalid nested presence are refused before execution. An empty mapping
requests only the complete response shape check. Every selected `returns: true` outcome gets
that shape check even without authored literals. No event, stored subject or view is invented.
Earlier scenario readers refuse the header; earlier versions refuse the new key.

## `ess-normalization/` — normalization recipes

| Version | Released in | What changed |
|---|---|---|
| `ess-normalization/1` | [0.19.0][r19] | Strict recipes, explicit external dispatch, ordered input/output schema boundaries, separate missing/null behavior. |
| `ess-normalization/2` | [0.20.0][r20] | Ordered string concatenation and joining, exact integer rendering, list concatenation, original collection indices, filtered mapping, first-match selection. |
| `ess-normalization/3` | [0.20.0][r20] | Model-owned stage roots, pinned to complete compiler provenance and explicit type selections. |
| `ess-normalization/4` | [0.20.0][r20] | Selected JSON field, array-item or root tokens captured as canonical standard base64 before first-stage validation. |
| `ess-normalization/5` | [0.20.0][r20] | Explicit input paths; token-preserving `binary64_literal` constants and finite `binary64` conversion steps. |
| `ess-normalization/6` | [0.20.0][r20] | Fixed string array preparation and checked `position` reads. |

Generated maps for the earlier formats stay byte-identical at the same generator version.

## Envelopes revised once

| Format | Revised in | What the later version carries |
|---|---|---|
| `ess-conformance-report/` | [0.19.0][r19] | `/2` separates passed, failed, error, unsupported and skipped counts, bound to the exact executed suite bytes. |
| `ess-schema-bundle/` | [0.19.0][r19] | `/2` is a document-root import with explicit, replay-checked root identity. Component-bundle `/1` bytes are unchanged. |
| `ess-impact/` | [0.19.0][r19] | `/3` versions the corrected dependency vocabulary and the embedded delta. |
| `ess-conformance-run/` | [0.20.0][r20] | `/2` is the checked detailed run output. |
| `ess-target-failure/` | [0.20.0][r20], [0.23.0][r23] | `/2`, then `/3` with the `accessor-resource` cause. |
| `ess-scenario/` | [0.23.0][r23], [0.35.0][r35], [0.39.0][r39] | `/2` authored setup establishes typed, isolated backend entity rows. `/3`, added in [0.35.0][r35], adds typed `fixtures:` and `{$fixture: name}` references resolved before the scenario starts. `/4`, added in [0.39.0][r39], adds literal `response:` assertions on an act. |
| `infra-observation/` | [0.1.0][r1], [0.33.0][r33] | `/2` is a reduced, deliberately partial recovery profile, not a superset of `/1`. `/3` is the full scan with each Secret value recorded as `{"present": true}`: the key name, no digest, no length. `/1` wrote each value's unsalted SHA-256 and byte length, which confirm a guessed low-entropy secret to anyone holding the file. Same fields, new meaning, so a `/1` reader must reject `/3`; this build still reads `/1` and discards its digests. |
| `infra-ir/` | [0.33.0][r33] | `/3` records each Secret key as present and nothing derived from its value, and is what every full observation with a Secret key compiles to, `/1` included. An IR without a Secret key keeps `/1` and its bytes. A persisted `/1` still reads, returned as `/3` with its Secret digests dropped and a different model digest, so nothing derived from it chains to the `/1` file's own digest; so drift reports a Secret's added and removed keys and never a changed value. An older reader refuses `/3`. |
| `infra-drift/` | [0.33.0][r33] | `/2` is the namespace topology profile. `/3` is the full-scan comparison with one meaning changed: a Secret's `changed_keys` is always empty, so an empty list means the value is unknown, where under `/1` it meant not rotated. Serialize-only; `/1` documents already written keep their meaning. |
| `ess-observed-bindings-report/` | [0.32.0][r32], [0.33.0][r33] | `/2` adds `OBS-BIND-008`: a container or native sidecar in a bound workload that no binding names is a violation. Same fields, new semantics; a document satisfied under `/1` can be violated under `/2`, so a `/1` reader must reject `/2`. The authored `ess-observed-bindings/1` input keeps its version and fields; it now claims the bound workload runs nothing else. `/3` adds each binding's `acknowledged` list, so a satisfied `OBS-BIND-008` no longer means every entry is bound; a `/2` reader must reject `/3`. |
| `ess-observed-bindings/` | [0.33.0][r33] | `/2` adds optional `foreign_containers`: per bound workload, containers this realization does not build, each with a `name` and a nonempty `reason`. `OBS-BIND-008` accounts for them without a binding; one running a declared image or artifact locator, or its `@sha256:` digest under another name, violates it, one naming no observed container or native sidecar leaves it unknown (plain init containers are not recorded), and one naming a bound container is refused. `/1` is read unchanged, acknowledges nothing and keeps its binding digest; a `/1` document carrying the key, even empty, is refused. An older reader refuses `/2`. The report moves to `ess-observed-bindings-report/3`, which adds each binding's `acknowledged` list; a satisfied `OBS-BIND-008` there no longer means every entry is bound, so a `/2` reader must reject `/3`. |
| `ess-composition/` | [0.38.0][r38], [0.40.0][r40] | `/2` lets a reference name any type the selected component's owned domains declare, and adds `conformances`: an assertion that a consumer's local type has an imported component type's shape, checked field by field, where the consumer may treat a required value as optional and nothing else may differ (`type_conformance_drift`). The client plan is unchanged. The earlier format keeps its meaning and bytes and refuses the key, even empty; an older reader refuses `/2`. `/3`, added in [0.40.0][r40], adds `reader: true` on a `conformances` entry. |

`ess-composition/3`, introduced in [0.40.0][r40]. A `conformances` entry may carry `reader: true`: the consumer's
type only reads the imported one, so it may also read a newtype as what it wraps, an enum as
`String`, enum variants by wire name, a struct or `String`-keyed map as `Map<String, Json>`, and a
subset of the fields, with any extra field compared by wire name against every imported field.
`Json` itself read as a map, and anything else that could reject an imported value, stays
`type_conformance_drift`. `reader: true` also asserts that the consumer's reader ignores keys it
does not declare; ESS-generated closed types do not, so a consumer reading through them must not
use `reader` for a field subset. An entry without the key is compared as before. Earlier formats
keep their meaning and bytes and refuse the key whatever its value, `null` included; an older
reader refuses `/3`.

## `ess-history/` — recorded concurrent histories

`ess-history/1`, introduced in [0.39.0][r39], is one run of several clients against a target:
each call's client, command, subject, invoke and return instants, `Returned` or `Indeterminate`
completion and outcome, the rows a view read answered, and the `retry_of` of a retried request.
`ess verify conform check-history` reads it; the Go and TypeScript concurrent explorers write it.
The document is specified in `models/concurrent-history/` and published as
`schemas/ess-history.schema.json`.

`ess-history-adapter/1`, introduced in [0.39.0][r39], maps each field of a JSON Lines call log to a
JSON pointer or declares it `absent`, for `ess verify conform import-history`.

## `ess-mutation-report/` and `ess-mutation-manifest/` — the mutation audit

`ess-mutation-report/1`, introduced in [0.34.0][r34], is what `ess verify conform mutate` writes.
`ess-mutation-manifest/1`, introduced in [0.37.0][r37], is what `--emit` writes and `--collect` reads.

`ess-mutation-report/2` and `ess-mutation-manifest/2` were introduced in [0.41.0][r41].
The report adds the `unwitnessed` verdict (`ESS-MUTATE-004`) with each mutant's
`added_refusals`, each mutant's `excluded` scenarios, and the baseline's `not_scored` list; a
baseline scenario reported `unsupported` or `skipped` is no longer red, and a mutant nothing
killed with an excluded scenario it changed is `inconclusive`. The manifest adds each
suite's `refused` list. `--collect` still reads a `/1` manifest and judges gained refusals by count.

## Still at version 1

Never revised, and a document that claims a higher number is refused:
`ess-realization/1`, `ess-realization-ir/1`, `ess-client-plan/1`,
`ess-docs/1`, `ess-build/1`, `ess-build-ir/1`, `ess-component/1`, `ess-component-ir/1`,
`ess-release/1`, `ess-release-bundle/1`, `ess-release-catalog/1`, `ess-runtime/1`,
`ess-runtime-ir/1`, `ess-stack/1`, `ess-stack-lock/1`, `ess-environment/1`, `ess-deployment/1`,
`ess-service-interface/1`, `ess-openapi-import/1`, `ess-browser-catalog/1`,
`ess-conformance-input/1`, `ess-conformance-replay/1`, `infra-spec/1`,
`infra-graph/1`, `infra-simulation/1`, `infra-projection/1`.

## Release numbers this page cites

Five versions appear in the [CHANGELOG][changelog] with no GitHub Release behind them.

| Version | Why | Where its code shipped |
|---|---|---|
| 0.3.0 | tagged in one clone, never pushed | 0.4.0 |
| 0.5.0 | tagged in one clone, never pushed | 0.5.1 |
| 0.15.0 | tagged in one clone, never pushed | 0.16.0 |
| 0.21.0 | tag pushed; the tree does not pass the current gate, so no release can be cut from it | the tag itself |
| 0.26.1 | never tagged | 0.27.0 |

`ess-conformance/5` is the one format version introduced by any of them, and the 0.21.0 tag
carries it. Everything else in the tables above sits on a version with a published release.

The tag `v0.3.0` is a fifth artifact of the same period, under the naming convention that
preceded bare versions. The release workflow triggers on bare versions only, so it never asked
for a release and promises none.

[changelog]: https://github.com/beyond10x/ess/blob/main/CHANGELOG.md
[r1]: https://github.com/beyond10x/ess/releases/tag/0.1.0
[r7]: https://github.com/beyond10x/ess/releases/tag/0.7.0
[r16]: https://github.com/beyond10x/ess/releases/tag/0.16.0
[r18]: https://github.com/beyond10x/ess/releases/tag/0.18.0
[r19]: https://github.com/beyond10x/ess/releases/tag/0.19.0
[r20]: https://github.com/beyond10x/ess/releases/tag/0.20.0
[r21]: https://github.com/beyond10x/ess/releases/tag/0.21.0
[r23]: https://github.com/beyond10x/ess/releases/tag/0.23.0
[r27]: https://github.com/beyond10x/ess/releases/tag/0.27.0

[r28]: https://github.com/beyond10x/ess/releases/tag/0.28.0

[r29]: https://github.com/beyond10x/ess/releases/tag/0.29.0

[r32]: https://github.com/beyond10x/ess/releases/tag/0.32.0

[r33]: https://github.com/beyond10x/ess/releases/tag/0.33.0

[r34]: https://github.com/beyond10x/ess/releases/tag/0.34.0

[r35]: https://github.com/beyond10x/ess/releases/tag/0.35.0

[r36]: https://github.com/beyond10x/ess/releases/tag/0.36.0

[r37]: https://github.com/beyond10x/ess/releases/tag/0.37.0

[r38]: https://github.com/beyond10x/ess/releases/tag/0.38.0

[r39]: https://github.com/beyond10x/ess/releases/tag/0.39.0
[r40]: https://github.com/beyond10x/ess/releases/tag/0.40.0
[r41]: https://github.com/beyond10x/ess/releases/tag/0.41.0
