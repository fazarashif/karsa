<!-- Shown when the file changed in another program since Karsa read it. -->
<script lang="ts">
  let { onreload, onkeep }: { onreload: () => void; onkeep: () => void } = $props();
</script>

<div class="conflict" role="alert">
  <span class="meta grow">This note changed outside Karsa.</span>
  <button class="btn sm" onclick={onreload}>Reload</button>
  <button class="btn sm primary" onclick={onkeep}>Keep mine</button>
</div>

<style>
  .conflict {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 8px 8px 12px;
    margin-bottom: 8px;
    border-radius: var(--radius-control);
    background: var(--deadline-tint);
  }

  .conflict .meta {
    color: var(--fg);
  }
</style>
