# A component declares the work it starts itself (`steps:`, ess/38)

Status: implemented for validation, the IR, the interpreter, `ess verify diff`, synthesis and the
generators (`story:a-component-declares-level-and-time-steps-that-validate-checks-and-the-interpreter-runs`).
Synthesis names each step as a refusal and writes no scenario for it yet; the Rust, Go, web and
command-line generators refuse each step by name.

## The gap

A system acts on its own in two forms: a worker runs a command for each row a view answers (a
queue's `Lease` for each pending job), and a sweeper runs a command every period (a cache's
`Expire` every minute). The time form is partly a periodic binding with a host contract; the level
form has no declaration. A product that keeps such steps in a table beside the model has nothing
that checks the table, and its suite never exercises the loop.

## The declaration

A component lists its steps, in declaration order:

```yaml
components:
  - component: worker
    owns: {domains: [demo.jobs]}
    accepts:
      commands: [demo.jobs.Submit, demo.jobs.Lease, demo.jobs.RecordSweep]
    steps:
      - {form: level, view: demo.jobs.PendingJobs, command: demo.jobs.Lease, every: PT5S}
      - {form: time, command: demo.jobs.RecordSweep, every: PT60S}
```

| key | meaning |
|---|---|
| `form` | `level`: the command runs once for each row `view` answers. `time`: the command runs every `every`. |
| `view` | the view whose rows start a level step. Required on `level`, refused on `time`. |
| `command` | the command the step runs. |
| `actor` | who runs it. Optional: unnamed, it is the one system actor of a system-issued command. |
| `every` | the period, `PT<seconds>S`, read as a periodic binding's `every:` is (the `Period` parse refusal unchanged). Required on `time`; on `level` the polling period an implementation may use. |

The grants stay the one place that says who may run a command
([system-issued-commands.md](system-issued-commands.md)): an unnamed actor is read from them, and a
named one is checked against them.

## What validation checks

From `ess/38`; below it `steps` is refused as `unsupported_format_version` naming `ess/38`.

- `command` and `view` are declared: else `undeclared_reference`, the declared names in the hint.
- A named actor is declared (`undeclared_reference`), may run the command, and for a level step
  may read the view unless no actor's grant names the view
  (`conflicting_declaration`, naming the grant that is missing).
- An unnamed actor: the command is system-issued and granted to one system actor. Else
  `missing_declaration`, naming the command and the actors granted it.
- A level step names a view and a time step `every` (`missing_declaration`); a time step naming a
  view is `conflicting_declaration`.
- A level step runs once per answered row (decision D2): each input of its command is read from the
  row's field of the same name. An input no field fills is `missing_declaration` naming the input
  and the view, unless the input is `Optional`; a field of another type is `type_mismatch`.

A model with no step keeps its IR bytes: `ResolvedComponent::steps` is skipped when empty.

## The interpreter (decision D3)

After each command a scenario sends with its input, the interpreted target runs the steps:

1. Each time step runs once for every whole period of the scenario's clock that elapsed since the
   scenario's first command decision and has not run yet. A command of a model with a time step,
   decided with no clock, is refused as unsupported, naming the time step.
2. The level steps run in rounds. Each round runs every level step in component-name and
   declaration order, its command once per row its view answers. The rounds end at the first round
   in which no level step's view answered a row.
3. A level step whose view never empties ends at `STEP_ROUNDS` (64) rounds, as an unsupported
   answer naming the bound and the views that still answer rows; never a silent stop.

A step's command is a decision like any other: granted to the step's actor, reading the clock once,
dispatching the bindings its events trigger. It does not start the steps again; the rounds do.

## Other targets

- `ess verify diff`: `component/<name>/step-added`, `step-removed` and `step-changed` from
  `ess-diff/24`. A step is identified by its form, view and command; one that runs as another actor
  or at another period is changed.
- Synthesis: one `NotSynthesisedYet` refusal per step, naming the form and this page.
- Generators: a `component_step` capability per step, refused with `step_loop_deferred` for every
  target, its detail naming the form, component, command and actor.
