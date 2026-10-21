<script lang="ts">
  import { untrack } from 'svelte';
  import { formatDate, formatTime, fromInputs, toDateInput, toTimeInput } from '../format.svelte';
  import Icon from '../Icon.svelte';
  import PickerField from '../PickerField.svelte';
  import { kindLabel, reminders, repeatLabel, type ReminderKind, type Repeat } from '../reminders.svelte';
  import { settings } from '../settings.svelte';
  import { panel } from '../ui.svelte';

  let { id, onback }: { id: number | null; onback: () => void } = $props();

  const existing = untrack(() => reminders.all.find((r) => r.id === id));
  /** A todo task's deadline: only the lead time is the reminder's own. */
  const linked = existing?.taskId != null;

  // A new reminder starts at the next full hour.
  const start = existing?.at ?? Math.ceil((Date.now() + 1) / 3_600_000) * 3_600_000;

  let title = $state(existing?.title ?? '');
  let date = $state(toDateInput(start));
  let time = $state(toTimeInput(start));
  let kind = $state<ReminderKind>(existing?.kind ?? 'general');
  let repeat = $state<Repeat>(existing?.repeat ?? 'none');
  let leadMin = $state(existing?.leadMin ?? untrack(() => settings.current.reminderLeadMin));
  // "Every few minutes": drink water every 30 minutes, 07:00–17:00, on weekdays.
  let everyMin = $state(existing?.everyMin ?? 30);
  let windowFrom = $state(existing?.windowFrom ?? 7 * 60);
  let windowUntil = $state(existing?.windowUntil ?? 17 * 60);
  let windowDays = $state<'weekdays' | 'daily'>(existing?.windowDays ?? 'weekdays');
  const interval = $derived(repeat === 'interval');
  const everyOptions = [15, 20, 30, 45, 60, 90, 120];
  const toInput = (minutes: number) => `${String(Math.floor(minutes / 60)).padStart(2, '0')}:${String(minutes % 60).padStart(2, '0')}`;
  const fromInput = (value: string) => {
    const [h, m] = value.split(':').map(Number);
    return h * 60 + m;
  };
  let confirmDelete = $state(false);
  let titleInput = $state<HTMLInputElement>();

  const kinds: { id: ReminderKind; icon: string }[] = [
    { id: 'general', icon: 'general' },
    { id: 'meeting', icon: 'meeting' },
    { id: 'deadline', icon: 'deadline' },
  ];
  const leads = [
    { label: 'At time', minutes: 0 },
    { label: '5m', minutes: 5 },
    { label: '10m', minutes: 10 },
    { label: '15m', minutes: 15 },
    { label: '30m', minutes: 30 },
  ];

  $effect(() => {
    reminders.clearError();
    if (!linked) titleInput?.focus();
  });

  async function save() {
    const input = interval
      ? { id, title, kind, at: Date.now(), repeat, leadMin: 0, everyMin, windowFrom, windowUntil, windowDays }
      : { id, title, kind, at: fromInputs(date, time), repeat, leadMin };
    if (await reminders.save(input)) onback();
  }

  async function remove() {
    if (id !== null && (await reminders.remove(id))) onback();
  }
</script>

