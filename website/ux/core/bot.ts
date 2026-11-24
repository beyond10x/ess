// The screen-only user (skill § 7). Deterministic code, no model. It acts only on controls that are
// wholly inside the viewport and not covered at their centre, found by accessible role and name,
// and it decides every move from what is on screen: it never reads where an out-of-view or hidden
// control sits. A click, a key press, a scroll gesture or a typed query is one action.
//
// When a control it is to use is not on screen it looks, in this order: through the visible scroll
// regions (a sidebar, an open menu), through a container a visible control opens (the menu
// button), then down the page to the end and back up; a target to reveal it looks for in the page
// first. A collapsed category is opened only when the card names it (an `open` step). With the
// keyboard it presses Tab until the focus is on the control, or until the focus has gone round the
// whole page.
import type { ElementHandle, JSHandle, Page } from '@playwright/test';
import { probeReadingLoad, probeVisible } from './probes';
import { taskMetrics, type StateRecord, type TaskMetrics } from './metrics';

export interface Control {
  role: Parameters<Page['getByRole']>[0];
  name: string; // a case-insensitive regular expression over the accessible name
}

export type Step =
  | { click: Control } // follow a visible control toward the target
  | { open: Control } // make sure a disclosure (a sidebar category) is open: no action if it is
  | { reveal: Control } // bring the target into view: the completion state
  | { theme: 'dark' | 'light'; via: Control }; // press `via` until the page shows that theme

export interface Truth {
  path: string;
  target?: Control;
  // The id of the heading whose section holds the target: the target counts only under it.
  anchor?: string;
  // A control that must be in view beside the target: in its table row for a cell, else in its
  // section.
  beside?: Control;
  theme?: 'dark' | 'light';
}

export interface Task {
  name: string;
  start: string;
  steps: Step[];
  truth: Truth;
}

// A control the reader sees that opens a container: the menu button. `hiddenWhen` is CSS that
// matches while the container is closed; `close` is the control that closes it again. A toggle
// given as CSS (a container's own control, such as a collapsed sidebar category's caret) is never
// used on the user's own: the card names such a control in an `open` step.
export interface Revealer {
  inside?: string;
  hiddenWhen?: string;
  toggle: string | Control;
  close?: Control;
}

export interface BotConfig {
  regions: Array<{ name: string; selector: string }>;
  primary: string[];
  revealers: Revealer[];
  mode: 'pointer' | 'keyboard';
  maxSteers: number;
  maxTabs: number;
}

export interface TaskRun {
  task: string;
  mode: 'pointer' | 'keyboard';
  completed: boolean;
  reason: string;
  path: string;
  states: StateRecord[];
  metrics: TaskMetrics;
  completion_miss: number;
  false_complete: number;
}

type Handle = ElementHandle<Element>;

const pattern = (c: Control) => new RegExp(c.name, 'i');
const describe = (c: Control) => `${c.role} /${c.name}/`;

// After an action: a full navigation may still be committing, so a destroyed context is retried.
async function settle(page: Page): Promise<void> {
  for (let attempt = 0; ; attempt++) {
    try {
      await page.waitForLoadState('load');
      await page.waitForLoadState('networkidle', { timeout: 3_000 }).catch(() => undefined);
      await frames(page);
      return;
    } catch (error) {
      if (attempt >= 5 || !/context was destroyed|navigat/i.test(String(error))) throw error;
      await page.waitForTimeout(100);
    }
  }
}

async function frames(page: Page): Promise<void> {
  await page.evaluate(() => new Promise<void>((resolve) => requestAnimationFrame(() => requestAnimationFrame(() => resolve()))));
}

async function regionOf(handle: Handle, config: BotConfig): Promise<string> {
  return handle.evaluate((el, regions) => {
    for (const r of regions) if (el.closest(r.selector)) return r.name;
    return 'other';
  }, config.regions);
}

// What the reader sees of a control: copies on screen and usable, and copies on screen but covered
// (under a sticky bar). Copies off screen or hidden are not the reader's to know.
interface Seen {
  usable: Handle[];
  covered: Handle[];
}

