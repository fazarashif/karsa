import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { FocusState } from './focus.svelte';
import type { MediaState } from './media.svelte';
import type { OutlookState } from './outlook.svelte';
import type { Reminder } from './reminders.svelte';
import type { AppSettings } from './settings.svelte';
import type { Snapshot } from './todo.svelte';

/** False when the UI is opened in a plain browser (vite dev) for design work. */
export const inTauri = '__TAURI_INTERNALS__' in window;

export type AlertPayload = {
  title: string;
  subtitle: string;
  kind: 'general' | 'meeting' | 'deadline' | 'focus';
  /** Focus alerts only: which actions to offer when a phase ends. */
  focus?: 'break-next' | 'focus-next' | 'auto';
  /** Reminder alerts: the reminder to snooze, and its event time. */
  reminder?: number;
  at?: number;
  /** Outlook meeting alerts: the meeting, for its note. */
  outlook?: string;
  /** Several reminders at once (or missed ones): offers Review instead of Snooze. */
  summary?: boolean;
  /** The end-of-day invitation: offers Later and Start. */
  rampung?: boolean;
};

/** Events sent by the Rust core. */
type Events = {
  'island://blur': null;
  /** Opens the panel, optionally on Settings or the Rampung ritual (from the tray). */
  'island://open': 'settings' | 'rampung' | null;
  'island://alert': AlertPayload;
  /** The quick-capture hotkey was pressed. */
  'island://capture': null;
  'clipboard://changed': null;
  'shelf://changed': null;
  /** Files dragged over the island (the core's own drop target); on drop they are already kept. */
  'shelf://drag': { kind: 'enter' | 'leave' | 'drop'; count: number; kept: number };
  'media://changed': MediaState;
  'outlook://changed': OutlookState;
  'island://fullscreen': boolean;
  'focus://state': FocusState;
  'reminders://changed': Reminder[];
  'todo://changed': Snapshot;
  'notes://changed': null;
  'settings://changed': AppSettings;
};

const local = new EventTarget();

export function on<K extends keyof Events>(name: K, cb: (payload: Events[K]) => void): () => void {
  if (inTauri) {
    let unlisten: UnlistenFn | undefined;
    let disposed = false;
    listen<Events[K]>(name, (e) => cb(e.payload)).then((u) => (disposed ? u() : (unlisten = u)));
    return () => {
      disposed = true;
      unlisten?.();
    };
  }
  const handler = (e: Event) => cb((e as CustomEvent<Events[K]>).detail);
  local.addEventListener(name, handler);
  return () => local.removeEventListener(name, handler);
}

/** Browser preview only: simulates an event the Rust core would normally send. */
export function emitLocal<K extends keyof Events>(name: K, payload: Events[K]) {
  local.dispatchEvent(new CustomEvent(name, { detail: payload }));
}

async function call(cmd: string, args?: Record<string, unknown>) {
  if (inTauri) await invoke(cmd, args);
}

export const core = {
  /** Shows the window once the first frame is ready, so there is no blank flash. */
  ready: () => call('ui_ready'),
  /** Size (CSS px) of the clickable area at the bottom-right; clicks elsewhere pass through. */
  setHitRegion: (width: number, height: number) => call('set_hit_region', { width, height }),
  /** Lets the window take keyboard focus (Expanded) or go back to never activating. */
  setInteractive: (active: boolean, restoreFocus = false) =>
    call('set_interactive', { active, restoreFocus }),
  uses12HourClock: async () => (inTauri ? invoke<boolean>('uses_12_hour_clock') : false),
};
