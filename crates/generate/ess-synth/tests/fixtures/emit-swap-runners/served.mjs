// The generated TypeScript runner's target for a generated service, as `served_test.go` is the Go
// runner's: every request goes over the line protocol the harness under `ESS_SERVED_BINARY` speaks,
// with the route table in `ESS_SERVED_ROUTES` as `name method path` triples. Nothing here decides a
// branch or an event: the generated service answers.
import { spawn } from 'node:child_process';
import { createInterface } from 'node:readline';
import { ErrUnsupported } from './dist/runtime.js';

export function servedTarget() {
  const routes = (process.env.ESS_SERVED_ROUTES ?? '').split(/\s+/).filter((word) => word !== '');
  const child = spawn(process.env.ESS_SERVED_BINARY, routes, {
    stdio: ['pipe', 'pipe', 'inherit'],
  });
  const waiting = [];
  createInterface({ input: child.stdout }).on('line', (line) => waiting.shift()(JSON.parse(line)));
  const ask = (request) =>
    new Promise((resolve) => {
      waiting.push(resolve);
      child.stdin.write(`${JSON.stringify(request)}\n`);
    });
  let published = [];
  let sequence = 0;
  return {
    identity: () => ({ name: 'emit-swap-served-generated', version: '1' }),
    async beginScenario() {
      const answer = await ask({ op: 'reset' });
      if (answer.ok !== true) throw new Error(`opening a scenario: ${JSON.stringify(answer)}`);
      published = [];
      sequence = 0;
    },
    endScenario() {
      published = [];
    },
    async executeCommand({ command, actor, input }) {
      const answer = await ask({
        op: 'command',
        command,
        actor: actor === '' ? null : actor,
        body: JSON.stringify(input),
      });
      const reply = answer.answer ?? {};
      if (answer.status === 403 && reply.refused === 'not granted') {
        return { notGranted: true, notGrantedActor: reply.actor ?? '' };
      }
      if (answer.status === 501) return {};
      if (typeof reply.outcome !== 'string') {
        throw new Error(`invoking \`${command}\`: ${JSON.stringify(answer)}`);
      }
      const result = { outcome: reply.outcome, directEvents: [] };
      if (typeof reply.error === 'string') {
        result.error = reply.error;
        result.errorPayload = Object.fromEntries(
          Object.entries(reply.payload ?? {}).filter(([, value]) => value !== null),
        );
      }
      for (const entry of reply.published ?? []) {
        const event = { event: entry.event, payload: entry.payload ?? {} };
        result.directEvents.push(event);
        published.push(event);
      }
      sequence += 1;
      result.consistency = `seq:${sequence}`;
      return result;
    },
    async queryView({ view }) {
      const answer = await ask({ op: 'view', view });
      const rows = answer.answer?.rows;
      if (!Array.isArray(rows)) throw new Error(`reading \`${view}\`: ${JSON.stringify(answer)}`);
      return { rows };
    },
    observeEvents({ event }) {
      return published.filter((seen) => seen.event === event);
    },
    configureExternalOutcome() {
      throw ErrUnsupported;
    },
    redeliverEvent() {
      throw ErrUnsupported;
    },
    observeInvocations() {
      throw ErrUnsupported;
    },
    close() {
      child.stdin.end();
    },
  };
}
