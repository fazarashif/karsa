import type { Expression } from './Eyes.svelte';

/**
 * Short expressions that answer an event (design v2, 2.2): kaget when an alert arrives,
 * seneng when a task is done, gumun when the focus streak grows, nyanyi when a track
 * starts, and mrengut's glance at overdue tasks. Each lasts a fixed time and never loops.
 * In its own module so the stores that trigger them do not import each other.
 */

/** Highest first; a reaction only replaces one of the same or lower rank. */
const RANK: Expression[] = ['kaget', 'seneng', 'gumun', 'mrengut', 'nyanyi'];

let current = $state<Expression | null>(null);
/** Bumped when a reaction ends, so the eyes blink once on the way back. */
let ended = $state(0);
let timer: ReturnType<typeof setTimeout> | undefined;

export const reaction = {
  /** The expression to show over the regular mood, or null. */
  get current() {
    return current;
  },
  get ended() {
    return ended;
  },
  /** Ranks a reaction against a regular mood: lower is stronger. */
  rank(expression: Expression) {
    const i = RANK.indexOf(expression);
    return i < 0 ? RANK.length : i;
  },
};

export function react(expression: Expression, ms: number) {
  if (current && reaction.rank(expression) > reaction.rank(current)) return;
  current = expression;
  clearTimeout(timer);
  timer = setTimeout(() => {
    current = null;
    ended++;
  }, ms);
}
