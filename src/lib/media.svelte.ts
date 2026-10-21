import { invoke } from '@tauri-apps/api/core';
import { inTauri, on } from './bridge';

export type MediaState = {
  /** Something has a media session (playing or paused). */
  active: boolean;
  playing: boolean;
  title: string;
  artist: string;
  /** A friendly app name, e.g. "Spotify". */
  app: string;
  /** Album art as a data: URL, or empty. */
  art: string;
  canPrevious: boolean;
  canNext: boolean;
};

const idle: MediaState = { active: false, playing: false, title: '', artist: '', app: '', art: '', canPrevious: false, canNext: false };

let state = $state<MediaState>(idle);

// The core pushes every change (track, play/pause, app).
on('media://changed', (next) => (state = next));

export const media = {
  get state() {
    return state;
  },
  load: async () => {
    if (inTauri) state = await invoke<MediaState>('media_state');
  },
  control: (action: 'toggle' | 'next' | 'previous') => {
    if (inTauri) void invoke('media_control', { action });
    else if (action === 'toggle') state = { ...state, playing: !state.playing };
  },
};

/** Browser preview: pretend something is playing (`karsa.media()` in the console). */
export function previewMedia(playing = true) {
  state = {
    active: true,
    playing,
    title: 'Gambang Suling',
    artist: 'Ki Narto Sabdo',
    app: 'Spotify',
    art: '',
    canPrevious: true,
    canNext: true,
  };
}
