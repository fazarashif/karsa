<!-- The Notes list's project switcher, as Todo's list menu: every note, no project, each
     project (a folder), then a new project and the manager. -->
<script lang="ts">
  import { onEscape } from '../escape';
  import Icon from '../Icon.svelte';
  import { notes } from '../notes.svelte';

  let { onclose, onmanage }: { onclose: () => void; onmanage: () => void } = $props();

  let menu = $state<HTMLDivElement>();
  let input = $state<HTMLInputElement>();
  let creating = $state(false);
  let name = $state('');
  let error = $state('');

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

  const count = (project: string | null) => notes.list.filter((n) => project === null || n.project === project).length;

  async function create(e: SubmitEvent) {
    e.preventDefault();
    if (!name.trim()) return;
    try {
      await notes.createProject(name);
      notes.selected = name.trim();
      onclose();
    } catch (err) {
      error = String(err);
    }
  }

  function pick(project: string | null) {
    notes.selected = project;
    onclose();
  }
</script>

<div class="menu" role="menu" bind:this={menu}>
  <div class="projects">
    {#each [null, '', ...notes.projects] as project (project ?? '*')}
      <button class="mi" role="menuitemradio" aria-checked={project === notes.selected} onclick={() => pick(project)}>
        {#if project}<Icon name="folder" size={16} />{/if}
        <span class="grow ell" class:muted={project === ''}>{project === null ? 'All notes' : project || 'No project'}</span>
        <span class="meta">{count(project)}</span>
        <span class="check" class:on={project === notes.selected}><Icon name="check" size={14} /></span>
      </button>
    {/each}
  </div>
  <div class="mdiv"></div>
  {#if creating}
    <form class="mi new" onsubmit={create}>
      <Icon name="add" size={16} />
      <input bind:this={input} bind:value={name} placeholder="Project name" aria-label="New project name" maxlength="80" />
    </form>
    {#if error}<p class="meta err">{error}</p>{/if}
  {:else}
    <button class="mi muted" role="menuitem" onclick={() => (creating = true)}><Icon name="add" size={16} />New project</button>
  {/if}
  <button class="mi muted" role="menuitem" onclick={onmanage}><Icon name="edit" size={16} />Manage projects…</button>
</div>

<style>
  .menu {
    top: 36px;
    left: -8px;
    width: 230px;
  }

  .projects {
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

  .err {
    margin: 4px 10px;
    color: var(--danger);
  }
</style>
