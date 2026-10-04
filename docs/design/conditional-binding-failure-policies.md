# Conditional bindings and per-refusal policies (#268/#194, #269)

Status: coordinator contract approved by final independent design review on 2026-10-03.
#268/#194 implemented (unit ess-w3-268-194-binding-conditions, 2026-10-04); #269 pending.
Source syntax is coordinated ess/22; ess/21 remains the one-time-response allocation.

#268/#194 as implemented, and where it stops short of this contract:

- `when.where` is admitted from ess/22 and refused below it naming ess/22, refused on a periodic
  cause, and resolved by `ess_domain::binding::condition`. The resolved condition sits beside the
  cause as `ResolvedBinding::condition` (serialized `where`, omitted when absent), as the delivery
  context does, so unconditioned bindings keep their bytes.
- The presence proof admits an Optional event field or accessor into a required input in domain
  validation and in the compiler; the IR keeps the source's declared type.
- The conformance interpreter evaluates the condition before mapping: False skips, and a proved
  accessor is observed as its Optional so absence is refused rather than unwrapped. Unknown is
  that binding's unmet obligation: it invokes nothing, its sibling bindings and every queued
  delivery still run, and the obligation is reported once they have.
- Synthesis takes every branch that publishes the event as a candidate trigger, in model order. A
  member the branch writes as a literal is fixed; one copied from an input is varied, only where no
  branch guards on that input. The positive aspects use the first branch and value set the
  condition holds for, and are refused aspect by aspect where no branch gives one. `condition-false`
  changes one compared leaf from that payload, else takes the first branch that fails with every
  member present. `condition-absent` carries one occurrence per Optional level the condition proves
  present, outermost first, each leaving that level out; the condition is then False, or Unknown
  where a comparison reads the level, and both invoke nothing. A negative witness whose setup or
  binding chain may publish the event again is not taken, and is refused by name where no other is
  left. Both are observed by `expect_no_invocation` (suite/36, /37), with a fifty-ask window in the
  native, Go and TypeScript runtimes. A same-row chain reaching a conditioned binding is unsettled
  with its reason.
- **Not done:** generated Rust, Go and web dispatch refuse a conditioned binding by name
  (`MissingRepresentation`, `bindings.<name>.when.where`) instead of evaluating it. A conditioned
  binding on an external (delivery-context) event gets no scenarios: each aspect, both condition
  witnesses included, is refused by name; choosing the delivered payload for the condition is #268
  slice 2. Browser composition, the diff/14 `binding/predicate-changed` kind, and ess-gen
  docs/graph/AsyncAPI rendering of the condition are not implemented.

## Payload condition (#268 and #194)

```yaml
when:
  event: demo.orders.OrderChanged
  where:
    all:
      - defined: event.shipping
      - event.kind == Ship
```

`where` is an optional typed predicate over the declared event payload, for local and external
event causes. Periodic causes have no event and refuse it. Outcome names, command input, stored
rows, caller, producer context and delivery context are not predicate roots. The event log/wire
shape does not gain a source-outcome discriminator. Branches publishing identical payloads are
indistinguishable to bindings, as they are to other event consumers.

Use the existing bounded selection predicate AST and evaluator, with root `event` instead of
`item`: Always, Never, Defined, equality/inequality to a typed literal, All, Any and Not. Reads
traverse at most three declared field segments through structs and Optional structs; comparisons
end at String or enum leaves and use their existing nominal/literal validation. Defined can test
an Optional struct or scalar path, so it can express #194 without a fabricated leaf comparison.
No functions, ordering, fact-to-fact comparison, lists, maps or quantifiers enter this slice.
Locations name the authored `when.where` operand, not an unrelated mapping field.

Evaluate against the admitted event payload before selections, conversions, mapped-input
construction or command invocation. True continues normal delivery. False consumes/skips this
binding occurrence successfully, invokes nothing and runs no failure policy; other bindings for
the event still run. Unknown is a typed unresolved-binding-condition failure, invokes nothing and
is reported as an unmet obligation; it cannot become a successful skip. Invalid required payload
data is rejected by normal event admission before condition evaluation. Null/absent Optional
members retain the evaluator's existing absence semantics.

### Presence proof for required mapped inputs

A condition only fixes #194 if it also makes mapping admission sound. A mapping of an Optional
event path to a required command input is admitted when the compiler proves that `where == true`
implies that path and every Optional ancestor are present, and that the present types are
assignable. The proof uses the same resolved path identity as mapping; no spelling-prefix trick
may refine a sibling or a different source root.

