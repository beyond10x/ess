# Fit review of externally driven changes, 2026-09-22 to 2026-09-30

Read-only audit of `origin/main` at `1bd946d6b` against the fit questions in
`.agents/skills/assessing-external-requests/SKILL.md` (C1 need vs syntax, C2 class, C3 existing
idiom, C4 consistency and target support, C5 generality, C6 surface cost, C7 alternatives). Issues
were read with `gh issue view`; planned stories with `aep plan artifact show`. Nothing was run:
every finding is from reading code and documentation, and says so where it matters.

Abbreviations: CL `CHANGELOG.md`; SV `website/docs/reference/spec-versions.md`; GP
`website/docs/guides/specify/guards-and-predicates.md`; CO `.../specify/commands-and-outcomes.md`;
BC `.../specify/bindings-and-components.md`; PR `website/docs/reference/predicates.md`; SY
`website/docs/guides/synthesize.md`. "DS" = raised by a downstream adopter.

## 1. Verdicts

### Shipped, 0.40.0–0.48.0

| # | Change (version) | Origin | Class | Verdict | Reason |
|---|---|---|---|---|---|
| S1 | `ess-composition/3` `reader: true` (0.40) | DS #191 | gap | keep, revise | a document can assert what ESS's own closed generated readers break (CL:943–947) |
| S2 | Go/TS runner surface (0.40) | DS #188 | defect | keep | runners back to the suite versions the release writes |
| S3 | `when_subject_state: [..]`, subjectless refusal (ess/18, 0.41) | DS #201 | gap | keep, revise | S4 in the same release expresses it; make it sugar |
| S4 | `state` inside `when_subject` (ess/18, 0.41) | DS #204 | gap | keep (canonical) | view filters already read `state` (PR:31) |
| S5 | `when_related:` (ess/18, 0.41) | DS #211 | gap | keep, revise | `via` differs from `{related:}`; `exists` has four meanings; no guide section |
| S6 | event-binding `context_fields` / `context_authority` / `context.` (0.41) | DS #195 | gap | keep, revise | adopted verbatim; duplicates periodic `host: {authority, context_fields}` / `host_context.` |
| S7 | `context.x` literal refused under pre-ess/18 headers (0.41) | follows S6 | breaking | revise | against the `{caller:}` / `response.item` legacy precedent (SV:160, CO:330–331) |
| S8 | mutation report/manifest /2, /3 (0.41, 0.42) | DS #203 #210 #218 | defect | keep | churn, each bump justified |
| S9 | first-declared-wins overlap (0.42) | DS #217 | defect | keep | matches Entity Runtime |
| S10 | `ess-diff/11` (0.42) | DS #219 | defect | keep | diff lagged ess/15 |
| S11 | relation via own identity (0.42) | DS #230 | defect | keep | relaxation |
| S12 | input refusal beside held-state branches (0.43) | DS #213/#227 | defect | keep | ESS contradicted its documented precedence |
| S13 | nested `{$instance}`, suite/32–33 (0.43) | DS #242 | defect | keep | fourth Rust-only suite pair in a row |
| S14 | `ESS-AUTHOR-037` (0.43) | DS #243 | defect | keep | such suites were unsatisfiable |
| S15 | `Json` in every target (0.44/0.45) | DS #224 | gap | keep | removed named refusals |
| S16 | generated behaviours, queries, invariants, ports (0.46) | DS epic | gap | keep, revise | fills error fields by same-name match the spec never states (SY:211–212) |
| S17 | `Actor`, `may()` (0.46) | DS epic | convenience | keep | data only |
| S18 | `handle`, `entry::Refused` (0.46) | DS epic | convenience | keep | — |
| S19 | `--layout crate` (0.46) | DS, one adopter | local policy | keep | opt-in, default unchanged |
| S20 | ess/19 error `payload:` sources (0.46) | DS | gap | keep, revise | completeness rule differs from events (CO:325 vs CO:358) |
| S21 | `ess-diff/12` (0.46.1) | DS #253 | defect | keep | diff lag again |
| S22 | `ess-ui/1` family (0.47/0.48) | mixed | new family | keep, revise | a second type grammar (`{list: T}`, `string`, `{record:}`) |
| S23 | served `published`, `Request.headers`, event naming (0.47) | DS story | gap | keep | wire break stated |
| S24 | 501 "committed, delivery failed" (0.47) + `committed`, `Refused::Undelivered` (0.48) | DS story + #260 | defect | revise | a success answered 501; both requester alternatives adopted; two breaking releases |
| S25 | results format, `verify conform report` (0.48) | DS story | gap | keep | CL:25 still says "(unreleased)" |

### Planned stories (`epic:downstream-reported-gaps`)

