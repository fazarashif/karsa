<script lang="ts">
  import { onEscape } from '../escape';
  import Icon from '../Icon.svelte';
  import { todo, topLevel } from '../todo.svelte';
  import { panel } from '../ui.svelte';
  import ListManager from './ListManager.svelte';
  import ListMenu from './ListMenu.svelte';
  import TaskDetail from './TaskDetail.svelte';
  import TaskRow from './TaskRow.svelte';

  type View = { kind: 'list' } | { kind: 'task'; id: number } | { kind: 'lists' };

  let view = $state<View>({ kind: 'list' });
  let menuOpen = $state(false);
  let showCompleted = $state(false);
  let draft = $state('');
  let input = $state<HTMLInputElement>();

  const list = $derived(todo.lists.find((l) => l.id === todo.currentListId) ?? todo.lists[0]);
  const tasks = $derived(topLevel(todo.tasks, list.id));
  const open = $derived(tasks.filter((t) => !t.done));
  const completed = $derived(tasks.filter((t) => t.done));

  // Another tab asked to show a task (e.g. a "From Todo" reminder).
  $effect(() => {
    const id = panel.openTaskId;
    if (id === null) return;
    const task = todo.tasks.find((t) => t.id === id);
    if (task) {
      todo.currentListId = task.listId;
      view = { kind: 'task', id: task.parentId ?? task.id };
    }
    panel.openTaskId = null;
  });

  // Esc in a sub-view goes back to the list instead of closing the panel.
  $effect(() => {
    if (view.kind !== 'list') return onEscape(() => (view = { kind: 'list' }));
  });

  // The quick-add field is ready to type into whenever the list is showing.
  $effect(() => {
    if (view.kind === 'list' && !menuOpen) input?.focus();
  });

  async function add(e: SubmitEvent) {
    e.preventDefault();
    if (!draft.trim()) return;
    await todo.createTask(list.id, draft);
    if (!todo.error) draft = '';
  }

  const openTask = (id: number) => (view = { kind: 'task', id });
</script>

{#if view.kind === 'task'}
  <TaskDetail id={view.id} onback={() => (view = { kind: 'list' })} />
{:else if view.kind === 'lists'}
  <ListManager onback={() => (view = { kind: 'list' })} />
{:else}
  <div class="view">
    <div class="tbar">
      <div class="switch-wrap">
        <button
          class="switch"
          class:open={menuOpen}
          aria-haspopup="menu"
          aria-expanded={menuOpen}
          onclick={() => (menuOpen = !menuOpen)}
        >
          <span class="ell">{list.name}</span>
          <Icon name="chevron-down" size={12} />
        </button>
        {#if menuOpen}
          <ListMenu
            onclose={() => (menuOpen = false)}
            onmanage={() => {
              menuOpen = false;
              view = { kind: 'lists' };
            }}
          />
        {/if}
      </div>
      {#if tasks.length > 0}<span class="meta">{open.length} open</span>{/if}
    </div>

    <form class="input add" onsubmit={add}>
      <Icon name="add" size={16} />
      <input bind:this={input} bind:value={draft} placeholder="Add a task…" aria-label="New task" maxlength="500" />
    </form>

    {#if todo.error}<p class="error meta">{todo.error}</p>{/if}

    {#if tasks.length === 0}
      <div class="empty">
        <span style:color="var(--ornament)"><Icon name="ornaments/kawung-mark" size={24} /></span>
        <span class="proverb">Alon-alon waton kelakon</span>
        <span class="muted">Slowly, as long as it gets done.</span>
        <span class="faint small">Type above and press Enter.</span>
      </div>
    {:else}
      <div class="scroll list">
        {#each open as task (task.id)}
          <TaskRow {task} onopen={openTask} />
        {/each}
        {#if completed.length > 0}
          <button class="sect caps" aria-expanded={showCompleted} onclick={() => (showCompleted = !showCompleted)}>
            <span class="chev" class:open={showCompleted}><Icon name="chevron-down" size={12} /></span>
            Completed · {completed.length}
          </button>
          {#if showCompleted}
            {#each completed as task (task.id)}
              <TaskRow {task} onopen={openTask} />
            {/each}
          {/if}
        {/if}
      </div>
    {/if}
  </div>
{/if}

<style>
  .tbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 28px;
    margin-bottom: 8px;
    padding: 0 4px;
  }

  .switch-wrap {
    position: relative;
    min-width: 0;
  }

  .switch {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    max-width: 240px;
    height: 28px;
    padding: 0 6px 0 8px;
    margin-left: -8px;
    border-radius: 8px;
    font-weight: var(--fw-semibold);
    transition: background-color var(--dur-fast) var(--ease-standard);
  }

  .switch:hover,
  .switch.open {
    background: var(--surface);
  }

  .list {
    margin-top: 8px;
  }

  .error {
    margin: 8px 4px 0;
    color: var(--danger);
  }

  .small {
    font-size: var(--fs-caption);
    line-height: var(--lh-caption);
  }

  .sect {
    display: flex;
    align-items: center;
    gap: 4px;
    width: 100%;
    height: 32px;
    padding: 0 4px;
    margin-top: 4px;
  }

  .sect:hover {
    color: var(--muted);
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
