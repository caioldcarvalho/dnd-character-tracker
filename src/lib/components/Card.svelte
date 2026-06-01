<script lang="ts">
  // A titled panel in the cockpit grid. Dense by default.
  let { title, span = 1, children } = $props<{
    title?: string;
    span?: number;
    children: import('svelte').Snippet;
  }>();
</script>

<section
  class="rounded border border-[var(--color-border)] bg-[var(--color-panel)] overflow-hidden flex flex-col"
  style="grid-column: span {span};"
>
  {#if title}
    <header
      class="px-2.5 py-1.5 text-[10px] font-semibold uppercase tracking-wider text-[var(--color-muted)] border-b border-[var(--color-border)] bg-[var(--color-panel-2)]"
    >
      {title}
    </header>
  {/if}
  <div class="p-2 flex-1">
    {@render children()}
  </div>
</section>
