<script lang="ts">
  import { selectedCommit } from '$lib/stores/repo';

  let expanded = $state(false);

  function formatFullDate(timestamp: string): string {
    return new Date(timestamp).toLocaleString();
  }

  function formatShortDate(timestamp: string): string {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / 86400000);
    if (diffDays < 1) return 'today';
    if (diffDays < 30) return `${diffDays}d ago`;
    return date.toLocaleDateString();
  }

  function toggle() {
    expanded = !expanded;
  }
</script>

{#if $selectedCommit}
  <div class="border-t border-border bg-card flex flex-col {expanded ? 'max-h-[50vh]' : ''}">
    <!-- Collapsed header row — always visible -->
    <button
      class="flex items-center gap-3 px-4 py-2 w-full text-left cursor-pointer hover:bg-accent/50 transition-colors shrink-0"
      onclick={toggle}
    >
      <svg
        class="shrink-0 text-muted-foreground transition-transform {expanded ? 'rotate-180' : ''}"
        width="14"
        height="14"
        viewBox="0 0 16 16"
      >
        <path fill="currentColor" d="M4.427 7.427l3.396 3.396a.25.25 0 0 0 .354 0l3.396-3.396A.25.25 0 0 0 11.396 7H4.604a.25.25 0 0 0-.177.427Z" />
      </svg>

      <span class="text-sm font-semibold text-foreground truncate flex-1">{$selectedCommit.summary}</span>

      <span class="text-xs text-muted-foreground shrink-0">{$selectedCommit.author.name}</span>
      <span class="text-xs text-muted-foreground shrink-0">{formatShortDate($selectedCommit.timestamp)}</span>
      <span class="font-mono text-[11px] text-muted-foreground shrink-0">{$selectedCommit.short_oid}</span>
    </button>

    <!-- Expanded body -->
    {#if expanded}
      <div class="overflow-y-auto px-4 pb-3 border-t border-border/50">
        {#if $selectedCommit.message !== $selectedCommit.summary}
          <pre class="mt-2 mb-3 p-2 bg-background rounded text-xs text-muted-foreground whitespace-pre-wrap font-mono">{$selectedCommit.message}</pre>
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
          <div class="flex gap-2 text-xs">
            <span class="text-muted-foreground min-w-[60px] font-medium">SHA</span>
            <span class="font-mono text-primary">{$selectedCommit.oid}</span>
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
  </div>
{/if}