async function look(page: Page, control: Control, wait = 0): Promise<Seen> {
  const seen: Seen = { usable: [], covered: [] };
  const locator = page.getByRole(control.role, { name: pattern(control), includeHidden: true });
  // A client-side navigation renders its page a moment after the URL changes: wait for the control
  // to exist (not to be visible) before deciding it is not there. Waiting costs no action.
  if (wait > 0) await locator.first().waitFor({ state: 'attached', timeout: wait }).catch(() => undefined);
  for (const handle of (await locator.elementHandles()) as Handle[]) {
    const v = await handle.evaluate(probeVisible);
    const shown = await handle.evaluate((el) => el.checkVisibility({ opacityProperty: true, visibilityProperty: true }));
    if (!shown || !v.inViewport) continue;
    (v.covered ? seen.covered : seen.usable).push(handle);
  }
  return seen;
}

// The id of the heading whose section holds an element: the element itself if it is such a
// heading, else the last heading with an id before it in the document.
async function sectionOf(handle: Handle): Promise<string> {
  return handle.evaluate((el) => {
    let id = '';
    for (const h of document.querySelectorAll('h1[id], h2[id], h3[id], h4[id], h5[id], h6[id]')) {
      if (h === el || h.compareDocumentPosition(el) & Node.DOCUMENT_POSITION_FOLLOWING) id = h.id;
      else break;
    }
    return id;
  });
}

export class ScreenOnlyUser {
  private states: StateRecord[] = [];
  private completionMiss = 0;
  private falseComplete = 0;

  constructor(
    private readonly page: Page,
    private readonly config: BotConfig,
  ) {}

  private viewport() {
    return this.page.viewportSize() ?? { width: 1280, height: 800 };
  }

  private async press(handle: Handle): Promise<void> {
    if (this.config.mode === 'pointer') {
      const box = await handle.boundingBox();
      if (!box) throw new Error('the control has no box');
      await this.page.mouse.click(box.x + box.width / 2, box.y + box.height / 2);
    } else {
      await this.page.keyboard.press('Enter');
    }
    await settle(this.page);
  }

  // Tab until the focus is on a copy of the control, until the focus has gone round the page, or
  // up to maxTabs presses. Returns the presses made and the focused copy, if reached.
  private async tabTo(control: Control): Promise<{ presses: number; handle: Handle | null }> {
    const locator = this.page.getByRole(control.role, { name: pattern(control) });
    const focused = async (): Promise<Handle | null> => {
      for (const h of (await locator.elementHandles()) as Handle[]) {
        if (await h.evaluate((el) => el === document.activeElement || el.contains(document.activeElement))) return h;
      }
      return null;
    };
    let first: JSHandle | null = null;
    for (let presses = 0; ; presses++) {
      const h = await focused();
      if (h) return { presses, handle: h };
      if (presses >= this.config.maxTabs) return { presses, handle: null };
      await this.page.keyboard.press('Tab');
      const now = await this.page.evaluateHandle(() => document.activeElement);
      if (first === null) first = now;
      else if (await this.page.evaluate(([a, b]) => a === b, [first, now])) return { presses: presses + 1, handle: null };
    }
  }

  // One look and, if the control is on screen, one use of it. Returns the actions it took, or null
  // when the control is not on screen (with the keyboard: not reached by Tab).
  private async use(control: Control, wait: number, done?: (h: Handle) => Promise<boolean>): Promise<{ actions: number; handle: Handle } | null> {
    if (this.config.mode === 'keyboard') {
      await look(this.page, control, wait);
      const { presses, handle } = await this.tabTo(control);
      if (!handle) {
        this.tabsSpent += presses;
        return null;
      }
      if (done && (await done(handle))) return { actions: presses, handle };
      await this.press(handle);
      return { actions: presses + 1, handle };
    }
    const handle = (await look(this.page, control, wait)).usable[0];
    if (!handle) return null;
    if (done && (await done(handle))) return { actions: 0, handle };
    await this.press(handle);
    return { actions: 1, handle };
  }

  private tabsSpent = 0;

