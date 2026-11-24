// Hermetic pages: every request that does not go to the local origin is aborted and counted.
import type { BrowserContext } from '@playwright/test';
import { installLayoutShiftObserver } from './probes';

export interface Hermetic {
  blocked: Map<string, number>;
}

export async function makeHermetic(context: BrowserContext, origin: string): Promise<Hermetic> {
  const blocked = new Map<string, number>();
  await context.route('**/*', (route) => {
    const url = route.request().url();
    if (url.startsWith(origin) || url.startsWith('data:') || url.startsWith('blob:')) return route.continue();
    const host = (() => {
      try {
        return new URL(url).host;
      } catch {
        return url.slice(0, 40);
      }
    })();
    blocked.set(host, (blocked.get(host) ?? 0) + 1);
    return route.abort('blockedbyclient');
  });
  await context.addInitScript(installLayoutShiftObserver);
  return { blocked };
}
