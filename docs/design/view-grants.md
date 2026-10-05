# View grants

beyond10x/ess#286, `story:feature-request-286`. The read-side counterpart of the command grant a
served surface enforces (beyond10x/ess#265). The model is
[`view-grants.example.yaml`](view-grants.example.yaml).

## The construct

From source `ess/22` an actor's `may:` may name a view. There is one grant table, so there is no
second `readable_by:` on the view. Names are unique across kinds, so a grant names a command or a
view and never both.

- A view some actor names is **read-granted**: only the actors naming it may read it.
- A view no actor names stays **open** to every caller. A document that names no view keeps its
  meaning, its IR bytes (`may_read` is left out when empty) and every generated artifact.
- Under `ess/21` and earlier, a grant naming a view is refused once, with
  `unsupported_format_version` naming `ess/22`, at `actor <name>.may`. It is not also reported as an
  undeclared command.
- From `ess/22` a grant naming neither kind is `undeclared_reference`, "which no domain declares as
  a command or a view", and its hint lists both kinds.

## Projections

| Projection | What it carries |
|---|---|
| IR | `ResolvedActor::may_read`, view handles; `EssIr::read_granted`, `readers`, `grants_reads` |
| OpenAPI | on a read-granted view's operation, `x-ess-may-read` and a `403` with the standard refusal body; the description says which views are read-checked |
| Documentation | "It may read …" on each actor that names a view |
| Rust | `may_read(actor)` and `Caller::may_read` in the types crate; `admit_read` in the served surface, run before a read-granted view's route and `handle` arm |
| Go | `readGrants`, `Caller.MayRead`, `AdmitRead` and `admitRead`, run before a read-granted view's route |

The refusal is the one a command answers an ungranted actor: `403`,
`{"refused": "not granted", "actor": <name or null>}`, checked before the view is read. No caller
is refused too.

## Conformance

`ess-conformance/34` and `/35` carry `read_as {actor}`: every later read of the scenario is sent as
that actor, or with `actor: null` as no actor at all (`query_view_anonymous`). `expect_not_granted`
may carry `actor: null` for a refusal naming none. A target reads as the actor through
`ConformanceTarget::query_view_as`, whose default reads as `query_view` does, so a target that
checks no read grant fails the denied scenario rather than being skipped.

For each read-granted view a served component answers, synthesis:

- sends every other read of it as the lowest-named actor naming it;
- files `<view>/grant/read/denied`: the view read as the lowest-named actor the grant does not name,
  where one is declared, and then as no actor, each followed by `expect_not_granted`, which after a
  `query_view` requires that read's refusal (a view every actor may read is still read as no
  actor);
- files `<view>/grant/read/admitted/<actor>` for each actor naming it: the read served.

The ids carry `read` because `<view>/grant/denied` could not be told from a command's
`<command>/grant/denied` by its rendered form. A view with parameters borrows the arrangement of
the shortest scenario that reads it; where none does, a note names it. A model that serves nothing
gets one note: enforcing a read grant is the caller's.

A read the scenario needed answered and the target refused is `failed`, in Rust, Go and
TypeScript alike. The demonstration `--callers actor-header` authentication of the generated Rust
and Go executables authenticates a request carrying more than one `Authorization` header as nobody.
ui-check reports an `actor: anonymous` document reading a read-granted view (`section_readable`)
and a page whose actor's grant does not name a read-granted view it reads (`page_actor_grants`).

The Go and TypeScript runtimes send each read as the actor in force (`ViewRequest.Actor`, `actor`)
and read a refusal from `NotGranted` / `notGranted` on the view result; they give the Rust verdicts
for an honest and a read-serving target (`tests/view_grant_runtimes.rs`). The browser coverage
admission refuses a suite carrying the read-grant ids or `read_as`.

## Not done

- `ess-diff` has no typed change for a read grant gained or lost.
- Concurrent-history recording (`sessions`) reads as no actor.
- Authored scenarios have no `read_as` act.
