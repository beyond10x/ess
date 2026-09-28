// The #174 job list (`tests/fixtures/view-paging.yaml`), the target `tests/view_paging.rs`
// implements in Rust. Modes: `correct`, `shared`, `ignores-paging`, `ignores-page`, `one-based`,
// `pages-before-ordering`, `total-is-page-length`, `no-total`. `shared` is correct: a correct
// target other users share.
import { unsupported } from './dist/runtime.js';

const mode = process.env.ESS_TARGET_MODE ?? 'correct';
let sequence = 0;
const byId = ([a], [b]) => (a < b ? -1 : a > b ? 1 : 0);
const id = (prefix, n) => `00000000-0000-4000-${prefix}-${String(n).padStart(12, '0')}`;

class Jobs {
  /** `[job_id, type]`, in the order they were made. */
  rows = [];
  identity() {
    return { name: 'view-paging', version: '1' };
  }
  beginScenario() {}
  endScenario() {}
  executeCommand({ command, input }) {
    if (command !== 'demo.jobs.CreateJob') {
      throw new Error(`unexpected command ${command}`);
    }
    const kind = typeof input.type === 'string' ? input.type : '';
    sequence += 1;
    // Minted descending, so the order rows are made in is the reverse of `job_id asc`.
    const job = id('8000', 900_000 - sequence);
    if (mode === 'shared') {
      this.rows.push([id('8000', sequence), kind]);
      this.rows.push([id('9000', sequence), kind]);
    }
    this.rows.push([job, kind]);
    return {
      outcome: 'created',
      consistency: 'write',
      directEvents: [{ event: 'demo.jobs.JobCreated', payload: { job_id: job, type: kind } }],
    };
  }
  queryView({ params }) {
    const wanted = typeof params.type === 'string' ? params.type : undefined;
    const admitted = this.rows.filter(([, kind]) => wanted === undefined || wanted === kind);
    const ordered = [...admitted].sort(byId);
    const whole = (value) => (Number.isInteger(value) && value >= 0 ? value : undefined);
    const [page, size] = [whole(params.page), whole(params.size)];
    let slice = ordered;
    if (page !== undefined && size !== undefined && mode !== 'ignores-paging') {
      const offset =
        mode === 'ignores-page' ? 0 : mode === 'one-based' ? Math.max(page - 1, 0) * size : page * size;
      const source = mode === 'pages-before-ordering' ? admitted : ordered;
      slice = source.slice(offset, offset + size).sort(byId);
    }
    const result = { rows: slice.map(([job, kind]) => ({ job_id: job, type: kind })) };
    if (mode === 'total-is-page-length') {
      result.total = result.rows.length;
    } else if (mode !== 'no-total') {
      result.total = admitted.length;
    }
    return result;
  }
  configureExternalOutcome() {
    throw unsupported('the model declares none');
  }
  redeliverEvent() {
    throw unsupported('unused');
  }
  observeEvents() {
    throw unsupported('unused');
  }
}

export function makeTarget() {
  return new Jobs();
}
