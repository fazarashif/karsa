import { invoke } from '@tauri-apps/api/core';
import { inTauri, on } from './bridge';

export type AppSettings = {
  autostart: boolean;
  hideInFullscreen: boolean;
  /** Lead time new reminders and todo deadlines start with. */
  reminderLeadMin: number;
  notesFolder: string;
  /** False while the notes folder is the default `Documents\Karsa\Notes`. */
  notesFolderCustom: boolean;
  version: string;
};

export type SettingsPatch = Partial<Pick<AppSettings, 'autostart' | 'hideInFullscreen' | 'reminderLeadMin'>>;

let current = $state<AppSettings>({
  autostart: true,
  hideInFullscreen: true,
  reminderLeadMin: 0,
  notesFolder: 'C:\\Users\\you\\Documents\\Karsa\\Notes',
  notesFolderCustom: false,
  version: '0.1.0',
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
