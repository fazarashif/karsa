import { invoke } from '@tauri-apps/api/core';
import { inTauri, on } from './bridge';

export type ShelfItem = {
  id: number;
  path: string;
  name: string;
  folder: boolean;
  /** Bytes, for files that still exist. */
  size: number | null;
  /** False when the original was moved or deleted. */
  exists: boolean;
  addedAt: number;
  /** The icon Explorer shows for it (a PNG data URL); null for a type Windows does not know. */
  icon: string | null;
  /** "Microsoft PowerPoint Presentation", "MD File" and the like. */
  typeName: string;
};

/** "PPTX" for "Slides.pptx"; empty when the name has no extension. */
export function extensionOf(name: string) {
  const dot = name.lastIndexOf('.');
  return dot > 0 ? name.slice(dot + 1).toUpperCase() : '';
}

let items = $state<ShelfItem[]>([]);
let error = $state<string | null>(null);

async function refresh() {
  try {
    items = inTauri ? await invoke<ShelfItem[]>('shelf_list') : [...previewItems];
    error = null;
  } catch (e) {
    error = String(e);
  }
}

async function run(cmd: string, args: Record<string, unknown> = {}, reload = true) {
  try {
    if (inTauri) await invoke(cmd, args);
    else previewRun(cmd, args);
    error = null;
  } catch (e) {
    error = String(e);
  }
  if (reload) await refresh();
}

on('shelf://changed', () => void refresh());

export const shelf = {
  get items() {
    return items;
  },
  get error() {
    return error;
  },
  load: refresh,
  /** Files dropped on the island. Returns how many were kept. */
  async add(paths: string[]) {
    try {
      const added = inTauri ? await invoke<number>('shelf_add', { paths }) : previewAdd(paths);
      await refresh();
      return added;
    } catch (e) {
      error = String(e);
      return 0;
    }
  },
  /** Files copied in Explorer (Ctrl+C), pasted onto the shelf. Returns how many. */
  async paste() {
    try {
      const added = inTauri ? await invoke<number>('shelf_paste') : 0;
      error = null;
      await refresh();
      return added;
    } catch (e) {
      error = String(e);
      return 0;
    }
  },
  /** "Add files…": the Windows file picker. */
  async pick() {
    try {
      const added = inTauri ? await invoke<number>('shelf_pick') : 0;
      error = null;
      await refresh();
      return added;
    } catch (e) {
      error = String(e);
      return 0;
    }
  },
  remove: (id: number) => run('shelf_remove', { id }),
  clear: () => run('shelf_clear'),
  open: (id: number) => run('shelf_open', { id }, false),
  reveal: (id: number) => run('shelf_reveal', { id }, false),
  copy: (ids: number[]) => run('shelf_copy', { ids }, false),
  /** Starts a Windows drag of the files; call from dragstart, while the button is down. */
  drag: (ids: number[]) => run('shelf_drag', { ids }, false),
};

/** Hands files dropped on the island to the core. Only WebView2 can turn a dropped `File`
    into its path, so they go as a web message (`accept_drops` in shelf.rs). Returns false
    outside WebView2 (the browser preview). */
export function dropFiles(files: FileList): boolean {
  const webview = (window as { chrome?: { webview?: { postMessageWithAdditionalObjects?: (message: string, objects: FileList) => void } } })
    .chrome?.webview;
  if (!webview?.postMessageWithAdditionalObjects) return false;
  webview.postMessageWithAdditionalObjects('karsa:shelf-drop', files);
  return true;
}

export function formatSize(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  const units = ['KB', 'MB', 'GB'];
  let value = bytes / 1024;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit++;
  }
  return `${value < 10 ? value.toFixed(1) : Math.round(value)} ${units[unit]}`;
}

// ---- Browser preview -----------------------------------------------------

let previewId = 3;
let previewItems: ShelfItem[] = [
  { id: 3, path: 'C:\\Users\\you\\Documents\\meeting.md', name: 'meeting.md', folder: false, size: 3_200, exists: true, addedAt: Date.now() - 5 * 60_000, icon: null, typeName: 'MD File' },
  { id: 2, path: 'C:\\Users\\you\\Downloads\\Q3 roadmap.pdf', name: 'Q3 roadmap.pdf', folder: false, size: 2_400_000, exists: true, addedAt: Date.now() - 20 * 60_000, icon: null, typeName: 'PDF File' },
  { id: 1, path: 'C:\\Users\\you\\Pictures\\whiteboard.jpg', name: 'whiteboard.jpg', folder: false, size: 840_000, exists: false, addedAt: Date.now() - 3 * 3_600_000, icon: null, typeName: 'JPG File' },
];

function previewAdd(paths: string[]) {
  for (const path of paths) {
    previewItems = [
      { id: previewId++, path, name: path.split(/[\\/]/).pop() ?? path, folder: false, size: 12_000, exists: true, addedAt: Date.now(), icon: null, typeName: '' },
      ...previewItems.filter((i) => i.path !== path),
    ];
  }
  return paths.length;
}

function previewRun(cmd: string, a: Record<string, any>) {
  if (cmd === 'shelf_remove') previewItems = previewItems.filter((i) => i.id !== a.id);
  if (cmd === 'shelf_clear') previewItems = [];
}
