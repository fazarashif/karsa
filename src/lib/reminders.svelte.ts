import { invoke } from '@tauri-apps/api/core';
import { inTauri, on } from './bridge';

export type ReminderKind = 'general' | 'meeting' | 'deadline';
export type Repeat = 'none' | 'daily' | 'weekdays' | 'weekly' | 'interval';

/** An "every N minutes" reminder's window, in minutes after local midnight. */
export type Interval = {
  everyMin: number;
  windowFrom: number;
  windowUntil: number;
  windowDays: 'weekdays' | 'daily';
};

export type Reminder = {
  id: number;
  title: string;
  kind: ReminderKind;
  /** Unix ms of the event itself (a meeting's start, a deadline). */
  at: number;
  repeat: Repeat;
  leadMin: number;
  snoozedUntil: number | null;
  fired: boolean;
  /** Set for a todo task's linked deadline. */
  taskId: number | null;
  /** "Every N minutes" reminders only. */
  everyMin: number | null;
  windowFrom: number | null;
  windowUntil: number | null;
  windowDays: 'weekdays' | 'daily' | null;
};

export type ReminderInput = {
  id: number | null;
  title: string;
  kind: ReminderKind;
  at: number;
  repeat: Repeat;
  leadMin: number;
} & Partial<Interval>;

let items = $state<Reminder[]>([]);
let error = $state<string | null>(null);

async function run(cmd: string, args: Record<string, unknown> = {}) {
  try {
    items = inTauri ? await invoke<Reminder[]>(cmd, args) : preview(cmd, args);
    error = null;
    return true;
  } catch (e) {
    error = String(e);
    return false;
  }
}

// The core pushes the list after it fires reminders or a todo deadline changes.
on('reminders://changed', (list) => (items = list));

export const reminders = {
  /** Upcoming reminders, soonest first. */
  get all() {
    return items;
  },
  get error() {
    return error;
  },
  clearError() {
    error = null;
  },
  load: () => run('reminders_list'),
  save: (input: ReminderInput) => run('reminders_save', { input }),
  remove: (id: number) => run('reminders_delete', { id }),
  snooze: (id: number, minutes = 5) => run('reminders_snooze', { id, minutes }),
};

function sameDay(a: number, b: Date) {
  const d = new Date(a);
  return d.getFullYear() === b.getFullYear() && d.getMonth() === b.getMonth() && d.getDate() === b.getDate();
}

/** The next reminder still ahead today: the pill's "next up" and Peek's row. "Every N
    minutes" nudges are left out; they would always be next. */
export function nextToday(list: Reminder[], now: Date) {
  return list
    .filter((r) => r.repeat !== 'interval' && r.at >= now.getTime() && sameDay(r.at, now))
    .sort((a, b) => a.at - b.at)[0];
}

export const kindLabel: Record<ReminderKind, string> = { general: 'General', meeting: 'Meeting', deadline: 'Deadline' };
export const repeatLabel: Record<Repeat, string> = {
  none: 'None',
  daily: 'Daily',
  weekdays: 'Weekdays',
  weekly: 'Weekly',
  interval: 'Every few minutes…',
};

const hhmm = (minutes: number) => `${String(Math.floor(minutes / 60)).padStart(2, '0')}:${String(minutes % 60).padStart(2, '0')}`;

/** "Every 30 min · 07:00–17:00 · weekdays" */
export function intervalLabel(r: Pick<Reminder, 'everyMin' | 'windowFrom' | 'windowUntil' | 'windowDays'>) {
  if (r.everyMin === null || r.windowFrom === null || r.windowUntil === null) return '';
  const every = r.everyMin % 60 === 0 ? `${r.everyMin / 60} h` : `${r.everyMin} min`;
  return `Every ${every} · ${hhmm(r.windowFrom)}–${hhmm(r.windowUntil)} · ${r.windowDays === 'daily' ? 'every day' : 'weekdays'}`;
}

// ---- Browser preview -----------------------------------------------------
// Keeps the list in memory for design work; nothing fires in a plain browser.

let previewId = 1;

function preview(cmd: string, a: Record<string, any>): Reminder[] {
  let list: Reminder[] = structuredClone($state.snapshot(items));
  if (cmd === 'reminders_save') {
    const input = a.input as ReminderInput;
    if (!input.title.trim()) throw new Error('Title cannot be empty');
    if (input.repeat === 'none' && input.at <= Date.now()) throw new Error('Pick a time in the future');
    if (input.repeat === 'interval' && (input.windowFrom ?? 0) >= (input.windowUntil ?? 0)) throw new Error('The end time must be after the start time');
    const interval =
      input.repeat === 'interval'
        ? { everyMin: input.everyMin ?? 30, windowFrom: input.windowFrom ?? 420, windowUntil: input.windowUntil ?? 1020, windowDays: input.windowDays ?? 'weekdays' }
        : { everyMin: null, windowFrom: null, windowUntil: null, windowDays: null };
    const at = input.repeat === 'interval' ? Date.now() + (input.everyMin ?? 30) * 60_000 : input.at;
    const existing = list.find((r) => r.id === input.id);
    if (existing) Object.assign(existing, { ...input, ...interval, at, id: existing.id, title: input.title.trim() });
    else list.push({ ...input, ...interval, at, id: previewId++, title: input.title.trim(), snoozedUntil: null, fired: false, taskId: null });
  } else if (cmd === 'reminders_delete') {
    list = list.filter((r) => r.id !== a.id);
  } else if (cmd === 'reminders_snooze') {
    const r = list.find((x) => x.id === a.id);
    if (r) r.snoozedUntil = Date.now() + a.minutes * 60_000;
  }
  return list.sort((x, y) => x.at - y.at || x.id - y.id);
}
