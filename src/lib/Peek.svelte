<script lang="ts">
  import { clock } from './clock.svelte';
  import { focus, formatCountdown } from './focus.svelte';
  import { dueLabelShort, formatDate, formatTime, isUrgent } from './format.svelte';
  import Icon from './Icon.svelte';
  import { nextToday, reminders } from './reminders.svelte';
  import { todo, upcomingTasks } from './todo.svelte';
  import { panel } from './ui.svelte';

  let { onclick }: { onclick: () => void } = $props();

  const next = $derived(nextToday(reminders.all, clock.now));
  // A task whose deadline is the next-up item already has its own row.
  const top = $derived(upcomingTasks(todo.tasks).filter((t) => t.id !== next?.taskId).slice(0, 3));
  const timer = $derived(focus.state);

  // The small buttons act on the timer without opening the panel.
  function act(e: MouseEvent, action: () => void) {
    e.stopPropagation();
    action();
  }
</script>

<div
  class="peek"
  role="button"
  tabindex="0"
  {onclick}
  onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && onclick()}
>
  <span class="head beat" style="--i: 0">
    <span class="date">{formatDate(clock.now)}</span>
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

  <span class="row beat" style="--i: 4">
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

  .head {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    padding-bottom: 9px;
  }

  .date {
    font: var(--fw-semibold) 14px/18px var(--font-text);
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
