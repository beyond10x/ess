---
format: aep.planning-md/1
id: epic:command-line-surface
kind: epic
status: implemented
title: A command line is a declared surface, not a hand-written one
revision: 4
---
# Epic: a command line is a declared surface, not a hand-written one

## Outcome

A component can say that its callers are people at a terminal, say where each command sits in the
tree they type, and have the parser, the obligations and the completion scripts generated from that
declaration. The epic is finished when a command-line surface is checked by `ess validate` before
anything is generated from it, and when the grammar a person types cannot drift from the
specification it came from.

## Why

Three targets already turn one IR into an implementation — `rust`, `go`, `web` — and the web target
is a whole front end. All three answer *what does this system do*. None answers the question an
operator asks at a shell: *what can I run, and what may I type after it*. That answer is a grammar,
and a grammar was the one surface the model could not state.

The cost of not stating it is not hypothetical. `connectors` serves `kubernetes.workloads` from its
personal-local daemon and has no command-line verb that reads a datasource, so an operator on that
machine cannot reach a projection the process beside them is already publishing. Nothing reported
it, because nothing knew what the surface was supposed to be.
