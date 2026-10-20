// The transcript target for the generated TypeScript runtime: every question is answered with the
// answer the Rust target gave the Rust reference runner to the same question
// (tests/support_go/mod.rs, Recorder). Questions are matched per scenario by method and key, in
// order; a request that differs from the recorded one is written to the divergence file and fails
// the call. `bare` hides every optional method, as for a target written before them.
import { readFileSync, writeFileSync } from 'node:fs';
import { runWith, unsupported } from './dist/runtime.js';

const [suiteFile, transcriptFile, divergenceFile, bare] = process.argv.slice(2);
const transcript = JSON.parse(readFileSync(transcriptFile, 'utf8'));
const divergences = [];
const repeatable = new Set(['query_view', 'observe_events', 'observe_invocations', 'await_outcome']);

function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(',')}]`;
  if (value !== null && typeof value === 'object') {
    return `{${Object.keys(value)
      .sort()
      .map((key) => `${JSON.stringify(key)}:${canonical(value[key])}`)
      .join(',')}}`;
  }
  return JSON.stringify(value ?? null);
}

function result(recorded) {
  return {
    outcome: recorded.outcome ?? undefined,
    error: recorded.error ?? undefined,
    errorPayload: recorded.error_payload ?? undefined,
    consistency: recorded.consistency ?? undefined,
    directEvents: recorded.direct_events ?? [],
    response: recorded.response ?? undefined,
  };
}

const target = () => {
  let scenario = '';
  let used = new Map();
  const next = (method, key, request, compared) => {
    const matching = (transcript[scenario] ?? []).filter(
      (entry) => entry.method === method && entry.key === key,
    );
    const n = used.get(`${method}\0${key}`) ?? 0;
    used.set(`${method}\0${key}`, n + 1);
    let entry = matching[n];
    if (entry === undefined && matching.length > 0 && repeatable.has(method)) {
      entry = matching[matching.length - 1];
    }
    if (entry === undefined) {
      divergences.push(
        `${scenario}: ${method} \`${key}\` call ${n + 1} was never made by the reference runner`,
      );
      throw new Error('transcript divergence');
    }
    if (compared) {
      const sent = canonical(request);
      const want = canonical(entry.request);
      if (sent !== want) {
        divergences.push(
          `${scenario}: ${method} \`${key}\` call ${n + 1} sent ${sent}, the reference runner sent ${want}`,
        );
        throw new Error('transcript divergence');
      }
    }
    if (entry.error === 'unsupported') throw unsupported('recorded');
    if (entry.error !== null) throw new Error(entry.error);
    return entry.result;
  };
  const answering = {
    identity: async () => ({ name: 'transcript', version: '1' }),
    beginScenario: async (context) => {
      scenario = context.scenario;
      used = new Map();
    },
    endScenario: async () => {},
    executeCommand: async (request) =>
      result(
        next(
          'execute_command',
          request.command,
          {
            actor: request.actor || null,
            caller: request.caller ?? null,
            input: request.input ?? {},
          },
          true,
        ),
      ),
    queryView: async (request) => {
      const recorded = next('query_view', request.view, request, false);
      return { rows: recorded.rows ?? [], total: recorded.total ?? undefined };
    },
    observeEvents: async (request) =>
      next('observe_events', request.event, request, false).map((event) => ({
        event: event.event,
        payload: event.payload,
      })),
    configureExternalOutcome: async (control) => {
      next('configure_external_outcome', `${control.command}/${control.outcome}`, {}, false);
    },
    redeliverEvent: async (request) => {
      next('redeliver_event', request.event, {}, false);
    },
    observeInvocations: async (request) =>
      next('observe_invocations', `${request.binding}|${request.command}`, {}, false),
  };
  if (bare !== '1') {
    // An await of a command the system issues itself (beyond10x/ess#542): the question is the
    // instance, the declared bound and the compared field names, never the runner's deadline.
    answering.awaitOutcome = async (request) => {
      const recorded = next(
        'await_outcome',
        request.command,
        {
          instance: request.instance,
          within: request.within ?? null,
          compared: [...request.compared].sort(),
        },
        true,
      );
      const elapsedMillis = recorded.elapsed_ms ?? undefined;
      if (recorded.answer === 'reached') {
        return {
          reached: true,
          result: result(recorded.result),
          input: recorded.input ?? {},
          elapsedMillis,
        };
      }
      return { reached: false, elapsedMillis };
    };
  }
  return answering;
};

const scope = {
  diagnostic() {},
  skip() {},
  async test(_name, body) {
    try {
      await body(this);
    } catch {}
  },
};
try {
  await runWith(scope, target, readFileSync(suiteFile, 'utf8'));
} catch (error) {
  console.error(String(error));
  process.exitCode = /^Error: \d+ required scenario\(s\) unsupported/.test(String(error)) ? 1 : 2;
}
writeFileSync(divergenceFile, divergences.join('\n'));
