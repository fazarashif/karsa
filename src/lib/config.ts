export type Mode = 'idle' | 'peek' | 'expanded' | 'alert' | 'drop';

export type Box = { width: number; height: number; radius: number };

/**
 * The island window never resizes. It stays at this size, anchored to the bottom-right
 * of the work area or to the top centre (the notch), and a window region makes
 * everything outside the box click-through. Must match `WINDOW_SIZE` in
 * src-tauri/src/island.rs and the window in tauri.conf.json.
 */
export const WINDOW = { width: 400, height: 540 } as const;

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

/** Peek grows by one row while something is playing (the "Playing" row). */
export const PEEK_MEDIA_ROW = 26;

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
