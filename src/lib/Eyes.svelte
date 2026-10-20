<script lang="ts" module>
  /** Mripat expressions; the names are internal and never shown. */
  export type Expression = 'melek' | 'fokus' | 'leren' | 'mesem' | 'merem';
</script>

<script lang="ts">
  import Icon from './Icon.svelte';
  import { crossfade } from './motion';

  let {
    expression = 'melek',
    width = 28,
    blinkKey = 0,
  }: { expression?: Expression; width?: number; blinkKey?: number } = $props();

  // Kedhep: blink once each time blinkKey changes (on hover). Never on a timer.
  let blinking = $state(false);
  $effect(() => {
    if (blinkKey === 0) return;
    blinking = false;
    requestAnimationFrame(() => (blinking = true));
  });
</script>

<span
  class="eyes"
  class:blinking
  style:width="{width}px"
  style:height="{width / 2}px"
  onanimationend={() => (blinking = false)}
>
  {#key expression}
    <span class="face" in:crossfade out:crossfade>
      <Icon name="eyes/eyes-{expression}" {width} height={width / 2} />
    </span>
  {/key}
</span>

<style>
  .eyes {
    position: relative;
    display: inline-block;
    flex: none;
  }

  .face {
    position: absolute;
    inset: 0;
  }

  .blinking {
    animation: kedhep var(--dur-blink) var(--ease-standard);
  }

  @keyframes kedhep {
    50% {
      transform: scaleY(0.18);
    }
  }
</style>
