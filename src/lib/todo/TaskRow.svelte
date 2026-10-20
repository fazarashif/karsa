<script lang="ts">
  import { clock } from '../clock.svelte';
  import { dueLabel, isUrgent } from '../format.svelte';
  import Icon from '../Icon.svelte';
  import { subtasksOf, todo, type Task } from '../todo.svelte';
  import Checkbox from './Checkbox.svelte';

  let { task, onopen }: { task: Task; onopen: (id: number) => void } = $props();

  let expanded = $state(false);
  const subtasks = $derived(subtasksOf(todo.tasks, task.id));
  const subtasksDone = $derived(subtasks.filter((s) => s.done).length);
</script>

<div class="task" class:done={task.done}>
  <span class="cb-slot">
    <Checkbox
      checked={task.done}
      label="Done: {task.title}"
      onchange={() => todo.updateTask(task.id, { done: !task.done })}
    />
  </span>
  <button class="main" onclick={() => onopen(task.id)}>
    <span class="title">{task.title}</span>
    {#if task.dueAt !== null || subtasks.length > 0}
      <span class="meta">
        {#if task.dueAt !== null}
          <span class:urgent={isUrgent(task, clock.now)}><Icon name="calendar" size={12} />{dueLabel(task, clock.now)}</span>
        {/if}
        {#if subtasks.length > 0}
          <span><Icon name="subtask" size={12} />{subtasksDone}/{subtasks.length}</span>
        {/if}
      </span>
    {/if}
  </button>
  {#if subtasks.length > 0}
    <button
      class="ibtn sm"
      aria-label={expanded ? 'Hide subtasks' : 'Show subtasks'}
      aria-expanded={expanded}
      onclick={() => (expanded = !expanded)}
    >
      <span class="chev" class:open={expanded}><Icon name="chevron-down" size={12} /></span>
    </button>
  {/if}
</div>

{#if expanded}
  {#each subtasks as sub (sub.id)}
    <div class="task sub" class:done={sub.done}>
      <span class="cb-slot">
        <Checkbox
          size={14}
          checked={sub.done}
          label="Done: {sub.title}"
          onchange={() => todo.updateTask(sub.id, { done: !sub.done })}
        />
      </span>
      <button class="main" onclick={() => onopen(task.id)}><span class="title">{sub.title}</span></button>
    </div>
  {/each}
{/if}

<style>
  .task {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 8px 4px;
    border-bottom: 1px solid var(--hairline);
  }

  .task.sub {
    padding: 6px 4px 6px 30px;
  }

  .cb-slot {
    display: flex;
    padding-top: 1px;
  }

  .sub .cb-slot {
    padding-top: 2px;
  }

  .main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
    text-align: left;
  }

  .title {
    overflow-wrap: anywhere;
  }

  .done .title {
    color: var(--faint);
    text-decoration: line-through;
  }

  .chev {
    display: grid;
    transform: rotate(-90deg);
    transition: transform var(--dur-base) var(--ease-standard);
  }

  .chev.open {
    transform: none;
  }
</style>
