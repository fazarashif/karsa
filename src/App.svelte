<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import { BOX, NOTCH, PEEK_MEDIA_ROW, PILL, SHAPE, TIMING, type Box, type Mode } from './lib/config';
  import { core, emitLocal, inTauri, on, type AlertPayload } from './lib/bridge';
  import { clock } from './lib/clock.svelte';
  import { handleEscape } from './lib/escape';
  import { focus, formatCountdown } from './lib/focus.svelte';
  import { formatTime, prefs, toDateInput } from './lib/format.svelte';
  import { contentOut } from './lib/motion';
  import { lyrics } from './lib/lyrics.svelte';
  import { media, previewMedia } from './lib/media.svelte';
  import { idleMood } from './lib/mood.svelte';
  import { pointer } from './lib/pointer.svelte';
  import { react } from './lib/reactions.svelte';
  import { nextToday, reminders } from './lib/reminders.svelte';
  import { settings } from './lib/settings.svelte';
  import { dropFiles } from './lib/shelf.svelte';
  import { todo } from './lib/todo.svelte';
  import { panel } from './lib/ui.svelte';
  import Alert from './lib/Alert.svelte';
  import Drop from './lib/Drop.svelte';
  import Panel from './lib/Panel.svelte';
  import Peek from './lib/Peek.svelte';
  import Pill, { type PillContent } from './lib/Pill.svelte';

  let mode = $state<Mode>('idle');
  let pillWidth = $state(72);
  /** Shown in Alert mode, or waiting in the pill until the user acknowledges it. */
  let alert = $state<AlertPayload | null>(null);
  let islandEl = $state<HTMLDivElement>();
  /** Bumped on hover so the eyes blink once (kedhep). */
  let blinkKey = $state(0);
  /** Settings → Position: a notch at the top centre instead of the bottom-right pill. */
  const notch = $derived(settings.current.islandPosition === 'top');

  // Priority (requirements 4.2): alert waiting, a reminder within 15 minutes, the focus
  // timer, the next reminder today, then the eyes. Never the clock: the taskbar has it.
  // The notch sits over title bars all day, so it leaves later reminders to Peek.
  const pillContent: PillContent = $derived.by(() => {
    // Only a waiting alert takes the pill; a new one first shows kaget eyes for a moment.
    if (alert && alertHeld) return { kind: 'alert', alert };
    const now = clock.now;
    const next = nextToday(reminders.all, now);
    const minutes = next ? Math.ceil((next.at - now.getTime()) / 60_000) : Infinity;
    const color = next ? `var(--${next.kind})` : '';
    if (next && minutes <= 15) {
      return { kind: 'due', title: next.title, detail: `in ${Math.max(1, minutes)}m`, color };
    }
    const timer = focus.state;
    if (timer.status !== 'ready') {
      return {
        kind: 'focus',
        isBreak: focus.isBreak,
        time: formatCountdown(focus.remainingMs),
        minutesLeft: Math.ceil(focus.remainingMs / 60_000),
        paused: timer.status === 'paused',
        waiting: timer.status === 'waiting',
      };
    }
    if (next && !notch) return { kind: 'due', title: next.title, detail: formatTime(next.at), color };
    // Lyric focus: the line being sung, in place of the title (design v2, 4.9).
    const playing = media.state;
    if (settings.current.lyricsEnabled && settings.current.lyricFocus && playing.playing && lyrics.status === 'timed' && lyrics.line) {
      return { kind: 'lyric', line: lyrics.line };
    }
    // What is playing, only while it plays: a track paused all afternoon is not news.
    if (settings.current.mediaInPill && playing.active && playing.playing) {
      return { kind: 'media', title: playing.title, artist: playing.artist, art: playing.art };
    }
    return { kind: 'eyes', expression: idleMood(now) };
  });

  let pointerInside = false;
  let fullscreen = false;
  /** The alert's hold time has passed, so moving the pointer away collapses it. */
  let alertHeld = $state(false);
  const heldDuringFullscreen: AlertPayload[] = [];
  let hoverTimer: ReturnType<typeof setTimeout> | undefined;
  let alertTimer: ReturnType<typeof setTimeout> | undefined;
  let regionTimer: ReturnType<typeof setTimeout> | undefined;
  /** The clickable area last sent to the core (plain, not reactive). */
  let region = { width: 0, height: 0 };

  const box: Box = $derived(
    mode === 'idle'
      ? { width: pillWidth, ...(notch ? NOTCH : PILL) }
      : mode === 'peek' && media.state.active
        ? { ...BOX.peek, height: BOX.peek.height + PEEK_MEDIA_ROW }
        : mode === 'expanded'
          ? { ...SHAPE[panel.view], radius: BOX.expanded.radius }
          : BOX[mode],
  );

  function applyRegion(width: number, height: number) {
    region = { width, height };
    core.setHitRegion(width, height);
  }

  // The clickable area grows at once so the morph is never clipped, and shrinks only
  // after the morph has mostly settled.
  $effect(() => {
    const { width, height } = box;
    clearTimeout(regionTimer);
    applyRegion(Math.max(region.width, width), Math.max(region.height, height));
    regionTimer = setTimeout(() => applyRegion(width, height), TIMING.regionSettle);
  });

  /** A countdown's rounded width can flip by 1 px between seconds ("20:37" is a hair
      narrower than "20:38"), or 2 px in the notch. Following that would run the 700 ms
      width spring every few seconds, so the pill grows at once but only shrinks for real
      changes. */
  function setPillWidth(width: number) {
    // The notch is centred: an even width keeps its edges on whole pixels.
    if (notch) width = Math.ceil(width / 2) * 2;
    if (width > pillWidth || pillWidth - width > 2) pillWidth = width;
  }

  function setMode(next: Mode) {
    clearTimeout(hoverTimer);
    mode = next;
  }

  /** When the mripat last glanced at overdue tasks (at most once per 10 minutes). */
  let lastGlance = 0;

  function overdue(now: number) {
    const midnight = new Date(now).setHours(0, 0, 0, 0);
    return todo.tasks.some((t) => !t.done && t.dueAt !== null && t.dueAt < (t.dueHasTime ? now : midnight));
  }

  function onPointer(inside: boolean) {
    pointerInside = inside;
    clearTimeout(hoverTimer);
    if (inside) {
      if (mode === 'idle') {
        blinkKey++;
        // Cheeky but quiet (design v2, C4): with tasks overdue, a short sulky glance after
        // the blink, no text and no nagging.
        const now = Date.now();
        if (now - lastGlance > 10 * 60_000 && overdue(now)) {
          lastGlance = now;
          setTimeout(() => react('mrengut', 1600), 160);
        }
        hoverTimer = setTimeout(() => (mode = alert ? 'alert' : 'peek'), notch ? TIMING.hoverOpenNotch : TIMING.hoverOpen);
      }
    } else if (mode === 'peek' || (mode === 'alert' && alertHeld)) {
      hoverTimer = setTimeout(() => (mode = 'idle'), TIMING.hoverClose);
    }
  }

  async function expand() {
    if (mode === 'expanded') return;
    setMode('expanded');
    await core.setInteractive(true);
  }

  function collapse(restoreFocus: boolean) {
    if (mode !== 'expanded') return;
    panel.rampung = false;
    panel.capture = false;
    panel.nowPlaying = false;
    setMode('idle');
    core.setInteractive(false, restoreFocus);
  }

  /** Alerts that arrived while another one was showing, oldest first. */
  const queued: AlertPayload[] = [];
  /** Counts presented alerts, so a delayed open belongs to the alert that scheduled it. */
  let alertSeq = 0;

  function showAlert(next: AlertPayload) {
    if (fullscreen) {
      heldDuringFullscreen.push(next);
      return;
    }
    if (alert) {
      queued.push(next);
      return;
    }
    present(next);
  }

  function present(next: AlertPayload) {
    alert = next;
    clearTimeout(alertTimer);
    // Never interrupt the open panel; the pill shows the alert once it closes.
    if (mode === 'expanded') {
      alertHeld = true;
      return;
    }
    alertHeld = false;
    react('kaget', 600);
    // A sequence number, not the object: $state hands back a proxy, never `next` itself.
    const seq = ++alertSeq;
    const open = () => {
      if (seq !== alertSeq || !alert || mode === 'expanded') return;
      setMode('alert');
      alertTimer = setTimeout(() => {
        alertHeld = true;
        if (mode === 'alert' && !pointerInside) mode = 'idle';
      }, TIMING.alertHold);
    };
    // The eyes jump first, then the alert grows out of the pill (design v2, section 5).
    if (mode === 'idle' && !matchMedia('(prefers-reduced-motion: reduce)').matches) {
      clearTimeout(alertTimer);
      alertTimer = setTimeout(open, 240);
    } else {
      open();
    }
  }

  function clearAlert() {
    clearTimeout(alertTimer);
    alert = null;
    alertHeld = false;
  }

  /** The current alert is handled; the next queued one, if any, takes its place. */
  function dismissAlert() {
    clearAlert();
    setMode('idle');
    presentQueued();
  }

  function presentQueued() {
    const next = queued.shift();
    if (next) present(next);
  }

  function reviewAlert() {
    dismissAlert();
    panel.tab = 'reminders';
    void expand();
  }

  /** "+ Meeting note" on a meeting alert: a new note titled after the meeting, at its time. */
  function meetingNoteAlert() {
    const meeting = alert;
    dismissAlert();
    if (!meeting) return;
    panel.startMeeting(meeting.title, meeting.at ?? Date.now());
    void expand();
  }

  function onFocusAction(action: 'start' | 'skip' | 'stop') {
    dismissAlert();
    focus[action]();
  }

  // Lyric focus needs the lyrics while the panel is closed too; turning lyrics off forgets them.
  $effect(() => {
    const s = settings.current;
    if (!s.lyricsEnabled) return lyrics.reset();
    if (s.lyricFocus && media.state.active && media.state.title) void lyrics.load();
  });

  // Rampung is offered once a day on weekdays at the time set in Settings, never while a
  // focus phase runs. "Later" offers it again 30 minutes on. Checked on the minute clock,
  // so it costs no timer of its own.
  let rampungOffered = '';
  let rampungLaterUntil = 0;
  $effect(() => {
    const now = clock.now;
    const s = settings.current;
    const status = focus.state.status;
    untrack(() => {
      const today = toDateInput(now.getTime());
      if (!s.rampungEnabled || s.rampungDoneDate === today || rampungOffered === today || panel.rampung) return;
      if (now.getDay() === 0 || now.getDay() === 6 || status === 'running' || status === 'paused') return;
      const [h, m] = s.rampungTime.split(':').map(Number);
      if (now.getHours() * 60 + now.getMinutes() < h * 60 + m || Date.now() < rampungLaterUntil) return;
      rampungOffered = today;
      showAlert({ title: 'Time to wrap up', subtitle: 'Rampung · a quick look back at today', kind: 'general', rampung: true });
    });
  });

  // Files dragged onto the island go on the shelf (the core keeps them on drop). Over the
  // pill, the island grows into a drop target; over the open panel, it shows the shelf.
  // WebView2 runs the drag, as Edge does; the page decides what it accepts.
  let dropCount = $state(0);
  let dropKept = $state<number | null>(null);
  let dropTimer: ReturnType<typeof setTimeout> | undefined;

  function onShelfDrag(note: { kind: 'enter' | 'leave' | 'drop'; count: number; kept: number }) {
    if (note.kind === 'enter') {
      if (mode === 'expanded') {
        panel.tab = 'shelf';
        panel.shelf = 'files';
        return;
      }
      clearTimeout(dropTimer);
      dropCount = note.count;
      dropKept = null;
      setMode('drop');
    } else if (note.kind === 'leave') {
      if (mode === 'drop' && dropKept === null) setMode('idle');
    } else {
      if (mode === 'expanded') return;
      dropKept = note.kept;
      setMode('drop');
      dropTimer = setTimeout(() => mode === 'drop' && setMode('idle'), 1800);
    }
  }

  let dragging = false;
  /** Ends a drag that went quiet without a dragleave, so the drop target never sticks. */
  let dragWatchdog: ReturnType<typeof setTimeout> | undefined;
  const isFileDrag = (e: DragEvent) => e.dataTransfer?.types.includes('Files') ?? false;
  const editable = (target: EventTarget | null) =>
    target instanceof HTMLElement && (target.isContentEditable || target.closest('input, textarea') !== null);

  function onDragOver(e: DragEvent) {
    if (isFileDrag(e)) {
      e.preventDefault();
      e.dataTransfer!.dropEffect = 'copy';
      clearTimeout(dragWatchdog);
      dragWatchdog = setTimeout(endDrag, 10_000);
      if (!dragging) {
        dragging = true;
        const count = [...e.dataTransfer!.items].filter((item) => item.kind === 'file').length;
        onShelfDrag({ kind: 'enter', count, kept: 0 });
      }
    } else if (!editable(e.target)) {
      // Anything else (a link, text) dropped outside a text field would open in the webview.
      e.preventDefault();
      e.dataTransfer!.dropEffect = 'none';
    }
  }

  /** Moving from one element to another (the morph replaces the one under the pointer)
      fires a dragleave whose relatedTarget is the element entered. Only one without it
      means the files left the window, which the region keeps to the island. */
  function onDragLeave(e: DragEvent) {
    if (!e.relatedTarget) endDrag();
  }

  function endDrag() {
    clearTimeout(dragWatchdog);
    if (!dragging) return;
    dragging = false;
    onShelfDrag({ kind: 'leave', count: 0, kept: 0 });
  }

  function onDrop(e: DragEvent) {
    if (!isFileDrag(e)) {
      if (!editable(e.target)) e.preventDefault();
      return;
    }
    e.preventDefault();
    clearTimeout(dragWatchdog);
    dragging = false;
    const files = e.dataTransfer!.files;
    // The core answers with a "drop" note once the files are on the shelf. Files that
    // exist only inside another app (an Outlook attachment) have no path to keep.
    if (!files.length) onShelfDrag({ kind: 'leave', count: 0, kept: 0 });
    else if (!dropFiles(files)) onShelfDrag({ kind: 'drop', count: files.length, kept: files.length });
  }

  function startRampung() {
    rampungOffered = toDateInput(Date.now());
    panel.rampung = true;
    void expand();
  }

  function onRampung(action: 'start' | 'later') {
    dismissAlert();
    if (action === 'start') {
      startRampung();
    } else {
      rampungOffered = '';
      rampungLaterUntil = Date.now() + 30 * 60_000;
    }
  }

  // A phase-end alert is moot once the waiting phase was started (or stopped) elsewhere,
  // e.g. from the Focus tab while the alert sat in the pill.
  let lastFocusStatus = untrack(() => focus.state.status);
  $effect(() => {
    const status = focus.state.status;
    if (lastFocusStatus === 'waiting' && status !== 'waiting') {
      untrack(() => {
        if (alert?.kind === 'focus' && alert.focus !== 'auto') {
          clearAlert();
          if (mode === 'alert') setMode('idle');
          presentQueued();
        }
      });
    }
    lastFocusStatus = status;
  });

  function snoozeAlert() {
    const snoozed = alert;
    dismissAlert();
    if (!snoozed) return;
    // Reminders snooze in the core (it survives a restart); the tray's test alert has no reminder.
    if (snoozed.reminder !== undefined) reminders.snooze(snoozed.reminder, 5);
    else setTimeout(() => showAlert(snoozed), TIMING.snooze);
  }

  function onPillClick() {
    if (alert) {
      alertHeld = true;
      setMode('alert');
      return;
    }
    if (pillContent.kind === 'focus') panel.tab = 'focus';
    if (pillContent.kind === 'media' || pillContent.kind === 'lyric') panel.showNowPlaying('pill');
    else panel.nowPlaying = false;
    void expand();
  }

  function onFullscreen(active: boolean) {
    fullscreen = active;
    if (active) {
      collapse(false);
      setMode('idle');
    } else {
      // Focus alerts keep their actions; several reminders become one summary.
      const held = heldDuringFullscreen.splice(0);
      const single = held.filter((a) => a.kind !== 'focus' && !a.summary);
      const rest = held.filter((a) => a.kind === 'focus' || a.summary);
      if (single.length > 1) {
        rest.push({
          title: `${single.length} missed reminders`,
          subtitle: single.map((a) => (a.at ? `${a.title} ${formatTime(a.at)}` : a.title)).join(' · '),
          kind: 'general',
          summary: true,
        });
      } else {
        rest.push(...single);
      }
      rest.forEach(showAlert);
    }
  }

  function onKeydown(e: KeyboardEvent) {
    // Esc first closes whatever is open inside the panel (a menu, a detail view).
    if (e.key === 'Escape' && mode === 'expanded' && !handleEscape()) collapse(true);
  }

  const demoAlert: AlertPayload = { title: 'Weekly sync', subtitle: 'Meeting · starts in 5 min', kind: 'meeting' };

  /** In a plain browser there is no Rust core, so these stand in for its events. */
  function installPreviewHelpers() {
    Object.assign(window, {
      karsa: {
        alert: (overrides: Partial<AlertPayload> = {}) =>
          emitLocal('island://alert', { ...demoAlert, ...overrides }),
        fullscreen: (active: boolean) => emitLocal('island://fullscreen', active),
        rampung: () => emitLocal('island://open', 'rampung'),
        capture: () => emitLocal('island://capture', null),
        media: previewMedia,
        drop: (count = 2) => {
          emitLocal('shelf://drag', { kind: 'enter', count, kept: 0 });
          setTimeout(() => emitLocal('shelf://drag', { kind: 'drop', count, kept: count }), 1500);
        },
        state: () => ({ mode, alert: $state.snapshot(alert), pillWidth }),
      },
    });
    window.addEventListener('pointerdown', (e) => {
      if (!islandEl?.contains(e.target as Node)) emitLocal('island://blur', null);
    });
  }

  onMount(() => {
    const offs = [
      on('island://blur', () => collapse(false)),
      on('island://open', (target) => {
        panel.nowPlaying = false;
        if (target === 'rampung') return startRampung();
        if (target) panel.tab = target;
        void expand();
      }),
      on('island://alert', showAlert),
      on('island://capture', () => {
        panel.rampung = false;
        panel.capture = true;
        void expand();
      }),
      on('island://fullscreen', onFullscreen),
    ];
    offs.push(on('shelf://drag', onShelfDrag));
    core.uses12HourClock().then((v) => (prefs.hour12 = v));
    panel.collapse = () => collapse(false);
    panel.closeAndReturn = () => collapse(true);
    todo.load();
    focus.load();
    reminders.load();
    settings.load();
    media.load();
    if (!inTauri) installPreviewHelpers();
    // Wait for the first painted frame before the core shows the window.
    requestAnimationFrame(() => requestAnimationFrame(() => core.ready()));
    return () => offs.forEach((off) => off());
  });
