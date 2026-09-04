# ESS Model-Driven Interpretation — Design v0.1

> **Repository:** `beyond10x/ess`
> **Status:** Proposed. Nothing is implemented against this page yet, and the argument in §2 is what
> it is asking for agreement on.
> **Audience:** Implementors of `ess-conformance` and of a new `ess-interpret`, and anybody holding a
> specification that nothing has ever executed.
> **Relationship to existing work:** Additive. One more implementation of a trait that already has
> two, and one more verb beside three that already exist. No existing target, suite, emitter or
> generated tree changes.

---

## 1. Nothing can run a specification nobody has implemented

`ess conform run --target <TARGET>` runs a suite against a target, and there are two:
`crates/verify/ess-conformance/src/reference.rs` says what they are in its own first lines —
"`Billing` is `examples/billing/` implemented by hand and in memory, and `Oracle` is
`examples/oracle-fixture/`". Both are hand-written, both are deliberately boring, and both exist so
that a generated suite has something known-good to be checked against: "No repository trait, no event
bus, no ports and adapters — an entity is a struct in a map, a projection is a filter over that map,
and a binding is a function call."

That is the right thing to have built and it does not generalise. A specification outside those two
examples has no target. So the honest state of the toolchain is:

| A specification can be | Today |
|---|---|
| validated | yes |
| compiled to typed IR | yes |
| projected to schema, OpenAPI, AsyncAPI, docs | yes |
| synthesised to Rust, Go, web, clap | yes — as types, ports and **unfilled obligations** |
| **executed** | **only if somebody hand-wrote it, and two people have** |

A consumer specification in this organisation currently carries 68 obligations, none filled, and has
never run. Every claim anybody makes about it is a claim about a document.

## 2. Gap register D-2 does not forbid this, and the wording matters

The obvious objection is that ESS refuses to choose behaviour. The web emitter says so directly, in
`crates/generate/ess-synth/src/web/mod.rs`:

> **It never chooses a realization.** Every command behaviour is an obligation, and this bridge fills
> none of them… Gap register D-2 says the machinery does not choose; a page is machinery.

Read D-2 itself, as `docs/plan/ess-wave-6-structural-synthesis.md:73-76` records it:

> One constraint on the linker, decided in `gap-register.md` D-2: it never chooses. Zero
> implementations for an obligation is an unsatisfied obligation; two is an ambiguity error naming
> both. **Selection among alternatives** is `Realization` material and stays proposed with it.

D-2 constrains **the linker**, and what it forbids is **selection among alternatives**. It is the rule
that stops the machinery silently picking one of two candidate implementations and shipping it. It is
not a rule that an implementation may not exist, and it cannot be — `reference.rs` is an
implementation, it is in this repository, and it does not violate D-2. It does not violate D-2 because
nobody's build silently selected it: an operator names it, with `--target`.

An interpreter is in exactly that position. `--target interpreted` is a choice a person makes, in the
same argument slot, with the same explicitness. Zero implementations for an obligation remains an
unsatisfied obligation; two remains an ambiguity naming both. Nothing about the linker changes,
because the interpreter is not linked — it reads the IR.

So the distinction this page asks agreement on is not *may behaviour be chosen* but **who is
choosing, and for what**:

| | chooses | for | D-2 |
|---|---|---|---|
| the linker | nothing — refuses zero, refuses two | code somebody ships | governs it |
| `--target billing` | the operator, explicitly | checking a suite | untouched |
| `--target interpreted` | the operator, explicitly | checking a suite | untouched |
| a synthesis target | nothing — emits obligations | code somebody ships | governs it |

**One thing to fix while here:** wave 6 cites `docs/plan/gap-register.md` and that file does not exist
at `11fc669`. The decision survives only as the sentence quoted above. Either the register comes back
or D-2 should be restated somewhere that is not a plan document, because a constraint whose home is
missing is a constraint the next reader will re-litigate. This page is currently the second-best copy
of it, which is the situation this repository refuses everywhere else.

## 3. What the model determines, and what it does not

An interpreter is only honest if it is precise about the boundary. Everything in the left column is
derivable from the IR and the interpreter must derive it. Everything in the right is not, and the
interpreter must refuse to invent it.