| Story | Class | Verdict | Reason |
|---|---|---|---|
| 229 `state` in `when_related` | gap | proceed after the format decision | same operand as S4; #204 gated it at a format |
| 257 key copied from a related row | defect | proceed | synthesis only |
| 265 ungranted actor refused | gap | redesign | an `ess-actor` header lets a client-settable header decide authorization; 403 body collision |
| 266 bindings that move state | defect? | pause | not reproduced; interpreter runs no bindings; immediate vs eventual undecided |
| 267 flow/delivery into `wrong_state` commands | defect | proceed, tighten | the `drop` acceptance is an unfalsifiable either/or |
| 268 binding reacts to one outcome | convenience/gap | redesign | use an event `where:` predicate (also closes #194) |
| 269 per-refusal failure policy | gap or convenience | pause | the accepted-no-op idiom may cover it; else reuse `retry.final`'s grammar |
| 270 `{related:}` sets beside `when_related` | defect | proceed | — |
| 271 `when_related` over an owns `via` | defect | proceed | share #193's code path |
| 272 `when_related` on an aggregate creator | defect | proceed | — |
| 273 identity fields in event expectations | defect | proceed (authoring), redesign (format) | narrowing `used_by` writes /18 steps under older headers |

## 2. Evidence (selected)

- **S1:** the requester's second option, adopted as proposed. ESS-generated closed types do not
  ignore unknown keys, "so a consumer reading through them must not use `reader` for a field subset"
  (CL:943–947), yet validation does not refuse it.
- **S3/S4:** held state is selectable five ways (`wrong_state`, `when_subject_state: S`,
  `when_subject_state: [..]`, `state` in `when_subject`, `when_state_changes`), and
  `when_subject_state` beside `when_subject` is refused as "one selection authority"
  (`crates/specify/ess-domain/src/command.rs:5240–5253`) while `{all: [state == Ready, …]}` is
  accepted. GP:139–140 is stale. The interpreter does not interpret `SubjectState`
  (`interpret/execute.rs:657–658`).
- **S5:** `via` is input-only (PR:27, 775) while `{related:}` also reads a subject field
  (`values-and-views.md:39`); `exists` means four things (PR:167, 357, 454–471); a missing own row
  answers 404 and a missing related row 409 (`ess-gen/src/http.rs:141–147`); no guide section.
- **S6/S7:** periodic bindings already have `host: {authority, context_fields}` read as
  `host_context.` (BC:180–199); the design note keeps them apart without a reason
  (`binding-mapping-bounded-accessor.md:628`). CL:808 breaks old documents under their own header,
  unlike `{caller:}` below ess/16 (SV:160) and `response.item` (CO:330–331).
- **S12:** #204 gated a new operand at a format; #227, #230 and #217 relaxed rules without one.
  SV:17–19 has no rule for relaxations.
- **S16/S20:** under ess/4 every event field is mapped (`{generated: true}` for the
  implementation's own values, CO:325–326); under ess/19 an error field "needs no line" (CO:358);
  generated Rust fills it "from the row… its field of the same name and type" (SY:208–213).
  CO:156 and CO:220 contradict SY:223.
- **S22:** `string`, `{list: T}`, `{record:}` in `schemas/ui/ess-ui.schema.yaml:42–43, 165–174, 269`
  beside ess/N's `List<T>` and `String`; `actor: {enum: [from_session, anonymous]}` (schema:163) is
  a third notion of the caller.
- **S24:** the adversary flagged the overload (`review-result:adversary-served-events-pass-1`);
  ESS declared it instead of removing it (CL:128–138); #260 offered two alternatives and both
  shipped (CL:48–68); `UNFINISHED` still documents "the realization is unfinished"
  (`ess-gen/src/http.rs:93–100`).

## 3. Worst five, with revisions

1. **Story 265's `ess-actor` header** — a client can claim any actor unless every shell strips the
   header. Revision: the shell passes a typed `Caller` into `dispatch`/`handle`; the route never
   reads the actor from headers; one documented 403 body distinguishable from caller-decided
   refusals. (Applied to the wave-1 unit on 2026-09-30.)
2. **501 for a committed success (S24)** — every client sees an error for a command that took
   effect. Revision (breaking, batch it): answer the command's own outcome plus
   `undelivered: [{binding, event}]`; 501 only when the port did not run.
3. **Five ways to select held state (S3/S4)** — correct rules refused, parallel code paths, a
   contradictory guide. Revision: `state` is the one operand; `when_subject_state` compiles to
   `state in [..]` combined with any `when_subject`; drop the "one selection authority" refusal;
   fix GP:139.
4. **Host-bound context in two shapes (S6/S7)** — revision in the next format: one
   `context: {authority, fields}` block and one `context.` prefix for event and periodic causes,
   old spellings as aliases; restore `context.x`'s literal meaning below ess/18.
5. **Three rules for payload filling (S16/S20)** — revision: a `payload:` entry maps every field,
   `{generated: true}` marks implementation values, the generator never fills by name.

Also: `exists` has four meanings; `via` spelled two ways; ess-diff bumped three times in three
days; suite pairs 28–33 Rust-only; `reader: true` unenforceable against closed types; ess-ui's own
type grammar; no format rule for relaxations; stale lines CO:156, CO:220, GP:139, CL:25.

## 4. Not established

Whether 266 reproduces; ess/19 support in Go/TS runners, Entity Runtime and the Go target; how many
distinct adopters stand behind the bot-filed issues (only #227 and #229 name a second); whether any
shell strips client headers today. No runtime observation backs any finding.
