<script lang="ts">
  import { toasts } from '../state.svelte';

  const kindColor: Record<string, string> = {
    good: 'var(--color-good)',
    warn: 'var(--color-warn)',
    info: 'var(--color-accent)'
  };
</script>

<!-- Fixed stack, bottom-right, above everything else -->
<div
  class="fixed bottom-4 right-4 z-50 flex flex-col-reverse gap-1.5 pointer-events-none"
  aria-live="polite"
  aria-atomic="false"
>
  {#each toasts.items as toast (toast.id)}
    <div
      role="status"
      class="pointer-events-auto flex items-start gap-2 rounded border bg-[var(--color-panel)] px-3 py-2 text-[11px] shadow-lg max-w-[280px]"
      style="border-color: {kindColor[toast.kind]}40; box-shadow: 0 2px 8px rgba(0,0,0,0.4);"
    >
      <!-- Coloured left accent bar -->
      <span
        class="mt-0.5 h-3 w-0.5 shrink-0 rounded-full"
        style="background: {kindColor[toast.kind]};"
      ></span>

      <span class="flex-1 leading-snug text-[var(--color-ink)]">{toast.msg}</span>

      <button
        type="button"
        onclick={() => toasts.dismiss(toast.id)}
        class="ml-1 shrink-0 text-[var(--color-muted)] hover:text-[var(--color-ink)] leading-none"
        aria-label="Dismiss"
      >×</button>
    </div>
  {/each}
</div>
