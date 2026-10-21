<!-- Rampung: wrapping up the day. A look back at what got done, a home for what is still
     open (tomorrow), a look at tomorrow, then "Rampung" closes the day. -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { clock } from '../clock.svelte';
  import { onEscape } from '../escape';
  import Eyes from '../Eyes.svelte';
  import { focus } from '../focus.svelte';
  import { formatTime, toDateInput } from '../format.svelte';
  import Icon from '../Icon.svelte';
  import { paribasan } from '../jawa';
  import { reminders } from '../reminders.svelte';
  import { settings } from '../settings.svelte';
  import { todo, type Task } from '../todo.svelte';
  import { panel } from '../ui.svelte';

  const DAY = 86_400_000;
  let finished = $state(false);

  const now = $derived(clock.now);
  const startOfToday = $derived(new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime());
  const startOfTomorrow = $derived(new Date(now.getFullYear(), now.getMonth(), now.getDate() + 1).getTime());
  const startOfAfter = $derived(new Date(now.getFullYear(), now.getMonth(), now.getDate() + 2).getTime());

  const doneToday = $derived(todo.tasks.filter((t) => t.done && t.completedAt !== null && t.completedAt >= startOfToday));
  const stillOpen = $derived(
    todo.tasks
      .filter((t) => t.parentId === null && !t.done && t.dueAt !== null && t.dueAt < startOfTomorrow)
      .sort((a, b) => a.dueAt! - b.dueAt!),
  );
  const tomorrow = $derived(
    [
      ...reminders.all
        .filter((r) => r.repeat !== 'interval' && r.at >= startOfTomorrow && r.at < startOfAfter)
        .map((r) => ({ key: `r${r.id}`, time: formatTime(r.at), title: r.title, color: `var(--${r.kind})`, at: r.at })),
      // Date-only deadlines have no reminder of their own.
      ...todo.tasks
        .filter((t) => !t.done && !t.dueHasTime && t.dueAt !== null && t.dueAt >= startOfTomorrow && t.dueAt < startOfAfter)
        .map((t) => ({ key: `t${t.id}`, time: 'All day', title: t.title, color: 'var(--deadline)', at: t.dueAt! })),
    ].sort((a, b) => a.at - b.at),
  );
  const proverb = $derived(paribasan(now));
  const dayName = $derived(new Intl.DateTimeFormat('en-GB', { weekday: 'long' }).format(now));

  const todayLine = $derived.by(() => {
    const tasks = doneToday.length;
    const minutes = focus.today.minutes;
    const parts = [`${tasks} ${tasks === 1 ? 'task' : 'tasks'} done`];
    if (minutes > 0) parts.push(minutes >= 60 ? `${Math.floor(minutes / 60)}h ${minutes % 60} min of focus` : `${minutes} min of focus`);
    return parts.join(' · ');
  });

  /** Same time tomorrow for a timed deadline; tomorrow (all day) for a date-only one. */
  function toTomorrow(task: Task) {
    if (!task.dueHasTime) return todo.setDue(task.id, startOfTomorrow, false);
    const due = new Date(task.dueAt!);
    const next = new Date(startOfTomorrow);
    next.setHours(due.getHours(), due.getMinutes());
    return todo.setDue(task.id, next.getTime(), true);
  }

  async function moveAll() {
    for (const task of stillOpen) await toTomorrow(task);
  }

  function overdueLabel(task: Task) {
    if (task.dueAt! >= startOfToday) return task.dueHasTime ? formatTime(task.dueAt!) : 'Today';
    const days = Math.round((startOfToday - new Date(task.dueAt!).setHours(0, 0, 0, 0)) / DAY);
    return days === 1 ? 'Yesterday' : `${days} days ago`;
  }

  async function rampung() {
    await settings.update({ rampungDoneDate: toDateInput(Date.now()) });
    finished = true;
    setTimeout(() => {
      panel.rampung = false;
      panel.collapse();
    }, 4000);
  }

  onMount(() => onEscape(() => (panel.rampung = false)));
</script>

<div class="view">
  {#if finished}
    <div class="closing">
      <Eyes expression="bungah" width={56} head="large" />
      <span class="proverb">Rampung</span>
      <span class="muted">{proverb.text}</span>
      <span class="faint small">{proverb.meaning}</span>
    </div>
  {:else}
    <div class="bar">
      <button class="back" onclick={() => (panel.rampung = false)}><Icon name="chevron-left" size={16} />Back</button>
      <span class="grow"></span>
      <span class="caps">Rampung</span>
    </div>

    <div class="scroll">
      <section>
        <h3 class="title">Wrapping up {dayName}</h3>
        <p class="muted line">{todayLine}</p>
      </section>

      <section>
        <div class="head">
          <span class="caps">Still open</span>
          {#if stillOpen.length > 1}
            <button class="btn text sm-text" onclick={moveAll}>Move all to tomorrow</button>
          {/if}
        </div>
        {#each stillOpen as task (task.id)}
          <div class="row">
            <span class="ell grow">{task.title}</span>
            <span class="num faint small">{overdueLabel(task)}</span>
            <button class="btn sm" onclick={() => toTomorrow(task)}>Tomorrow</button>
          </div>
        {:else}
          <p class="faint line">Nothing left over from today.</p>
        {/each}
      </section>

      <section>
        <span class="caps">Tomorrow</span>
        {#each tomorrow as item (item.key)}
          <div class="row">
            <span class="num time" style:color={item.color}>{item.time}</span>
            <span class="ell grow">{item.title}</span>
          </div>
        {:else}
          <p class="faint line">Nothing planned yet.</p>
        {/each}
      </section>
    </div>

    <div class="footer">
      <button class="btn lg primary done" onclick={rampung}><Icon name="ornaments/kawung-mark" size={10} />Rampung</button>
    </div>
  {/if}
</div>

<style>
  section {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 4px 4px 14px;
  }

  .title {
    margin: 0;
    font: var(--fw-semibold) var(--fs-title) / var(--lh-title) var(--font-text);
  }

  .line {
    margin: 0;
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-height: 22px;
  }

  .sm-text {
    height: 22px;
    font-size: var(--fs-caption);
  }

  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 36px;
    border-bottom: 1px solid var(--hairline);
  }

  .row:last-child {
    border-bottom: 0;
  }

  .time {
    flex: none;
    width: 52px;
    font-size: var(--fs-caption);
  }

  .small {
    font-size: var(--fs-caption);
  }

  .footer {
    padding-top: 10px;
  }

  .done {
    width: 100%;
  }

  .closing {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    text-align: center;
    padding: 0 24px 24px;
  }
</style>
