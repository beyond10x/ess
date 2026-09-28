// The #176 queue (`tests/fixtures/defined-over-optional-aggregates.yaml`), the target
// `tests/defined_over_optional_aggregates.rs` implements in Rust: metrics are held only while
// paused. Modes: `correct`, `keeps-metrics-on-resume`, `empty-metrics-on-resume` (resume leaves an
// empty struct, which is still defined).
import { unsupported } from './dist/runtime.js';

const mode = process.env.ESS_TARGET_MODE ?? 'correct';
let minted = 0;

class Queues {
  rows = [];
  identity() {
    return { name: 'queues-fixture', version: '1' };
  }
  beginScenario() {}
  endScenario() {}
  executeCommand({ command, input }) {
    minted += 1;
    const consistency = `seq:${minted}`;
    const took = (outcome, event, id) => ({
      outcome,
      consistency,
      directEvents: [{ event, payload: { queue_id: id } }],
    });
    // The row a command acts on where it is in `from`; otherwise the declared conflict, naming the
    // state it is in where there is a row at all.
    const find = (from) => {
      const row = this.rows.find((held) => held.queue_id === input.queue_id);
      if (row !== undefined && row.state === from) {
        return [row, undefined];
      }
      const refused = { outcome: 'wrong-state', error: 'demo.queue.QueueStateConflict', consistency };
      if (row !== undefined) {
        refused.errorFields = { state: row.state };
      }
      return [undefined, refused];
    };
    switch (command) {
      case 'demo.queue.OpenQueue': {
        const id = `00000000-0000-4000-8000-${String(minted).padStart(12, '0')}`;
        this.rows.push({ queue_id: id, state: 'Running' });
        return took('opened', 'demo.queue.QueueOpened', id);
      }
      case 'demo.queue.PauseQueue': {
        const [row, refused] = find('Running');
        if (row === undefined) {
          return refused;
        }
        row.state = 'Paused';
        row.metrics = input.metrics;
        return took('paused', 'demo.queue.QueuePaused', row.queue_id);
      }
      case 'demo.queue.ResumeQueue': {
        const [row, refused] = find('Paused');
        if (row === undefined) {
          return refused;
        }
        row.state = 'Running';
        if (mode === 'empty-metrics-on-resume') {
          row.metrics = {};
        } else if (mode !== 'keeps-metrics-on-resume') {
          row.metrics = null;
        }
        return took('resumed', 'demo.queue.QueueResumed', row.queue_id);
      }
      default:
        throw unsupported(`command ${command}`);
    }
  }
  queryView({ view }) {
    if (view !== 'demo.queue.QueueById') {
      throw unsupported(`view ${view}`);
    }
    return { rows: this.rows.map((row) => ({ ...row })) };
  }
  observeEvents() {
    return [];
  }
  configureExternalOutcome() {
    throw new Error('nothing here is externally decided');
  }
  redeliverEvent() {
    throw new Error('no bindings');
  }
}

export function makeTarget() {
  return new Queues();
}
