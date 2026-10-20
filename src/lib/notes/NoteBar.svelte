<!-- The top bar shared by both editors: back (panel), Meeting badge, Open full (panel), More. -->
<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { inTauri } from '../bridge';
  import { onEscape } from '../escape';
  import Icon from '../Icon.svelte';
  import { notes, type NoteSession } from '../notes.svelte';
  import { panel } from '../ui.svelte';

  let {
    session,
    layout,
    meeting = false,
    title,
    onback,
  }: { session: NoteSession; layout: 'panel' | 'window'; meeting?: boolean; title: string; onback?: () => void } = $props();

  let menuOpen = $state(false);
  let confirmDelete = $state(false);
  let menu = $state<HTMLDivElement>();
  let error = $state('');

  $effect(() => {
    if (!menuOpen) return;
    const offEscape = onEscape(close);
    const outside = (e: PointerEvent) => {
      const target = e.target as Element;
      if (!menu?.contains(target) && !target.closest('.more')) close();
    };
    window.addEventListener('pointerdown', outside, true);
    return () => {
      offEscape();
      window.removeEventListener('pointerdown', outside, true);
    };
  });

  function close() {
    menuOpen = false;
    confirmDelete = false;
  }

  /** Moves the note to its own window and closes the panel. */
  async function openFull() {
    await session.flush();
    if (!session.path) return;
    await notes.openWindow(session.path, title);
    onback?.();
    panel.collapse();
  }

  async function reveal() {
    close();
    await session.flush();
    if (session.path) await notes.reveal(session.path);
  }

  async function remove() {
    try {
      session.discard();
      if (session.path) await notes.remove(session.path);
      session.path = null;
      close();
      if (layout === 'window' && inTauri) await getCurrentWindow().close();
      else onback?.();
    } catch (e) {
      error = String(e);
    }
  }
</script>

<div class="bar">
  {#if layout === 'panel'}
    <button class="back" onclick={onback}><Icon name="chevron-left" size={16} />Notes</button>
  {/if}
  <span class="grow"></span>
  {#if meeting}
    <span class="badge"><Icon name="meeting" size={12} />Meeting</span>
  {/if}
  {#if layout === 'panel'}
    <button class="ibtn" aria-label="Open full" title="Open full" disabled={!session.path} onclick={openFull}>
      <Icon name="open-full" size={16} />
    </button>
  {/if}
  <div class="more-wrap">
    <button class="ibtn more" aria-label="More" title="More" aria-haspopup="menu" aria-expanded={menuOpen} onclick={() => (menuOpen = !menuOpen)}>
      <Icon name="more" size={16} />
    </button>
    {#if menuOpen}
      <div class="menu" role="menu" bind:this={menu}>
        <button class="mi" role="menuitem" disabled={!session.path} onclick={reveal}><Icon name="open-full" size={16} />Show in folder</button>
        {#if confirmDelete}
          <button class="mi danger" role="menuitem" onclick={remove}><Icon name="delete" size={16} />Confirm: move to Recycle Bin</button>
        {:else}
          <button class="mi danger" role="menuitem" disabled={!session.path} onclick={() => (confirmDelete = true)}>
            <Icon name="delete" size={16} />Delete note
          </button>
        {/if}
        {#if error}<p class="meta err">{error}</p>{/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 18px;
    padding: 0 7px;
    margin-right: 4px;
    border-radius: var(--radius-badge);
    color: var(--meeting);
    background: var(--meeting-tint);
    font: var(--fw-semibold) var(--fs-caption) / 18px var(--font-text);
  }

  .more-wrap {
    position: relative;
  }

  .menu {
    top: 32px;
    right: 0;
    width: 230px;
  }

  .mi.danger {
    color: var(--danger);
  }

  .mi:disabled {
    opacity: var(--o-disabled);
  }

  .err {
    margin: 4px 10px;
    color: var(--danger);
  }
</style>
