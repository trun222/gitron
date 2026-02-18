<script lang="ts">
  import { selectedCommit } from '$lib/stores/repo';

  function formatFullDate(timestamp: string): string {
    return new Date(timestamp).toLocaleString();
  }
</script>

{#if $selectedCommit}
  <div class="px-4 py-3 border-t border-border bg-card max-h-[200px] overflow-y-auto">
    <div class="flex items-start justify-between gap-3 mb-2">
      <h3 class="text-sm font-semibold text-foreground">{$selectedCommit.summary}</h3>
      <span class="font-mono text-[11px] text-muted-foreground shrink-0">{$selectedCommit.oid}</span>
    </div>

    {#if $selectedCommit.message !== $selectedCommit.summary}
      <pre class="mb-2 p-2 bg-background rounded text-xs text-muted-foreground whitespace-pre-wrap font-mono">{$selectedCommit.message}</pre>
    {/if}

    <div class="flex flex-col gap-1">
      <div class="flex gap-2 text-xs">
        <span class="text-muted-foreground min-w-[60px] font-medium">Author</span>
        <span class="text-secondary-foreground">
          {$selectedCommit.author.name}
          &lt;{$selectedCommit.author.email}&gt;
        </span>
      </div>
      <div class="flex gap-2 text-xs">
        <span class="text-muted-foreground min-w-[60px] font-medium">Date</span>
        <span class="text-secondary-foreground">{formatFullDate($selectedCommit.timestamp)}</span>
      </div>
      {#if $selectedCommit.parents.length > 0}
        <div class="flex gap-2 text-xs">
          <span class="text-muted-foreground min-w-[60px] font-medium">Parents</span>
          <span class="text-secondary-foreground">
            {#each $selectedCommit.parents as parent, i}
              <span class="font-mono text-primary">{parent.substring(0, 7)}</span>
              {#if i < $selectedCommit.parents.length - 1},&nbsp;{/if}
            {/each}
          </span>
        </div>
      {/if}
    </div>
  </div>
{/if}
