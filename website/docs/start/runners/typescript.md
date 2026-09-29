---
title: The TypeScript runner
description: The files ess writes for a TypeScript conformance package, the Target interface an implementation provides, and how to run it with npm.
---

# The TypeScript runner

`ess verify conform synthesize --target typescript` writes the suite a specification obliges as an
ESM test package that `node --test` runs. [Run your first conformance suite](../first-conformance-run.md)
walks through one; this page lists what the package holds and what your implementation provides.
It needs Node.js 20 or later.

## What ess writes

From the `tasks` project of the earlier pages:

```shell-session ess-tutorial
$ cd ~/tasks
$ ess verify conform synthesize --path . --target typescript --out typescript
6 scenario(s) (0 authored), 0 refusal(s), 13 file(s) written to typescript
```

The package is `essconform`, one directory below `--out`:

```text ess-tutorial files=tasks/typescript/essconform
README.md
ir.json
package.json
src/coordinate.ts
src/explore.ts
src/fixtures.ts
src/index.ts
src/predicate.ts
src/reading.ts
src/response.ts
src/runtime.ts
suite.json
tsconfig.json
```

| File | What it is |
|---|---|
| `suite.json` | the suite: one scenario per outcome, lifecycle move and refusal |
| `ir.json` | the compiled specification, which the explorer interprets as its reference model |
| `src/runtime.ts` | the runner: `run`, `runWith`, the `Target` interface and its request and result types |
| `src/predicate.ts`, `src/response.ts`, `src/reading.ts`, `src/coordinate.ts`, `src/fixtures.ts` | what the runner asserts with: guards, returned values, clock readings and supplied values |
| `src/explore.ts` | `explore` and `exploreConcurrent`, seeded random command sequences |
| `src/index.ts` | the entry point your test imports from |
| `package.json`, `tsconfig.json` | the package: `npm test` compiles `src/` into `dist/` and runs `dist/*.test.js` |
| `README.md` | the wiring for this suite, and the report format it requires |

Beside the package, `typescript/.ess-output/state.json` records which files `ess` wrote there. Nothing in `essconform` is edited by hand; a file
you add, such as `src/conformance.test.ts`, is kept when you regenerate.

The package has no runtime dependencies. Before you add anything to it, it installs and typechecks
without a finding:

```shell-session ess-tutorial requires=node
$ cd typescript/essconform
$ npm install
…
$ npm run --silent typecheck
```

## What your implementation provides

One object satisfying `Target`, exactly as `src/runtime.ts` declares it. A method marked `?` is
optional:

```ts ess-tutorial interface=tasks/typescript/essconform/src/runtime.ts
export interface Target {
  fixtureValues?(
    scenario: ScenarioContext,
    contract: FixtureContract,
  ): Answer<Record<string, Node>>;
  identity(): Answer<Identity>;
  beginScenario(scenario: ScenarioContext): Answer<void>;
  endScenario(scenario: ScenarioContext): Answer<void>;
  executeCommand(request: CommandRequest): Answer<CommandResult>;
  executeCommandWithoutInput?(request: AbsentInputRequest): Answer<CommandResult>;
  queryView(request: ViewRequest): Answer<ViewResult>;
  observeEvents(request: EventObservationRequest): Answer<ObservedEvent[]>;
  configureExternalOutcome(control: ExternalOutcomeControl): Answer<void>;
  configureExternalOutcomeRepeatedly?(
    control: ExternalOutcomeControl & { times: number },
  ): Answer<void>;
  redeliverEvent(request: RedeliveryRequest): Answer<void>;
  observeInvocations(request: InvocationObservationRequest): Answer<Invocation[]>;
}
```

| Method | What it answers |
|---|---|
| `identity` | the implementation's name and version, for the report |
| `beginScenario`, `endScenario` | bracket one scenario; state from one scenario must not satisfy another |
| `executeCommand` | run one command; return the outcome taken, the declared error if it refused, and the events it emitted directly |
| `queryView` | read one view; a `read_your_writes` view must already show the command that just returned |
| `observeEvents` | the events seen for one activity, away from the command that caused them |
| `configureExternalOutcome` | force the next answer of an outcome declared `external:` |
| `redeliverEvent` | deliver an event a second time, for `delivery: at_least_once` |
| `observeInvocations` | the commands one binding invoked and what it passed |

A method the implementation cannot answer throws `ErrUnsupported`. The scenario is then reported
as skipped, which is a different fact from failed, and a run with a skipped scenario is
`inconclusive`, not `passed`. A specification that declares backend setup, periodic hosts or clock
readings asks for a further interface beside `Target` (`EntitySetupTarget`, `PeriodicTarget`,
`ClockReadingTarget`); the generated `README.md` names the ones its suite needs.

## Run it

Hand a factory to `run` from one test file under `src/`; the runner builds one target per
scenario:

```ts
import { test } from "node:test";
import { run } from "./index.js";
import type { Target } from "./index.js";

await test("conformance", async (t) => {
  await run(t, (): Target => newTarget());
});
```

Then run `npm test` with `ESS_REPORT_FORMAT=2` set; `ESS_REPORT_OUT` names a file for the report:

```shell-session
$ ESS_REPORT_FORMAT=2 ESS_REPORT_OUT=report.json npm test
```

[Verify conformance](../../guides/verify-conformance.md#hold-your-own-implementation-to-the-suite)
covers authored scenarios, coverage and the report in full.
