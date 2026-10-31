---
format: aep.planning-md/3
id: story:a-guard-on-a-related-identity-input-is-decided-on-the-row-sent
kind: story
status: active
title: 'A when: guard on a related row''s via input is decided on the row sent'
summary: Synthesis decides an input guard on the identity of the captured related row it sends, steers that row to meet it, or refuses (ess#693)
tags:
- ess-issues-3
refs:
- provider: github
  reference: beyond10x/ess#693
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- informed_by: approval-record:d214-batch-3-consumer-blocked-first
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-10-29T17:06:40Z", actor: "human:timo", revision: 8, executor: "agent:ess-issues-lead", correlation: "ess-issues-3"}
- {from: "proposed", to: "active", at: "2026-10-29T17:06:41Z", actor: "human:timo", revision: 9, executor: "agent:ess-issues-lead", correlation: "ess-issues-3"}
---
## Problem

beyond10x/ess#693. A command guards an input with `when:` and also names that input as the `via:`
of a `when_related:` guard. Synthesis decides the `when:` guard on a candidate literal for the
input, then sends the input as the captured related row. That row's identity is a different value,
so the scenario's send contradicts the guard it witnesses.

Minimal model (neutral names; `demo.slots.Kind` is an enum `[a, b, c]`, `Slot` has identity
`kind` and a Boolean `enabled`, `OpenSlot(kind, enabled)` creates one):

```yaml
  - name: demo.slots.UseSlot
    input:
      - {name: kind, type: demo.slots.Kind}
    outcomes:
      - name: refused-b
        when: kind == b
        error: demo.slots.KindRefused
        payload:
          demo.slots.KindRefused: {kind: input.kind}
      - name: missing
        when_related: {via: input.kind, exists: false}
        error: demo.slots.SlotMissing
        payload:
          demo.slots.SlotMissing: {kind: input.kind}
      - name: disabled
        when_related:
          via: input.kind
          predicate: enabled == false
        error: demo.slots.SlotDisabled
        payload:
          demo.slots.SlotDisabled: {kind: input.kind}
      - name: used
        emits: [demo.slots.SlotUsed]
        payload:
          demo.slots.SlotUsed: {kind: input.kind}
```

Checked on `ess 0.77.0` (a repro outside the repository):

```text
$ ess verify conform run --target interpreted --path . --suite suite.json --report-format 2
demo v1 against interpreted 0.77.0 — failed
  passed demo.slots.OpenSlot/outcome/opened
  passed demo.slots.UseSlot/outcome/disabled
  failed demo.slots.UseSlot/outcome/refused-b
  passed demo.slots.UseSlot/outcome/used
  4 scenarios: 3 passed, 1 failed, 0 error, 0 unsupported
```

`refused-b` opens slots `c`, `a` and `b`, then sends `UseSlot` with `kind` bound to the captured
`a` row; the target answers `disabled`. The issue's probes: with `kind == a`, `disabled` and `used`
fail instead; with `kind == c` or `{kind: [b]}`, `refused-b` fails. The same holds on 0.77.0 for a
newtype-of-String identity with `when: 'kind == "x"'`, and for a guard on the `via:` of a second
related row (`PairSlots(kind, other)` with `when: 'other == "x"'`): there `refused-x` fails, and
so does the `missing` scenario's extra send for that guard.

The issue's Expected: the `refused-b` scenario sends the row whose identity satisfies `kind == b`,
or synthesis refuses the outcome. It never writes a scenario whose act contradicts the guard it
witnesses.

## Fit review

Checked 2026-10-29 on the `ess 0.77.0` release binary and on this tree. `synthesize/related_guard.rs`
is unchanged since 0.74.0 (`git diff --stat 0.74.0 HEAD`), so the 2026-10-28 scoping's line numbers
in it still hold.

1. **Need.** A command refuses or accepts a request by the value of an input, and the same input
   names a related row. Synthesis writes a scenario whose send contradicts that guard. On 0.77.0,
   `refused-b` sends the captured row of kind `a`, and the interpreted target answers `disabled`
   ("failed demo.slots.UseSlot/outcome/refused-b … 4 scenarios: 3 passed, 1 failed"). This
   reproduces with a newtype-of-String identity (`when: 'kind == "x"'`: refused-b sends the row
   created as `kind-138600`). It also reproduces on a second related row's `via:`
   (`PairSlots(kind, other)`, `when: 'other == "x"'`): `refused-x` fails, and the `missing`
   scenario's extra send for that guard expects `refused-x` and observes `missing`. All three are
   minimal neutral models from a repro outside the repository. The requester proposed no syntax.
   Their Expected is: send the row whose identity satisfies the guard, or refuse the outcome.
2. **Class: defect.** Synthesis writes an act that contradicts the guard it witnesses. Under the
   precedence rule an input-guarded refusal answers at `Phase::InputRefusal`
   (`crates/specify/ess-domain/src/command/precedence.rs:512`), so the target is right to answer
   on the value actually sent.
3. **Already expressible.** Not applicable to a defect.
4. **Fit.** No authored surface. The defect chain, read in the tree:
   - `run_as` routes every branch of a command with a related guard, `refused-b` included, through
     `related_guard::prepare_at_in` (`synthesize.rs:4566`, `:4651-4660`; `routes` at
     `synthesize/related_guard.rs:178`).
   - `arranged_at` (`related_guard.rs:3044`) calls `with_row` → `with_row_where` (`:4118`). That
     function collects every input guard (`:4137`) and accepts a row once
     `takes` → `chooses` (`:4209`, `:396`) → `selects` (`:1456`) decides the guard (`input_guard`,
     `:1403`) on the candidate input's own `kind`, whatever identity the row was created under.
   - `arranged_at` then binds `kind` to the row (`:3164`), and `supply` sends the binding over the
     candidate value (`synthesize.rs:7482`).
   - The only value correction, `fixed` (`:3128`, from `subject_identity` `:3185`, #521), covers
     the branch's own subject and reads `arranged_identity` (`synthesize.rs:12959`). That function
     keeps struct identities only (`.filter(Node::Map)`), so a scalar via identity is never
     grounded.
   - On a command reading several related rows, the rows beside are arranged first (`beside`,
     `:1104`) and bound by `place_beside` (`:3261`). `Around::admits` (`:1350`) and the extra
     sends of the `exists: false` scenario (`missing_sends` `:3908`, `send_each_without_row`
     `:3970`) decide input guards on candidate values, not on those rows' identities.

   Other targets are untouched: the interpreter is right, and generated code, `ess verify diff`
   and the entity runtime read no synthesized arrangement.
5. **Second adopter.** Quotas keyed by a `region` enum: `ChargeQuota(region)` refuses `region == eu`
   for policy, above `when_related: {via: input.region, predicate: exhausted == true}`.
6. **Cost.** No format, keyword or diagnostic code. The refusal reuses ESS-SYNTH-003
   `GuardUnsatisfiable`. Suite bytes change only where an input guard reads a related `via:`
   field. A text scan of the repository's YAML found none: 75 files carry `via: input.`, 5 commands
   carry both a `via:` and a `when:`, and in none of them does the `when:` read the via field. So
   neither bytes table should move (inferred from a scan of the text, not from a build).
7. **Considered.**
   - (a) Change nothing.
   - (b) Take input-guarded refusals out of `related_guard::routes` and witness them with a literal
     through `reach`: that fixes `refused-b` only. `disabled` and `used` still send `a` under the
     guard `kind == a` (the issue's probe).
   - (c) Chosen: decide every input guard on the identity of the row actually sent, steer the
     row's creator toward an identity the guards need, and refuse where no arrangeable row meets
     them. This is the issue's Expected, as proposed.

Out of scope, seen in the reproduction: with the three-variant enum, `UseSlot/missing` is refused
ESS-SYNTH-001 ("too few values to name an identity no other scenario sends",
`synthesize/existence.rs:163`). That refusal is honest and is not changed here.

## Decisions

**Accept as proposed.** The scenario sends a row whose identity meets the guard it witnesses, or
synthesis refuses the outcome. It never writes a contradicting act.

- **Decide on the row sent.** In `with_row_where` (`related_guard.rs:4118`), `takes` (`:4209`) sets
  the via field to the identity the candidate row was created under, then calls `chooses`. This
  happens only where some input guard of the command reads that field. The identity is read from
  the row's own creating step through `super::determined_payload` (`synthesize.rs:8704`), for
  scalar and struct identities alike, by a reader in `related_guard.rs`. The shared
  `arranged_identity` (`synthesize.rs:12959`) is left unchanged. The input returned for the row is
  the one decided this way.
- **Steer the row.** Beside `grounded_on_input` in the hints (`:4243`), add predicates that carry
  each input guard over the via field onto the related entity's identity (`kind == b` becomes a
  row hint `kind == b`). `search_under` then tries a row of kind `b` for `refused-b`, and rows of
  another kind for `disabled` and `used`. Whether the hints reach the creator's identity input is
  inferred. If they do not, the implementor steers that input directly through the `chosen`/`fixed`
  path, and the acceptance below is unchanged.
- **Rows beside.** On a command reading several related rows, each beside row's via field is
  likewise decided on that row's identity in `takes`/`Around::admits` (`:1350`) and in the extra
  sends of the `exists: false` scenario (`missing_sends` `:3908`, `send_each_without_row`
  `:3970`). An extra send whose guard the bound row refutes is not written. Steering a beside row
  toward a guard is not part of this story: where the beside row refutes the branch's own guard,
  the branch is refused.
- **Refusal.** Where no arrangeable row meets the guards, the existing search failure stands, which
  is ESS-SYNTH-003 `GuardUnsatisfiable`, the refusal the issue reports for a two-value enum.
- **D135 / D214.** No wire bytes, generated API, format or documented contract changes. Bytes
  change only for a command whose input guard reads a related via field, and no repository model
  has one (Fit review Q6). Such a scenario failed on a conforming target before the change, so
  this is not breaking. A refusal that replaces a contradicting scenario is a synthesis refusal,
  not a refusal of input: `ess specify validate` still accepts the model, and `synthesize` exits 0.
  If the batch's adopter replay finds an adopter suite that gains one, that case ships as a
  synthesis `note:` that keeps the 0.77.0 scenario, per D214, and
  `no_row_meeting_the_guard_refuses_by_name` is changed to pin the note.

## Acceptance

New test file `crates/verify/ess-conformance/tests/related_guard_on_the_identity_input.rs`. Its
models are the neutral shapes in `## Problem`, as fixtures under `tests/fixtures/` or inline.

- `refused_b_is_sent_the_row_of_kind_b`: for `UseSlot` with `when: kind == b`, the synthesized
  `refused-b` sends `kind` as a captured row created with `kind: b`.
  `cargo test -p ess-conformance --locked --test related_guard_on_the_identity_input refused_b_is_sent_the_row_of_kind_b`
- `siblings_are_sent_a_row_the_guard_refutes`: `disabled` and `used` each send a captured row
  whose creating `kind` is not `b`.
  `cargo test -p ess-conformance --locked --test related_guard_on_the_identity_input siblings_are_sent_a_row_the_guard_refutes`
- `interpreted_target_passes_every_guard_probe`: for each guard `kind == b`, `kind == a`,
  `kind == c` and `{kind: [b]}`, every `UseSlot` scenario written passes on the interpreted target,
  and no guarded branch is left without a scenario or a refusal.
  `cargo test -p ess-conformance --locked --test related_guard_on_the_identity_input interpreted_target_passes_every_guard_probe`
- `a_string_identity_guard_is_decided_on_the_value_sent`: with `Kind` a newtype of String and
  `when: 'kind == "x"'`, `refused-b` sends a row created with `kind: "x"`, and every scenario
  passes on the interpreted target.
  `cargo test -p ess-conformance --locked --test related_guard_on_the_identity_input a_string_identity_guard_is_decided_on_the_value_sent`
- `no_row_meeting_the_guard_refuses_by_name`: with a two-variant `Kind` and guards that leave no
  arrangeable row for a branch (the issue's probe), that branch has no scenario, and its refusal
  is ESS-SYNTH-003 naming the guard.
  `cargo test -p ess-conformance --locked --test related_guard_on_the_identity_input no_row_meeting_the_guard_refuses_by_name`
- `a_guard_on_a_beside_via_is_never_contradicted`: for `PairSlots(kind, other)` with
  `when: 'other == "x"'`, every scenario written passes on the interpreted target, `missing`
  included, where 0.77.0 fails `refused-x` and `missing`. `refused-x` is either written that way or
  refused with ESS-SYNTH-003 naming `other == "x"`.
  `cargo test -p ess-conformance --locked --test related_guard_on_the_identity_input a_guard_on_a_beside_via_is_never_contradicted`
- Bytes tables: `cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table`. Every
  existing row of both TSVs is unchanged, and a new fixture carrying `format: ess/` only adds its
  own rows.
- Kept green, one `--test` at a time: `cargo test -p ess-conformance --locked --test interpreted_related_guards`,
  `--test related_guard`, `--test related_guard_captured_struct` (#521),
  `--test related_guard_multiple_vias`, `--test related_guard_pass1_fixes`,
  `--test related_guard_pass2_fixes`, `--test adversary_related_guard_pass1`,
  `--test adversary_related_guard_pass2`, `--test aggregate_related_guard`; then
  `cargo test -p ess-cli --locked` and
  `cargo clippy -p ess-conformance -p ess-cli --locked --all-targets -- -D warnings`.

## Scope

Derived 2026-10-29 from this tree. Each line is **cited** (read in the tree) or **inferred**.

- **Primary surface:** `crates/verify/ess-conformance/src/synthesize/related_guard.rs`,
  `with_row_where` (`:4118-4303`: `takes` `:4209`, hints `:4243`). Cited.
- **Symbols:** `with_row_where`, `chooses` (`:396`), `selects` (`:1456`, unchanged: it decides on
  the input it is given), `input_guard` (`:1403`), `arranged_at` (`:3044`, the bind at `:3164`),
  `Around::admits` (`:1350`), `missing_sends` (`:3908`), `send_each_without_row` (`:3970`), and a
  new scalar identity reader in `related_guard.rs` over `super::determined_payload`
  (`synthesize.rs:8704`). Cited.
- **Not touched:** `arranged_identity` (`synthesize.rs:12959`) and `subject_identity` (`:3185`)
  keep their struct-only reading (#521). `subject_fact.rs` is touched only if the identity hint
  has to live beside `grounded_on_input` (`subject_fact.rs:1135`). Inferred, and avoided if
  possible.
- **New test:** `crates/verify/ess-conformance/tests/related_guard_on_the_identity_input.rs` plus
  its fixtures. Inferred.
- **Size:** M. The single-row change is small. Steering the creator's identity, and the
  several-row sites, carry the uncertainty: whether `subject_fact::search_under` reaches a chosen
  enum variant through hints is not read.
- **Collisions in batch 3, by function:**
  - U11 itself: the #645 story edits `synthesize.rs` `arranged` only, so they share no function.
  - U14 (`story:a-cascade-onto-rows-whose-creator-needs-the-subject-active-is-witnessed`): the
    creator drives in `related_guard.rs` (`drive` `:1810`, `drive_toward` `:1937`, `drive_sharing`
    `:2098`) call `with_row` → `with_row_where` (`:1699`, `:1750`, `:2159`). A cascade creator whose
    input guard reads its own via field would run the changed `takes`. They share a function, not
    lines. Merge after U14, or rebase and rerun U14's tests. Inferred: U14 has no scope recorded
    yet. U14's likely edits (`present_in` `:440`, `set_effects.rs` `affects_segment`) are other
    functions.
  - U13 (`story:a-guard-on-the-input-addressed-parent-row-holds-on-every-command-of-its-children`):
    its creator case (a predicate guard on the input-addressed parent row) is arranged by the same
    `drive`/`with_row_where` path, and its reader case by `row_set.rs`. Same shared-function risk
    as U14. Inferred.
  - U15: authored scenario reading, no shared file.
- **Bytes tables:** existing rows do not move (Fit review Q6, a text scan, level 3, unproven until
  the table test runs).
