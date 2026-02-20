<script lang="ts">
  import { outputEntries, outputPanelOpen, clearOutput } from '$lib/stores/output';
  import { tick } from 'svelte';

  let scrollContainer: HTMLDivElement | undefined = $state();
  let prevCount = $state(0);

  // Auto-scroll to bottom when new entries are added
  $effect(() => {
    const count = $outputEntries.length;
    if (count > prevCount && scrollContainer) {
      tick().then(() => {
        if (scrollContainer) {
          scrollContainer.scrollTop = scrollContainer.scrollHeight;
        }
      });
    }
    prevCount = count;
  });

  function formatTime(date: Date): string {
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
  }
</script>

{#if $outputPanelOpen && $outputEntries.length > 0}
  <div class="flex flex-col h-[200px] border-t border-border bg-card shrink-0">
    <!-- Header -->
    <div class="flex items-center justify-between px-3 py-1.5 border-b border-border">
      <span class="text-xs font-medium text-foreground">Output</span>
      <div class="flex items-center gap-1">
        <button
          class="text-[11px] text-muted-foreground hover:text-foreground px-1.5 py-0.5 rounded hover:bg-accent transition-colors cursor-pointer"
          onclick={clearOutput}
        >
          Clear
        </button>
        <button
          class="text-muted-foreground hover:text-foreground p-0.5 rounded hover:bg-accent transition-colors cursor-pointer"
          onclick={() => outputPanelOpen.set(false)}
          aria-label="Close output panel"
        >
          <svg viewBox="0 0 16 16" width="14" height="14">
            <path fill="currentColor" d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Scrollable log area -->
    <div
      bind:this={scrollContainer}
      class="flex-1 overflow-y-auto px-3 py-2 font-mono leading-relaxed"
      style="font-size: var(--editor-font-size)"
    >
      {#each $outputEntries as entry (entry.id)}
        <div class="mb-2">
          <div class="flex items-center gap-2 text-muted-foreground text-[10px] mb-0.5">
            <span>{formatTime(entry.timestamp)}</span>
            <span class="font-semibold uppercase">{entry.operation}</span>
            {#if entry.success}
              <span class="text-git-added">OK</span>
            {:else}
              <span class="text-destructive">FAILED</span>
            {/if}
          </div>
          {#if entry.stdout}
            <pre class="whitespace-pre-wrap text-foreground m-0">{entry.stdout}</pre>
          {/if}
          {#if entry.stderr}
            <pre class="whitespace-pre-wrap m-0 {entry.success ? 'text-muted-foreground' : 'text-destructive'}">{entry.stderr}</pre>
          {/if}
        </div>
      {/each}
    </div>
  </div>
{/if}
