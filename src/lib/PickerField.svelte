<!-- A date or time field. The native input only provides the picker; the text shown
     follows Karsa's own format instead of the browser locale's. -->
<script lang="ts">
  import Icon from './Icon.svelte';

  let {
    type,
    value,
    display,
    placeholder,
    label,
    disabled = false,
    onchange,
    onclear,
  }: {
    type: 'date' | 'time';
    /** The native value: "YYYY-MM-DD" or "HH:MM", or "" when empty. */
    value: string;
    display: string;
    placeholder: string;
    label: string;
    disabled?: boolean;
    onchange: (value: string) => void;
    onclear?: () => void;
  } = $props();
</script>

<span class="input picker" class:disabled>
  <Icon name={type === 'date' ? 'calendar' : 'clock'} size={16} />
  <span class="shown" class:unset={!value}>{value ? display : placeholder}</span>
  <input
    class="native"
    {type}
    {value}
    {disabled}
    aria-label={label}
    onclick={(e) => e.currentTarget.showPicker?.()}
    onchange={(e) => onchange(e.currentTarget.value)}
  />
  {#if onclear && value && !disabled}
    <button class="ibtn sm clear" aria-label="Remove {label.toLowerCase()}" onclick={onclear}>
      <Icon name="close" size={12} />
    </button>
  {/if}
</span>

<style>
  .picker {
    position: relative;
  }

  .picker.disabled {
    opacity: var(--o-disabled);
  }

  .shown {
    flex: 1;
    color: var(--fg);
    font-variant-numeric: tabular-nums;
  }

  /* Not "empty": that is the global empty-state class, which centres its text. */
  .shown.unset {
    color: var(--faint);
  }

  /* Invisible, but covers the field so a click opens the native picker. */
  .picker .native {
    position: absolute;
    inset: 0;
    width: 100%;
    opacity: 0;
  }

  .clear {
    position: relative;
    z-index: 1;
  }
</style>
