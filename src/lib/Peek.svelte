<script lang="ts">
  import { clock } from './clock.svelte';
  import { focus, formatCountdown } from './focus.svelte';
  import { dueLabelShort, formatDate, formatTime, isUrgent } from './format.svelte';
  import Eyes from './Eyes.svelte';
  import Icon from './Icon.svelte';
  import { pasaran } from './jawa';
  import { media, positionAt, sourceLabel } from './media.svelte';
  import { mood } from './mood.svelte';
  import { nextToday, reminders } from './reminders.svelte';
  import { todo, upcomingTasks } from './todo.svelte';
  import { panel } from './ui.svelte';

  let { onclick }: { onclick: () => void } = $props();

  const next = $derived(nextToday(reminders.all, clock.now));
  // A task whose deadline is the next-up item already has its own row.
  const playing = $derived(media.state);
  const top = $derived(upcomingTasks(todo.tasks).filter((t) => t.id !== next?.taskId).slice(0, 3));
  const timer = $derived(focus.state);

  // The Playing row's progress line moves once a second while playing.
  let now = $state(Date.now());
  $effect(() => {
    if (!playing.playing) return;
    now = Date.now();
    const t = setInterval(() => (now = Date.now()), 1000);
    return () => clearInterval(t);
  });
  const progress = $derived(playing.durationMs > 0 ? positionAt(playing, now) / playing.durationMs : 0);

  // The small buttons act on the timer without opening the panel.
  function act(e: MouseEvent, action: () => void) {
    e.stopPropagation();
    action();
  }
</script>

<div
  class="peek"
  class:with-media={playing.active}
  role="button"
  tabindex="0"
  {onclick}
  onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && onclick()}
