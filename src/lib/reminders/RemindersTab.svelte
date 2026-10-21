<script lang="ts">
  import { clock } from '../clock.svelte';
  import { onEscape } from '../escape';
  import { dayFromToday, formatDate, formatTime } from '../format.svelte';
  import Icon from '../Icon.svelte';
  import { intervalLabel, kindLabel, reminders, repeatLabel, type Reminder, type ReminderKind } from '../reminders.svelte';
  import { todo } from '../todo.svelte';
  import { panel } from '../ui.svelte';
  import ReminderForm from './ReminderForm.svelte';

  /** null: the list; { id: null }: a new reminder; { id }: editing. */
  let editing = $state<{ id: number | null } | null>(null);

  $effect(() => {
    if (editing) return onEscape(() => (editing = null));
  });

  type Row = {
    key: string;
    at: number;
    allDay: boolean;
    title: string;
    kind: ReminderKind;
    meta: { icon: string; text: string }[];
    open: () => void;
  };

  function metaFor(r: Reminder) {
    const meta: Row['meta'] = [];
    if (r.taskId !== null) meta.push({ icon: 'link', text: 'From Todo' });
    if (r.repeat === 'interval') meta.push({ icon: 'repeat', text: intervalLabel(r) });
    else if (r.repeat !== 'none') meta.push({ icon: 'repeat', text: repeatLabel[r.repeat] });
    if (r.leadMin > 0) meta.push({ icon: 'clock', text: `${r.leadMin} min before` });
    if (r.snoozedUntil !== null) meta.push({ icon: 'snooze', text: `Snoozed · ${formatTime(r.snoozedUntil)}` });
    return meta;
  }

  // Reminders, plus date-only task deadlines from today on (those never alert, but
  // they belong in the picture of what is coming).
  const rows = $derived.by(() => {
    const today = dayFromToday(0);
    const list: Row[] = reminders.all.map((r) => ({
      key: `r${r.id}`,
      at: r.at,
      allDay: false,
      title: r.title,
      kind: r.kind,
      meta: metaFor(r),
      open: () => (editing = { id: r.id }),
    }));
    for (const t of todo.tasks) {
      if (t.parentId !== null || t.done || t.dueAt === null || t.dueHasTime || t.dueAt < today) continue;
      list.push({
        key: `t${t.id}`,
        at: t.dueAt,
        allDay: true,
        title: t.title,
        kind: 'deadline',
        meta: [{ icon: 'link', text: 'From Todo' }],
        open: () => panel.showTask(t.id),
      });
    }
    return list.sort((a, b) => a.at - b.at || Number(b.allDay) - Number(a.allDay));
  });

  const groups = $derived.by(() => {
    void clock.now; // relabel Today/Tomorrow when the day changes
    const today = dayFromToday(0);
    const tomorrow = dayFromToday(1);
    const dayAfter = dayFromToday(2);
    const result: { label: string; rows: Row[] }[] = [];
    for (const row of rows) {
      const label =
        row.at < today ? 'Earlier' : row.at < tomorrow ? 'Today' : row.at < dayAfter ? 'Tomorrow' : formatDate(row.at);
      const last = result.at(-1);
      if (last?.label === label) last.rows.push(row);
      else result.push({ label, rows: [row] });
    }
    return result;
  });
</script>

{#if editing}
  <ReminderForm id={editing.id} onback={() => (editing = null)} />
{:else}
  <div class="view">
    <div class="bar">
      <span class="caps grow">Upcoming</span>
      <button class="btn sm" onclick={() => (editing = { id: null })}><Icon name="add" size={16} />New reminder</button>
    </div>

    {#if groups.length === 0}
      <div class="empty">
        <span style:color="var(--ornament)"><Icon name="reminders" size={24} /></span>
        <span class="proverb">Nothing coming up</span>
        <span class="muted">Reminders and timed task deadlines show here.</span>
      </div>
    {:else}
      <div class="scroll">
        {#each groups as group (group.label)}
          <div class="day caps">{group.label}</div>
          {#each group.rows as row (row.key)}
            <button class="rem" onclick={row.open}>
              {#if row.allDay}
                <span class="time all-day caps">All day</span>
              {:else}
                <span class="time">{formatTime(row.at)}</span>
              {/if}
              <span class="main">
                <span class="title">{row.title}</span>
                <span class="meta">
                  <span class="badge {row.kind}">{kindLabel[row.kind]}</span>
                  {#each row.meta as m (m.text)}
                    <span><Icon name={m.icon} size={12} />{m.text}</span>
                  {/each}
                </span>
              </span>
            </button>
          {/each}
        {/each}
      </div>
    {/if}
  </div>
{/if}

<style>
  .day {
    display: flex;
    align-items: center;
    height: 28px;
    padding: 0 4px;
    margin-top: 6px;
  }

  .day:first-child {
    margin-top: 0;
  }

  .rem {
    width: 100%;
    display: flex;
    gap: 12px;
    padding: 8px 4px;
    border-bottom: 1px solid var(--hairline);
    text-align: left;
    border-radius: 0;
    transition: background-color var(--dur-fast) var(--ease-standard);
  }

  .rem:hover {
    background: var(--surface);
  }

  .time {
    flex: none;
    width: 40px;
    font-weight: var(--fw-semibold);
    font-variant-numeric: tabular-nums;
  }

  .time.all-day {
    line-height: 18px;
  }

  .main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .title {
    overflow-wrap: anywhere;
  }

  .badge {
    height: 18px;
    padding: 0 7px;
    border-radius: var(--radius-badge);
    font: var(--fw-semibold) var(--fs-caption) / 18px var(--font-text);
  }

  .badge.general {
    color: var(--general);
    background: var(--general-tint);
  }

  .badge.meeting {
    color: var(--meeting);
    background: var(--meeting-tint);
  }

  .badge.deadline {
    color: var(--deadline);
    background: var(--deadline-tint);
  }
</style>
