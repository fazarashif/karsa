<!-- The last four weeks of focus as a small woven cloth: one kawung mark per day, stronger
     with more focus minutes, Monday to Sunday, plus the current streak. -->
<script lang="ts">
  import { clock } from '../clock.svelte';
  import { focus } from '../focus.svelte';
  import { formatDate, toDateInput } from '../format.svelte';
  import Icon from '../Icon.svelte';

  const WEEKS = 4;

  const minutesByDay = $derived.by(() => {
    const map = new Map<string, number>();
    for (const [endedAt, ms] of focus.history) {
      const key = toDateInput(endedAt);
      map.set(key, (map.get(key) ?? 0) + Math.round(ms / 60_000));
    }
    return map;
  });

  const cells = $derived.by(() => {
    const today = new Date(clock.now);
    today.setHours(0, 0, 0, 0);
    // Start on the Monday WEEKS - 1 weeks before this week's Monday.
    const start = new Date(today);
    start.setDate(start.getDate() - ((today.getDay() + 6) % 7) - (WEEKS - 1) * 7);
    return Array.from({ length: WEEKS * 7 }, (_, i) => {
      const day = new Date(start);
      day.setDate(start.getDate() + i);
      const minutes = minutesByDay.get(toDateInput(day.getTime())) ?? 0;
      const future = day > today;
      const level = future ? -1 : minutes === 0 ? 0 : minutes < 25 ? 1 : minutes < 75 ? 2 : 3;
      return { key: toDateInput(day.getTime()), day, minutes, level, today: day.getTime() === today.getTime() };
    });
  });

  // Days in a row with focus, counted back from today (or from yesterday while today has
  // nothing yet, so the streak is not broken before the day is over).
  const streak = $derived.by(() => {
    const day = new Date(clock.now);
    day.setHours(0, 0, 0, 0);
    if (!minutesByDay.has(toDateInput(day.getTime()))) day.setDate(day.getDate() - 1);
    let count = 0;
    while (minutesByDay.has(toDateInput(day.getTime()))) {
      count++;
      day.setDate(day.getDate() - 1);
    }
    return count;
  });

  const label = (c: (typeof cells)[number]) =>
    `${formatDate(c.day)} · ${c.minutes ? `${c.minutes} min of focus` : 'no focus'}`;
</script>

<div class="kain">
  <span class="caps">Kain</span>
  <span class="grid" role="img" aria-label="Focus over the last four weeks">
    {#each cells as c (c.key)}
      <span class="cell l{c.level}" class:today={c.today} title={c.level >= 0 ? label(c) : undefined}>
        {#if c.level > 0}<Icon name="ornaments/kawung-mark" size={10} />{:else if c.level === 0}<i></i>{/if}
      </span>
    {/each}
  </span>
  <span class="grow"></span>
  <span class="streak">
    {#if streak > 1}
      <span class="num strong">{streak}</span><span class="muted">days in a row</span>
    {:else if streak === 1}
      <span class="muted">Focused {minutesByDay.has(toDateInput(clock.now.getTime())) ? 'today' : 'yesterday'}</span>
    {:else}
      <span class="faint">Each finished session weaves a thread</span>
    {/if}
  </span>
</div>

<style>
  .kain {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 4px;
    border-top: 1px solid var(--hairline);
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(7, 10px);
    gap: 3px;
  }

  .cell {
    width: 10px;
    height: 10px;
    display: grid;
    place-items: center;
    border-radius: 3px;
  }

  .cell i {
    width: 3px;
    height: 3px;
    border-radius: 50%;
    background: var(--track);
  }

  .cell.l1 {
    color: var(--ornament);
  }

  .cell.l2 {
    color: var(--sogan);
  }

  .cell.l3 {
    color: var(--accent);
  }

  .cell.today {
    box-shadow: 0 0 0 1px var(--hairline-strong);
  }

  .streak {
    display: flex;
    align-items: baseline;
    gap: 5px;
    font-size: var(--fs-caption);
    text-align: right;
  }

  .streak .num {
    font-size: var(--fs-body);
    color: var(--accent);
  }
</style>
