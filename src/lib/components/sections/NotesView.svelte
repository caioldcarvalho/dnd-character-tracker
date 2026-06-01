<script lang="ts">
  import Card from '../Card.svelte';
  import { app } from '../../state.svelte';

  const notes = $derived(app.sheet?.notes ?? '');
  const count = $derived(notes.length);

  function onInput(e: any) {
    app.setNotes(e.currentTarget.value);
  }
</script>

{#if app.sheet}
  <Card title="Notes">
    <div class="flex flex-col gap-1">
      <textarea
        value={notes}
        oninput={onInput}
        placeholder="Backstory, session notes, reminders…"
        spellcheck="false"
        class="w-full p-2 rounded bg-[var(--color-panel-2)] border border-[var(--color-border)] text-[12px] text-[var(--color-ink)] font-mono leading-relaxed resize-y focus:border-[var(--color-accent)] focus:outline-none"
        style="min-height: 60vh;"
      ></textarea>
      <span class="text-[10px] text-[var(--color-muted)] self-end">
        <span class="num">{count}</span> chars
      </span>
    </div>
  </Card>
{:else}
  <Card title="Notes">
    <p class="text-[11px] text-[var(--color-muted)]">No character.</p>
  </Card>
{/if}
