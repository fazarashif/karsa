import { invoke } from '@tauri-apps/api/core';
import { inTauri, on } from './bridge';
import type { NewMeeting } from './meeting';

/** A meeting read from classic Outlook on this PC (outlook.rs). Read only. */
export type OutlookMeeting = {
  /** Stable for this meeting (an occurrence of a recurring one has its own); a meeting
      note started from it keeps it as `outlook:`. */
  key: string;
  title: string;
  /** Unix ms. */
  start: number;
  end: number;
  allDay: boolean;
  location: string;
  /** Empty when the user organizes it. */
  organizer: string;
  /** Without the user and the organizer; at most 30. */
  attendees: string[];
  attendeeCount: number;
  canceled: boolean;
  declined: boolean;
};

export type OutlookState = {
  enabled: boolean;
  /** "closed": Outlook is not running, so what was last read stays. */
  status: 'off' | 'reading' | 'ok' | 'closed' | 'error';
  message: string;
  /** When the meetings were last read (Unix ms), or 0. */
  readAt: number;
  /** Minutes before a meeting that Karsa alerts; -1 for none. */
  leadMin: number;
  meetings: OutlookMeeting[];
};

/** A meeting note takes the organizer and this many people at most: a town hall's fifty
    names would bury the note. */
const NOTE_PEOPLE = 12;

let state = $state<OutlookState>({ enabled: false, status: 'off', message: '', readAt: 0, leadMin: 10, meetings: [] });

on('outlook://changed', (next) => (state = next));

export const outlook = {
  get state() {
    return state;
  },
  get meetings() {
    return state.meetings;
  },
  load: async () => {
    state = inTauri ? await invoke<OutlookState>('outlook_state') : previewState();
  },
  /** Reads Outlook again now (the Calendar tab asks when it opens). */
  refresh: () => {
    if (inTauri && state.enabled) void invoke('outlook_refresh');
  },
  find: (key: string) => state.meetings.find((m) => m.key === key),
};

/** Whether Karsa alerts for it, and counts it as "next up". */
export const remindable = (m: OutlookMeeting) => !m.allDay && !m.canceled && !m.declined;

/** Meetings as the pill and Peek count upcoming items, beside reminders. */
export function asUpcoming(meetings: OutlookMeeting[]) {
  return meetings.filter(remindable).map((m) => ({ title: m.title || 'Meeting', at: m.start, kind: 'meeting' as const, repeat: 'none' as const, taskId: null }));
}

/** What a new meeting note for it starts with: title, time, the organizer and the first
    attendees, and the link back to the meeting. */
export function noteFrom(m: OutlookMeeting): NewMeeting {
  const people = [m.organizer, ...m.attendees].filter(Boolean).slice(0, NOTE_PEOPLE);
  return { title: m.title || 'Meeting', at: m.start, attendees: people, outlook: m.key };
}

// ---- Browser preview -----------------------------------------------------

function previewState(): OutlookState {
  const at = (days: number, hour: number, minute = 0) => {
    const d = new Date();
    d.setDate(d.getDate() + days);
    d.setHours(hour, minute, 0, 0);
    return d.getTime();
  };
  const meeting = (key: string, title: string, start: number, minutes: number, more: Partial<OutlookMeeting> = {}): OutlookMeeting => ({
    key,
    title,
    start,
    end: start + minutes * 60_000,
    allDay: false,
    location: 'Microsoft Teams Meeting',
    organizer: 'Dewi Lestari',
    attendees: ['Bima Pratama', 'Sari Wulandari', 'Agus Santoso'],
    attendeeCount: 3,
    canceled: false,
    declined: false,
    ...more,
  });
  return {
    enabled: true,
    status: 'ok',
    message: '',
    readAt: Date.now(),
    leadMin: 10,
    meetings: [
      meeting('preview-1', 'Weekly sync', at(0, 10), 30),
      meeting('preview-2', 'S/4 HANA progress review', at(0, 14, 30), 60, { attendeeCount: 41 }),
      meeting('preview-3', 'Vendor demo', at(0, 16), 45, { canceled: true }),
      meeting('preview-4', 'Sprint planning', at(1, 9), 90),
    ],
  };
}
