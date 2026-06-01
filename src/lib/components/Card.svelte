<script lang="ts">
  // A titled panel in the cockpit grid. Dense by default.
  let { title, span = 1, children } = $props<{
    title?: string;
    span?: number;
    children: import('svelte').Snippet;
  }>();
</script>

<!-- No `overflow-hidden`: it would clip absolutely-positioned dropdowns (the
     Picker) and any tall expanded content. The header gets its own rounded top
     corners so the panel still looks clean. -->
<section
  class="rounded border border-[var(--color-border)] bg-[var(--color-panel)] flex flex-col"
  style="grid-column: span {span};"
>
  {#if title}
    <header
      class="rounded-t px-2.5 py-1.5 text-[10px] font-semibold uppercase tracking-wider text-[var(--color-muted)] border-b border-[var(--color-border)] bg-[var(--color-panel-2)]"
    >
      {title}
    </header>
  {/if}
  <div class="p-2 flex-1">
    {@render children()}
  </div>
</section>
