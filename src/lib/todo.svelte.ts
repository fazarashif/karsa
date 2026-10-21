import { invoke } from '@tauri-apps/api/core';
import { inTauri, on } from './bridge';
import { react } from './reactions.svelte';

export type List = { id: number; name: string; position: number };

export type Task = {
  id: number;
  listId: number;
  parentId: number | null;
  title: string;
  done: boolean;
  /** Unix ms. For a date without a time, local midnight of that day. */
  dueAt: number | null;
  dueHasTime: boolean;
  position: number;
  completedAt: number | null;
  /** The meeting note this task came from, relative to the notes folder. */
  notePath: string | null;
};

export type Snapshot = { lists: List[]; tasks: Task[] };

/** The list that always exists (created by the core's first migration). */
export const INBOX_ID = 1;

let snapshot = $state<Snapshot>({ lists: [{ id: INBOX_ID, name: 'Inbox', position: 0 }], tasks: [] });
let error = $state<string | null>(null);
/** The list shown in the Todo tab; kept here so it survives closing the panel. */
let currentListId = $state(INBOX_ID);

async function run(cmd: string, args: Record<string, unknown> = {}) {
  try {
    snapshot = inTauri ? await invoke<Snapshot>(cmd, args) : preview(cmd, args);
    error = null;
    if (!snapshot.lists.some((l) => l.id === currentListId)) currentListId = INBOX_ID;
  } catch (e) {
    error = String(e);
  }
}

// Changes made in another window (e.g. "Add to todo" in a note window) arrive here.
on('todo://changed', (next) => {
  snapshot = next;
  if (!snapshot.lists.some((l) => l.id === currentListId)) currentListId = INBOX_ID;
});

export const todo = {
  get lists() {
    return snapshot.lists;
  },
  get tasks() {
    return snapshot.tasks;
  },
  get error() {
    return error;
  },
  get currentListId() {
    return currentListId;
  },
  set currentListId(id: number) {
    currentListId = id;
  },
  load: () => run('todo_snapshot'),
  createList: (name: string) => run('todo_create_list', { name }),
  renameList: (id: number, name: string) => run('todo_rename_list', { id, name }),
  deleteList: (id: number) => run('todo_delete_list', { id }),
  moveList: (id: number, direction: -1 | 1) => run('todo_move_list', { id, direction }),
  createTask: (listId: number, title: string, parentId: number | null = null, notePath: string | null = null) =>
    run('todo_create_task', { listId, title, parentId, notePath }),
  updateTask: (id: number, patch: { title?: string; done?: boolean; listId?: number }) => {
    if (patch.done) react('seneng', 1200);
    return run('todo_update_task', { id, ...patch });
  },
  setDue: (id: number, dueAt: number | null, hasTime: boolean) => run('todo_set_due', { id, dueAt, hasTime }),
  /** Adds a task with its deadline, e.g. "+ Task" on a calendar day. */
  createTaskDue: async (listId: number, title: string, dueAt: number, hasTime: boolean) => {
    await run('todo_create_task', { listId, title, parentId: null, notePath: null });
    const created = snapshot.tasks
      .filter((t) => t.listId === listId && t.parentId === null && t.title === title.trim())
      .sort((a, b) => b.id - a.id)[0];
    if (created) await run('todo_set_due', { id: created.id, dueAt, hasTime });
  },
  deleteTask: (id: number) => run('todo_delete_task', { id }),
};

// ---- Queries -------------------------------------------------------------

const byPosition = (a: Task, b: Task) => a.position - b.position || a.id - b.id;

export function topLevel(tasks: Task[], listId: number) {
  return tasks.filter((t) => t.listId === listId && t.parentId === null).sort(byPosition);
}

export function subtasksOf(tasks: Task[], parentId: number) {
  return tasks.filter((t) => t.parentId === parentId).sort(byPosition);
}

export function openCount(tasks: Task[], listId: number) {
  return tasks.filter((t) => t.listId === listId && t.parentId === null && !t.done).length;
}