Use conservative positive presence facts for both True and False of the finite AST: Defined(p)
when true proves p and its traversed Optional ancestors; a typed non-null comparison that is
definitely true or false proves the operands it must read are present; Not swaps the two fact
sets; All-true and Any-false union their child facts, while All-false and Any-true intersect them.
Always/Never contribute no presence facts. An empty proof is safe and may refuse an unsafe
mapping; it cannot silently unwrap. For an Optional parent with a separate Optional child,
Defined(parent) does not prove the child present. Without sufficient proof the existing
ESS-BINDING-015 obligation/refusal remains. Runtime performs the same presence checks before a
required input is built, even when a statically valid condition guards it.

Keep unconditional mapping bytes and meanings unchanged. The resolved event cause gains an
optional, omitted-when-absent typed predicate plus resolved read/projection authority; source22
fences its serialization. All consumers match the new condition explicitly. Domain/compiler
validation agrees for parsed and programmatically assembled specifications.

### Zero invocation is observable

Add `ExpectNoInvocation { binding, command }`, serialized as `expect_no_invocation`, under
ordinary suite/36 and inventory suite/37. These are new allocations for this binding observation,
not amendments to released or held suite/34–35. It observes the complete invocation set for that
binding/command under the scenario correlation for the runner's whole eventual window. Any
attempt, including a refused attempt or one with wrong inputs, fails immediately. No attempts
passes only at the deadline. An unsupported observer is unsupported, never pass. There is no
input filter and no inference from absent events.

The scenario's setup must not invoke that binding; if it necessarily does, synthesis names the
unarrangeable negative witness rather than inventing a log baseline. The required fixture has an
independent setup route. Zero-invocation is distinct from #267 drop's one failed attempt and zero
retries. Readers for all earlier suite formats refuse this instruction/version, while unaffected
suites retain their existing minimum version and bytes. Native, Go, TypeScript and browser suite
admission/execution must all implement the new step before required acceptance can pass.

The positive/negative conformance fixture varies exactly one discriminator/Optional fact at a
time. Named controls `condition_true_invokes`, `condition_false_never_invokes`,
`optional_parent_absent_skips_before_mapping`, `optional_child_still_requires_proof`,
`other_binding_still_invokes`, and `late_unwanted_invocation_fails` run through real generated
Rust/Go dispatch and the binding-running native fixture. Mutants ignoring the predicate, treating
Unknown as false, unwrapping before testing, and sharing another binding's condition fail.
The mapped required input is observed at the actual invocation seam, not predicted by the test.

## Refusal-selected failure policy (#269)

The accepted direction is policy-keyed selection, not a map keyed by condition kinds or a second
outcome grammar. Complete the earlier illustrative syntax by retaining escalation's required event:

```yaml
on_failure:
  escalate:
    emits: demo.orders.DispatchEscalated
    except: [already_done]
  drop: [already_done]
```

In selected mode, keys are the closed `drop`, `retry`, `escalate` policies, each at most once.
Each policy has exactly one selector: `outcomes: [names]` or `except: [names]`. Drop and unbounded
retry allow the list shorthand for `outcomes`. Escalate always uses a block with required `emits`.
Retry's block may additionally have the existing `attempts` and `final`; `final` requires a bound,
and attempts includes the first invocation and is at least two. Unknown keys, duplicate policy
keys, mixing selectors, empty positive lists and duplicate/overlapping aliases are refused.
An empty `except` is the explicit all-refusals complement and is legal.

Exactly one policy has `except`. It is the explicit fallback for failures of an invoked command
port that carry no declared command outcome, as well as its selected declared refusals. Every
other policy is positive.
Resolve names exactly as retry.final: an outcome name must carry an error; a qualified error name
expands to every refusal of the invoked command reporting it. A condition word such as wrong_state
has no special meaning; it is legal only if it is actually that command's refusal outcome name.
Resolve aliases first. The resulting sets over declared refusal outcomes must be disjoint and
exhaustive, including the complement; reject overlap even when two aliases name the same policy.
Unqualified unknown names, another command's outcomes and accepting outcomes are refused.

No selector means the existing universal policy syntax and exact semantics remain. Below source22,
new selector syntax is refused by source-version admission without changing the original universal
policy parser/diagnostics. Capture new raw shapes until version admission so a pre22 new mapping
does not bypass the intended format refusal. Do not reinterpret an old retry.final or escalation
emits block as a selector. The schema exposes both strict old and new shapes with the source fence.

### Attempts and actual refusal authority

