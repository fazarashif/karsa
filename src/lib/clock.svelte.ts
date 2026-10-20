/** Minute-resolution clock. Wakes once per minute, aligned to the minute boundary. */
let now = $state(new Date());

function msToNextMinute() {
  return 60_000 - (Date.now() % 60_000) + 20;
}

function tick() {
  now = new Date();
  setTimeout(tick, msToNextMinute());
}

setTimeout(tick, msToNextMinute());

// Timers are throttled while the window is hidden (e.g. during fullscreen apps).
document.addEventListener('visibilitychange', () => {
  if (!document.hidden) now = new Date();
});

export const clock = {
  get now() {
    return now;
  },
};
