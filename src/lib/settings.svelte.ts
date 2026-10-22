import { invoke } from '@tauri-apps/api/core';
import { inTauri, on } from './bridge';

export type AppSettings = {
  autostart: boolean;
  hideInFullscreen: boolean;
  /** Bottom-right above the taskbar, or a notch at the top centre. */
  islandPosition: 'corner' | 'top';
  /** Lead time new reminders and todo deadlines start with. */
  reminderLeadMin: number;
  notesFolder: string;
  /** False while the notes folder is the default `Documents\Karsa\Notes`. */
  notesFolderCustom: boolean;
  version: string;
  /** Shown in the greeting; empty when not set. */
  name: string;
  /** Local "YYYY-MM-DD", or empty. */
  birthday: string;
  rampungEnabled: boolean;
  /** Local "HH:MM", weekdays. */
  rampungTime: string;
  /** Local "YYYY-MM-DD" of the last finished Rampung. */
  rampungDoneDate: string;
  /** "Win+Shift+K" and the like; empty when quick capture has no hotkey. */
  captureHotkey: string;
  /** False when another app already holds the hotkey. */
  captureHotkeyOk: boolean;
  clipboardEnabled: boolean;
  clipboardPaused: boolean;
  /** Show what is playing in the pill when nothing else needs it. */
  mediaInPill: boolean;
  /** Look up lyrics on lrclib.net (off until turned on: Karsa's only network use). */
  lyricsEnabled: boolean;
  /** Show the current lyric line in the idle pill (needs lyrics). */
  lyricFocus: boolean;
  /** Read meetings from classic Outlook on this PC (off until turned on). */
  outlookEnabled: boolean;
  /** Minutes before an Outlook meeting that Karsa alerts; -1 for no alerts. */
  outlookLeadMin: number;
};

export type SettingsPatch = Partial<
  Pick<
    AppSettings,
    | 'autostart'
    | 'hideInFullscreen'
    | 'islandPosition'
    | 'reminderLeadMin'
    | 'name'
    | 'birthday'
    | 'rampungEnabled'
    | 'rampungTime'
    | 'rampungDoneDate'
    | 'captureHotkey'
    | 'clipboardEnabled'
    | 'clipboardPaused'
    | 'mediaInPill'
    | 'lyricsEnabled'
    | 'lyricFocus'
    | 'outlookEnabled'
    | 'outlookLeadMin'
  >
>;

let current = $state<AppSettings>({
  autostart: true,
  hideInFullscreen: true,
  islandPosition: 'corner',
  reminderLeadMin: 0,
  notesFolder: 'C:\\Users\\you\\Documents\\Karsa\\Notes',
  notesFolderCustom: false,
  version: '0.1.0',
  name: '',
  birthday: '',
  rampungEnabled: true,
  rampungTime: '17:00',
  rampungDoneDate: '',
  captureHotkey: 'Win+Shift+K',
  captureHotkeyOk: true,
  clipboardEnabled: true,
  clipboardPaused: false,
  mediaInPill: true,
  lyricsEnabled: false,
  lyricFocus: false,
  outlookEnabled: false,
  outlookLeadMin: 10,
});
let error = $state<string | null>(null);

async function run(cmd: string, args: Record<string, unknown> = {}) {
  try {
    const next = inTauri ? await invoke<AppSettings | null>(cmd, args) : preview(cmd, args);
    if (next) current = next;
    error = null;
  } catch (e) {
    error = String(e);
  }
}

// The tray can switch Start with Windows too.
on('settings://changed', (next) => (current = next));

export const settings = {
  get current() {
    return current;
  },
  get error() {
    return error;
  },
  load: () => run('settings_get'),
  update: (patch: SettingsPatch) => run('settings_update', { patch }),
  /** Opens the system folder picker; nothing changes if it is cancelled. */
  chooseNotesFolder: () => run('settings_choose_notes_folder'),
  resetNotesFolder: () => run('settings_reset_notes_folder'),
  openNotesFolder: () => (inTauri ? invoke('settings_open_notes_folder').catch((e) => (error = String(e))) : undefined),
};

// ---- Browser preview -----------------------------------------------------

function preview(cmd: string, a: Record<string, any>): AppSettings | null {
  const s = structuredClone($state.snapshot(current));
  if (cmd === 'settings_update') Object.assign(s, a.patch);
  if (cmd === 'settings_choose_notes_folder') Object.assign(s, { notesFolder: 'D:\\OneDrive\\Notes', notesFolderCustom: true });
  if (cmd === 'settings_reset_notes_folder') Object.assign(s, { notesFolder: 'C:\\Users\\you\\Documents\\Karsa\\Notes', notesFolderCustom: false });
  return s;
}
