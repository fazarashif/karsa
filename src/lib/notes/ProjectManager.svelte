<!-- Projects, as Todo's list manager: rename, delete (only an empty one) and add. Each is a
     folder in the notes folder, so a change here is a change in File Explorer too. -->
<script lang="ts">
  import Icon from '../Icon.svelte';
  import { notes } from '../notes.svelte';

  let { onback }: { onback: () => void } = $props();

  let confirming = $state<string | null>(null);
  let draft = $state('');
  let error = $state('');

  const noteCount = (project: string) => notes.list.filter((n) => n.project === project).length;

  async function run(action: () => Promise<void>) {
    error = '';
    try {
      await action();
    } catch (e) {
      error = String(e);
    }
  }

  function rename(input: HTMLInputElement, current: string) {
    const value = input.value.trim();
    if (value && value !== current) {
      void run(async () => {
        await notes.renameProject(current, value);
        if (notes.selected === current) notes.selected = value;
      });
    } else {
      input.value = current;
    }
  }

  async function create(e: SubmitEvent) {
    e.preventDefault();
    if (!draft.trim()) return;
    await run(() => notes.createProject(draft));
    if (!error) draft = '';
  }

  function remove(project: string) {
    confirming = null;
    void run(() => notes.deleteProject(project));
  }
</script>

<div class="view">
  <div class="bar">
    <button class="back" onclick={onback}><Icon name="chevron-left" size={16} />Notes</button>
    <span class="grow"></span>
    <span class="caps">Projects</span>
  </div>

  <div class="scroll">
    {#if notes.projects.length === 0}
      <p class="hint faint">A project is a folder in your notes folder. Make one here or in File Explorer.</p>
    {/if}
    {#each notes.projects as project (project)}
      <div class="row">
        {#if confirming === project}
          <span class="grow ell">Delete “{project}”? Its empty folder goes to the Recycle Bin.</span>
          <button class="btn sm text muted" onclick={() => (confirming = null)}>Cancel</button>
          <button class="btn sm danger" onclick={() => remove(project)}>Delete</button>
        {:else}
          <span class="input sm grow">
            <Icon name="folder" size={16} />
            <input
              value={project}
              aria-label="Project name"
              maxlength="80"
              onblur={(e) => rename(e.currentTarget, project)}
              onkeydown={(e) => e.key === 'Enter' && e.currentTarget.blur()}
            />
          </span>
          <span class="meta count">{noteCount(project)}</span>
          <button
            class="ibtn danger"
            aria-label="Delete project"
            title={noteCount(project) ? 'Move or delete its notes first' : 'Delete project'}
            disabled={noteCount(project) > 0}
            onclick={() => (confirming = project)}
          >
            <Icon name="delete" size={16} />
          </button>
        {/if}
      </div>
    {/each}

    <form class="input sm new" onsubmit={create}>
      <Icon name="add" size={16} />
      <input bind:value={draft} placeholder="New project…" aria-label="New project" maxlength="80" />
    </form>
    {#if error}<p class="meta err">{error}</p>{/if}
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

  .count {
    min-width: 20px;
    text-align: right;
  }

  .new {
    margin-top: 6px;
  }

  .hint {
    margin: 4px 4px 8px;
    font-size: var(--fs-caption);
  }

  .err {
    margin: 6px 4px;
    color: var(--danger);
  }
</style>
