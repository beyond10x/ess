---
format: aep.planning-md/3
id: story:an-older-command-input-is-read-on-replay-through-a-declared-upcast
kind: story
status: archived
title: Replay reads a recorded command input in an older shape through a declared upcast
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 5
transitions:
- {from: "draft", to: "archived", at: "2026-10-08T03:51:50Z", actor: "human:timo", revision: 5}
---
## Need

An adopter changes a command input field's type: a text field becomes a record
(`receipt: String` → `receipt: {revision, evidence}`). Recorded invocations made with the old
shape must still replay. On ess 0.56.0 replay of such history fails with
`body.receipt: expected an object, found a string`, and the specification has no declared way to
say how an older input is read: neither an input version nor an upcast from the old shape to the
new one. The adopter's own conversion design validates but does not generate (2 unmet
capabilities); without the conversion it generates (132 capabilities, synthesis 181 scenarios,
0 refusals) and replay still fails. The adopter keeps a hand-written host guard until ESS can say
this.

## Reproduction

Not yet minimised. A reproducer with neutral nouns goes into `.engineering/repro/` with the fit
review: one command whose input field changes from String to a record between two specification
revisions, and one recorded invocation in the old shape.

## Next

Assess with `.agents/skills/assessing-external-requests/SKILL.md` before any build. Questions for
the fit review: whether an older input is a new authored construct (and so a source format
`ess/N`), whether `ess verify diff` already classifies the change and can name the reading
obligation, and which surfaces it touches (replay in `ess-conformance`, generated readers in
`ess-gen`/`ess-synth`). Where it lands is not established; whether it needs `synthesize/` decides
whether it may go before the other session's synthesis waves end.

## Fit review

Assessed with `.agents/skills/assessing-external-requests/SKILL.md` on 2026-10-08 against ess 0.56.0.
Reproducer: `.engineering/repro/upcast/` (README has every command and its output).

