<!-- The calendar (design v2, 4.5 and 6.3): a month of Karsa's own data, and the selected
     day's agenda with quick add. Monday first, as in the kain. -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { addDays, agenda, daySummary, marks, monthGrid, sameDay, startOfDay, type Data } from '../calendar';
  import { clock } from '../clock.svelte';
  import Eyes from '../Eyes.svelte';
  import { focus } from '../focus.svelte';
  import { formatDate, formatTime, fromInputs, toDateInput, toTimeInput } from '../format.svelte';
  import Icon from '../Icon.svelte';
  import { paribasan, specialDay } from '../jawa';
  import { monthIn } from '../motion';
  import { notes } from '../notes.svelte';
  import { intervalLabel, reminders } from '../reminders.svelte';
  import { settings } from '../settings.svelte';
  import Checkbox from '../todo/Checkbox.svelte';
  import { INBOX_ID, todo } from '../todo.svelte';
  import { panel } from '../ui.svelte';

  const today = $derived(startOfDay(clock.now));
  let month = $state(startOfDay(new Date(new Date().getFullYear(), new Date().getMonth(), 1)));
  let selected = $state(startOfDay(new Date()));
  /** Direction of the last month change, for the slide. */
  let dir = $state(1);

  const data: Data = $derived({ tasks: todo.tasks, reminders: reminders.all, notes: notes.list, focus: focus.history });
  const days = $derived(monthGrid(month));
  const dayMarks = $derived(marks(days, data));
  const items = $derived(agenda(selected, today, data));
  const summary = $derived(daySummary(selected, data));
  const monthTitle = $derived(new Intl.DateTimeFormat('en-GB', { month: 'long', year: 'numeric' }).format(month));
  const emptyMonth = $derived(
    days.every((d) => {
      const m = dayMarks.get(d.getTime())!;
      return d.getMonth() !== month.getMonth() || (!m.deadline && !m.meeting && !m.general && m.focusMin === 0);
    }) && items.length === 0,
  );
  const proverb = $derived(paribasan(clock.now));
  const dayLabel = (d: Date) => new Intl.DateTimeFormat('en-GB', { day: 'numeric', month: 'long' }).format(d);

  function showMonth(next: Date, direction: number) {
    dir = direction;
    month = new Date(next.getFullYear(), next.getMonth(), 1);
  }

  function select(day: Date) {
    selected = startOfDay(day);
    if (day.getMonth() !== month.getMonth() || day.getFullYear() !== month.getFullYear()) {
      showMonth(day, day < month ? -1 : 1);
    }
    adding = null;
  }

  const goToday = () => select(new Date());

  // ---- Quick add ----------------------------------------------------------
  let adding = $state<'task' | 'reminder' | null>(null);
  let title = $state('');
  let time = $state('09:00');
  let error = $state('');

  function startAdding(kind: 'task' | 'reminder') {
    adding = kind;
    title = '';
    error = '';
  }

  async function add() {
    const text = title.trim();
    if (!text) return;
    if (adding === 'task') {
      await todo.createTaskDue(INBOX_ID, text, selected.getTime(), false);
      adding = null;
    } else if (adding === 'reminder') {
      const at = fromInputs(toDateInput(selected.getTime()), time);
      const ok = await reminders.save({ id: null, title: text, kind: 'general', at, repeat: 'none', leadMin: settings.current.reminderLeadMin });
      if (ok) adding = null;
      else error = reminders.error ?? '';
      reminders.clearError();
    }
  }

  function addKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      void add();
    } else if (e.key === 'Escape') {
      e.stopPropagation();
      adding = null;
    }
  }

  // A late task moved with the native date picker keeps its time of day.
  function moveTask(id: number, value: string) {
    const task = todo.tasks.find((t) => t.id === id);
    if (!task || !value) return;
    const at = task.dueHasTime && task.dueAt !== null ? fromInputs(value, toTimeInput(task.dueAt)) : fromInputs(value, '');
    void todo.setDue(id, at, task.dueHasTime);
  }

  // PgUp/PgDn move by month, T goes to today, the arrows move the selected day and Enter
  // starts a task on it. Ignored while typing.
  function onKeydown(e: KeyboardEvent) {
    const typing = e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement;
    if (typing || e.ctrlKey || e.altKey || e.metaKey) return;
    const step: Record<string, number> = { ArrowLeft: -1, ArrowRight: 1, ArrowUp: -7, ArrowDown: 7 };
    if (e.key in step) select(addDays(selected, step[e.key]));
    else if (e.key === 'PageUp') showMonth(new Date(month.getFullYear(), month.getMonth() - 1, 1), -1);
    else if (e.key === 'PageDown') showMonth(new Date(month.getFullYear(), month.getMonth() + 1, 1), 1);
    else if (e.key.toLowerCase() === 't') goToday();
    else if (e.key === 'Enter') startAdding('task');
    else return;
    e.preventDefault();
  }

  onMount(() => {
    void notes.refresh();
    void reminders.load();
  });
