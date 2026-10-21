import { invoke } from '@tauri-apps/api/core';
import { inTauri, on } from './bridge';

export type ClipItem = { id: number; text: string; copiedAt: number };

let items = $state<ClipItem[]>([]);
let query = $state('');
let error = $state<string | null>(null);
/** Only while the Shelf shows the clipboard; the list is not kept up to date otherwise. */
let watching = false;
let queryTimer: ReturnType<typeof setTimeout> | undefined;

async function refresh() {
  try {
    items = inTauri ? await invoke<ClipItem[]>('clipboard_list', { query: query.trim() || null }) : preview();
    error = null;
  } catch (e) {
    error = String(e);
  }
}

async function run(cmd: string, args: Record<string, unknown> = {}) {
  try {
    if (inTauri) await invoke(cmd, args);
    else previewRun(cmd, args);
    error = null;
  } catch (e) {
    error = String(e);
  }
  await refresh();
}

on('clipboard://changed', () => watching && void refresh());

export const clipboard = {
  get items() {
    return items;
  },
  get query() {
    return query;
  },
  set query(value: string) {
    query = value;
    clearTimeout(queryTimer);
    queryTimer = setTimeout(refresh, 150);
  },
  get error() {
    return error;
  },
  /** Starts following changes; returns the function that stops. */
  watch() {
    watching = true;
    void refresh();
    return () => (watching = false);
  },
  copy: (id: number) => run('clipboard_copy', { id }),
  remove: (id: number) => run('clipboard_delete', { id }),
  clear: () => run('clipboard_clear'),
};

// ---- Browser preview -----------------------------------------------------

let previewItems: ClipItem[] = [
  { id: 3, text: 'https://github.com/fazarashif/karsa', copiedAt: Date.now() - 4 * 60_000 },
  { id: 2, text: 'Meeting moved to 15:30, room Kawung 2', copiedAt: Date.now() - 52 * 60_000 },
  { id: 1, text: 'SELECT * FROM tasks WHERE done = 0 ORDER BY due_at;', copiedAt: Date.now() - 5 * 3_600_000 },
];

function preview() {
  const q = query.trim().toLowerCase();
  return previewItems.filter((i) => !q || i.text.toLowerCase().includes(q)).sort((a, b) => b.copiedAt - a.copiedAt);
}

function previewRun(cmd: string, a: Record<string, any>) {
  if (cmd === 'clipboard_delete') previewItems = previewItems.filter((i) => i.id !== a.id);
  if (cmd === 'clipboard_clear') previewItems = [];
  if (cmd === 'clipboard_copy') {
    const item = previewItems.find((i) => i.id === a.id);
    if (item) {
      item.copiedAt = Date.now();
      void navigator.clipboard?.writeText(item.text);
    }
  }
}