1. **Need, apart from the syntax.** The domain fact: "an invocation recorded before a command's
   input changed shape must still be accepted, and the specification says what it means." The
   reproducer has `catalog.items.Accept` with `receipt: String` in v1 (`before.yaml`) and
   `receipt: catalog.items.Receipt {revision: Integer, evidence: String}` in v2 (`after.yaml`),
   plus one invocation recorded against v1 (`scenarios/recorded-v1.scenario.yaml`).
   - Against v1: `ess verify conform run --target interpreted --path before.yaml --scenarios
     scenarios/recorded-v1.scenario.yaml --report-format 2` passes, 5 of 5.
   - Against v2 it is refused before it runs: `refusal[ESS-AUTHOR-015] … the input of
     catalog.items.Accept: receipt: expected catalog.items.Receipt as a mapping, found a string`,
     exit 1 (`crates/verify/ess-conformance/src/authored.rs:3582`).
   - The adopter's replay is not an `ess` verb. It is the adopter's own test, posting recorded
     bodies to the server `ess generate synthesize --layout crate` generates. There the message is
     `body.receipt: expected an object, found a string`, answered 400. Path: route decode
     `crates/generate/ess-synth/src/rust/http.rs:1109`, generated `decode_command_*`
     (`rust/wire.rs:439`), `json::member_at` (`rust/json.rs:775-776`), message
     `rust/json.rs:505`. Go has the same shape (`go/http.rs:1990`). The recorded format is the
     adopter's own JSON log of request bodies. ESS has no format for recorded command inputs:
     `ess-history/1` and `/2` record no inputs (`ess verify conform check-history --help`).
   - *Requester's proposed syntax (theirs):* a declared "upcast" from the old input shape to the
     new one, or an input version. Their own attempt, a `conversions:` entry from the new struct to
     `String`, validates and leaves 2 obligations. Reproduced: `15 capabilities: 13 generated, 2
     obligation(s)`, the command behaviour and the conversion algorithm (README, "The conversion
     design").
2. **Class: convenience.** It is not a defect: refusing a body that does not match the declared
   input is documented. "absence of a required field is a refusal, never a default"
   (`rust/json.rs:773-774`); `ESS-AUTHOR-015` "write a value of the type the model declares
   there". It is not a gap either, because the fact can be stated today (3). What the idiom gives
   up is grouping the two receipt facts in one struct.
3. **Can it already be expressed? Yes: an additive `Optional` input.** Keep the field that recorded
   invocations carry at its recorded type, and add the new fact as `Optional`. A guard reads it
   only when it is present (optional input narrowing, ess/16, `docs/design/optional-input-narrowing.md`).
   `after-additive.yaml` keeps `receipt: String`, adds `receipt_revision: Optional<Integer>`, and
   guards `stale-revision` with `when: defined(receipt_revision)` plus
   `when_subject: {predicate: {all: [state == Open, revision != input.receipt_revision]}}`.
   - `ess specify validate --path after-additive.yaml`: `catalog v2 — 1 file(s), valid`.
   - `ess verify conform run --target interpreted --path after-additive.yaml --scenarios scenarios
     --report-format 2`: `4 scenarios: 4 passed`. That includes the v1-recorded invocation
     unchanged and a new one refused as `stale-revision`.
   - `ess generate synthesize --path after-additive.yaml --layout crate`:
     `13 capabilities: 13 generated, 0 obligation(s), 0 refused`. The decoder reads `receipt` as
     text and `receipt_revision` as `None` when the member is absent.
   - Also expressible today: a second command beside the first (`Accept` keeps the text receipt,
     a new command takes the struct). Each recorded invocation replays against the command it was
     recorded for.
   - Checked and not applicable: `conversions:` (`crates/specify/ess-domain/src/types.rs:2236`) is
     a crossing between two values the model reads. It never touches the wire decoder, and its
     algorithm is an obligation. Unions are tagged on the wire (`{"kind": …}`), so a bare string is
     not one of their variants. `fixture_inputs:` (ess/13) names fixture values and is not a
     reader.
4. **Does a declared upcast fit what is there?** Not as requested.
   - *Semantics.* The old record has no `revision`. An upcast `String -> Receipt` has to invent
     one: a constant, the subject's stored revision (which makes the stale check always pass), or a
     parse of the text. A constant or a subject read is a guess, and AGENTS.md says "Imports never
     guess". A parse is an algorithm, which ESS states as an obligation; that is the 2 unmet
     capabilities the adopter already met. The honest model is that an old receipt names no
     revision. The `Optional` input says exactly that.
   - *Vocabulary.* "Upcast" and "input version" appear nowhere in the authored language, the
     design pages or the crates (grep of `docs/`, `website/docs/`, `crates/*/*/src`). The nearest
     word is `conversions:`, and it means something else (above).
   - *Generality.* The same question arises for view parameters, event payloads read by bindings,
     and stored entity fields. A command-input-only construct would leave those siblings unable to
     say it, which is a red flag.
   - *Targets an upcast would have to handle or refuse:* domain input validation
     (`ess-domain/src/command.rs:2522`), IR (`ess-compiler/src/ir.rs:1693-1699`), Rust, Go and web
     decoders (`ess-synth/src/rust/wire.rs:439`, `go/http.rs`, `web/bridge.rs:337`), input
     schema/OpenAPI projection (`oneOf` of two shapes, `ess-gen`/`ess-openapi`), authored scenario
     input checking (`ess-conformance/src/authored.rs:3582`), synthesis of an obliged "old shape is
     read as …" scenario (a new family under `crates/verify/ess-conformance/src/synthesize/` plus
     its registration in `synthesize.rs`), `ess verify diff` (a new change kind), generated docs,
     and Entity Runtime, whose handling I don't know.
   - *A related finding, not the request.* `ess verify diff --compatibility` answers the
     `String -> Receipt` input change as `unknown for callers, readers` and `"history":
     "compatible"`, and `--fail-on breaking` passes it. Commands default to `[U, U, C]`
     (`crates/verify/ess-diff/src/compatibility.rs:379`). History is defined as "stored entity
     state and emitted events" (`compatibility.rs:13`), so recorded command inputs are outside
     every dimension. A scalar replaced by a struct cannot read either way, the same reason a union
     variant gaining a required payload is already decided as breaking for every use
     (`compatibility.rs:44`). So diff could name this change as breaking for callers. It cannot
     name it today.
5. **A second, unrelated adopter.** A delivery service logs every inbound `ScheduleDelivery`
   request and rebuilds state by replaying the log. `address: String` becomes a struct
   `{street, postcode, city}`. The need is real wherever command logs are replayed. But the old
   text cannot be split without a parser, so an upcast is again an algorithm or a guess. The
   idiom covers it: keep `address`, add `address_parts: Optional<Address>`.
