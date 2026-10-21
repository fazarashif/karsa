import type { TransitionConfig } from 'svelte/transition';

const reducedMotion = () => matchMedia('(prefers-reduced-motion: reduce)').matches;

/**
 * Old content leaves quickly so it never overlaps the new layout for long. New content
 * enters through the `.beat` CSS class (see app.css), one gamelan beat per row.
 */
export function contentOut(_node: Element, { duration = 100 } = {}): TransitionConfig {
  if (reducedMotion()) return { duration: 0 };
  return { duration, css: (t) => `opacity:${t};transform:scale(${0.98 + 0.02 * t})` };
}

/** Tab body: the old one fades out, the new one rises 4 px into place. */
export function tabOut(_node: Element): TransitionConfig {
  if (reducedMotion()) return { duration: 0 };
  return { duration: 80, css: (t) => `opacity:${t}` };
}

export function tabIn(_node: Element): TransitionConfig {
  if (reducedMotion()) return { duration: 0 };
  return { delay: 40, duration: 160, css: (t, u) => `opacity:${t};transform:translateY(${4 * u}px)` };
}

/** A calendar month slides 12 px in the direction of travel as it fades in. */
export function monthIn(_node: Element, { dir = 1 }: { dir?: number } = {}): TransitionConfig {
  if (reducedMotion()) return { duration: 0 };
  return { duration: 200, css: (t, u) => `opacity:${t};transform:translateX(${12 * dir * u}px)` };
}

/** Lyric focus: a new line rises 6 px into place as the old one leaves upwards. */
export function lyricIn(_node: Element): TransitionConfig {
  if (reducedMotion()) return { duration: 0 };
  return { duration: 220, easing: (t) => 1 - Math.pow(1 - t, 3), css: (t, u) => `opacity:${t};transform:translateY(${6 * u}px)` };
}

export function lyricOut(_node: Element): TransitionConfig {
  if (reducedMotion()) return { duration: 0 };
  return { duration: 120, css: (t, u) => `opacity:${t};transform:translateY(${-6 * u}px)` };
}

/** Eye expressions crossfade. */
export function crossfade(_node: Element): TransitionConfig {
  if (reducedMotion()) return { duration: 0 };
  return { duration: 160, css: (t) => `opacity:${t}` };
}