| Determined by the model | Not determined |
|---|---|
| which outcome a command may take, and the error a `wrong_state` branch reports | an outcome declared `external:` — §12 says it is decided outside |
| the field writes an outcome makes — `sets:`, from `input.<field>` or a literal | the value of an identity nothing supplies |
| the transition an outcome takes, and from which states it is legal | what time it is |
| the events emitted, and each payload's mapping | how a projection is stored |
| whether a `wrong_state` branch refuses or accepts — `refuses:` | |
| what must hold of an instance at rest — `invariants:` | |
| a view's rows: `source`, `filter`, `params`, `order_by`, `shape` | |
| which binding reacts to which event, with what mapping, and its `delivery` and `on_failure` | |

The right column is already a solved problem in the suite format. `ConformanceTarget`
(`crates/verify/ess-conformance/src/target.rs`) has `configure_external_outcome` at `:115` for the
first, and `mark_instant` at `:164` with `observe_elapsed` at `:194` for the third — both added for
authored scenarios claiming a length of time. A scenario supplies what the model does not, which is
what a scenario is for. The interpreter mints identities and a scenario binds them by name with
`capture:`.

**So the interpreter does not fill obligations. It executes what the model already decided, and asks
the scenario for the rest.** An obligation is what a *shipped* implementation owes. A test double owes
nothing, because nobody ships it, and this one owes less than the two hand-written doubles do — they
had to decide 52 command behaviours by hand and could have decided one of them wrong.

## 4. Three things not to shortcut

The value of an interpreter is that it is right by construction where the hand-written doubles are
right by attention. Three places where construction is not enough and `reference.rs` already shows
the way.

**An eventual view is really eventual.** `reference.rs` makes `consistency: eventual` lag by
`Billing::DEFAULT_LAG` further reads, and its own doc says why making it immediate would be cheating:
"a suite that never waits never tests the word `eventual`, and the first real projection would find
that out in production." An interpreter with a single in-memory map is *tempted* into immediacy by its
own architecture, which makes this the first thing it will get wrong. Reproduce the lag.

**An unmet obligation is not a delivery failure.** `crates/generate/ess-synth/src/rust/system.rs:23-27`
draws this line for the emitted pump: a port refusing because its behaviour is owed "is a fact about
the workspace being unfinished, not a fact about a delivery, and escalating it would publish a domain
event for a defect no provider caused — manufactured evidence." The interpreter has no unfilled ports,
so it will be tempted to drop the distinction. It must keep it, because a scenario that scripts an
external outcome and gets no answer is in exactly that state.

**A refusal the model does not declare is still a refusal.** `reference.rs`'s second cheat-refusal is
this one. An interpreter that answers `wrong-state` for anything it finds inconvenient turns the
suite green by making every branch reachable.

## 5. It is verifiable before it is trusted, and that is the point of building it first

`examples/billing` and `examples/oracle-fixture` each have a committed suite and a hand-written
target. Run the interpreter against both.

- Where the interpreter and `Billing` agree, two independently-derived implementations of one document
  agree, which is worth more than either alone.
- Where they disagree, one of them is wrong **and the specification says which**. That is a
  falsifiable check, available at zero authoring cost, and no test written for the interpreter in
  isolation is as strong.
- `faulty.rs` exists because design §26 requires that a fault leave unrelated scenarios green. Run it
  against the interpreter too: a fault that reddens everything means the interpreter has coupled
  things the specification keeps apart.

The gate for trusting the interpreter is therefore not "its tests pass" but **"billing's suite and
oracle's suite are as green against it as against the hand-written targets, and the fault matrix still
discriminates."** Until that holds, nothing downstream of it means anything.

## 6. What this is not

- **Not a realization.** Nothing links it, nothing ships it, and it is not a candidate the linker may
  select. If it ever becomes one, D-2 applies to it and this page is void.
- **Not behavioural synthesis.** It emits no code. Wave 6 lists behavioural synthesis among what is
  deliberately not in scope, and this does not smuggle it in: interpreting a document at test time and
  emitting an implementation are different acts with different consequences, and only the second one
  leaves something behind that somebody could deploy.
- **Not a reason to stop filling obligations.** It makes filling them *postponable*, which is
  different, and a specification that runs only under an interpreter has still never been implemented.
  The interpreter's job is to make a specification falsifiable early. A consumer that mistakes it for
  a runtime has misread it, and the page that renders it should say so where somebody will read it.
