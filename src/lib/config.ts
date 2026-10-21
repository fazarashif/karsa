export type Mode = 'idle' | 'peek' | 'expanded' | 'alert' | 'drop';

export type Box = { width: number; height: number; radius: number };

/**
 * The island window never resizes. It stays at this size, anchored to the bottom-right
 * of the work area or to the top centre (the notch), and a window region makes
 * everything outside the box click-through. Must match `WINDOW_SIZE` in
 * src-tauri/src/island.rs and the window in tauri.conf.json.
 */
export const WINDOW = { width: 560, height: 640 } as const;

/** Idle pill. Its width follows the label, clamped by CSS min/max width. */
export const PILL = { height: 32, radius: 16 } as const;

/** The idle pill as a notch at the top: slimmer, with only its bottom corners rounded.
    Must match `--notch-h` and `--radius-notch` in design/tokens.css. */
export const NOTCH = { height: 26, radius: 12 } as const;

/** Fixed sizes of the larger states. Each content component renders at exactly this size. */
export const BOX: Record<Exclude<Mode, 'idle'>, Box> = {
  peek: { width: 320, height: 172, radius: 22 },
  expanded: { width: 380, height: 520, radius: 26 },
  alert: { width: 320, height: 112, radius: 22 },
  /** Files dragged over the island: a target big enough to drop on. */
  drop: { width: 320, height: 112, radius: 22 },
};

/** Peek grows by the Playing row and its progress line while something plays (210 high). */
export const PEEK_MEDIA_ROW = 38;

/** What the open panel shows; each has its own shape (design v2, section 3). */
export type View =
  | 'todo'
  | 'notes'
  | 'editor'
  | 'focus'
  | 'reminders'
  | 'calendar'
  | 'shelf'
  | 'settings'
  | 'rampung'
  | 'capture'
  | 'nowplaying'
  | 'nowplaying-lyrics';

/** The open island's shape per view. Every one uses the panel radius. Must match the
    shape tokens in design/tokens.css. */
export const SHAPE: Record<View, { width: number; height: number }> = {
  todo: { width: 380, height: 520 },
  notes: { width: 380, height: 520 },
  editor: { width: 420, height: 620 },
  focus: { width: 380, height: 500 },
  reminders: { width: 380, height: 520 },
  calendar: { width: 400, height: 600 },
  shelf: { width: 400, height: 520 },
  settings: { width: 400, height: 560 },
  rampung: { width: 380, height: 480 },
  // Quick capture becomes a 360 × 140 card in M9; until then it shares the panel.
  capture: { width: 380, height: 520 },
  nowplaying: { width: 520, height: 212 },
  'nowplaying-lyrics': { width: 520, height: 420 },
};

export const TIMING = {
  /** How long a shrinking morph keeps its old clickable area; the spring is ~95% done by then. */
  regionSettle: 450,
  hoverOpen: 300,
  /** The notch sits where the pointer passes on its way to tabs and title bars. */
  hoverOpenNotch: 450,
  hoverClose: 400,
  alertHold: 8000,
  snooze: 5 * 60_000,
} as const;
