import type { Expression } from './Eyes.svelte';
import { focus } from './focus.svelte';
import { toDateInput } from './format.svelte';
import { specialDay } from './jawa';
import { settings } from './settings.svelte';
import { todayStatus, todo } from './todo.svelte';

/**
 * The mripat's expression when no timer is shown: sulking right after a focus session
 * was cut short, asleep at night, proud on the user's birthday or weton day or once the
 * day is wrapped up, content when today's tasks are all done, otherwise awake.
 */
export function idleMood(now: Date): Expression {
  if (focus.sulking) return 'mrengut';
  const hour = now.getHours();
  if (hour >= 22 || hour < 5) return 'merem';
  const s = settings.current;
  if (specialDay(now, s.birthday) || s.rampungDoneDate === toDateInput(now.getTime())) return 'bungah';
  const today = todayStatus(todo.tasks, now);
  if (today.total > 0 && today.open === 0) return 'mesem';
  return 'melek';
}

/** The face in Peek and the panel header: the timer's expression while one runs. */
export function mood(now: Date): Expression {
  const timer = focus.state;
  if (timer.status === 'running' || timer.status === 'paused') return focus.isBreak ? 'leren' : 'fokus';
  return idleMood(now);
}