  // A scroll gesture (or a Page Down / Page Up) over the page or a region: 80 % of what it shows.
  // Returns false when there was nothing left to scroll that way.
  private async scrollOnce(region: string | null, down: boolean): Promise<boolean> {
    return this.page.evaluate(
      ([css, d]) => {
        const el = css ? document.querySelector(css) : null;
        const box = el ?? document.scrollingElement ?? document.documentElement;
        const shown = el ? el.clientHeight : window.innerHeight;
        const before = box.scrollTop;
        const room = d ? box.scrollHeight - shown - before : before;
        if (room < 1) return false;
        box.scrollTo({ top: before + (d ? 1 : -1) * Math.round(shown * 0.8), behavior: 'instant' });
        return true;
      },
      [region, down] as const,
    );
  }

  // The scroll regions on screen, other than the page: marked so they can be scrolled by CSS.
  private async regions(): Promise<string[]> {
    return this.page.evaluate(() => {
      const out: string[] = [];
      let n = 0;
      for (const el of document.querySelectorAll('body *')) {
        const s = getComputedStyle(el);
        if (!(s.overflowY === 'auto' || s.overflowY === 'scroll') || el.scrollHeight <= el.clientHeight + 1) continue;
        const r = el.getBoundingClientRect();
        if (r.width < 1 || r.height < 1 || r.bottom <= 0 || r.top >= window.innerHeight || r.right <= 0 || r.left >= window.innerWidth) continue;
        if (!el.checkVisibility({ visibilityProperty: true, opacityProperty: true })) continue;
        el.setAttribute('data-ux-region', String(n));
        out.push(`[data-ux-region="${n++}"]`);
      }
      return out;
    });
  }

  // Scroll one region (or the page) from where it is down to its end, then up to its top, looking
  // after each gesture. Returns the gestures made and whether the control came on screen.
  private async scan(control: Control, region: string | null, budget: number): Promise<{ gestures: number; found: boolean }> {
    let gestures = 0;
    const startedAtTop = await this.page.evaluate((css) => (css ? document.querySelector(css)?.scrollTop ?? 0 : window.scrollY) < 1, region);
    for (const down of startedAtTop ? [true] : [true, false]) {
      while (gestures < budget && (await this.scrollOnce(region, down))) {
        gestures++;
        await frames(this.page);
        if ((await look(this.page, control)).usable.length > 0) return { gestures, found: true };
      }
    }
    return { gestures, found: false };
  }

  private async revealerApplies(revealer: Revealer): Promise<Handle | null> {
    if (typeof revealer.toggle === 'string') return null;
    if (revealer.hiddenWhen && !(await this.page.evaluate((css) => !!document.querySelector(css), revealer.hiddenWhen))) return null;
    return (await look(this.page, revealer.toggle)).usable[0] ?? null;
  }

  // Bring a control on screen without using it. Returns the actions and steers spent and whether
  // it is on screen now. `navigation`: the control leads somewhere (click), so the navigation on
  // screen is searched before the page; a target in the content (reveal) is looked for in the page first.
  private async seek(control: Control, navigation: boolean): Promise<{ actions: number; steers: number; found: boolean }> {
    let actions = 0;
    let steers = 0;
    const budget = () => this.config.maxSteers - steers;
    const onScreen = async () => (this.config.mode === 'keyboard' ? false : (await look(this.page, control)).usable.length > 0);
    const scanRegions = async () => {
      for (const region of await this.regions()) {
        const r = await this.scan(control, region, budget());
        actions += r.gestures;
        steers += r.gestures;
        if (r.found) return true;
      }
      return false;
    };
    const scanPage = async () => {
      const r = await this.scan(control, null, budget());
      actions += r.gestures;
      steers += r.gestures;
      return r.found;
    };
    const opened: Revealer[] = [];
    const tryRevealers = async () => {
      const tried = new Set<string>();
      for (;;) {
        let progressed = false;
        for (const revealer of this.config.revealers) {
          if (typeof revealer.toggle === 'string' || tried.has(revealer.toggle.name)) continue;
          if (!(await this.revealerApplies(revealer))) continue;
          tried.add(revealer.toggle.name);
          const used = await this.use(revealer.toggle, 0);
          if (!used) continue;
          actions += used.actions;
          steers += 1;
          opened.push(revealer);
          progressed = true;
          if (this.config.mode === 'keyboard' || (await onScreen()) || (await scanRegions())) return true;
          break;
        }
        if (!progressed) return false;
      }
    };
    const closeOpened = async () => {
      for (const revealer of opened.reverse()) {
        if (!revealer.close) continue;
        const used = await this.use(revealer.close, 0);
        if (used) actions += used.actions;
      }
    };
    if (navigation) {
      if (this.config.mode === 'pointer' && (await scanRegions())) return { actions, steers, found: true };
      if (await tryRevealers()) return { actions, steers, found: true };
      await closeOpened();
      if (this.config.mode === 'keyboard') return { actions, steers, found: false };
      if (await scanPage()) return { actions, steers, found: true };
    } else {
      if ((await scanPage()) || (await scanRegions())) return { actions, steers, found: true };
    }
    return { actions, steers, found: false };
  }