After every failed invocation, select the policy from its actual declared outcome (or the explicit
fallback for an untyped transport/adapter failure returned by that invocation). A logical attempt
begins at the dispatcher-to-command-port boundary with a fully constructed, admitted input. Count
and record it once before calling the port; a network failure returned from the port still consumed
that attempt, even if the remote command's execution is unknown. No field of an error invents a
declared refusal outcome. An accepting outcome ends processing. Drop
ends immediately. Escalate emits its declared event once and ends. Retry uses the existing delivery
schedule; no new timing guarantee is implied. A bounded retry compares its total invocation count
for this occurrence with attempts; switching between refusal outcomes never resets that count.
If a selected retry outcome is in its resolved final set, it drops immediately. Final aliases must
resolve wholly inside that retry policy's selected set; otherwise validation refuses them. A final
refusal does not silently reroute to escalation. An unbounded retry remains unbounded in the same
sense as existing `on_failure: retry` and must not be advertised as a finitely completed aggregate
cut.

Conditions run before the command, so a false #268 predicate is neither a refusal nor an attempt.
The selected policy cannot cause it to invoke. A mapping, host conversion or selection error before
the command port has a valid complete input is an explicit unmet-binding-input obligation, with
zero command attempts and no policy selection, automatic retry or escalation. This is the same
fail-closed authority boundary as an unresolved condition, not a transport refusal of an invoked
command. Do not apply a bounded retry whose invocation count cannot advance. Universal old-policy
models retain their existing published behavior; this defines selected-mode admission/execution
and requires a named unsupported/obligation result if the host cannot honor the boundary.

Escalation is reached only after a recorded attempt and therefore has that attempt's actual full
input for the existing typed escalation builder. Preserve its current explicit host obligation
for constructing the declared event payload; the generated runtime must never fabricate it from
a partial mapping or expected suite values. A builder failure remains a reported obligation/error,
does not emit a partial event and does not restart the original command's retry loop. The required
fixture supplies and tests the builder, including untyped command-port failure with valid input.

### Typed representation and consumers

Keep the existing failure/escalation/retry fields and bytes for universal policies. For a selected
policy, add one typed optional refusal-policy table (omitted otherwise): resolved outcome identities
map to concrete Drop/Escalate(event handle)/Retry(bound or unbounded) descriptors, plus a typed
fallback descriptor. Ordered collections only. The compiler may derive legacy fields from the
fallback for internal compatibility, but consumers must use an explicit new ByRefusal arm of
`ResolvedBinding::on_failure`; no consumer is allowed to read those fields and silently apply the
fallback to every declared refusal. Validate that the redundant internal view agrees with the table.
Source22 is the serialization fence; no generic ESS IR envelope or unchecked IR reader is added.

Generated Rust and Go dispatch, native binding fixtures, web hosting over Rust, synthesis, docs,
AsyncAPI annotations and graph descriptions handle the selected policy explicitly. Entity Runtime
keeps a named unsupported disposition if its binding authority cannot represent it. Diff/14 is the
coordinated new allocation for `binding/predicate-changed` and `binding/refusal-policy-changed`,
both with semantic relation Changed and typed, normalized before/after content (optional predicate,
or complete resolved refusal table/fallback). No direction or equivalence is guessed. All earlier
diff readers refuse /14; new readers retain strict old formats and rederive ids/content as before.
Universal-policy changes keep their existing `failure-changed` kind and minimum version. Unchanged
models keep byte-identical projections. A generation obligation is not successful execution of
the required Rust/Go profile.

### Required policy controls

Synthesis witnesses every declared selected refusal via an actual arrangable branch or a declared
external injection; an unforceable branch is a named coverage limitation, never a fabricated
result. The required fixture makes drop, escalation, bounded retry and retry.final forceable and
observes all command attempts and effects. Existing positive count/every-invocation instructions
are sufficient for these per-entry witnesses; no outcome is inferred merely from an input name.

Named controls `selected_drop_once`, `selected_escalation_once`, `selected_retry_to_success`,
`selected_retry_exhausts_total_budget`, `selected_retry_final_stops`,
`outcome_aliases_expand_to_same_policy`, and `unknown_failure_uses_explicit_fallback` run healthy
and faulty dispatchers. Separate actual generated-runtime tests force a sequence of different
refusals during one retry and verify no budget reset and correct reselection. Choosing the wrong
policy, escalating twice, retrying a dropped/final refusal, omitting an attempt, and applying the
fallback to every refusal must each fail. Add `pre_input_failure_is_an_obligation` with zero
attempts/retries/escalations, `untyped_port_failure_consumes_attempt_budget` with an exhausted
bounded fallback, and `fallback_escalation_uses_actual_complete_input`; an escalation-builder
failure publishes nothing and does not reenter retry. Optional condition absence composes with every policy
and still invokes zero times. All applicable native/generated Rust/generated Go and native/Go/TS
suite controls must execute, with actual browser composition before bundle closure.

Independent design review precedes implementation; independent implementation review, affected
tests, strict lint, old-reader controls and regenerated owned outputs precede integration. These
contracts do not claim transitive causal completion for aggregate observations.