<div class="view">
  <div class="bar">
    <button class="back" onclick={onback}><Icon name="chevron-left" size={16} />Reminders</button>
    <span class="grow"></span>
    <span class="meta">{id === null ? 'New reminder' : 'Edit reminder'}</span>
  </div>

  <div class="scroll form">
    {#if linked}
      <p class="note meta">
        <Icon name="link" size={12} />From Todo. The title and time follow the task.
        <button class="btn text sm-text" onclick={() => existing && panel.showTask(existing.taskId!)}>Open task</button>
      </p>
    {/if}

    <label class="field">
      <span class="caps">Title</span>
      <span class="input" class:disabled={linked}>
        <input bind:this={titleInput} bind:value={title} maxlength="500" disabled={linked} aria-label="Title" />
      </span>
    </label>

    {#if !interval}
    <div class="pair">
      <div class="field">
        <span class="caps">Date</span>
        <PickerField
          type="date"
          value={date}
          display={date ? formatDate(fromInputs(date, '')) : ''}
          placeholder="Pick a date"
          label="Date"
          disabled={linked}
          onchange={(value) => (date = value)}
        />
      </div>
      <div class="field">
        <span class="caps">Time</span>
        <PickerField
          type="time"
          value={time}
          display={date && time ? formatTime(fromInputs(date, time)) : ''}
          placeholder="Pick a time"
          label="Time"
          disabled={linked}
          onchange={(value) => (time = value)}
        />
      </div>
    </div>
    {/if}

    {#if !linked}
      <div class="field">
        <span class="caps">Type</span>
        <div class="seg" role="radiogroup" aria-label="Type">
          {#each kinds as k (k.id)}
            <button role="radio" aria-checked={kind === k.id} class:on={kind === k.id} onclick={() => (kind = k.id)}>
              {#if kind === k.id}<span style:color="var(--{k.id})"><Icon name={k.icon} size={14} /></span>{/if}
              {kindLabel[k.id]}
            </button>
          {/each}
        </div>
      </div>

      <label class="field">
        <span class="caps">Repeat</span>
        <span class="input">
          <select bind:value={repeat} aria-label="Repeat">
            {#each Object.entries(repeatLabel) as [value, label] (value)}
              <option {value}>{label}</option>
            {/each}
          </select>
          <Icon name="chevron-down" size={12} />
        </span>
      </label>

      {#if interval}
        <div class="pair">
          <label class="field">
            <span class="caps">Every</span>
            <span class="input">
              <select bind:value={everyMin} aria-label="Every">
                {#each everyOptions as minutes (minutes)}
                  <option value={minutes}>{minutes % 60 === 0 ? `${minutes / 60} hour${minutes > 60 ? 's' : ''}` : `${minutes} minutes`}</option>
                {/each}
              </select>
              <Icon name="chevron-down" size={12} />
            </span>
          </label>
          <div class="field">
            <span class="caps">On</span>
            <div class="seg" role="radiogroup" aria-label="On">
              <button role="radio" aria-checked={windowDays === 'weekdays'} class:on={windowDays === 'weekdays'} onclick={() => (windowDays = 'weekdays')}>Weekdays</button>
              <button role="radio" aria-checked={windowDays === 'daily'} class:on={windowDays === 'daily'} onclick={() => (windowDays = 'daily')}>Every day</button>
            </div>
          </div>
        </div>
        <div class="pair">
          <div class="field">
            <span class="caps">From</span>
            <PickerField
              type="time"
              value={toInput(windowFrom)}
              display={formatTime(fromInputs(toDateInput(Date.now()), toInput(windowFrom)))}
              placeholder="07:00"
              label="From"
              onchange={(value) => value && (windowFrom = fromInput(value))}
            />
          </div>
          <div class="field">
            <span class="caps">Until</span>
            <PickerField
              type="time"
              value={toInput(windowUntil)}
              display={formatTime(fromInputs(toDateInput(Date.now()), toInput(windowUntil)))}
              placeholder="17:00"
              label="Until"
              onchange={(value) => value && (windowUntil = fromInput(value))}
            />
          </div>
        </div>
      {/if}
    {/if}

    {#if !interval}
      <div class="field">
        <span class="caps">Remind me</span>
        <div class="chips">
          {#each leads as lead (lead.minutes)}
            <button class="chip" class:on={leadMin === lead.minutes} onclick={() => (leadMin = lead.minutes)}>{lead.label}</button>
          {/each}
        </div>
      </div>
    {/if}

    {#if reminders.error}<p class="error meta">{reminders.error}</p>{/if}
  </div>

  <div class="footer">
    {#if id !== null && !linked}
      {#if confirmDelete}
        <button class="btn danger" onclick={remove}><Icon name="delete" size={14} />Confirm delete</button>
      {:else}
        <button class="btn danger" onclick={() => (confirmDelete = true)}><Icon name="delete" size={14} />Delete</button>
      {/if}
    {/if}
    <span class="grow"></span>
    <button class="btn" onclick={onback}>Cancel</button>
    <button class="btn primary" onclick={save}>Save</button>
  </div>
</div>

<style>
  .input.disabled {
    opacity: var(--o-disabled);
  }

  .note {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 0;
  }

  .sm-text {
    height: 22px;
    margin-left: auto;
    font-size: var(--fs-caption);
  }

  .seg {
    display: flex;
    padding: 2px;
    border-radius: var(--radius-control);
    background: var(--surface);
  }

  .seg button {
    flex: 1;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 5px;
    border-radius: 8px;
    font-weight: var(--fw-semibold);
    color: var(--muted);
    transition: background-color var(--dur-fast) var(--ease-standard), color var(--dur-fast) var(--ease-standard);
  }

  .seg button:hover {
    color: var(--fg);
  }

  .seg button.on {
    background: var(--surface-hover);
    color: var(--fg);
  }

  .chips {
    display: flex;
    gap: 6px;
  }

  .error {
    margin: 0;
    color: var(--danger);
  }

  .footer {
    display: flex;
    align-items: center;
    gap: 8px;
    padding-top: 12px;
  }
</style>
