import { invoke } from '@tauri-apps/api/core';
import { inTauri, on } from './bridge';

export type ReminderKind = 'general' | 'meeting' | 'deadline';
export type Repeat = 'none' | 'daily' | 'weekdays' | 'weekly';

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
};

export type ReminderInput = {
  id: number | null;
  title: string;
  kind: ReminderKind;
  at: number;
  repeat: Repeat;
  leadMin: number;
};

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

/** The next reminder still ahead today: the pill's "next up" and Peek's row. */
export function nextToday(list: Reminder[], now: Date) {
  return list.filter((r) => r.at >= now.getTime() && sameDay(r.at, now)).sort((a, b) => a.at - b.at)[0];
}

export const kindLabel: Record<ReminderKind, string> = { general: 'General', meeting: 'Meeting', deadline: 'Deadline' };
export const repeatLabel: Record<Repeat, string> = { none: 'None', daily: 'Daily', weekdays: 'Weekdays', weekly: 'Weekly' };

// ---- Browser preview -----------------------------------------------------
// Keeps the list in memory for design work; nothing fires in a plain browser.

let previewId = 1;

function preview(cmd: string, a: Record<string, any>): Reminder[] {
  let list: Reminder[] = structuredClone($state.snapshot(items));
  if (cmd === 'reminders_save') {
    const input = a.input as ReminderInput;
    if (!input.title.trim()) throw new Error('Title cannot be empty');
    if (input.repeat === 'none' && input.at <= Date.now()) throw new Error('Pick a time in the future');
    const existing = list.find((r) => r.id === input.id);
    if (existing) Object.assign(existing, { ...input, id: existing.id, title: input.title.trim() });
    else list.push({ ...input, id: previewId++, title: input.title.trim(), snoozedUntil: null, fired: false, taskId: null });
  } else if (cmd === 'reminders_delete') {
    list = list.filter((r) => r.id !== a.id);
  } else if (cmd === 'reminders_snooze') {
    const r = list.find((x) => x.id === a.id);
    if (r) r.snoozedUntil = Date.now() + a.minutes * 60_000;
  }
  return list.sort((x, y) => x.at - y.at || x.id - y.id);
}
