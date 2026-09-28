// The #171 jobs model (`tests/fixtures/current-time-guard.yaml`): a job may not start more than 60
// seconds in the past, by the machine's clock. Modes: `correct`, `tolerates-120s`, `tolerates-0s`,
// `reads-a-fixed-clock` (decides against 2023, not now).
const mode = process.env.ESS_TARGET_MODE ?? 'correct';
const tolerance = mode === 'tolerates-120s' ? 120 : mode === 'tolerates-0s' ? 0 : 60;
const now = () => (mode === 'reads-a-fixed-clock' ? 1_700_000_000_000 : Date.now());
let minted = 0;

class Jobs {
  rows = [];
  identity() {
    return { name: 'jobs-fixture', version: '1' };
  }
  beginScenario() {}
  endScenario() {}
  executeCommand({ command, input }) {
    minted += 1;
    const consistency = `seq:${minted}`;
    if (command !== 'demo.jobs.ScheduleJob') {
      throw new Error(`unexpected command ${command}`);
    }
    const startsAt = Date.parse(input.starts_at);
    if (Number.isNaN(startsAt)) {
      throw new Error(`${input.starts_at} is no instant`);
    }
    if (startsAt < now() - tolerance * 1000) {
      return { outcome: 'start-in-past', error: 'demo.jobs.StartInPast', consistency };
    }
    const id = `00000000-0000-4000-8000-${String(minted).padStart(12, '0')}`;
    this.rows.push({ job_id: id, starts_at: input.starts_at });
    return {
      outcome: 'scheduled',
      consistency,
      directEvents: [
        { event: 'demo.jobs.JobScheduled', payload: { job_id: id, starts_at: input.starts_at } },
      ],
    };
  }
  queryView({ view }) {
    if (view !== 'demo.jobs.JobDetails') {
      throw new Error(`unexpected view ${view}`);
    }
    return { rows: [...this.rows] };
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
  return new Jobs();
}
