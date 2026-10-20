<script lang="ts">
  import type { AlertPayload } from './bridge';
  import Eyes from './Eyes.svelte';
  import Icon from './Icon.svelte';
  import { typeColor } from './Pill.svelte';

  let {
    alert,
    onsnooze,
    ondismiss,
    onfocus,
    onreview,
    onmeeting,
  }: {
    alert: AlertPayload;
    onsnooze: () => void;
    ondismiss: () => void;
    onfocus: (action: 'start' | 'skip' | 'stop') => void;
    onreview: () => void;
    onmeeting: () => void;
  } = $props();
</script>

<div class="alert" role="alert">
  <div class="top beat" style="--i: 0">
    {#if alert.kind === 'focus'}
      <span class="mark"><Eyes expression="mesem" /></span>
    {:else}
      <span class="mark gong" style:color={typeColor(alert.kind)}><Icon name="ornaments/kawung-mark" size={10} /></span>
    {/if}
    <div class="text">
      <span class="title ell">{alert.title}</span>
      <span class="muted ell">{alert.subtitle}</span>
    </div>
  </div>
  <div class="actions beat" style="--i: 1">
    {#if alert.focus === 'break-next'}
      <button class="btn sm" onclick={() => onfocus('skip')}>Skip</button>
      <button class="btn sm primary" onclick={() => onfocus('start')}>Start break</button>
    {:else if alert.focus === 'focus-next'}
      <button class="btn sm" onclick={() => onfocus('stop')}>Stop</button>
      <button class="btn sm primary" onclick={() => onfocus('start')}>Start focus</button>
    {:else if alert.focus === 'auto'}
      <button class="btn sm" onclick={() => onfocus('skip')}>Skip</button>
      <button class="btn sm primary" onclick={ondismiss}>Dismiss</button>
    {:else if alert.summary}
      <button class="btn sm" onclick={onreview}>Review</button>
      <button class="btn sm primary" onclick={ondismiss}>Dismiss</button>
    {:else}
      {#if alert.kind === 'meeting'}
        <!-- Shortened from "New meeting note" so three actions fit (DESIGN.md 4.5). -->
        <button class="btn text meeting" onclick={onmeeting}>+ Meeting note</button>
        <span class="spacer"></span>
      {/if}
      <button class="btn sm" onclick={onsnooze}><Icon name="snooze" size={14} />Snooze 5m</button>
      <button class="btn sm primary" onclick={ondismiss}>Dismiss</button>
    {/if}
  </div>
</div>

<style>
  .alert {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    box-sizing: border-box;
    width: var(--alert-w);
    height: var(--alert-h);
    padding: 16px 16px 14px 18px;
  }

  .top {
    display: flex;
    gap: 10px;
    align-items: flex-start;
  }

  .mark {
    margin-top: 5px;
  }

  .text {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .title {
    font: var(--fw-semibold) var(--fs-title) / var(--lh-title) var(--font-text);
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .meeting {
    color: var(--meeting);
  }

  .spacer {
    flex: 1;
  }
</style>
