import { invoke } from '@tauri-apps/api/core';
import { emitLocal, inTauri, on, type AlertPayload } from './bridge';
import { dayFromToday } from './format.svelte';

export type Phase = 'focus' | 'short_break' | 'long_break';
export type Status = 'ready' | 'running' | 'paused' | 'waiting';

export type FocusState = {
  phase: Phase;
  status: Status;
  /** Wall-clock ms at which the running phase ends. */
  endsAt: number | null;
  /** Time left while paused, ready or waiting. */
  remainingMs: number;
  durationMs: number;
  /** The focus session this phase belongs to, counted from 1 within the cycle. */
  session: number;
  settings: {
    focusMin: number;
    shortBreakMin: number;
    longBreakMin: number;
    sessionsPerCycle: number;
    autoStart: boolean;
  };
};

const settings = { focusMin: 25, shortBreakMin: 5, longBreakMin: 15, sessionsPerCycle: 4, autoStart: false };

let state = $state<FocusState>({
  phase: 'focus',
  status: 'ready',
  endsAt: null,
  remainingMs: settings.focusMin * 60_000,
  durationMs: settings.focusMin * 60_000,
  session: 1,
  settings,
});
let today = $state({ sessions: 0, minutes: 0 });
/** Finished focus sessions of the last HISTORY_DAYS days: [ended at, length in ms]. */
const HISTORY_DAYS = 120;
let history = $state<[number, number][]>([]);
/** True for two minutes after a focus session is stopped before its end (mripat mrengut). */
let sulking = $state(false);
let sulkTimer: ReturnType<typeof setTimeout> | undefined;
/** Wall clock, refreshed once per second only while a phase is running. */
let now = $state(Date.now());
let ticker: ReturnType<typeof setTimeout> | undefined;

function tick() {
  now = Date.now();
  ticker = setTimeout(tick, 1000 - (now % 1000) + 5);
}

function apply(next: FocusState) {
  state = next;
  clearTimeout(ticker);
  if (next.status === 'running') tick();
  void refreshStats();
}

async function refreshStats() {
  const since = dayFromToday(0);
  today = inTauri ? await invoke('focus_stats', { since }) : previewStats(since);
  const historySince = dayFromToday(-HISTORY_DAYS);
  history = inTauri
    ? await invoke<[number, number][]>('focus_history', { since: historySince })
    : [...previewHistory(), ...previewSessions.filter((s) => s.endedAt >= historySince).map((s): [number, number] => [s.endedAt, s.minutes * 60_000])];
}

async function command(cmd: string) {
  apply(inTauri ? await invoke<FocusState>(cmd) : preview(cmd));
}

// The core pushes every change, including phases that end while the panel is closed.
on('focus://state', apply);

export const focus = {
  get state() {
    return state;
  },
  get today() {
    return today;
  },
  get history() {
    return history;
  },
  /** Time left right now; recomputed every second while running. */
  get remainingMs() {
    return state.status === 'running' && state.endsAt !== null ? Math.max(0, state.endsAt - now) : state.remainingMs;
  },
  get progress() {
    return state.durationMs > 0 ? 1 - focus.remainingMs / state.durationMs : 0;
  },
  get isBreak() {
    return state.phase !== 'focus';
  },
  get sulking() {
    return sulking;
  },
  load: async () => apply(inTauri ? await invoke<FocusState>('focus_state') : $state.snapshot(state)),
  start: () => command('focus_start'),
  pause: () => command('focus_pause'),
  resume: () => command('focus_resume'),
  skip: () => command('focus_skip'),
  stop: () => {
    if (state.phase === 'focus' && (state.status === 'running' || state.status === 'paused')) {
      sulking = true;
      clearTimeout(sulkTimer);
      sulkTimer = setTimeout(() => (sulking = false), 120_000);
    }
    return command('focus_stop');
  },
  /** Saves timer settings; a running phase keeps its length. */
  updateSettings: async (patch: Partial<FocusState['settings']>) => {
    if (inTauri) return apply(await invoke<FocusState>('focus_update_settings', { patch }));
    const s: FocusState = structuredClone($state.snapshot(state));
    Object.assign(s.settings, patch);
    if (s.status === 'ready' || s.status === 'waiting') {
      const min = s.phase === 'focus' ? s.settings.focusMin : s.phase === 'short_break' ? s.settings.shortBreakMin : s.settings.longBreakMin;
      Object.assign(s, { durationMs: min * 60_000, remainingMs: min * 60_000 });
    }
    apply(s);
  },
};

