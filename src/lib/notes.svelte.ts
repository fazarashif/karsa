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
  /** The project (a folder in the notes folder) it is in, or "" for none. */
  project: string;
};

export type NoteFile = { path: string; content: string; modified: number };
type Saved = { path: string; modified: number };

let list = $state<NoteSummary[]>([]);
let query = $state('');
let searchTimer: ReturnType<typeof setTimeout> | undefined;
/** The projects: folders in the notes folder (notes.rs). */
let projects = $state<string[]>([]);
/** The project the list shows: null for every note, "" for notes in no project. */
let selected = $state<string | null>(null);

async function refresh() {
  [list, projects] = inTauri
    ? await Promise.all([
        invoke<NoteSummary[]>('notes_list', { query: query.trim() || null }),
        invoke<string[]>('notes_projects'),
      ])
    : [previewList(), previewProjects()];
  // A project renamed or deleted elsewhere (in File Explorer) is no longer shown.
  if (selected && !projects.includes(selected)) selected = null;
}

/** The project a note's path is in: its first folder, unless that is the top `Meetings`. */
export function projectOf(path: string) {
  const parts = path.split('/');
  return parts.length > 1 && parts[0].toLowerCase() !== 'meetings' ? parts[0] : '';
}

/** Project changes, then a fresh list (the core also tells every window). */
async function changeProjects(cmd: string, args: Record<string, string>, preview: () => void) {
  if (inTauri) await invoke(cmd, args);
  else preview();
  await refresh();
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
  get projects() {
    return projects;
  },
  get selected() {
    return selected;
  },
  set selected(value: string | null) {
    selected = value;
  },
  refresh,
  read: (path: string): Promise<NoteFile> => (inTauri ? invoke('notes_read', { path }) : previewRead(path)),
  /** `project` places a new note ("" for none); a saved note stays in its own. */
  save: (path: string | null, content: string, baseModified: number | null, force = false, project = ''): Promise<Saved> =>
    inTauri ? invoke('notes_save', { path, content, baseModified, force, project }) : previewSave(path, content, project),
  /** Moves a note into another project ("" for none). */
  move: (path: string, project: string): Promise<Saved> =>
    inTauri ? invoke('notes_move', { path, project }) : previewMove(path, project),
  createProject: (name: string) =>
    changeProjects('notes_project_create', { name }, () => previewProjectSet.add(name.trim())),
  renameProject: (name: string, newName: string) =>
    changeProjects('notes_project_rename', { name, newName }, () => previewRename(name, newName.trim())),
  /** Only an empty project can go (its folder to the Recycle Bin). */
  deleteProject: (name: string) =>
    changeProjects('notes_project_delete', { name }, () => {
      if ([...previewFiles.keys()].some((p) => projectOf(p) === name)) throw new Error('Move or delete its notes first');
      previewProjectSet.delete(name);
    }),
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
  /** Where a new note is saved first ("" for no project); a saved note's path says it. */
  project = $state('');
  #base: number | null;
  #pending: string | null = null;
  #timer: ReturnType<typeof setTimeout> | undefined;
  #saving: Promise<void> | null = null;

  constructor(path: string | null, modified: number | null, project = '') {
    this.path = path;
    this.#base = modified;
    this.project = project;
  }

  /** Moves the note into another project ("" for none), after saving what was typed. */
  async moveTo(project: string) {
    await this.flush();
    if (!this.path) {
      this.project = project;
      return;
    }
    const moved = await notes.move(this.path, project);
    this.path = moved.path;
    this.#base = moved.modified;
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
      .save(this.path, content, this.#base, force, this.project)
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
const previewProjectSet = new Set<string>(['Karsa']);

function previewProjects() {
  const names = new Set([...previewProjectSet, ...[...previewFiles.keys()].map(projectOf).filter(Boolean)]);
  return [...names].sort((a, b) => a.toLowerCase().localeCompare(b.toLowerCase()));
}

function previewRename(name: string, newName: string) {
  previewProjectSet.delete(name);
  previewProjectSet.add(newName);
  for (const [path, file] of [...previewFiles]) {
    if (projectOf(path) !== name) continue;
    previewFiles.delete(path);
    const moved = newName + path.slice(name.length);
    previewFiles.set(moved, { ...file, path: moved });
  }
}

async function previewMove(path: string, project: string) {
  const file = await previewRead(path);
  previewFiles.delete(path);
  return previewSave(null, file.content, project);
}

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
        project: projectOf(f.path),
      };
    })
    .sort((a, b) => b.date - a.date);
}

async function previewRead(path: string) {
  const file = previewFiles.get(path);
  if (!file) throw new Error('This note no longer exists');
  return file;
}

async function previewSave(path: string | null, content: string, project = '') {
  const meeting = isMeeting(content);
  const title = (meeting ? parseMeeting(content).title : noteTitle(content)).replace(/[<>:"/\\|?*]/g, ' ').trim();
  const dir = path ? projectOf(path) : project;
  const next = (dir ? `${dir}/` : '') + (meeting ? `Meetings/${parseMeeting(content).date.slice(0, 10)} ${title}.md` : `${title}.md`);
  if (path) previewFiles.delete(path);
  const saved = { path: next, content, modified: Date.now() };
  previewFiles.set(next, saved);
  void refresh();
  return { path: next, modified: saved.modified };
}