  private async glanceOf(handle: Handle, steers: number): Promise<1 | 2 | 3> {
    if (steers > 0) return 3;
    return this.config.primary.includes(await regionOf(handle, this.config)) ? 1 : 2;
  }

  private path(): string {
    return new URL(this.page.url()).pathname.replace(/\/$/, '') || '/';
  }

  private async noteFalseComplete(truth: Truth): Promise<void> {
    if (!truth.target || this.path() === truth.path.replace(/\/$/, '')) return;
    if ((await look(this.page, truth.target)).usable.length > 0) this.falseComplete++;
  }

  // Use a control: look, and when it is not on screen seek it, then use it. `done`: the control is
  // in the wanted state already (an open category), so it is not pressed.
  private async click(control: Control, label: string, done?: (h: Handle) => Promise<boolean>): Promise<StateRecord> {
    // The first look waits for a client-side navigation to render the page; the state's words are
    // read after it, so they are the words of the page the reader acts on.
    const seen = await look(this.page, control, 2_000);
    const words = await this.page.evaluate(probeReadingLoad);
    this.tabsSpent = 0;
    let glance: 1 | 2 | 3 | null = null;
    if (this.config.mode === 'pointer' && seen.usable[0]) glance = await this.glanceOf(seen.usable[0], 0);
    const direct = await this.use(control, 2_000, done);
    if (direct) return { step: label, glance: glance ?? (direct.actions > 1 ? 3 : await this.glanceOf(direct.handle, 0)), actions: direct.actions, steers: 0, words };
    const sought = await this.seek(control, true);
    let actions = sought.actions + this.tabsSpent;
    if (sought.found) {
      this.tabsSpent = 0;
      const used = await this.use(control, 0, done);
      if (used) return { step: label, glance: 3, actions: actions + used.actions, steers: sought.steers, words };
      actions += this.tabsSpent;
    }
    const why = sought.steers >= this.config.maxSteers ? `after ${sought.steers} steers` : 'and nothing on screen reveals it';
    throw new Error(`${label}: the control is not on screen ${why} (${actions} actions)`);
  }

  // A disclosure the reader opens on the way (a sidebar category): already open costs a look and
  // no action.
  private async open(control: Control, label: string): Promise<StateRecord> {
    return this.click(control, label, async (h) => (await h.getAttribute('aria-expanded')) === 'true');
  }

  private async arrivedCovered(target: Control): Promise<void> {
    const seen = await look(this.page, target);
    if (seen.usable.length === 0 && seen.covered.length > 0) this.completionMiss++;
  }

  private async reveal(target: Control, label: string): Promise<StateRecord> {
    const first = (await look(this.page, target, 2_000)).usable[0];
    const words = await this.page.evaluate(probeReadingLoad);
    if (first) return { step: label, glance: await this.glanceOf(first, 0), actions: 0, steers: 0, words };
    // On arrival the target is on screen but under something: the reader can miss it (gate 3).
    await this.arrivedCovered(target);
    let actions = 0;
    // The reader's first move on a long page: a visible link that names the target (the table of
    // contents), if there is one.
    if (this.config.mode === 'pointer') {
      const link = (await look(this.page, { role: 'link', name: `^${target.name.replace(/^\^|\$$/g, '')}$` })).usable[0];
      if (link) {
        await this.press(link);
        actions += 1;
        const after = (await look(this.page, target)).usable[0];
        if (after) return { step: label, glance: 2, actions, steers: 0, words };
        await this.arrivedCovered(target);
      }
    }
    if (this.config.mode === 'keyboard') {
      // Page Down from where the page is, then Page Up: the keyboard's scroll gestures.
      const r = await this.scan(target, null, this.config.maxSteers);
      actions += r.gestures;
      if (r.found) return { step: label, glance: 3, actions, steers: r.gestures, words };
      throw new Error(`${label}: the target is not on screen after ${r.gestures} Page Down and Page Up presses`);
    }
    const sought = await this.seek(target, false);
    actions += sought.actions;
    if (sought.found) return { step: label, glance: 3, actions, steers: sought.steers, words };
    throw new Error(`${label}: the target is not on screen after ${sought.steers} steers, in any visible or revealable form`);
  }