/** "18:42", or "1:05:00" for phases of an hour or more. */
export function formatCountdown(ms: number) {
  const total = Math.ceil(ms / 1000);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  const two = (n: number) => String(n).padStart(2, '0');
  return h ? `${h}:${two(m)}:${two(s)}` : `${two(m)}:${two(s)}`;
}

// ---- Browser preview -----------------------------------------------------
// Mirrors focus.rs closely enough for design work in a plain browser.

const previewSessions: { endedAt: number; minutes: number }[] = [];
let previewEnd: ReturnType<typeof setTimeout> | undefined;

/** A few weeks of made-up sessions so the kain has something to show in a browser. */
function previewHistory(): [number, number][] {
  const minutes = [0, 25, 50, 0, 100, 75, 25, 0, 0, 50, 125, 25, 50, 0, 25, 75, 50, 0, 25, 50];
  return minutes.flatMap((m, i) => (m ? [[dayFromToday(-(minutes.length - i)) + 15 * 3_600_000, m * 60_000]] : []));
}

function previewStats(since: number) {
  const done = previewSessions.filter((s) => s.endedAt >= since);
  return { sessions: done.length, minutes: done.reduce((sum, s) => sum + s.minutes, 0) };
}

function preview(cmd: string): FocusState {
  const s: FocusState = structuredClone($state.snapshot(state));
  const length = (p: Phase) =>
    60_000 * (p === 'focus' ? s.settings.focusMin : p === 'short_break' ? s.settings.shortBreakMin : s.settings.longBreakMin);
  const enter = (phase: Phase, session: number, status: Status) => {
    Object.assign(s, { phase, session, status, durationMs: length(phase), remainingMs: length(phase) });
    s.endsAt = status === 'running' ? Date.now() + length(phase) : null;
  };
  const following = (): [Phase, number] =>
    s.phase === 'focus'
      ? [s.session >= s.settings.sessionsPerCycle ? 'long_break' : 'short_break', s.session]
      : ['focus', s.phase === 'short_break' ? s.session + 1 : 1];

  if (cmd === 'focus_start' && s.status === 'ready') enter('focus', 1, 'running');
  else if (cmd === 'focus_start' && s.status === 'waiting') enter(s.phase, s.session, 'running');
  else if (cmd === 'focus_pause' && s.status === 'running') {
    Object.assign(s, { status: 'paused', remainingMs: Math.max(0, s.endsAt! - Date.now()), endsAt: null });
  } else if (cmd === 'focus_resume' && s.status === 'paused') {
    Object.assign(s, { status: 'running', endsAt: Date.now() + s.remainingMs });
  } else if (cmd === 'focus_skip' && s.status !== 'ready') enter(...following(), 'running');
  else if (cmd === 'focus_stop') enter('focus', 1, 'ready');

  clearTimeout(previewEnd);
  if (s.status === 'running') previewEnd = setTimeout(() => previewComplete(following), s.endsAt! - Date.now());
  return s;

  function previewComplete(next: () => [Phase, number]) {
    const finished = $state.snapshot(state);
    if (finished.phase === 'focus') previewSessions.push({ endedAt: Date.now(), minutes: finished.durationMs / 60_000 });
    const s2: FocusState = structuredClone(finished);
    const [phase, session] = next();
    Object.assign(s2, { phase, session, status: 'waiting', durationMs: length(phase), remainingMs: length(phase), endsAt: null });
    apply(s2);
    const of = s2.settings.sessionsPerCycle;
    const alert: AlertPayload =
      finished.phase === 'focus'
        ? { title: 'Focus session done', subtitle: `Session ${session} of ${of} · ${phase === 'long_break' ? 'long' : 'short'} break next`, kind: 'focus', focus: 'break-next' }
        : { title: "Break's over", subtitle: `Session ${session} of ${of} next`, kind: 'focus', focus: 'focus-next' };
    emitLocal('island://alert', alert);
  }
}
