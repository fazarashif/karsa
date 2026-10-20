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
    const at = fromInputs(date, time);
    if (await reminders.save({ id, title, kind, at, repeat, leadMin })) onback();
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
    {/if}

    <div class="field">
      <span class="caps">Remind me</span>
      <div class="chips">
        {#each leads as lead (lead.minutes)}
          <button class="chip" class:on={leadMin === lead.minutes} onclick={() => (leadMin = lead.minutes)}>{lead.label}</button>
        {/each}
      </div>
    </div>

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
