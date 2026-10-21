<script lang="ts">
  import Eyes, { type Expression } from '../Eyes.svelte';
  import { focus, formatCountdown } from '../focus.svelte';
  import { formatTime } from '../format.svelte';
  import Icon from '../Icon.svelte';
  import Kain from './Kain.svelte';

  const s = $derived(focus.state);
  const of = $derived(s.settings.sessionsPerCycle);
  const breakName = $derived(s.phase === 'long_break' ? 'Long break' : 'Short break');

  const label = $derived.by(() => {
    if (s.status === 'ready') return { text: 'Ready', color: 'var(--faint)' };
    if (s.status === 'waiting')
      return { text: s.phase === 'focus' ? `Up next · session ${s.session} of ${of}` : `Up next · ${breakName.toLowerCase()}`, color: 'var(--muted)' };
    if (s.phase === 'focus') return { text: `Focus · session ${s.session} of ${of}`, color: 'var(--focus)' };
    return { text: breakName, color: 'var(--muted)' };
  });

  const eyes: Expression = $derived(
    s.status === 'ready' ? 'melek' : s.status === 'waiting' ? 'mesem' : s.phase === 'focus' ? 'fokus' : 'leren',
  );

  const caption = $derived.by(() => {
    if (s.status === 'ready') return `${s.settings.focusMin} min focus · ${s.settings.shortBreakMin} min break`;
    if (s.status === 'running' && s.endsAt !== null) return `Ends at ${formatTime(s.endsAt)}`;
    if (s.status === 'paused') return 'Paused';
    return "Start when you're ready";
  });

  // Dots for the current cycle: finished sessions full, the running one half.
  const dots = $derived(
    Array.from({ length: of }, (_, i) => {
      const done = s.phase === 'focus' ? s.session - 1 : s.session;
      if (s.status === 'ready') return 'todo';
      if (i < done) return 'done';
      if (i === s.session - 1 && s.phase === 'focus' && (s.status === 'running' || s.status === 'paused')) return 'half';
      return 'todo';
    }),
  );

  const todayText = $derived.by(() => {
    const { sessions, minutes } = focus.today;
    if (sessions === 0) return 'No sessions yet';
    const time = minutes >= 60 ? `${Math.floor(minutes / 60)}h ${minutes % 60} min` : `${minutes} min`;
    return `${sessions} ${sessions === 1 ? 'session' : 'sessions'} · ${time}`;
  });

  function primary() {
    if (s.status === 'running') focus.pause();
    else if (s.status === 'paused') focus.resume();
    else focus.start();
  }
</script>

<div class="focus-tab">
  <div class="center">
    <span class="caps" style:color={label.color}>{label.text}</span>
    <Eyes expression={eyes} width={56} />
    <span class="timer">{formatCountdown(focus.remainingMs)}</span>
    <span class="bar" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow={Math.round(focus.progress * 100)}>
      <i style:transform="scaleX({focus.progress})"></i>
    </span>
    <span class="muted">{caption}</span>

    {#if s.status === 'ready'}
      <button class="btn lg primary start" onclick={() => focus.start()}><Icon name="play" size={16} />Start focus</button>
    {:else}
      <div class="controls">
        <button class="round" aria-label="Stop" title="Stop" onclick={() => focus.stop()}><Icon name="stop" size={16} /></button>
        <button
          class="round primary"
          aria-label={s.status === 'running' ? 'Pause' : s.status === 'paused' ? 'Resume' : 'Start'}
          title={s.status === 'running' ? 'Pause' : s.status === 'paused' ? 'Resume' : 'Start'}
          onclick={primary}
        >
          <Icon name={s.status === 'running' ? 'pause' : 'play'} size={20} />
        </button>
        <button class="round" aria-label="Skip" title="Skip" onclick={() => focus.skip()}><Icon name="skip" size={16} /></button>
      </div>
    {/if}
  </div>

  <Kain />

  <div class="stats">
    <span class="caps">Today</span>
    <span class="dots" aria-hidden="true">
      {#each dots as dot, i (i)}<i class={dot}></i>{/each}
    </span>
    <span class="grow"></span>
    <span class="muted">{todayText}</span>
  </div>
</div>

<style>
  .focus-tab {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  .center {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
  }

  .timer {
    font: var(--fw-semibold) var(--fs-timer) / var(--lh-timer) var(--font-display);
    font-variant-numeric: tabular-nums;
    letter-spacing: -0.01em;
  }

  .bar {
    position: relative;
    width: 240px;
    height: 4px;
    border-radius: 2px;
    background: var(--track);
    overflow: hidden;
  }

  .bar i {
    position: absolute;
    inset: 0;
    border-radius: 2px;
    background: var(--focus);
    transform-origin: 0 0;
  }

  .start {
    margin-top: 8px;
  }

  .controls {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-top: 8px;
  }

  .round {
    width: 40px;
    height: 40px;
    border-radius: 20px;
    display: grid;
    place-items: center;
    background: var(--surface);
    color: var(--fg);
    transition:
      background-color var(--dur-fast) var(--ease-standard),
      transform var(--dur-fast) var(--ease-standard);
  }

  .round:hover {
    background: var(--surface-hover);
  }

  .round:active {
    background: var(--surface-pressed);
    transform: scale(0.95);
  }

  .round.primary {
    width: 56px;
    height: 56px;
    border-radius: 28px;
    background: var(--accent);
    color: var(--on-accent);
  }

  .round.primary:hover {
    background: var(--accent-hover);
  }

  .round.primary:active {
    background: var(--accent-pressed);
  }

  .stats {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 4px 0;
    border-top: 1px solid var(--hairline);
  }

  .dots {
    display: flex;
    gap: 6px;
  }

  .dots i {
    width: 8px;
    height: 8px;
    border-radius: 4px;
    background: var(--track);
  }

  .dots i.done {
    background: var(--focus);
  }

  .dots i.half {
    background: linear-gradient(90deg, var(--focus) 50%, var(--track) 50%);
  }
</style>
