<script lang="ts">
  import { selectedCommit, commitFiles, selectedCommitFile, selectCommitFile } from '$lib/stores/repo';
  import type { FileStatusType } from '$lib/api/types';

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

  function statusBadge(status: FileStatusType): { char: string; cls: string } {
    switch (status) {
      case 'Added':
        return { char: 'A', cls: 'text-[var(--color-git-added)] bg-[var(--color-git-added-bg)]' };
      case 'Deleted':
        return { char: 'D', cls: 'text-[var(--color-git-deleted)] bg-[var(--color-git-deleted-bg)]' };
      default:
        return { char: 'M', cls: 'text-[var(--color-git-modified)] bg-[var(--color-git-modified-bg)]' };
    }
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

        {#if $commitFiles && $commitFiles.length > 0}
          <div class="mt-3 border-t border-border/50 pt-2">
            <h4 class="text-[11px] font-semibold text-muted-foreground uppercase tracking-wide mb-1">
              Files changed ({$commitFiles.length})
            </h4>
            <div class="flex flex-col">
              {#each $commitFiles as file (file.path)}
                {@const badge = statusBadge(file.status)}
                <button
                  type="button"
                  class="flex items-center gap-2 px-1 py-0.5 rounded text-xs font-mono cursor-pointer transition-colors text-left
                    {$selectedCommitFile === file.path ? 'bg-accent text-foreground' : 'text-secondary-foreground hover:bg-accent/50'}"
                  onclick={() => selectCommitFile(file.path)}
                >
                  <span class="text-[10px] font-bold w-4 h-4 flex items-center justify-center rounded-sm shrink-0 {badge.cls}">{badge.char}</span>
                  <span class="truncate">{file.path}</span>
                </button>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </div>
{/if}
