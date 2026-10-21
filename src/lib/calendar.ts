/**
 * The calendar's view of Karsa's own data (design v2, 4.5 and 6.3): which days carry
 * deadlines, meetings and reminders, how long you focused, and each day's agenda.
 * Pure functions over the stores' data; nothing is fetched or kept.
 */
import type { NoteSummary } from './notes.svelte';
import type { Reminder } from './reminders.svelte';
import type { Task } from './todo.svelte';

const DAY = 86_400_000;

export function startOfDay(d: Date | number) {
  const x = new Date(d);
  x.setHours(0, 0, 0, 0);
  return x;
}

export function addDays(d: Date, days: number) {
  const x = new Date(d);
  x.setDate(x.getDate() + days);
  return x;
}

export function sameDay(a: Date | number, b: Date | number) {
  return startOfDay(a).getTime() === startOfDay(b).getTime();
}

/** Six weeks from the Monday on or before the 1st, so every month has the same height. */
export function monthGrid(month: Date) {
  const first = new Date(month.getFullYear(), month.getMonth(), 1);
  const start = addDays(first, -((first.getDay() + 6) % 7));
  return Array.from({ length: 42 }, (_, i) => addDays(start, i));
}

/** Whether a repeating reminder falls on the day (the time of day comes from `at`). */
function repeatsOn(r: Reminder, day: Date) {
  const weekday = day.getDay();
  switch (r.repeat) {
    case 'daily':
      return true;
    case 'weekdays':
      return weekday >= 1 && weekday <= 5;
    case 'weekly':
      return new Date(r.at).getDay() === weekday;
    case 'interval':
      return r.windowDays === 'daily' || (weekday >= 1 && weekday <= 5);
    default:
      return sameDay(r.at, day);
  }
}

/** The reminder's time on that day, in ms. */
function timeOn(r: Reminder, day: Date) {
  const at = new Date(r.at);
  const d = new Date(day);
  d.setHours(at.getHours(), at.getMinutes(), 0, 0);
  return d.getTime();
}

export type Data = { tasks: Task[]; reminders: Reminder[]; notes: NoteSummary[]; focus: [number, number][] };

/** Reminders a task already stands for (its linked deadline) are left to the task. */
const ownReminders = (data: Data) => data.reminders.filter((r) => r.taskId === null);

export type Marks = { deadline: boolean; meeting: boolean; general: boolean; focusMin: number };

/** Dots and the focus bar for each day of a grid, keyed by local midnight. */
export function marks(days: Date[], data: Data) {
  const map = new Map<number, Marks>();
  for (const day of days) map.set(day.getTime(), { deadline: false, meeting: false, general: false, focusMin: 0 });
  const mark = (ms: number) => map.get(startOfDay(ms).getTime());
  for (const t of data.tasks) {
    if (t.dueAt !== null && !t.done) {
      const m = mark(t.dueAt);
      if (m) m.deadline = true;
    }
  }
  for (const n of data.notes) {
    const m = n.kind === 'meeting' ? mark(n.date) : undefined;
    if (m) m.meeting = true;
  }
  for (const r of ownReminders(data)) {
    // Every-N-minutes nudges would put a dot on every weekday; they stay in the agenda.
    if (r.repeat === 'interval') continue;
    for (const day of days) {
      if (!repeatsOn(r, day)) continue;
      const m = map.get(day.getTime())!;
      if (r.kind === 'meeting') m.meeting = true;
      else if (r.kind === 'deadline') m.deadline = true;
      else m.general = true;
    }
  }
  for (const [endedAt, ms] of data.focus) {
    const m = mark(endedAt);
    if (m) m.focusMin += Math.round(ms / 60_000);
  }
  return map;
}

export type AgendaItem =
  | { kind: 'late'; task: Task }
  | { kind: 'task'; task: Task; at: number | null }
  | { kind: 'reminder'; reminder: Reminder; at: number; note: NoteSummary | null }
  | { kind: 'note'; note: NoteSummary }
  | { kind: 'interval'; reminder: Reminder };

/** One day's agenda, in design v2's order: late (today only), timed items by time,
    untimed deadlines, then every-N reminders. */
export function agenda(day: Date, today: Date, data: Data) {
  const start = startOfDay(day).getTime();
  const end = start + DAY;
  const items: AgendaItem[] = [];
  if (sameDay(day, today)) {
    for (const t of data.tasks) {
      if (t.parentId === null && !t.done && t.dueAt !== null && t.dueAt < start) items.push({ kind: 'late', task: t });
    }
  }
  const timed: { at: number; item: AgendaItem }[] = [];
  const meetings = data.notes.filter((n) => n.kind === 'meeting' && n.date >= start && n.date < end);
  const usedNotes = new Set<string>();
  for (const r of ownReminders(data)) {
    if (!repeatsOn(r, day)) continue;
    if (r.repeat === 'interval') continue;
    const at = timeOn(r, day);
    const note = r.kind === 'meeting' ? (meetings.find((n) => n.title === r.title) ?? null) : null;
    if (note) usedNotes.add(note.path);
    timed.push({ at, item: { kind: 'reminder', reminder: r, at, note } });
  }
  for (const n of meetings) if (!usedNotes.has(n.path)) timed.push({ at: n.date, item: { kind: 'note', note: n } });
  const untimed: AgendaItem[] = [];
  for (const t of data.tasks) {
    if (t.parentId !== null || t.dueAt === null || t.dueAt < start || t.dueAt >= end) continue;
    if (t.dueHasTime) timed.push({ at: t.dueAt, item: { kind: 'task', task: t, at: t.dueAt } });
    else untimed.push({ kind: 'task', task: t, at: null });
  }
  items.push(...timed.sort((a, b) => a.at - b.at).map((x) => x.item), ...untimed);
  for (const r of ownReminders(data)) if (r.repeat === 'interval' && repeatsOn(r, day)) items.push({ kind: 'interval', reminder: r });
  return items;
}

/** Tasks finished on the day, and minutes of focus. */
export function daySummary(day: Date, data: Data) {
  const start = startOfDay(day).getTime();
  const end = start + DAY;
  const done = data.tasks.filter((t) => t.completedAt !== null && t.completedAt >= start && t.completedAt < end).length;
  const sessions = data.focus.filter(([endedAt]) => endedAt >= start && endedAt < end);
  return { done, focusMin: sessions.reduce((sum, [, ms]) => sum + Math.round(ms / 60_000), 0), sessions: sessions.length };
}
