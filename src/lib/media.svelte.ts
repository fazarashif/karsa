import { invoke } from '@tauri-apps/api/core';
import { inTauri, on } from './bridge';
import { react } from './reactions.svelte';

/** One app with media, for the session switcher. */
export type MediaSession = { id: string; app: string; title: string; playing: boolean };

export type MediaState = {
  /** Something has a media session (playing or paused). */
  active: boolean;
  playing: boolean;
  title: string;
  artist: string;
  album: string;
  /** A friendly app name, e.g. "Spotify" or "Chrome". */
  app: string;
  /** The site for a browser, when Karsa can tell it ("YouTube Music"); else empty. */
  site: string;
  /** The shown session's id in `sessions`. */
  id: string;
  /** The app's icon (a PNG data: URL), or empty. */
  appIcon: string;
  /** Album art as a data: URL, or empty. */
  art: string;
  canPrevious: boolean;
  canNext: boolean;
  canSeek: boolean;
  canShuffle: boolean;
  canRepeat: boolean;
  shuffle: boolean;
  repeat: 'off' | 'all' | 'one';
  /** The position when the app last reported it, the length, and when (Unix ms). */
  positionMs: number;
  durationMs: number;
  positionAt: number;
  rate: number;
  sessions: MediaSession[];
  /** The app's volume (0 to 1) when Windows has an audio session for it. */
  volume: number | null;
  muted: boolean;
};

export type MediaAction =
  | { kind: 'toggle' }
  | { kind: 'next' }
  | { kind: 'previous' }
  | { kind: 'seek'; ms: number }
  | { kind: 'shuffle'; on: boolean }
  | { kind: 'repeat'; mode: 'off' | 'all' | 'one' }
  | { kind: 'select'; id: string }
  | { kind: 'volume'; level: number }
  | { kind: 'mute'; on: boolean };

const idle: MediaState = {
  active: false,
  playing: false,
  title: '',
  artist: '',
  album: '',
  app: '',
  site: '',
  id: '',
  appIcon: '',
  art: '',
  canPrevious: false,
  canNext: false,
  canSeek: false,
  canShuffle: false,
  canRepeat: false,
  shuffle: false,
  repeat: 'off',
  positionMs: 0,
  durationMs: 0,
  positionAt: 0,
  rate: 1,
  sessions: [],
  volume: null,
  muted: false,
};

let state = $state<MediaState>(idle);

/** The track last seen playing, so resuming it does not count as a new one. */
let lastTrack = '';
const trackOf = (s: MediaState) => `${s.app}\n${s.title}\n${s.artist}`;

// The core pushes every change (track, play/pause, position, app).
on('media://changed', (next) => {
  if (next.playing && next.title && trackOf(next) !== lastTrack) {
    lastTrack = trackOf(next);
    react('nyanyi', 1600);
  }
  state = next;
});

export const media = {
  get state() {
    return state;
  },
  load: async () => {
    if (!inTauri) return;
    state = await invoke<MediaState>('media_state');
    if (state.playing) lastTrack = trackOf(state);
  },
  control: (action: MediaAction) => {
    if (inTauri) void invoke('media_control', { action });
    else previewControl(action);
  },
};

/** Where the track is now: the app reports its position only when something changes, so
    it moves on from the last report at the playback rate while playing. */
export function positionAt(s: MediaState, now: number) {
  const moved = s.playing ? (now - s.positionAt) * (s.rate || 1) : 0;
  return Math.max(0, Math.min(s.durationMs || Infinity, s.positionMs + moved));
}

/** "3:07" */
export function formatPlayTime(ms: number) {
  const total = Math.max(0, Math.floor(ms / 1000));
  return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, '0')}`;
}

/** "YouTube Music · Chrome", or just the app. */
export function sourceLabel(s: Pick<MediaState, 'site' | 'app'>) {
  return s.site ? `${s.site} · ${s.app}` : s.app;
}

// ---- Browser preview -----------------------------------------------------

function previewControl(action: MediaAction) {
  const now = Date.now();
  const pos = positionAt(state, now);
  if (action.kind === 'toggle') state = { ...state, playing: !state.playing, positionMs: pos, positionAt: now };
  else if (action.kind === 'seek') state = { ...state, positionMs: action.ms, positionAt: now };
  else if (action.kind === 'shuffle') state = { ...state, shuffle: action.on };
  else if (action.kind === 'repeat') state = { ...state, repeat: action.mode };
  else if (action.kind === 'volume') state = { ...state, volume: action.level, muted: false };
  else if (action.kind === 'mute') state = { ...state, muted: action.on };
  else if (action.kind === 'select') state = { ...state, id: action.id };
}

/** Browser preview: pretend something is playing (`karsa.media()` in the console). */
export function previewMedia(playing = true) {
  state = {
    ...idle,
    active: true,
    playing,
    title: 'Gambang Suling',
    artist: 'Ki Narto Sabdo',
    album: 'Langgam Jawa',
    app: 'Chrome',
    site: 'YouTube Music',
    id: 'Chrome',
    canPrevious: true,
    canNext: true,
    canSeek: true,
    positionMs: 42_000,
    durationMs: 223_000,
    positionAt: Date.now(),
    sessions: [
      { id: 'Chrome', app: 'Chrome', title: 'Gambang Suling', playing },
      { id: 'Spotify.exe', app: 'Spotify', title: 'Lir-ilir', playing: false },
    ],
    volume: 0.62,
  };
}