</script>

<svelte:window onkeydown={onKeydown} />

<div class="calendar">
  <div class="nav">
    <span class="title">{monthTitle}</span>
    <span class="nav-buttons">
      <button class="btn text" onclick={goToday} title="Today (T)">Today</button>
      <button
        class="ibtn"
        aria-label="Previous month"
        title="Previous month (PgUp)"
        onclick={() => showMonth(new Date(month.getFullYear(), month.getMonth() - 1, 1), -1)}
      ><Icon name="chevron-left" size={16} /></button>
      <button
        class="ibtn"
        aria-label="Next month"
        title="Next month (PgDn)"
        onclick={() => showMonth(new Date(month.getFullYear(), month.getMonth() + 1, 1), 1)}
      ><Icon name="chevron-right" size={16} /></button>
    </span>
  </div>

  <div class="weekdays caps" aria-hidden="true">
    {#each ['M', 'T', 'W', 'T', 'F', 'S', 'S'] as d, i (i)}<span>{d}</span>{/each}
  </div>

  <div class="grid-wrap">
    {#key month.getTime()}
      <div class="grid" role="grid" aria-label={monthTitle} in:monthIn={{ dir }}>
        {#each days as day (day.getTime())}
          {@const m = dayMarks.get(day.getTime())!}
          {@const other = day.getMonth() !== month.getMonth()}
          {@const weekend = day.getDay() === 0 || day.getDay() === 6}
          {@const weton = specialDay(day, settings.current.birthday) === 'weton'}
          {@const count = [m.deadline, m.meeting, m.general].filter(Boolean).length}
          <button
            class="day"
            class:other
            class:weekend
            class:today={sameDay(day, today)}
            class:selected={sameDay(day, selected)}
            role="gridcell"
            aria-selected={sameDay(day, selected)}
            aria-label="{dayLabel(day)}, {count} {count === 1 ? 'item' : 'items'}{weton ? ', weton' : ''}"
            onclick={() => select(day)}
          >
            <span class="date num">{day.getDate()}</span>
            {#if weton}<span class="weton"><Icon name="ornaments/kawung-mark" size={7} /></span>{/if}
            {#if count}
              <span class="dots">
                {#if m.deadline}<i style:background="var(--deadline)"></i>{/if}
                {#if m.meeting}<i style:background="var(--meeting)"></i>{/if}
                {#if m.general}<i style:background="var(--general)"></i>{/if}
              </span>
            {/if}
            {#if m.focusMin > 0}
              <span class="focus-bar" style:width="{Math.min(32, Math.max(6, (m.focusMin / 120) * 32))}px"></span>
            {/if}
          </button>
        {/each}
      </div>
    {/key}
  </div>

  <div class="agenda">
    <div class="agenda-head">
      <span class="agenda-title">{formatDate(selected)}{#if sameDay(selected, today)}<span class="muted">{' · today'}</span>{/if}</span>
      <button class="btn text sm" onclick={() => startAdding('task')}><Icon name="add" size={12} />Task</button>
      <button class="btn text sm" onclick={() => startAdding('reminder')}><Icon name="add" size={12} />Reminder</button>
    </div>

    {#if adding}
      <div class="add">
        <!-- svelte-ignore a11y_autofocus -->
        <input
          class="add-input"
          bind:value={title}
          placeholder={adding === 'task' ? `Task on ${formatDate(selected)}` : 'Reminder'}
          aria-label={adding === 'task' ? 'New task' : 'New reminder'}
          autofocus
          onkeydown={addKeydown}
        />
        {#if adding === 'reminder'}
          <input class="add-time num" type="time" bind:value={time} aria-label="Time" onkeydown={addKeydown} />
        {/if}
      </div>
      <span class="hint faint">Enter to add · Esc to cancel</span>
      {#if error}<span class="error">{error}</span>{/if}
    {/if}

    {#if emptyMonth && !adding}
      <div class="empty">
        <Eyes expression="merem" width={48} head="large" />
        <span class="proverb">{proverb.text}</span>
        <span class="faint">{proverb.meaning}</span>
        <span class="muted">Nothing planned this month.</span>
      </div>
    {:else}
      <div class="scroll rows">
        {#each items as item (item.kind + ('task' in item ? item.task.id : 'reminder' in item ? item.reminder.id : item.note.path))}
          {#if item.kind === 'late'}
            <div class="row">
              <span class="when late caps">Late</span>
              <Checkbox checked={false} size={14} label="Done: {item.task.title}" onchange={() => todo.updateTask(item.task.id, { done: true })} />
              <button class="what ell link" onclick={() => panel.showTask(item.task.id)}>{item.task.title}</button>
              <label class="btn text sm move">
                Move to…
                <input
                  type="date"
                  class="picker"
                  aria-label="Move {item.task.title} to"
                  onclick={(e) => e.currentTarget.showPicker?.()}
                  onchange={(e) => moveTask(item.task.id, e.currentTarget.value)}
                />
              </label>
            </div>
          {:else if item.kind === 'task'}
            <div class="row" class:done={item.task.done}>
              <span class="when num" class:deadline={item.at !== null}>{item.at !== null ? formatTime(item.at) : 'All day'}</span>
              <Checkbox
                checked={item.task.done}
                size={14}
                label="{item.task.done ? 'Not done' : 'Done'}: {item.task.title}"
                onchange={() => todo.updateTask(item.task.id, { done: !item.task.done })}
              />
              <button class="what ell link" onclick={() => panel.showTask(item.task.id)}>{item.task.title}</button>
            </div>
          {:else if item.kind === 'reminder'}
            <div class="row">
              <span class="when num" style:color="var(--{item.reminder.kind})">{formatTime(item.at)}</span>
              <span class="what ell">{item.reminder.title}</span>
              {#if item.reminder.repeat !== 'none'}<span class="faint" title="Repeats"><Icon name="repeat" size={12} /></span>{/if}
              {#if item.reminder.kind === 'meeting'}<span class="badge meeting">Meeting</span>{/if}
              {#if item.note}
                <button class="btn text sm" onclick={() => panel.showNote(item.note!.path)}>Note · {item.note.openActions} open</button>
              {/if}
            </div>
          {:else if item.kind === 'note'}
            <div class="row">
              <span class="when num" style:color="var(--meeting)">{formatTime(item.note.date)}</span>
              <span class="what ell">{item.note.title}</span>
              <button class="btn text sm" onclick={() => panel.showNote(item.note.path)}>Note · {item.note.openActions} open</button>
            </div>
          {:else}
            <div class="row">
              <span class="when faint"><Icon name="repeat" size={12} /></span>
              <span class="what ell muted">{item.reminder.title} · {intervalLabel(item.reminder).replace('Every', 'every')}</span>
            </div>
          {/if}
        {:else}
          {#if !adding}<p class="nothing muted">Nothing on {formatDate(selected)}</p>{/if}
        {/each}
        {#if summary.focusMin > 0 || summary.done > 0}
          <div class="row summary faint">
            <span class="when"><Icon name="ornaments/kawung-mark" size={10} /></span>
            <span class="what">
              {#if summary.focusMin > 0}Focus {summary.focusMin} min{/if}{#if summary.focusMin > 0 && summary.done > 0} · {/if}{#if summary.done > 0}{summary.done} done{/if}
            </span>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .calendar {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  .nav {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 32px;
    padding: 0 0 0 4px;
  }

  .title {
    font: var(--fw-semibold) 15px/20px var(--font-text);
  }

  .nav-buttons {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .weekdays {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    height: 18px;
    align-items: center;
    text-align: center;
    color: var(--faint);
  }

  .grid-wrap {
    overflow: hidden;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
  }

  .day {
    position: relative;
    height: var(--day-cell-h);
    border-radius: var(--radius-day);
    display: flex;
    justify-content: center;
    transition: background var(--dur-fast) var(--ease-standard);
  }

  .day:hover {
    background: var(--surface);
  }

  .day.selected {
    background: var(--selected-bg);
  }

  .day.today {
    box-shadow: inset 0 0 0 1.5px var(--accent);
  }

  .date {
    margin-top: 6px;
    font: var(--fw-semibold) 13px/16px var(--font-text);
  }

  .day.weekend .date {
    color: var(--muted);
  }

  .day.other .date {
    font-weight: 400;
    color: var(--disabled);
  }

  .weton {
    position: absolute;
    top: 4px;
    right: 5px;
    color: var(--accent);
    line-height: 0;
  }

  .dots {
    position: absolute;
    bottom: 9px;
    left: 0;
    right: 0;
    display: flex;
    justify-content: center;
    gap: 3px;
  }

  .dots i {
    width: var(--dot);
    height: var(--dot);
    border-radius: 50%;
  }

  .focus-bar {
    position: absolute;
    bottom: 4px;
    left: 50%;
    height: var(--focus-bar-h);
    border-radius: 1px;
    background: var(--focus);
    transform: translateX(-50%);
  }

  .agenda {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    margin-top: 8px;
    padding-top: 10px;
    border-top: 1px solid var(--hairline);
  }

  .agenda-head {
    display: flex;
    align-items: center;
    gap: 2px;
    height: 28px;
    padding-left: 4px;
  }

  .agenda-title {
    flex: 1;
    font-weight: var(--fw-semibold);
  }

  .agenda-head :global(.icon) {
    margin-right: 2px;
  }

  .add {
    display: flex;
    gap: 6px;
    margin: 6px 0 2px;
  }

  .add-input,
  .add-time {
    box-sizing: border-box;
    height: var(--control-h-sm);
    padding: 0 10px;
    border: 1px solid var(--hairline-strong);
    border-radius: var(--radius-control);
    background: var(--surface);
    outline: none;
  }

  .add-input {
    flex: 1;
    min-width: 0;
  }

  .add-input:focus,
  .add-time:focus {
    border-color: var(--accent);
  }

  .add-time {
    width: 92px;
  }

  .hint {
    padding-left: 4px;
    font-size: var(--fs-caption);
  }

  .error {
    padding-left: 4px;
    font-size: var(--fs-caption);
    color: var(--danger);
  }

  .rows {
    padding-top: 4px;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 28px;
    padding: 2px 4px;
  }

  .row.done .what {
    color: var(--faint);
    text-decoration: line-through;
  }

  .when {
    flex: none;
    width: 40px;
    font-size: var(--fs-caption);
    color: var(--muted);
  }

  .when.deadline,
  .when.late {
    color: var(--deadline);
  }

  .what {
    flex: 1;
    text-align: left;
  }

  .link:hover {
    color: var(--accent);
  }

  .badge {
    flex: none;
    padding: 0 6px;
    border-radius: var(--radius-badge);
    font: var(--fw-semibold) var(--fs-caption) / 18px var(--font-text);
  }

  .badge.meeting {
    background: var(--meeting-tint);
    color: var(--meeting);
  }

  .move {
    position: relative;
    flex: none;
    overflow: hidden;
  }

  /* The native date picker sits invisibly over "Move to…". */
  .picker {
    position: absolute;
    inset: 0;
    opacity: 0;
    cursor: pointer;
  }

  .summary {
    margin-top: 4px;
    font-size: var(--fs-caption);
  }

  .nothing {
    margin: 10px 4px;
  }

  .empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 6px;
    text-align: center;
  }

  .proverb {
    margin-top: 6px;
    font-weight: var(--fw-semibold);
    font-style: italic;
  }
</style>
