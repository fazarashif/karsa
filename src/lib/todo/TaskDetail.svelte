<script lang="ts">
  import { untrack } from 'svelte';
  import { dayFromToday, formatDate, formatTime, fromInputs, toDateInput, toTimeInput } from '../format.svelte';
  import Icon from '../Icon.svelte';
  import PickerField from '../PickerField.svelte';
  import { subtasksOf, todo } from '../todo.svelte';
  import { panel } from '../ui.svelte';
  import Checkbox from './Checkbox.svelte';

  let { id, onback }: { id: number; onback: () => void } = $props();

  const task = $derived(todo.tasks.find((t) => t.id === id));
  const subtasks = $derived(subtasksOf(todo.tasks, id));
  const subtasksDone = $derived(subtasks.filter((s) => s.done).length);

  let title = $state(untrack(() => task?.title ?? ''));
  let subDraft = $state('');
  let confirmDelete = $state(false);

  const dateValue = $derived(task?.dueAt != null ? toDateInput(task.dueAt) : '');
  const timeValue = $derived(task?.dueAt != null && task.dueHasTime ? toTimeInput(task.dueAt) : '');

  const quickDays = [
    { label: 'Today', days: 0 },
    { label: 'Tomorrow', days: 1 },
    { label: 'Next week', days: 7 },
    { label: 'None', days: null },
  ] as const;
  const activeQuick = $derived(
    quickDays.find((q) => (q.days === null ? !dateValue : dateValue === toDateInput(dayFromToday(q.days))))?.label,
  );

  // The task was deleted (here or elsewhere): leave the detail view.
  $effect(() => {
    if (!task) onback();
  });

  function saveTitle() {
    if (!task) return;
    if (title.trim() && title.trim() !== task.title) todo.updateTask(task.id, { title });
    else title = task.title;
  }

  function setDue(date: string, time: string) {
    if (!task) return;
    if (!date) todo.setDue(task.id, null, false);
    else todo.setDue(task.id, fromInputs(date, time), time !== '');
  }

  function quick(days: number | null) {
    setDue(days === null ? '' : toDateInput(dayFromToday(days)), days === null ? '' : timeValue);
  }

  async function addSubtask(e: SubmitEvent) {
    e.preventDefault();
    if (!task || !subDraft.trim()) return;
    await todo.createTask(task.listId, subDraft, task.id);
    subDraft = '';
  }

  function deleteTask() {
    if (task) todo.deleteTask(task.id);
    onback();
  }
</script>

{#if task}
  <div class="view">
    <div class="bar">
      <button class="back" onclick={onback}><Icon name="chevron-left" size={16} />Todo</button>
      <span class="grow"></span>
      {#if confirmDelete}
        <button class="btn sm text muted" onclick={() => (confirmDelete = false)}>Cancel</button>
        <button class="btn sm danger" onclick={deleteTask}><Icon name="delete" size={14} />Delete task</button>
      {:else}
        <button class="ibtn danger" aria-label="Delete task" onclick={() => (confirmDelete = true)}>
          <Icon name="delete" size={16} />
        </button>
      {/if}
    </div>

    <div class="scroll form">
      {#if task.notePath}
        <button class="btn text from-note" onclick={() => panel.showNote(task.notePath!)}>
          <Icon name="meeting" size={14} />From a meeting note · Open
        </button>
      {/if}
      <label class="field">
        <span class="caps">Title</span>
        <span class="input">
          <input
            bind:value={title}
            maxlength="500"
            onblur={saveTitle}
            onkeydown={(e) => e.key === 'Enter' && e.currentTarget.blur()}
          />
        </span>
      </label>

      <label class="field">
        <span class="caps">List</span>
        <span class="input">
          <select
            value={task.listId}
            onchange={(e) => todo.updateTask(task.id, { listId: Number(e.currentTarget.value) })}
          >
            {#each todo.lists as list (list.id)}
              <option value={list.id}>{list.name}</option>
            {/each}
          </select>
          <Icon name="chevron-down" size={12} />
        </span>
      </label>

      <div class="field">
        <span class="caps">Due</span>
        <div class="chips">
          {#each quickDays as q (q.label)}
            <button class="chip" class:on={activeQuick === q.label} onclick={() => quick(q.days)}>{q.label}</button>
          {/each}
        </div>
        <div class="pair">
          <PickerField
            type="date"
            value={dateValue}
            display={task.dueAt !== null ? formatDate(task.dueAt) : ''}
            placeholder="Pick a date"
            label="Due date"
            onchange={(value) => setDue(value, timeValue)}
          />
          <PickerField
            type="time"
            value={timeValue}
            display={timeValue ? formatTime(task.dueAt!) : ''}
            placeholder="Add a time"
            label="Due time"
            onchange={(value) => setDue(dateValue || toDateInput(dayFromToday(0)), value)}
            onclear={() => setDue(dateValue, '')}
          />
        </div>
        {#if timeValue}<span class="meta">Karsa reminds you at the due time.</span>{/if}
      </div>

      <div class="field">
        <span class="caps">Subtasks{subtasks.length ? ` · ${subtasksDone}/${subtasks.length}` : ''}</span>
        {#each subtasks as sub (sub.id)}
          <div class="sub" class:done={sub.done}>
            <Checkbox
              size={14}
              checked={sub.done}
              label="Done: {sub.title}"
              onchange={() => todo.updateTask(sub.id, { done: !sub.done })}
            />
            <span class="grow sub-title">{sub.title}</span>
            <button class="ibtn sm danger" aria-label="Delete subtask" onclick={() => todo.deleteTask(sub.id)}>
              <Icon name="close" size={12} />
            </button>
          </div>
        {/each}
        <form class="input sm" onsubmit={addSubtask}>
          <Icon name="add" size={16} />
          <input bind:value={subDraft} placeholder="Add a subtask…" aria-label="New subtask" maxlength="500" />
        </form>
      </div>
    </div>
  </div>
{/if}

<style>
  .chips {
    display: flex;
    gap: 6px;
    margin-bottom: 4px;
  }

  .from-note {
    align-self: flex-start;
    color: var(--meeting);
    margin-left: -6px;
  }

  .sub {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 4px;
    border-bottom: 1px solid var(--hairline);
  }

  .sub-title {
    overflow-wrap: anywhere;
  }

  .sub.done .sub-title {
    color: var(--faint);
    text-decoration: line-through;
  }

  .sub + form {
    margin-top: 6px;
  }
</style>
