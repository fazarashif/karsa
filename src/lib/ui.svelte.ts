import type { View } from './config';
import type { NewMeeting } from './meeting';

export type Tab = 'todo' | 'focus' | 'notes' | 'reminders' | 'calendar' | 'shelf' | 'settings';

export type NoteRequest = { path: string } | { meeting: NewMeeting };

/** The panel's tab. Kept here so it survives closing the panel, and so the pill and
    Peek can open the panel on a specific tab (e.g. Focus while a timer runs). */
let tab = $state<Tab>('todo');
/** A task to open in the Todo tab, e.g. from a "From Todo" row in Reminders. */
let openTaskId = $state<number | null>(null);
/** A note to open, or a meeting note to create, in the Notes tab. */
let noteRequest = $state<NoteRequest | null>(null);
/** Set by the app shell so a view can close the panel (e.g. after "Open full"). */
let collapse: () => void = () => {};
/** Set by the app shell: closes the panel and gives focus back to the app the user was in. */
let closeAndReturn: () => void = () => {};
/** The Rampung ritual covers the tabs while it runs. */
let rampung = $state(false);
/** Quick capture (the global hotkey) covers the tabs while it is open. */
let capture = $state(false);
/** Which half of the Shelf tab is showing. */
let shelf = $state<'clipboard' | 'files'>('clipboard');
/** A note is open in the Notes tab, which needs the tall editor shape. */
let editing = $state(false);
/** Now playing covers the panel (opened from the media pill or Peek's Playing row). */
let nowPlaying = $state(false);
/** Now playing shows its lyrics area; kept while the app runs. */
let lyricsOpen = $state(false);
/** Opened from the panel's header (back goes to the tabs) or from the pill or Peek. */
let nowPlayingFrom = $state<'panel' | 'pill'>('pill');

export const panel = {
  /** What the panel shows, which decides the island's shape. */
  get view(): View {
    if (nowPlaying) return lyricsOpen ? 'nowplaying-lyrics' : 'nowplaying';
    if (capture) return 'capture';
    if (rampung) return 'rampung';
    if (tab === 'notes' && editing) return 'editor';
    return tab;
  },
  get nowPlaying() {
    return nowPlaying;
  },
  set nowPlaying(value: boolean) {
    nowPlaying = value;
  },
  get nowPlayingFrom() {
    return nowPlayingFrom;
  },
  /** Opens Now playing over the panel. */
  showNowPlaying(from: 'panel' | 'pill') {
    nowPlayingFrom = from;
    nowPlaying = true;
  },
  get lyricsOpen() {
    return lyricsOpen;
  },
  set lyricsOpen(value: boolean) {
    lyricsOpen = value;
  },
  get editing() {
    return editing;
  },
  set editing(value: boolean) {
    editing = value;
  },
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
  /** Switches to the Notes tab with a new meeting note, e.g. from a meeting reminder; an
      Outlook meeting that already has one opens it. */
  startMeeting(meeting: NewMeeting) {
    noteRequest = { meeting };
    tab = 'notes';
  },
  get rampung() {
    return rampung;
  },
  set rampung(value: boolean) {
    rampung = value;
  },
  get shelf() {
    return shelf;
  },
  set shelf(value: 'clipboard' | 'files') {
    shelf = value;
  },
  get capture() {
    return capture;
  },
  set capture(value: boolean) {
    capture = value;
  },
  get closeAndReturn() {
    return closeAndReturn;
  },
  set closeAndReturn(fn: () => void) {
    closeAndReturn = fn;
  },
  get collapse() {
    return collapse;
  },
  set collapse(fn: () => void) {
    collapse = fn;
  },
};
