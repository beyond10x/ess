// Adversary target for `restart.yaml` (beyond10x/ess#297): `restart-target.mjs` with more modes.
//
//   durable               every entry and the identity counter are written to the directory
//   counter-reset         every entry is written and the counter is not: each process counts from zero
//   counter-reset-second  the counter is written, and the third process (after the second restart)
//                         starts it from zero again
//   constant              every creation mints the same identity, restart or not
//   pretend               the counter-reset backend, and a `restart` that answers without restarting
//   token                 the durable backend, and every read must name the last command's token
//
// `adversary_restart_target.go` is the same target in Go.

import { spawn } from 'node:child_process';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createInterface } from 'node:readline';
import { fileURLToPath } from 'node:url';

import { unsupported } from './dist/index.js';

const BACKEND = fileURLToPath(new URL('./adversary-restart-backend.mjs', import.meta.url));

/** The process id of every backend any target started. */
export const processes = new Set();

export function newTarget(mode) {
  const backend = mode === 'pretend' ? 'counter-reset' : mode === 'token' ? 'durable' : mode;
  // The consistency token of the last command, which every read must name in `token` mode.
  let issued = 0;
  let dir = '';
  let child = null;
  let lines = null;

  const read = async () => {
    const next = await lines.next();
    if (next.done) throw new Error('the backend answered nothing');
    return JSON.parse(next.value);
  };
  const call = async (request) => {
    child.stdin.write(`${JSON.stringify(request)}\n`);
    return read();
  };
  const start = async () => {
    child = spawn(process.execPath, [BACKEND], {
      env: { ...process.env, ESS_RESTART_BACKEND: backend, ESS_RESTART_DIR: dir },
      stdio: ['pipe', 'pipe', 'inherit'],
    });
    lines = createInterface({ input: child.stdout })[Symbol.asyncIterator]();
    const ready = await read();
    if (ready.ready !== child.pid) {
      throw new Error(`the backend answered as process ${ready.ready}, started as ${child.pid}`);
    }
    processes.add(child.pid);
  };
  const stop = async () => {
    if (child === null) return;
    const running = child;
    child = null;
    if (running.exitCode === null && running.signalCode === null) {
      const exited = new Promise((resolve) => running.once('exit', resolve));
      running.stdin.end();
      await exited;
    }
  };

  return {
    identity: () => ({ name: 'adversary-restart-target', version: '1' }),
    beginScenario: async () => {
      dir = mkdtempSync(join(tmpdir(), 'ess-adv-restart-'));
      await start();
    },
    endScenario: async () => {
      await stop();
      rmSync(dir, { recursive: true, force: true });
    },
    executeCommand: async (request) => {
      if (request.command !== 'restart.ledger.RecordEntry') {
        throw unsupported(`${request.command} is not a command of restart.yaml`);
      }
      const answer = await call({ op: 'record', amount: request.input.amount });
      issued += 1;
      return {
        outcome: 'recorded',
        consistency: mode === 'token' ? `t${issued}` : '',
        directEvents: [
          {
            event: 'restart.ledger.EntryRecorded',
            payload: { entry_id: answer.entry_id, amount: request.input.amount },
          },
        ],
      };
    },
    queryView: async (request) => {
      if (request.view !== 'restart.ledger.Entries') {
        throw unsupported(`${request.view} is not a view of restart.yaml`);
      }
      if (mode === 'token' && request.atLeast !== `t${issued}`) {
        throw new Error(`read at token ${JSON.stringify(request.atLeast)}, the last command answered t${issued}`);
      }
      const answer = await call({ op: 'list' });
      return { rows: answer.rows };
    },
    observeEvents: () => [],
    configureExternalOutcome: () => {
      throw unsupported('no external outcome');
    },
    redeliverEvent: () => {
      throw unsupported('no redelivery');
    },
    observeInvocations: () => {
      throw unsupported('no bindings');
    },
    restart: async () => {
      if (mode === 'pretend') return;
      await stop();
      await start();
    },
  };
}
