// The screen-only user (skill § 7). Deterministic code, no model. It acts only on controls that are
// wholly inside the viewport and not covered at their centre, found by accessible role and name.
// A click, a key press, a scroll gesture or a typed query is one action.
import type { ElementHandle, Page } from '@playwright/test';
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
  theme?: 'dark' | 'light';
}

export interface Task {
  name: string;
  start: string;
  steps: Step[];
  truth: Truth;
}

// A container that hides controls until one of its own controls is used: a collapsed sidebar
// category, a closed mobile menu. `toggle` is CSS below the container (starting `:scope`) or a
// control found by role and name anywhere on the page.
export interface Revealer {
  inside: string;
  hiddenWhen?: string;
  toggle: string | Control;
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

// After an action: a full navigation may still be committing, so a destroyed context is retried.
async function settle(page: Page): Promise<void> {
  for (let attempt = 0; ; attempt++) {
    try {
      await page.waitForLoadState('load');
      await page.waitForLoadState('networkidle', { timeout: 3_000 }).catch(() => undefined);
      await page.evaluate(
        () => new Promise<void>((resolve) => requestAnimationFrame(() => requestAnimationFrame(() => resolve()))),
      );
      return;
    } catch (error) {
      if (attempt >= 5 || !/context was destroyed|navigat/i.test(String(error))) throw error;
      await page.waitForTimeout(100);
    }
  }
}

async function regionOf(handle: Handle, config: BotConfig): Promise<string> {
  return handle.evaluate((el, regions) => {
    for (const r of regions) if (el.closest(r.selector)) return r.name;
    return 'other';
  }, config.regions);
}

interface Found {
  usable: Handle[];
  outOfView: Handle[];
  hidden: Handle[];
}

// Where the reader looks: the box `probeVisible` measures (its own text, for an element larger
// than the viewport), in page-viewport coordinates.
async function focusBox(handle: Handle): Promise<{ x: number; y: number; width: number; height: number } | null> {
  return (await handle.evaluate(probeVisible)).rect;
}

async function find(page: Page, control: Control, wait = 0): Promise<Found> {
  const found: Found = { usable: [], outOfView: [], hidden: [] };
  const locator = page.getByRole(control.role, { name: pattern(control), includeHidden: true });
  // A client-side navigation renders its page a moment after the URL changes: wait for the control
  // to exist (not to be visible) before deciding it is not there. Waiting costs no action.
  if (wait > 0) await locator.first().waitFor({ state: 'attached', timeout: wait }).catch(() => undefined);
  const handles = (await locator.elementHandles()) as Handle[];
  for (const handle of handles) {
    const v = await handle.evaluate(probeVisible);
    const shown = await handle.evaluate((el) => el.checkVisibility({ opacityProperty: true, visibilityProperty: true }));
    if (v.inViewport && !v.covered && shown) found.usable.push(handle);
    else if (shown && v.rect && v.rect.width > 0) found.outOfView.push(handle);
    else found.hidden.push(handle);
  }
  return found;
}

export class ScreenOnlyUser {
  private states: StateRecord[] = [];
  private completionMiss = 0;
  private falseComplete = 0;

  constructor(
    private readonly page: Page,
    private readonly config: BotConfig,
  ) {}

  private async activate(handle: Handle): Promise<number> {
    if (this.config.mode === 'pointer') {
      const box = await handle.boundingBox();
      if (!box) throw new Error('the control has no box');
      await this.page.mouse.click(box.x + box.width / 2, box.y + box.height / 2);
      await settle(this.page);
      return 1;
    }
    let presses = 0;
    for (; presses < this.config.maxTabs; presses++) {
      const focused = await handle.evaluate((el) => el === document.activeElement || el.contains(document.activeElement));
      if (focused) break;
      await this.page.keyboard.press('Tab');
    }
    const focused = await handle.evaluate((el) => el === document.activeElement || el.contains(document.activeElement));
    if (!focused) throw new Error(`not reached by ${this.config.maxTabs} Tab presses`);
    await this.page.keyboard.press('Enter');
    await settle(this.page);
    return presses + 1;
  }

  // Scroll gestures needed to bring an out-of-view control to the middle of the viewport.
  private async scrollsTo(handle: Handle): Promise<number> {
    const box = await focusBox(handle);
    const height = this.page.viewportSize()?.height ?? 800;
    return box ? Math.ceil(Math.abs(box.y - height / 2) / (height * 0.8)) : Infinity;
  }

  // One steer: open the container that hides the control, or scroll toward an out-of-view copy of
  // it, whichever costs fewer actions (a menu tap beats scrolling to the footer's copy of a link).
  private async steer(found: Found): Promise<number> {
    const out = found.outOfView[0];
    if (out && this.config.mode === 'pointer' && (await this.scrollsTo(out)) <= 1) return this.scroll(out);
    const revealed = await this.openContainer(found.hidden);
    if (revealed > 0) return revealed;
    if (out) return this.scroll(out);
    return found.hidden.length === 0 ? this.openUnrendered() : 0;
  }

