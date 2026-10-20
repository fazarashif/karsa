<script lang="ts">
  import Icon from '../Icon.svelte';
  import { INBOX_ID, todo } from '../todo.svelte';

  let { onback }: { onback: () => void } = $props();

  let confirming = $state<number | null>(null);
  let draft = $state('');

  const taskCount = (listId: number) => todo.tasks.filter((t) => t.listId === listId && t.parentId === null).length;

  function rename(input: HTMLInputElement, id: number, current: string) {
    const value = input.value.trim();
    if (value && value !== current) todo.renameList(id, value);
    else input.value = current;
  }

  async function create(e: SubmitEvent) {
    e.preventDefault();
    if (!draft.trim()) return;
    await todo.createList(draft);
    draft = '';
  }

  function remove(id: number) {
    confirming = null;
    todo.deleteList(id);
  }
</script>

<div class="view">
  <div class="bar">
    <button class="back" onclick={onback}><Icon name="chevron-left" size={16} />Todo</button>
    <span class="grow"></span>
    <span class="caps">Lists</span>
  </div>

  <div class="scroll">
    {#each todo.lists as list, i (list.id)}
      <div class="row">
        {#if confirming === list.id}
          <span class="grow ell">Delete “{list.name}” and {taskCount(list.id)} tasks?</span>
          <button class="btn sm text muted" onclick={() => (confirming = null)}>Cancel</button>
          <button class="btn sm danger" onclick={() => remove(list.id)}>Delete</button>
        {:else}
          <span class="input sm grow">
            <input
              value={list.name}
              aria-label="List name"
              maxlength="100"
              onblur={(e) => rename(e.currentTarget, list.id, list.name)}
              onkeydown={(e) => e.key === 'Enter' && e.currentTarget.blur()}
            />
          </span>
          <button class="ibtn" aria-label="Move up" disabled={i === 0} onclick={() => todo.moveList(list.id, -1)}>
            <span class="up"><Icon name="chevron-down" size={12} /></span>
          </button>
          <button
            class="ibtn"
            aria-label="Move down"
            disabled={i === todo.lists.length - 1}
            onclick={() => todo.moveList(list.id, 1)}
          >
            <Icon name="chevron-down" size={12} />
          </button>
          <button
            class="ibtn danger"
            aria-label="Delete list"
            disabled={list.id === INBOX_ID}
            onclick={() => (confirming = list.id)}
          >
            <Icon name="delete" size={16} />
          </button>
        {/if}
      </div>
    {/each}

    <form class="input sm new" onsubmit={create}>
      <Icon name="add" size={16} />
      <input bind:value={draft} placeholder="New list…" aria-label="New list" maxlength="100" />
    </form>
  </div>
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: 4px;
    min-height: 40px;
    padding: 0 0 4px;
  }

  .up {
    display: grid;
    transform: rotate(180deg);
  }

  .new {
    margin-top: 6px;
  }
</style>