/** Open top-level tasks across all lists: soonest due first, undated last. */
export function upcomingTasks(tasks: Task[]) {
  return tasks
    .filter((t) => t.parentId === null && !t.done)
    .sort((a, b) => (a.dueAt ?? Infinity) - (b.dueAt ?? Infinity) || byPosition(a, b));
}

function sameDay(a: Date, b: Date) {
  return a.getFullYear() === b.getFullYear() && a.getMonth() === b.getMonth() && a.getDate() === b.getDate();
}

/** Top-level tasks due today, and how many are still open (drives the smiling eyes). */
export function todayStatus(tasks: Task[], now: Date) {
  const due = tasks.filter((t) => t.parentId === null && t.dueAt !== null && sameDay(new Date(t.dueAt), now));
  return { total: due.length, open: due.filter((t) => !t.done).length };
}

// ---- Browser preview -----------------------------------------------------
// In a plain browser (npm run dev) there is no core, so the same commands run on a copy
// in memory. It mirrors todo.rs closely enough for design work; nothing is saved.

let previewId = 100;

function preview(cmd: string, a: Record<string, any>): Snapshot {
  const s: Snapshot = structuredClone($state.snapshot(snapshot));
  const find = (id: number) => s.tasks.find((t) => t.id === id);
  const title = (value: string) => {
    if (!value?.trim()) throw new Error('Title cannot be empty');
    return value.trim();
  };
  switch (cmd) {
    case 'todo_create_list':
      s.lists.push({ id: previewId++, name: title(a.name), position: s.lists.length });
      break;
    case 'todo_rename_list':
      s.lists.find((l) => l.id === a.id)!.name = title(a.name);
      break;
    case 'todo_delete_list':
      if (a.id === INBOX_ID) throw new Error('The Inbox cannot be deleted');
      s.lists = s.lists.filter((l) => l.id !== a.id);
      s.tasks = s.tasks.filter((t) => t.listId !== a.id);
      break;
    case 'todo_move_list': {
      const i = s.lists.findIndex((l) => l.id === a.id);
      const j = i + Math.sign(a.direction);
      if (j >= 0 && j < s.lists.length) [s.lists[i], s.lists[j]] = [s.lists[j], s.lists[i]];
      s.lists.forEach((l, k) => (l.position = k));
      break;
    }
    case 'todo_create_task': {
      const parent = a.parentId ? find(a.parentId) : undefined;
      const listId = parent?.listId ?? a.listId;
      const siblings = s.tasks.filter((t) => t.parentId === (a.parentId ?? null) && t.listId === listId);
      const positions = siblings.map((t) => t.position);
      const position = parent ? Math.max(0, ...positions) + 1 : Math.min(1, ...positions) - 1;
      s.tasks.push({
        id: previewId++,
        listId,
        parentId: a.parentId ?? null,
        title: title(a.title),
        done: false,
        dueAt: null,
        dueHasTime: false,
        position,
        completedAt: null,
        notePath: a.notePath ?? null,
      });
      break;
    }
    case 'todo_update_task': {
      const t = find(a.id)!;
      if (a.title !== undefined) t.title = title(a.title);
      if (a.done !== undefined) {
        t.done = a.done;
        t.completedAt = a.done ? Date.now() : null;
      }
      if (a.listId !== undefined) for (const x of s.tasks) if (x.id === a.id || x.parentId === a.id) x.listId = a.listId;
      break;
    }
    case 'todo_set_due': {
      const t = find(a.id)!;
      t.dueAt = a.dueAt;
      t.dueHasTime = a.dueAt !== null && a.hasTime;
      break;
    }
    case 'todo_delete_task':
      s.tasks = s.tasks.filter((t) => t.id !== a.id && t.parentId !== a.id);
      break;
  }
  s.lists.sort((x, y) => x.position - y.position || x.id - y.id);
  s.tasks.sort(byPosition);
  return s;
}