6. **Cost.**
   - Idiom: none. No format, no key, no diagnostic. The adopter's generated API keeps `receipt` as
     text and gains one `Option<i64>` input.
   - Diff finding (option C below): no source format. A changed verdict changes what the JSON
     document says. The module says a reader re-derives the answer "and refuses one that does not
     match" (`compatibility.rs:29-31`), so it may need an `ess-diff/N` bump (inferred; confirm in
     design). Touches `crates/verify/ess-diff/src/compatibility.rs` only, with tests.
   - Declared upcast (option D below): new authored key, so a source format bump.
     `SUPPORTED_FORMATS` ends at 23 (`crates/specify/ess-domain/src/system.rs:53-55`), so it would be
     `ess/24`. It could ride `release-plan:ess-24-one-language`, but that plan's intent is "the
     constructs adopted … read as one language", consolidation rather than new constructs
     (`story:response-and-struct-admit-undeclared-fields-when-declared-ignored` is its one new
     key). Also a new conformance suite version (scenarios carrying an older-shape input), a new
     diff change kind, input schemas that become `oneOf`, and generated decoders on three targets.
     It touches `synthesize.rs` / `synthesize/**`.
7. **What else was considered.**

   | option | what | surface | touches `synthesize/` | cost |
   |---|---|---|---|---|
   | A | **change nothing in ESS; idiom:** keep the recorded field's type, add the new fact as an `Optional` input | none | no | 0; adopter rewrites one command (shown working) |
   | B | idiom variant: a second command for the new shape, the old one kept for recorded invocations | none | no | 0; adopter keeps two commands and their routing |
   | C | diff decides a scalar ↔ struct/list/map input change as breaking for callers, so `--fail-on breaking` catches the change before it ships | ess-diff verdict only | no | small; maybe `ess-diff/N` |
   | D | declared upcast on a command input (`from:` old type, a value expression for each new member) | new key, `ess/24`, suite and diff formats, 3 decoders, schema `oneOf` | yes | large; the missing member is still a guess or an obligation |
   | E | let `conversions:` apply at decode | changes the meaning of an existing construct | yes | large; the algorithm stays an obligation (the adopter's 2 unmet capabilities) |

   The requester's proposal (D) is refused as written. It has to invent a value the old record
   does not hold, it introduces a word for something ESS spells `Optional`, and it covers command
   inputs only. A is taken because it says the domain fact exactly: an old receipt names no
   revision. C is kept as a separate, small change, because the diff should have told the adopter
   before the change shipped.

## Decisions

**Recommended: decline, with the idiom (option A).** The coordinator decides; nothing below is
taken yet.

- The idiom: keep the field that recorded invocations carry at its recorded type, and add the new
  fact as an `Optional` input that a guard reads only when present. It is shown working in
  `.engineering/repro/upcast/after-additive.yaml`: valid, 4 of 4 conformance scenarios pass
  including the v1-recorded invocation, 13 of 13 capabilities generated, 0 obligations. The
  `specification` artifact that states the idiom has not been written yet. It is the next step if
  this decision stands.
- Requester's proposal (a declared upcast or an input version, option D): refused as written. The
  old record lacks a member the new shape requires, so an upcast is a guess or an obligation, and
  it would add `ess/24` surface that `synthesize/**` has to carry.
- Separate follow-up, recommended (option C): `ess verify diff --compatibility` should decide a
  command input changed between a scalar and a struct, list or map as breaking for callers.
  Today it answers `unknown` and `history: compatible`
  (`crates/verify/ess-diff/src/compatibility.rs:379`), so `--fail-on breaking` lets the change
  through. It touches no source format and not `synthesize/`, so it can go before the other
  session's synthesis waves end. A story for it is not written.
- Message to the requester (to send once decided): decline with the idiom; the reproducer path; C
  if accepted.

## Closed (2026-10-08)

Declined, with the idiom: keep the old input field's type and add the new information as an `Optional` input, guarding any refusal that reads it with `defined(<field>)` (`.engineering/repro/upcast/after-additive.yaml`). A declared upcast would add a source-format key for a case the idiom covers. The gap the request exposed, `ess verify diff` letting a scalar-to-record input change pass as `unknown`, is `story:a-scalar-to-record-input-change-is-breaking-for-callers`.