>
  <span class="head beat" style="--i: 0">
    <span class="date"><Eyes expression={mood(clock.now)} width={22} head="small" glance />{formatDate(clock.now)}<span class="pasaran">{pasaran(clock.now)}</span></span>
    <span class="muted num">{formatTime(clock.now)}</span>
  </span>
  <span class="sep beat" style="--i: 1"><Icon name="ornaments/kawung-mark" size={8} /></span>

  <!-- Clicking the timer row opens the panel on the Focus tab. -->
  <span class="row beat" style="--i: 2" role="presentation" onclick={() => timer.status !== 'ready' && (panel.tab = 'focus')}>
    <span class="key">Focus</span>
    {#if timer.status === 'ready'}
      <span class="value muted">No session running</span>
    {:else}
      <span class="value timer">
        {#if focus.isBreak}<span class="muted">Break</span>{/if}
        <span class="num strong" style:color={focus.isBreak || timer.status === 'paused' ? 'var(--fg)' : 'var(--focus)'}>
          {formatCountdown(focus.remainingMs)}
        </span>
        <span class="bar"><i style:transform="scaleX({focus.progress})"></i></span>
      </span>
      <button
        class="ibtn sm"
        aria-label={timer.status === 'running' ? 'Pause' : timer.status === 'paused' ? 'Resume' : 'Start'}
        onclick={(e) => act(e, timer.status === 'running' ? focus.pause : timer.status === 'paused' ? focus.resume : focus.start)}
      >
        <Icon name={timer.status === 'running' ? 'pause' : 'play'} size={12} />
      </button>
      <button class="ibtn sm" aria-label="Stop" onclick={(e) => act(e, focus.stop)}><Icon name="stop" size={12} /></button>
    {/if}
  </span>

  <span class="row beat" style="--i: 3">
    <span class="key">Next up</span>
    {#if next}
      <span class="value ell">{next.title}</span>
      <span class="num due" style:color="var(--{next.kind})">{formatTime(next.at)}</span>
    {:else}
      <span class="value muted">Nothing else today</span>
    {/if}
  </span>

  {#if playing.active}
    <!-- Clicking the row opens Now playing (Peek's own click then opens the panel). -->
    <span class="row beat media" style="--i: 4" role="presentation" onclick={() => panel.showNowPlaying('pill')}>
      <span class="key">Playing</span>
      <span class="value ell" title="{playing.title}{playing.artist ? ` · ${playing.artist}` : ''} ({sourceLabel(playing)})">
        {#if playing.appIcon}<img class="source" src={playing.appIcon} alt="" />{/if}{playing.title}{#if playing.artist}<span class="muted"> · {playing.artist}</span>{/if}
      </span>
      <button class="ibtn sm" aria-label="Previous" disabled={!playing.canPrevious} onclick={(e) => act(e, () => media.control({ kind: 'previous' }))}>
        <Icon name="previous" size={12} />
      </button>
      <button class="ibtn sm" aria-label={playing.playing ? 'Pause' : 'Play'} onclick={(e) => act(e, () => media.control({ kind: 'toggle' }))}>
        <Icon name={playing.playing ? 'pause' : 'play'} size={12} />
      </button>
      <button class="ibtn sm" aria-label="Next" disabled={!playing.canNext} onclick={(e) => act(e, () => media.control({ kind: 'next' }))}>
        <Icon name="next" size={12} />
      </button>
    </span>
    {#if playing.durationMs > 0}
      <span class="progress beat" style="--i: 4" aria-hidden="true"><i style:transform="scaleX({progress})"></i></span>
    {/if}
  {/if}

  <span class="row beat" style="--i: {playing.active ? 5 : 4}">
    <span class="key">Tasks</span>
    <span class="lines">
      {#each top as task (task.id)}
        <span class="line">
          <span class="ell grow">{task.title}</span>
          {#if task.dueAt !== null}
            <span class="num due" class:muted={!isUrgent(task, clock.now)} class:urgent={isUrgent(task, clock.now)}>
              {dueLabelShort(task, clock.now)}
            </span>
          {/if}
        </span>
      {:else}
        <span class="muted">No open tasks</span>
      {/each}
    </span>
  </span>
</div>

<style>
  .peek {
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    width: var(--peek-w);
    height: var(--peek-h);
    padding: 14px 18px 12px;
    text-align: left;
    overflow: hidden;
  }

  /* Taller while something is playing (PEEK_MEDIA_ROW in config.ts). */
  .peek.with-media {
    height: var(--peek-media-h);
  }

  /* The source app's icon before the title. */
  .source {
    width: 12px;
    height: 12px;
    margin-right: 6px;
    border-radius: 3px;
    vertical-align: -1px;
  }

  .progress {
    position: relative;
    display: block;
    height: var(--pill-progress-h);
    margin: 2px 0 6px 70px;
    border-radius: 1px;
    background: var(--track);
    overflow: hidden;
  }

  .progress i {
    position: absolute;
    inset: 0;
    background: var(--gading);
    transform-origin: 0 0;
  }

  .head {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    padding-bottom: 9px;
  }

  .date {
    display: flex;
    align-items: center;
    gap: 8px;
    font: var(--fw-semibold) 14px/18px var(--font-text);
  }

  .pasaran {
    margin-left: -2px;
    font: var(--fw-semibold) var(--fs-caption) / 18px var(--font-text);
    letter-spacing: var(--tracking-caps);
    text-transform: uppercase;
    color: var(--muted);
  }

  .sep {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 4px;
    color: var(--ornament);
  }

  .sep::before,
  .sep::after {
    content: '';
    flex: 1;
    height: 1px;
    background: var(--hairline);
  }

  .row {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 4px 0;
    line-height: 18px;
  }

  .key {
    flex: none;
    width: 58px;
    font: var(--fw-semibold) var(--fs-caption) / 18px var(--font-text);
    letter-spacing: var(--tracking-caps);
    text-transform: uppercase;
    color: var(--faint);
  }

  .value {
    flex: 1;
    min-width: 0;
  }

  .timer {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .bar {
    position: relative;
    width: 96px;
    height: 3px;
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

  .row .ibtn {
    margin: -1px 0;
  }

  .lines {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .line {
    display: flex;
    gap: 8px;
  }

  .due {
    flex: none;
  }
</style>
