export type Tab = 'todo' | 'focus' | 'notes' | 'reminders' | 'settings';

export type NoteRequest = { path: string } | { meeting: { title: string; at: number } };

/** The panel's tab. Kept here so it survives closing the panel, and so the pill and
    Peek can open the panel on a specific tab (e.g. Focus while a timer runs). */
let tab = $state<Tab>('todo');
/** A task to open in the Todo tab, e.g. from a "From Todo" row in Reminders. */
let openTaskId = $state<number | null>(null);
/** A note to open, or a meeting note to create, in the Notes tab. */
let noteRequest = $state<NoteRequest | null>(null);
/** Set by the app shell so a view can close the panel (e.g. after "Open full"). */
let collapse: () => void = () => {};

export const panel = {
  get tab() {
    return tab;
  },
  set tab(value: Tab) {
    tab = value;
  },
  get openTaskId() {
    return openTaskId;
  },
  set openTaskId(value: number | null) {
    openTaskId = value;
  },
  get noteRequest() {
    return noteRequest;
  },
  set noteRequest(value: NoteRequest | null) {
    noteRequest = value;
  },
  /** Switches to the Todo tab with a task's detail view open. */
  showTask(id: number) {
    openTaskId = id;
    tab = 'todo';
  },
  /** Switches to the Notes tab with a note open. */
  showNote(path: string) {
    noteRequest = { path };
    tab = 'notes';
  },
  /** Switches to the Notes tab with a new meeting note, e.g. from a meeting reminder. */
  startMeeting(title: string, at: number) {
    noteRequest = { meeting: { title, at } };
    tab = 'notes';
  },
  get collapse() {
    return collapse;
  },
  set collapse(fn: () => void) {
    collapse = fn;
  },
};
