import { invoke } from '@tauri-apps/api/core';
import { inTauri, on } from './bridge';
import { isMeeting, noteTitle, parseMeeting } from './meeting';

export type NoteSummary = {
  /** Relative to the notes folder, with forward slashes. */
  path: string;
  kind: 'note' | 'meeting';
  title: string;
  snippet: string;
  topic: string;
  /** The meeting's date, or when the file last changed. */
  date: number;
  openActions: number;
  doneActions: number;
  /** The Outlook meeting a meeting note was started from, or "". */
  outlook: string;
};

export type NoteFile = { path: string; content: string; modified: number };
type Saved = { path: string; modified: number };

let list = $state<NoteSummary[]>([]);
let query = $state('');
let searchTimer: ReturnType<typeof setTimeout> | undefined;

async function refresh() {
  list = inTauri ? await invoke<NoteSummary[]>('notes_list', { query: query.trim() || null }) : previewList();
}

// Any window that saves or deletes a note tells the others.
on('notes://changed', () => void refresh());

export const notes = {
  get list() {
    return list;
  },
  get query() {
    return query;
  },
  set query(value: string) {
    query = value;
    clearTimeout(searchTimer);
    searchTimer = setTimeout(refresh, 150);
  },
  refresh,
  read: (path: string): Promise<NoteFile> => (inTauri ? invoke('notes_read', { path }) : previewRead(path)),
  save: (path: string | null, content: string, baseModified: number | null, force = false): Promise<Saved> =>
    inTauri ? invoke('notes_save', { path, content, baseModified, force }) : previewSave(path, content),
  remove: async (path: string) => {
    if (inTauri) await invoke('notes_delete', { path });
    else previewFiles.delete(path);
    await refresh();
  },
  /** The meeting note started from an Outlook meeting, looked for in all notes (the list
      may be filtered by a search). */
  findOutlook: async (key: string) => {
    const all = inTauri ? await invoke<NoteSummary[]>('notes_list', { query: null }) : previewList(true);
    return all.find((n) => n.kind === 'meeting' && n.outlook === key) ?? null;
  },
  reveal: (path: string) => (inTauri ? invoke('notes_reveal', { path }) : Promise.resolve()),
  openWindow: (path: string, title: string) => (inTauri ? invoke('notes_open_window', { path, title }) : Promise.resolve()),
};

/**
 * One open note: saves edits half a second after typing stops, one save at a time, and
 * notices when the file was changed by another program since it was read.
 */
export class NoteSession {
  path = $state<string | null>(null);
  status = $state<'saved' | 'saving' | 'unsaved' | 'conflict' | 'error'>('saved');
  error = $state('');
  #base: number | null;
  #pending: string | null = null;
  #timer: ReturnType<typeof setTimeout> | undefined;
  #saving: Promise<void> | null = null;

  constructor(path: string | null, modified: number | null) {
    this.path = path;
    this.#base = modified;
  }

  edit(content: string) {
    this.#pending = content;
    if (this.status !== 'conflict') this.status = 'unsaved';
    clearTimeout(this.#timer);
    this.#timer = setTimeout(() => void this.flush(), 500);
  }

  /** Saves now (e.g. before leaving the editor). */
  async flush(force = false) {
    clearTimeout(this.#timer);
    while (this.#saving) await this.#saving;
    const content = this.#pending;
    if (content === null || (this.status === 'conflict' && !force)) return;
    this.#pending = null;
    this.status = 'saving';
    this.#saving = notes
      .save(this.path, content, this.#base, force)
      .then((saved) => {
        this.path = saved.path;
        this.#base = saved.modified;
        this.status = this.#pending === null ? 'saved' : 'unsaved';
      })
      .catch((e) => {
        this.#pending ??= content;
        this.status = String(e) === 'conflict' ? 'conflict' : 'error';
        this.error = String(e);
      })
      .finally(() => (this.#saving = null));
    await this.#saving;
    // Typing during the save left newer text: schedule it.
    if (this.#pending !== null && this.#statusNow() === 'unsaved') this.edit(this.#pending);
  }

  /** Read through a method: the status changes while a save is awaited. */
  #statusNow() {
    return this.status;
  }

  /** Drops unsaved edits (the note is being deleted). */
  discard() {
    clearTimeout(this.#timer);
    this.#pending = null;
  }

  /** Conflict: overwrite the file with what is in Karsa. */
  keepMine() {
    return this.flush(true);
  }

  /** Conflict: drop Karsa's edits and load the file as it is on disk. */
  async reload() {
    this.#pending = null;
    const file = await notes.read(this.path!);
    this.#base = file.modified;
    this.status = 'saved';
    return file.content;
  }
}

// ---- Browser preview -----------------------------------------------------

const previewFiles = new Map<string, NoteFile>();

function previewList(all = false): NoteSummary[] {
  const q = all ? '' : query.trim().toLowerCase();
  return [...previewFiles.values()]
    .filter((f) => !q || f.content.toLowerCase().includes(q))
    .map((f) => {
      const meeting = isMeeting(f.content);
      const m = meeting ? parseMeeting(f.content) : null;
      const lines = f.content.split('\n').filter((l) => l.trim());
      return {
        path: f.path,
        kind: meeting ? ('meeting' as const) : ('note' as const),
        title: m ? m.title : noteTitle(f.content),
        snippet: m ? m.notes.split('\n')[0] ?? '' : lines[1] ?? '',
        topic: m?.topic ?? '',
        date: f.modified,
        openActions: m?.actions.filter((a) => !a.done).length ?? 0,
        doneActions: m?.actions.filter((a) => a.done).length ?? 0,
        outlook: m?.outlook ?? '',
      };
    })
    .sort((a, b) => b.date - a.date);
}

async function previewRead(path: string) {
  const file = previewFiles.get(path);
  if (!file) throw new Error('This note no longer exists');
  return file;
}

async function previewSave(path: string | null, content: string) {
  const meeting = isMeeting(content);
  const title = (meeting ? parseMeeting(content).title : noteTitle(content)).replace(/[<>:"/\\|?*]/g, ' ').trim();
  const next = meeting ? `Meetings/${parseMeeting(content).date.slice(0, 10)} ${title}.md` : `${title}.md`;
  if (path) previewFiles.delete(path);
  const saved = { path: next, content, modified: Date.now() };
  previewFiles.set(next, saved);
  void refresh();
  return { path: next, modified: saved.modified };
}
