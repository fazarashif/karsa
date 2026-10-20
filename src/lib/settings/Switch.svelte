<script lang="ts">
  let { checked, label, onchange }: { checked: boolean; label: string; onchange: (next: boolean) => void } = $props();
</script>

<button class="switch" class:on={checked} role="switch" aria-checked={checked} aria-label={label} onclick={() => onchange(!checked)}>
  <span class="knob"></span>
</button>

<style>
  .switch {
    flex: none;
    position: relative;
    width: 34px;
    height: 20px;
    border-radius: 10px;
    background: var(--surface-hover);
    transition: background-color var(--dur-fast) var(--ease-standard);
  }

  .switch:hover {
    background: var(--surface-pressed);
  }

  .switch.on {
    background: var(--accent);
  }

  .switch.on:hover {
    background: var(--accent-hover);
  }

  .knob {
    position: absolute;
    top: 3px;
    left: 3px;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--muted);
    transition: transform var(--dur-fast) var(--ease-standard), background-color var(--dur-fast) var(--ease-standard);
  }

  .switch.on .knob {
    transform: translateX(14px);
    background: var(--on-accent);
  }
</style>
