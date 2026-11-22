// In-page probes, passed to page.evaluate. Each function is self-contained: Playwright sends its
// source to the browser, so nothing outside the function body may be referenced.

export interface Spill {
  kind: 'page' | 'viewport' | 'box';
  element: string;
  right: number;
  limit: number;
}

export interface Truncation {
  element: string;
  text: string;
}

export interface LayoutProbe {
  spills: Spill[];
  truncated: Truncation[];
}

// Gate 1 and the truncation rule.
//   page:     document wider than the viewport (horizontal page scroll).
//   viewport: a visible element, not inside a horizontal scroll container, that starts inside the
//             viewport and ends past it (an element parked wholly off-screen, like a skip link
//             before it has focus, is not cut by the viewport; one that widens the page is a
//             `page` spill).
//   box:      visible content (an element with its own text, or a replaced element: img, svg,
//             video, canvas, iframe, table, pre), in flow, not inside a horizontal scroll
//             container, whose right edge is past its parent's border box while the parent lets it
//             overflow visibly. Layout boxes without content of their own (Infima's `.row` with
//             its negative margins) are not content running past anything.
// Visible: not display none, not visibility hidden, and no ancestor at opacity 0 or aria-hidden.
// Truncated: text cut by an ellipsis, a line clamp or a clipping box, with no title or aria-label
// carrying the full value.
export function probeLayout(): LayoutProbe {
  const doc = document.documentElement;
  const viewportWidth = doc.clientWidth;
  const spills: Spill[] = [];
  const truncated: Truncation[] = [];
  const name = (el: Element): string => {
    const id = el.id ? `#${el.id}` : '';
    const classes = Array.from(el.classList)
      .filter((c) => !/^[a-zA-Z]+_[a-zA-Z0-9_-]{5,}$/.test(c))
      .slice(0, 2)
      .map((c) => `.${c}`)
      .join('');
    const parent = el.parentElement && el.parentElement !== document.body ? el.parentElement : null;
    const parentName = parent ? `${parent.tagName.toLowerCase()}${parent.id ? `#${parent.id}` : ''}${parent.classList[0] ? `.${parent.classList[0]}` : ''} > ` : '';
    return `${parentName}${el.tagName.toLowerCase()}${id}${classes}`;
  };
  if (doc.scrollWidth > viewportWidth + 1) {
    spills.push({ kind: 'page', element: 'html', right: doc.scrollWidth, limit: viewportWidth });
  }
  const scrolls = (style: CSSStyleDeclaration) => style.overflowX === 'auto' || style.overflowX === 'scroll';
  const replaced = new Set(['IMG', 'SVG', 'svg', 'VIDEO', 'CANVAS', 'IFRAME', 'TABLE', 'PRE']);
  const walk = (el: Element, inScroll: boolean, parentRight: number, parentClips: boolean, parentHidden: boolean) => {
    const style = getComputedStyle(el);
    if (style.display === 'none') return;
    // checkVisibility() also sees content a closed <details> keeps out of rendering
    // (content-visibility), which still reports boxes.
    const faded = parentHidden || style.opacity === '0' || el.getAttribute('aria-hidden') === 'true' || !el.checkVisibility();
    const hidden = faded || style.visibility === 'hidden';
    const rect = el.getBoundingClientRect();
    const sized = rect.width > 1 && rect.height > 1;
    const positioned = style.position === 'absolute' || style.position === 'fixed';
    const content =
      replaced.has(el.tagName) ||
      Array.from(el.childNodes).some((n) => n.nodeType === Node.TEXT_NODE && (n.textContent ?? '').trim() !== '');
    if (!hidden && sized && !inScroll) {
      if (rect.right > viewportWidth + 1 && rect.left < viewportWidth) {
        spills.push({ kind: 'viewport', element: name(el), right: Math.round(rect.right), limit: viewportWidth });
      } else if (content && !positioned && !parentClips && rect.right > parentRight + 1) {
        spills.push({ kind: 'box', element: name(el), right: Math.round(rect.right), limit: Math.round(parentRight) });
      }
    }
    if (!hidden && sized) {
      const ellipsis = style.textOverflow === 'ellipsis';
      const clamp = style.getPropertyValue('-webkit-line-clamp');
      const clamped = clamp !== '' && clamp !== 'none';
      const clips = style.overflowX === 'hidden' || style.overflowX === 'clip';
      const ownText = Array.from(el.childNodes).some((n) => n.nodeType === Node.TEXT_NODE && (n.textContent ?? '').trim() !== '');
      const cut = (el as HTMLElement).scrollWidth > (el as HTMLElement).clientWidth + 1 || (clamped && (el as HTMLElement).scrollHeight > (el as HTMLElement).clientHeight + 1);
      if (ownText && cut && (ellipsis || clamped || clips)) {
        const full = (el.textContent ?? '').trim();
        const label = el.closest('[title]')?.getAttribute('title') ?? el.getAttribute('aria-label') ?? '';
        if (!label.includes(full.slice(0, 40))) {
          truncated.push({ element: name(el), text: full.slice(0, 60) });
        }
      }
    }
    const childInScroll = inScroll || scrolls(style);
    const clipsChildren = style.overflowX !== 'visible';
    for (const child of Array.from(el.children)) {
      walk(child, childInScroll, rect.right, clipsChildren, faded);
    }
  };
  walk(document.body, false, viewportWidth, false, false);
  // A spill reported for an element is not reported again for its descendants.
  const seen = new Set<string>();
  const unique = spills.filter((s) => {
    const key = `${s.kind}|${s.element}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
  return { spills: unique, truncated };
}

export interface VisibilityProbe {
  present: boolean;
  inViewport: boolean;
  covered: boolean;
  rect: { x: number; y: number; width: number; height: number } | null;
}

// Gate 2 for one element: wholly inside the viewport, and the element itself is what is hit at its
// centre and at four points just inside its corners.
export function probeVisible(el: Element | null): VisibilityProbe {
  if (!el) return { present: false, inViewport: false, covered: false, rect: null };
  const vw = document.documentElement.clientWidth;
  const vh = window.innerHeight;
  // An element larger than the viewport (a table cell beside a long description) can never be
  // wholly in view: what the reader looks for is its own text, so that is what is measured.
  let r = el.getBoundingClientRect();
  if (r.height > vh || r.width > vw) {
    const range = document.createRange();
    range.selectNodeContents(el);
    const t = range.getBoundingClientRect();
    if (t.width > 0 && t.height > 0) r = t;
  }
  const style = getComputedStyle(el);
  const shown = r.width > 0 && r.height > 0 && style.visibility !== 'hidden' && style.display !== 'none';
  const inViewport = shown && r.left >= -0.5 && r.top >= -0.5 && r.right <= vw + 0.5 && r.bottom <= vh + 0.5;
  const inset = Math.min(3, r.width / 4, r.height / 4);
  const points = [
    [r.left + r.width / 2, r.top + r.height / 2],
    [r.left + inset, r.top + inset],
    [r.right - inset, r.top + inset],
    [r.left + inset, r.bottom - inset],
    [r.right - inset, r.bottom - inset],
  ];
  const covered = !shown || points.some(([x, y]) => {
    if (x < 0 || y < 0 || x >= vw || y >= vh) return false;
    const hit = document.elementFromPoint(x, y);
    return !hit || !(hit === el || el.contains(hit) || hit.contains(el));
  });
  return { present: true, inViewport, covered, rect: { x: r.x, y: r.y, width: r.width, height: r.height } };
}

// Words of visible text inside the viewport: the reading load of one state.
export function probeReadingLoad(): number {
  const vw = document.documentElement.clientWidth;
  const vh = window.innerHeight;
  const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
  const range = document.createRange();
  let words = 0;
  for (let node = walker.nextNode(); node; node = walker.nextNode()) {
    const text = (node.textContent ?? '').trim();
    if (!text) continue;
    const parent = node.parentElement;
    if (!parent || !parent.checkVisibility({ opacityProperty: true, visibilityProperty: true })) continue;
    range.selectNodeContents(node);
    const r = range.getBoundingClientRect();
    if (r.width === 0 || r.bottom <= 0 || r.top >= vh || r.right <= 0 || r.left >= vw) continue;
    words += text.split(/\s+/).length;
  }
  return words;
}

// Installed before any page script: sums layout shifts without recent input.
export function installLayoutShiftObserver(): void {
  const w = window as unknown as { __uxLayoutShift: number };
  w.__uxLayoutShift = 0;
  try {
    new PerformanceObserver((list) => {
      for (const entry of list.getEntries() as unknown as Array<{ value: number; hadRecentInput: boolean }>) {
        if (!entry.hadRecentInput) w.__uxLayoutShift += entry.value;
      }
    }).observe({ type: 'layout-shift', buffered: true });
  } catch {
    // An engine without layout-shift entries reports 0.
  }
}

export function readLayoutShift(): number {
  return (window as unknown as { __uxLayoutShift?: number }).__uxLayoutShift ?? 0;
}

// Reduced motion: with prefers-reduced-motion: reduce emulated, nothing animates and the page does
// not scroll smoothly.
export function probeMotion(): { running: string[]; smoothScroll: boolean } {
  const running = document
    .getAnimations()
    .filter((a) => a.playState === 'running')
    .filter((a) => {
      const timing = a.effect?.getComputedTiming();
      return Number(timing?.duration ?? 0) > 0.01;
    })
    .map((a) => {
      const target = (a.effect as KeyframeEffect | null)?.target;
      const label = (a as CSSAnimation).animationName ?? (a as CSSTransition).transitionProperty ?? 'animation';
      return `${target ? target.tagName.toLowerCase() + (target.classList[0] ? '.' + target.classList[0] : '') : '?'}:${label}`;
    });
  return { running, smoothScroll: getComputedStyle(document.documentElement).scrollBehavior === 'smooth' };
}
