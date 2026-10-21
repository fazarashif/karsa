<!-- Shown while files are dragged over the island, and briefly after they are dropped. -->
<script lang="ts">
  import Icon from './Icon.svelte';

  let { count, kept }: { count: number; kept: number | null } = $props();
</script>

<div class="drop">
  <span class="mark beat" style="--i: 0">
    <Icon name={kept === null ? 'shelf' : 'check-circle'} size={22} />
  </span>
  <span class="text beat" style="--i: 1">
    {#if kept === null}
      <span class="title">Drop to keep on the shelf</span>
      <span class="muted">{count === 1 ? '1 item' : `${count} items`} · Karsa keeps a link, not a copy</span>
    {:else}
      <span class="title">On the shelf</span>
      <span class="muted">{kept === 1 ? '1 item' : `${kept} items`} · find them under Shelf in the panel</span>
    {/if}
  </span>
</div>

<style>
  .drop {
    display: flex;
    align-items: center;
    gap: 14px;
    box-sizing: border-box;
    width: var(--alert-w);
    height: var(--alert-h);
    padding: 16px 18px;
    border: 1px dashed transparent;
  }

  .mark {
    flex: none;
    color: var(--accent);
  }

  .text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .title {
    font: var(--fw-semibold) var(--fs-title) / var(--lh-title) var(--font-text);
  }
</style>