  // No copy of the control exists in the page at all: a closed container (the mobile menu) renders
  // its content only once opened. Open the first such container whose toggle the reader sees.
  private async openUnrendered(): Promise<number> {
    for (const revealer of this.config.revealers) {
      if (typeof revealer.toggle === 'string' || !revealer.hiddenWhen) continue;
      const closed = await this.page.evaluate((css) => !!document.querySelector(css), revealer.hiddenWhen);
      if (!closed) continue;
      const toggle = (await find(this.page, revealer.toggle)).usable[0];
      if (toggle) return this.activate(toggle);
    }
    return 0;
  }

  private async scroll(out: Handle): Promise<number> {
    {
      if (this.config.mode === 'keyboard') return 0; // Tab moves focus and scrolls by itself
      const where = await out.evaluate((el) => {
        let a: Element | null = el.parentElement;
        while (a && a !== document.body) {
          const s = getComputedStyle(a);
          if ((s.overflowY === 'auto' || s.overflowY === 'scroll') && a.scrollHeight > a.clientHeight + 1) {
            const r = a.getBoundingClientRect();
            return { x: r.left + r.width / 2, y: r.top + Math.min(r.height / 2, 200) };
          }
          a = a.parentElement;
        }
        return { x: document.documentElement.clientWidth / 2, y: window.innerHeight / 2 };
      });
      const box = await focusBox(out);
      const viewport = this.page.viewportSize() ?? { width: 1280, height: 800 };
      const below = (box?.y ?? 0) > viewport.height / 2;
      // The gestures a reader makes to bring it to the middle, made at once; each is one action.
      const gestures = Math.min(200, Math.max(1, await this.scrollsTo(out)));
      await this.page.mouse.move(where.x, where.y);
      for (let i = 0; i < gestures; i++) await this.page.mouse.wheel(0, (below ? 1 : -1) * Math.round(viewport.height * 0.8));
      await this.page.waitForTimeout(100);
      return gestures;
    }
  }

  private async openContainer(hidden: Handle[]): Promise<number> {
    for (const hiddenHandle of hidden) {
      for (const revealer of this.config.revealers) {
        const applies = await hiddenHandle.evaluate(
          (el, r) => !!el.closest(r.inside) && (!r.hiddenWhen || !!document.querySelector(r.hiddenWhen)),
          { inside: revealer.inside, hiddenWhen: revealer.hiddenWhen },
        );
        if (!applies) continue;
        let toggle: Handle | null = null;
        if (typeof revealer.toggle === 'string') {
          const css = revealer.toggle;
          const container = await hiddenHandle.evaluateHandle((el, inside) => el.closest(inside), revealer.inside);
          toggle = (await (container as Handle).$(css)) as Handle | null;
          if (toggle) {
            const v = await toggle.evaluate(probeVisible);
            if (!v.inViewport || v.covered) {
              // The toggle itself is out of view or hidden: work on it first.
              const inner = await this.steerTo(toggle);
              if (inner === 0) toggle = null;
              else return inner;
            }
          }
        } else {
          toggle = (await find(this.page, revealer.toggle)).usable[0] ?? null;
        }
        if (toggle) return this.activate(toggle);
      }
    }
    return 0;
  }

  private async steerTo(handle: Handle): Promise<number> {
    const v = await handle.evaluate(probeVisible);
    const shown = await handle.evaluate((el) => el.checkVisibility({ opacityProperty: true, visibilityProperty: true }));
    if (!shown) return this.steer({ usable: [], outOfView: [], hidden: [handle] });
    if (!v.inViewport || v.covered) return this.steer({ usable: [], outOfView: [handle], hidden: [] });
    return 0;
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
    if ((await find(this.page, truth.target)).usable.length > 0) this.falseComplete++;
  }

  // Bring a control into a usable state and act on it; returns the state record.
  private async click(control: Control, label: string): Promise<StateRecord> {
    const words = await this.page.evaluate(probeReadingLoad);
    let actions = 0;
    let steers = 0;
    for (;;) {
      const found = await find(this.page, control, 2_000);
      // With the keyboard, Tab brings an out-of-view control into view by itself.
      const target = found.usable[0] ?? (this.config.mode === 'keyboard' ? found.outOfView[0] : undefined);
      if (target) {
        const glance = found.usable[0] ? await this.glanceOf(target, steers) : 3;
        actions += await this.activate(target);
        return { step: label, glance, actions, steers, words };
      }
      if (steers >= this.config.maxSteers) {
        throw new Error(`${label}: no visible control after ${steers} steers (${found.outOfView.length} out of view, ${found.hidden.length} hidden)`);
      }
      const spent = await this.steer(found);
      if (spent === 0) {
        throw new Error(`${label}: no visible control and nothing visible reveals it (${found.outOfView.length} out of view, ${found.hidden.length} hidden)`);
      }
      actions += spent;
      steers += 1;
    }
  }

