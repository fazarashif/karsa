<script lang="ts">
  import { onEscape } from '../escape';
  import Icon from '../Icon.svelte';
  import { openCount, todo } from '../todo.svelte';

  let { onclose, onmanage }: { onclose: () => void; onmanage: () => void } = $props();

  let menu = $state<HTMLDivElement>();
  let input = $state<HTMLInputElement>();
  let creating = $state(false);
  let name = $state('');

  $effect(() => onEscape(onclose));

  // Close on any click outside the menu (the switcher button toggles it itself).
  $effect(() => {
    const outside = (e: PointerEvent) => {
      const target = e.target as Element;
      if (!menu?.contains(target) && !target.closest('.switch')) onclose();
    };
    window.addEventListener('pointerdown', outside, true);
    return () => window.removeEventListener('pointerdown', outside, true);
  });

  $effect(() => {
    if (creating) input?.focus();
  });

  async function create(e: SubmitEvent) {
    e.preventDefault();
    if (!name.trim()) return;
    const before = new Set(todo.lists.map((l) => l.id));
    await todo.createList(name);
    const created = todo.lists.find((l) => !before.has(l.id));
    if (created) todo.currentListId = created.id;
    onclose();
  }

  function pick(id: number) {
    todo.currentListId = id;
    onclose();
  }
</script>

<div class="menu" role="menu" bind:this={menu}>
  <div class="lists">
    {#each todo.lists as list (list.id)}
      <button class="mi" role="menuitemradio" aria-checked={list.id === todo.currentListId} onclick={() => pick(list.id)}>
        <span class="grow ell">{list.name}</span>
        <span class="meta">{openCount(todo.tasks, list.id)}</span>
        <span class="check" class:on={list.id === todo.currentListId}><Icon name="check" size={14} /></span>
      </button>
    {/each}
  </div>
  <div class="mdiv"></div>
  {#if creating}
    <form class="mi new" onsubmit={create}>
      <Icon name="add" size={16} />
      <input bind:this={input} bind:value={name} placeholder="List name" aria-label="New list name" maxlength="100" />
    </form>
  {:else}
    <button class="mi muted" role="menuitem" onclick={() => (creating = true)}><Icon name="add" size={16} />New list</button>
  {/if}
  <button class="mi muted" role="menuitem" onclick={onmanage}><Icon name="edit" size={16} />Manage lists…</button>
</div>

<style>
  .menu {
    top: 36px;
    left: -8px;
  }

  .lists {
    max-height: 224px;
    overflow-y: auto;
  }

  .check {
    color: var(--focus);
    visibility: hidden;
  }

  .check.on {
    visibility: visible;
  }

  .new {
    background: var(--surface);
  }

  .new input {
    flex: 1;
    min-width: 0;
    border: 0;
    outline: none;
    background: transparent;
    color: var(--fg);
    caret-color: var(--accent);
    user-select: text;
  }
</style>
