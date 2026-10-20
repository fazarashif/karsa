<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import { BOX, PILL, TIMING, type Box, type Mode } from './lib/config';
  import { core, emitLocal, inTauri, on, type AlertPayload } from './lib/bridge';
  import { clock } from './lib/clock.svelte';
  import { handleEscape } from './lib/escape';
  import { focus, formatCountdown } from './lib/focus.svelte';
  import { formatTime, prefs } from './lib/format.svelte';
  import { contentOut } from './lib/motion';
  import { nextToday, reminders } from './lib/reminders.svelte';
  import { settings } from './lib/settings.svelte';
  import { todayStatus, todo } from './lib/todo.svelte';
  import { panel } from './lib/ui.svelte';
  import Alert from './lib/Alert.svelte';
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

  // Priority (requirements 4.2): alert waiting, a reminder within 15 minutes, the focus
  // timer, the next reminder today, then the eyes. Never the clock: the taskbar has it.
  const pillContent: PillContent = $derived.by(() => {
    if (alert) return { kind: 'alert', alert };
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
    if (next) return { kind: 'due', title: next.title, detail: formatTime(next.at), color };
    const today = todayStatus(todo.tasks, now);
    const hour = now.getHours();
    const expression = today.total > 0 && today.open === 0 ? 'mesem' : hour >= 22 || hour < 5 ? 'merem' : 'melek';
    return { kind: 'eyes', expression };
  });

  let pointerInside = false;
  let fullscreen = false;
  /** The alert's hold time has passed, so moving the pointer away collapses it. */
  let alertHeld = false;
  const heldDuringFullscreen: AlertPayload[] = [];
  let hoverTimer: ReturnType<typeof setTimeout> | undefined;
  let alertTimer: ReturnType<typeof setTimeout> | undefined;
  let regionTimer: ReturnType<typeof setTimeout> | undefined;
  /** The clickable area last sent to the core (plain, not reactive). */
  let region = { width: 0, height: 0 };

  const box: Box = $derived(
    mode === 'idle' ? { width: pillWidth, height: PILL.height, radius: PILL.radius } : BOX[mode],
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
      narrower than "20:38"). Following that would run the 700 ms width spring every few
      seconds, so the pill grows at once but only shrinks for real changes. */
  function setPillWidth(width: number) {
    if (width > pillWidth || pillWidth - width > 1) pillWidth = width;
  }

  function setMode(next: Mode) {
    clearTimeout(hoverTimer);
    mode = next;
  }

  function onPointer(inside: boolean) {
    pointerInside = inside;
    clearTimeout(hoverTimer);
    if (inside) {
      if (mode === 'idle') {
        blinkKey++;
        hoverTimer = setTimeout(() => (mode = alert ? 'alert' : 'peek'), TIMING.hoverOpen);
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
    setMode('idle');
    core.setInteractive(false, restoreFocus);
  }

  /** Alerts that arrived while another one was showing, oldest first. */
  const queued: AlertPayload[] = [];

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
    setMode('alert');
    alertTimer = setTimeout(() => {
      alertHeld = true;
      if (mode === 'alert' && !pointerInside) mode = 'idle';
    }, TIMING.alertHold);
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
      on('island://open', (tab) => {
        if (tab) panel.tab = tab;
        void expand();
      }),
      on('island://alert', showAlert),
      on('island://fullscreen', onFullscreen),
    ];
    core.uses12HourClock().then((v) => (prefs.hour12 = v));
    panel.collapse = () => collapse(false);
    todo.load();
    focus.load();
    reminders.load();
    settings.load();
    if (!inTauri) installPreviewHelpers();
    // Wait for the first painted frame before the core shows the window.
    requestAnimationFrame(() => requestAnimationFrame(() => core.ready()));
    return () => offs.forEach((off) => off());
  });
</script>

<svelte:window onkeydown={onKeydown} />

<main class="stage" class:preview={!inTauri}>
  <!-- Outside the panel, clicks must not move focus into the page: that would let the
       webview activate the window and pull focus from the user's app. -->
  <div
    class="island"
    bind:this={islandEl}
    style:width="{box.width}px"
    style:height="{box.height}px"
    style:border-radius="{box.radius}px"
    onmouseenter={() => onPointer(true)}
    onmouseleave={() => onPointer(false)}
    onmousedown={(e) => mode !== 'expanded' && e.preventDefault()}
    role="presentation"
  >
    {#key mode}
      <div class="layer" out:contentOut>
        {#if mode === 'idle'}
          <Pill content={pillContent} {blinkKey} onwidth={setPillWidth} onclick={onPillClick} />
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
          />
        {:else if mode === 'expanded'}
          <Panel />
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
</style>
