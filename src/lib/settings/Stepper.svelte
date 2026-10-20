<script lang="ts">
  import Icon from '../Icon.svelte';

  let {
    value,
    min,
    max,
    step = 1,
    unit,
    label,
    onchange,
  }: {
    value: number;
    min: number;
    max: number;
    step?: number;
    unit: string;
    label: string;
    onchange: (next: number) => void;
  } = $props();

  // Steps land on multiples of the step (23 → 25 → 30), then stay within the range.
  const down = $derived(Math.max(min, Math.ceil(value / step) * step - step));
  const up = $derived(Math.min(max, Math.floor(value / step) * step + step));
</script>

<span class="stepper" role="group" aria-label={label}>
  <button class="ibtn sm" aria-label="Less" disabled={value <= min} onclick={() => onchange(down)}>
    <Icon name="minus" size={14} />
  </button>
  <span class="value num">{value} <span class="muted">{unit}</span></span>
  <button class="ibtn sm" aria-label="More" disabled={value >= max} onclick={() => onchange(up)}>
    <Icon name="add" size={14} />
  </button>
</span>

<style>
  .stepper {
    flex: none;
    display: inline-flex;
    align-items: center;
    gap: 2px;
    height: 28px;
    padding: 0 4px;
    border-radius: var(--radius-button);
    background: var(--surface);
  }

  .value {
    min-width: 76px;
    text-align: center;
    font-weight: var(--fw-semibold);
  }

  .ibtn.sm:hover {
    background: var(--surface-hover);
  }
</style>
