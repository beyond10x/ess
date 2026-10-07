// An implementation of `restart.yaml` that runs as a child process over a state directory, and the
// restart faults the explorer must catch (beyond10x/ess#297).
//
// `beginScenario` makes a fresh state directory and starts `restart-backend.mjs` in a new Node
// process. `restart` closes that process's input, waits for it to exit and starts a new process over
// the same directory, so a restart here is a real process restart. The mode selects what the
// backend keeps:
//
//   durable          every entry and the identity counter are written to the directory
//   counter-reset    every entry is written and the counter is not: each process counts from zero
//   volatile         nothing is written: a restarted process starts empty
//   no-restart       the target defines no `restart` at all
//   restart-refused  `restart` throws ErrUnsupported
//   token            durable, and every command answers a consistency token that every later read
//                    must name: a read after a restart at any other token is refused
//
// `restart_target.go` is the same target in Go.

import { spawn } from 'node:child_process';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createInterface } from 'node:readline';
import { fileURLToPath } from 'node:url';

import { unsupported } from './dist/index.js';

const BACKEND = fileURLToPath(new URL('./restart-backend.mjs', import.meta.url));

/** The process id of every backend any target started. */
export const processes = new Set();

export function newTarget(mode) {
  let dir = '';
  let child = null;
  let lines = null;
  // The commands answered; `token` mode answers the nth with token `tn`.
  let issued = 0;

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
      env: { ...process.env, ESS_RESTART_BACKEND: mode, ESS_RESTART_DIR: dir },
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

  const target = {
    identity: () => ({ name: 'restart-target', version: '1' }),
    beginScenario: async () => {
      dir = mkdtempSync(join(tmpdir(), 'ess-restart-'));
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
        throw new Error(
          `read at token ${JSON.stringify(request.atLeast)}, the last command answered t${issued}`,
        );
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
  };
  if (mode !== 'no-restart') {
    target.restart = async () => {
      if (mode === 'restart-refused') throw unsupported('this deployment cannot be restarted');
      await stop();
      await start();
    };
  }
  return target;
}