</script>

<svelte:window onkeydown={onKeydown} ondragenter={onDragOver} ondragover={onDragOver} ondragleave={onDragLeave} ondrop={onDrop} />

<main class="stage" class:preview={!inTauri}>
  <!-- Outside the panel, clicks must not move focus into the page: that would let the
       webview activate the window and pull focus from the user's app. -->
  <div
    class="island"
    class:notch
    bind:this={islandEl}
    style:width="{box.width}px"
    style:height="{box.height}px"
    style:border-radius={notch ? `0 0 ${box.radius}px ${box.radius}px` : `${box.radius}px`}
    onmouseenter={() => onPointer(true)}
    onmouseleave={() => {
      onPointer(false);
      pointer.leave();
    }}
    onmousemove={(e) => pointer.move(e.clientX, e.clientY)}
    onmousedown={(e) => mode !== 'expanded' && e.preventDefault()}
    role="presentation"
  >
    {#key mode}
      <div class="layer" out:contentOut>
        {#if mode === 'idle'}
          <Pill content={pillContent} {notch} {blinkKey} onwidth={setPillWidth} onclick={onPillClick} />
        {:else if mode === 'peek'}
          <Peek onclick={() => void expand()} />
        {:else if mode === 'alert' && alert}
          <Alert
            {alert}
            onsnooze={snoozeAlert}
            ondismiss={dismissAlert}
            onfocus={onFocusAction}
            onreview={reviewAlert}
            onmeeting={meetingNoteAlert}
            onrampung={onRampung}
          />
        {:else if mode === 'expanded'}
          <Panel />
        {:else if mode === 'drop'}
          <Drop count={dropCount} kept={dropKept} />
        {/if}
      </div>
    {/key}
  </div>
</main>

<style>
  .stage {
    position: fixed;
    inset: 0;
  }

  /* Browser preview only: a stand-in desktop so the black island is visible. */
  .stage.preview {
    background: linear-gradient(160deg, #6b7b93, #a9b3c2 55%, #d9dde3);
  }

  .island {
    position: absolute;
    right: 0;
    bottom: 0;
    overflow: hidden;
    background: var(--island-bg);
    /* The only elevation: a faint edge keeps black readable over dark apps. */
    box-shadow: var(--edge);
    contain: layout paint;
    transition:
      width var(--morph-duration) var(--spring),
      height var(--morph-duration) var(--spring),
      border-radius var(--radius-duration) ease-out;
  }

  /* Content keeps its own fixed size and is revealed as the box grows around it. */
  .layer {
    position: absolute;
    right: 0;
    bottom: 0;
    width: max-content;
    transform-origin: bottom right;
  }

  /* The notch hangs from the middle of the top edge and grows downwards and outwards.
     Its widths are even, so its edges stay on whole pixels. */
  .island.notch,
  .notch .layer {
    top: 0;
    right: auto;
    bottom: auto;
    left: 50%;
    translate: -50% 0;
  }

  .island.notch {
    box-shadow: var(--edge-notch);
  }

  .notch .layer {
    transform-origin: top center;
  }
</style>
