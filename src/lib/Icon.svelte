<script lang="ts" module>
  // Every SVG in src/assets, inlined at build time (~16 KB total), keyed like "icons/add".
  const files = import.meta.glob('../assets/**/*.svg', { query: '?raw', import: 'default', eager: true });
  const svgs = new Map(
    Object.entries(files).map(([path, svg]) => [path.replace('../assets/', '').replace('.svg', ''), svg as string]),
  );
</script>

<script lang="ts">
  let {
    name,
    size = 16,
    width,
    height,
  }: { name: string; size?: number; width?: number; height?: number } = $props();

  const svg = $derived(svgs.get(name.includes('/') ? name : `icons/${name}`) ?? '');
</script>

<span class="icon" style:width="{width ?? size}px" style:height="{height ?? size}px" aria-hidden="true">
  {@html svg}
</span>

<style>
  .icon {
    display: inline-grid;
    flex: none;
  }

  .icon :global(svg) {
    width: 100%;
    height: 100%;
  }
</style>
