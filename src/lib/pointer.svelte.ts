/** Where the cursor is while it is over the island (page coordinates), so the mripat
    can glance at it. Updated at most once per frame, and only while the cursor moves
    over the island: nothing runs when it is elsewhere. */
let position = $state<{ x: number; y: number } | null>(null);
let frame = 0;
let latest = { x: 0, y: 0 };

export const pointer = {
  get position() {
    return position;
  },
  move(x: number, y: number) {
    latest = { x, y };
    if (!frame) {
      frame = requestAnimationFrame(() => {
        frame = 0;
        position = latest;
      });
    }
  },
  leave() {
    cancelAnimationFrame(frame);
    frame = 0;
    position = null;
  },
};
