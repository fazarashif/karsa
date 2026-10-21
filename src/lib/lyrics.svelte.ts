import { invoke } from '@tauri-apps/api/core';
import { inTauri, on } from './bridge';
import { media, positionAt, type MediaState } from './media.svelte';

/**
 * Lyrics for what is playing (design v2, 4.9). They are looked up only while the user has
 * turned lyrics on and something shows them (Now playing's lyrics area or lyric focus).
 * The current line moves on a timer set for the next line's timestamp, never by polling.
 */
export type LyricsStatus = 'timed' | 'plain' | 'instrumental' | 'missing' | 'off' | 'error' | 'loading';

type Lyrics = {
  key: string;
  status: Exclude<LyricsStatus, 'loading'>;
  /** [start ms, text] */
  lines: [number, string][];
  plain: string;
  offsetMs: number;
  message: string;
};

let status = $state<LyricsStatus>('off');
let lines = $state<[number, string][]>([]);
let plain = $state('');
let key = $state('');
let offsetMs = $state(0);
let message = $state('');
/** The index of the line being sung, or -1 before the first one. */
let current = $state(-1);
/** The track the lyrics belong to. */
let loadedFor = '';
let timer: ReturnType<typeof setTimeout> | undefined;

const trackOf = (s: MediaState) => `${s.app}\n${s.title}\n${s.artist}\n${s.album}`;

function apply(result: Lyrics) {
  status = result.status;
  lines = result.lines;
  plain = result.plain;
  key = result.key;
  offsetMs = result.offsetMs;
  message = result.message;
  schedule();
}

/** Finds the line being sung and sets a timer for the next one while playing. */
function schedule() {
  clearTimeout(timer);
  const s = media.state;
  if (status !== 'timed' || !lines.length) {
    current = -1;
    return;
  }
  const at = positionAt(s, Date.now()) + offsetMs;
  let i = -1;
  while (i + 1 < lines.length && lines[i + 1][0] <= at) i++;
  current = i;
  if (s.playing && i + 1 < lines.length) {
    const wait = (lines[i + 1][0] - at) / (s.rate || 1);
    timer = setTimeout(schedule, Math.max(30, wait + 20));
  }
}

// A seek, pause or new track moves the lines; read after the media store has the state.
on('media://changed', () => queueMicrotask(schedule));

export const lyrics = {
  get status() {
    return status;
  },
  get lines() {
    return lines;
  },
  get plain() {
    return plain;
  },
  get current() {
    return current;
  },
  /** The line being sung, or empty. */
  get line() {
    return current >= 0 ? lines[current][1] : '';
  },
  get offsetMs() {
    return offsetMs;
  },
  get message() {
    return message;
  },
  /** Loads the lyrics for the playing track, once per track (`again` asks LRCLIB anew). */
  async load(again = false) {
    const s = media.state;
    const track = trackOf(s);
    if (!s.active || !s.title) return;
    if (!again && track === loadedFor) return;
    loadedFor = track;
    status = 'loading';
    lines = [];
    current = -1;
    try {
      const result: Lyrics = inTauri
        ? await invoke('lyrics_get', { title: s.title, artist: s.artist, album: s.album, durationMs: s.durationMs, again })
        : previewLyrics();
      if (trackOf(media.state) === track) apply(result);
    } catch (e) {
      status = 'error';
      message = String(e);
    }
  },
  /** Moves the timing by `delta` ms for this song, and remembers it. */
  nudge(delta: number) {
    offsetMs = Math.max(-30_000, Math.min(30_000, offsetMs + delta));
    if (inTauri && key) void invoke('lyrics_offset', { key, offsetMs });
    schedule();
  },
  /** Forgets the loaded track, e.g. when lyrics are turned on or off. */
  reset() {
    loadedFor = '';
    status = 'off';
    lines = [];
    current = -1;
    clearTimeout(timer);
  },
};

// ---- Browser preview -----------------------------------------------------

function previewLyrics(): Lyrics {
  const words = ['The line before this one', 'A quiet line about the morning', 'The line being sung right now', 'Then the line that comes next', 'And the one after it', 'A last line to close the verse'];
  return {
    key: 'preview',
    status: 'timed',
    lines: Array.from({ length: 40 }, (_, i) => [i * 5000, words[i % words.length]] as [number, string]),
    plain: '',
    offsetMs: 0,
    message: '',
  };
}