  // A disclosure the reader opens on the way (a sidebar category): already open costs a look and
  // no action.
  private async open(control: Control, label: string): Promise<StateRecord> {
    const found = await find(this.page, control, 2_000);
    for (const handle of [...found.usable, ...found.outOfView]) {
      if ((await handle.getAttribute('aria-expanded')) === 'true') {
        const words = await this.page.evaluate(probeReadingLoad);
        return { step: label, glance: found.usable.includes(handle) ? await this.glanceOf(handle, 0) : 3, actions: 0, steers: 0, words };
      }
    }
    return this.click(control, label);
  }

  private async reveal(target: Control, label: string): Promise<StateRecord> {
    const words = await this.page.evaluate(probeReadingLoad);
    let actions = 0;
    let steers = 0;
    let usedLink = false;
    for (;;) {
      // The first look waits for a client-side navigation to render the page.
      const found = await find(this.page, target, actions === 0 && steers === 0 && !usedLink ? 2_000 : 0);
      if (found.usable[0]) {
        const glance = steers > 0 ? 3 : usedLink ? 2 : await this.glanceOf(found.usable[0], 0);
        return { step: label, glance, actions, steers, words };
      }
      if (found.outOfView[0] && !usedLink) {
        // The reader's first move on a long page: a visible link that names the target (the
        // table of contents), if there is one.
        const link = (await find(this.page, { role: 'link', name: `^${target.name.replace(/^\^|\$$/g, '')}$` })).usable[0];
        if (link) {
          actions += await this.activate(link);
          usedLink = true;
          const after = await find(this.page, target);
          if (!after.usable[0] && after.outOfView[0]) {
            const v = await after.outOfView[0].evaluate(probeVisible);
            if (v.rect && v.rect.y < (this.page.viewportSize()?.height ?? 800)) this.completionMiss++;
          }
          continue;
        }
      }
      if (steers >= this.config.maxSteers) {
        throw new Error(`${label}: the target is not in view after ${steers} steers`);
      }
      const spent = this.config.mode === 'keyboard' ? await this.keyScroll(found) : await this.steer(found);
      if (spent === 0) throw new Error(`${label}: the target is not on this page in any visible or revealable form`);
      actions += spent;
      steers += 1;
    }
  }

  private async keyScroll(found: Found): Promise<number> {
    const presses = found.outOfView[0] ? Math.min(200, Math.max(1, await this.scrollsTo(found.outOfView[0]))) : 1;
    for (let i = 0; i < presses; i++) await this.page.keyboard.press('PageDown');
    await this.page.waitForTimeout(100);
    return presses;
  }

  private async theme(theme: 'dark' | 'light', via: Control, label: string): Promise<StateRecord> {
    const words = await this.page.evaluate(probeReadingLoad);
    let actions = 0;
    let steers = 0;
    let glance: 1 | 2 | 3 = 1;
    for (let presses = 0; presses < 3; presses++) {
      if ((await this.page.getAttribute('html', 'data-theme')) === theme) break;
      const before = { actions, steers };
      const record = await this.click(via, label);
      actions = before.actions + record.actions;
      steers = before.steers + record.steers;
      glance = Math.max(glance, record.glance) as 1 | 2 | 3;
    }
    if ((await this.page.getAttribute('html', 'data-theme')) !== theme) {
      throw new Error(`${label}: the page does not show the ${theme} theme after 3 presses`);
    }
    return { step: label, glance, actions, steers, words };
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
          this.states.push(await this.click(step.click, `${index + 1} click ${step.click.role} /${step.click.name}/`));
          await this.noteFalseComplete(task.truth);
        } else if ('open' in step) {
          this.states.push(await this.open(step.open, `${index + 1} open ${step.open.role} /${step.open.name}/`));
        } else if ('reveal' in step) {
          this.states.push(await this.reveal(step.reveal, `${index + 1} reveal ${step.reveal.role} /${step.reveal.name}/`));
        } else {
          this.states.push(await this.theme(step.theme, step.via, `${index + 1} theme ${step.theme}`));
        }
      }
    } catch (error) {
      reason = (error as Error).message;
    }
    let completed = reason === '';
    if (completed && this.path() !== task.truth.path.replace(/\/$/, '')) {
      completed = false;
      reason = `ended on ${this.path()}, the truth is ${task.truth.path}`;
    }
    if (completed && task.truth.target && (await find(this.page, task.truth.target)).usable.length === 0) {
      completed = false;
      reason = `the target ${task.truth.target.role} /${task.truth.target.name}/ is not in view at the end`;
    }
    if (completed && task.truth.theme && (await this.page.getAttribute('html', 'data-theme')) !== task.truth.theme) {
      completed = false;
      reason = `the page ends in the ${await this.page.getAttribute('html', 'data-theme')} theme, the truth is ${task.truth.theme}`;
    }
    return {
      task: task.name,
      mode: this.config.mode,
      completed,
      reason,
      path: this.path(),
      states: this.states,
      metrics: taskMetrics(this.states),
      completion_miss: this.completionMiss,
      false_complete: this.falseComplete,
    };
  }
}