  private async theme(theme: 'dark' | 'light', via: Control, label: string): Promise<StateRecord> {
    const words = await this.page.evaluate(probeReadingLoad);
    let actions = 0;
    let steers = 0;
    let glance: 1 | 2 | 3 = 1;
    for (let presses = 0; presses < 3; presses++) {
      if ((await this.page.getAttribute('html', 'data-theme')) === theme) break;
      const record = await this.click(via, label);
      actions += record.actions;
      steers += record.steers;
      glance = Math.max(glance, record.glance) as 1 | 2 | 3;
    }
    if ((await this.page.getAttribute('html', 'data-theme')) !== theme) {
      throw new Error(`${label}: the page does not show the ${theme} theme after 3 presses`);
    }
    return { step: label, glance, actions, steers, words };
  }

  // The truth at the end: the path, the target on screen under its anchor with its neighbour
  // beside it, the theme. Returns the reason it does not hold, or ''.
  private async judge(truth: Truth): Promise<string> {
    if (this.path() !== truth.path.replace(/\/$/, '')) return `ended on ${this.path()}, the truth is ${truth.path}`;
    if (truth.target) {
      let targets = (await look(this.page, truth.target)).usable;
      if (targets.length === 0) return `the target ${describe(truth.target)} is not in view at the end`;
      if (truth.anchor) {
        const under: Handle[] = [];
        for (const t of targets) if ((await sectionOf(t)) === truth.anchor) under.push(t);
        if (under.length === 0) return `the target ${describe(truth.target)} is in view, but not under the anchor #${truth.anchor}`;
        targets = under;
      }
      if (truth.beside) {
        const beside = (await look(this.page, truth.beside)).usable;
        let found = false;
        for (const t of targets) {
          for (const b of beside) {
            const together = await t.evaluate((el, other) => {
              const row = el.closest('tr');
              return row ? row.contains(other) : true;
            }, b);
            if (together && (!truth.anchor || (await sectionOf(b)) === (await sectionOf(t)))) found = true;
          }
        }
        if (!found) return `the target ${describe(truth.target)} is in view, but no ${describe(truth.beside)} beside it`;
      }
    }
    if (truth.theme && (await this.page.getAttribute('html', 'data-theme')) !== truth.theme) {
      return `the page ends in the ${await this.page.getAttribute('html', 'data-theme')} theme, the truth is ${truth.theme}`;
    }
    return '';
  }

  async run(task: Task, origin: string): Promise<TaskRun> {
    this.states = [];
    this.completionMiss = 0;
    this.falseComplete = 0;
    let reason = '';
    await this.page.goto(origin + task.start);
    await settle(this.page);
    try {
      for (const [index, step] of task.steps.entries()) {
        if ('click' in step) {
          this.states.push(await this.click(step.click, `${index + 1} click ${describe(step.click)}`));
          await this.noteFalseComplete(task.truth);
        } else if ('open' in step) {
          this.states.push(await this.open(step.open, `${index + 1} open ${describe(step.open)}`));
        } else if ('reveal' in step) {
          this.states.push(await this.reveal(step.reveal, `${index + 1} reveal ${describe(step.reveal)}`));
        } else {
          this.states.push(await this.theme(step.theme, step.via, `${index + 1} theme ${step.theme}`));
        }
      }
    } catch (error) {
      reason = (error as Error).message;
    }
    if (reason === '') reason = await this.judge(task.truth);
    return {
      task: task.name,
      mode: this.config.mode,
      completed: reason === '',
      reason,
      path: this.path(),
      states: this.states,
      metrics: taskMetrics(this.states),
      completion_miss: this.completionMiss,
      false_complete: this.falseComplete,
    };
  }
}
