<script lang="ts">
  import {
    selectedFileDiff,
    selectedFile,
    clearFileSelection,
    selectNextFile,
    selectPrevFile,
    stageSelectedFile,
    unstageSelectedFile,
  } from '$lib/stores/repo';
  import { getHighlighter, detectLanguage, tokenizeLine } from '$lib/highlight';
  import type { Highlighter } from 'shiki';
  import type { DiffLineType, FileStatusType } from '$lib/api/types';

  let highlighter: Highlighter | null = $state(null);

  getHighlighter().then((h) => {
    highlighter = h;
  });

  let language = $derived(
    $selectedFile ? detectLanguage($selectedFile.path) : 'text',
  );

  function statusColor(status: FileStatusType): string {
    switch (status) {
      case 'Added':
        return 'text-[var(--color-git-added)] bg-[var(--color-git-added-bg)]';
      case 'Deleted':
        return 'text-[var(--color-git-deleted)] bg-[var(--color-git-deleted-bg)]';
      default:
        return 'text-[var(--color-git-modified)] bg-[var(--color-git-modified-bg)]';
    }
  }

  function lineBackground(origin: DiffLineType): string {
    switch (origin) {
      case 'Addition':
        return 'bg-[var(--color-git-added-bg)]';
      case 'Deletion':
        return 'bg-[var(--color-git-deleted-bg)]';
      default:
        return '';
    }
  }

  function originChar(origin: DiffLineType): string {
    switch (origin) {
      case 'Addition':
        return '+';
      case 'Deletion':
        return '-';
      default:
        return ' ';
    }
  }

  function originColor(origin: DiffLineType): string {
    switch (origin) {
      case 'Addition':
        return 'text-git-added';
      case 'Deletion':
        return 'text-git-deleted';
      default:
        return 'text-muted-foreground/30';
    }
  }

  function skippedLines(hunks: NonNullable<typeof $selectedFileDiff>['hunks'], index: number): number | null {
    if (!hunks || index === 0) return null;
    const prevHunk = hunks[index - 1];
    const currHunk = hunks[index];
    const prevLast = prevHunk.lines.findLast((l) => l.new_lineno !== null);
    const currFirst = currHunk.lines.find((l) => l.new_lineno !== null);
    if (!prevLast?.new_lineno || !currFirst?.new_lineno) return null;
    const gap = currFirst.new_lineno - prevLast.new_lineno - 1;
    return gap > 0 ? gap : null;
  }

  function handleKeydown(e: KeyboardEvent) {
    const target = e.target as HTMLElement;
    if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') return;

    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectNextFile();
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectPrevFile();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      clearFileSelection();
    } else if (e.key === 's' && !e.shiftKey && !e.ctrlKey && !e.metaKey) {
      e.preventDefault();
      stageSelectedFile();
    } else if (e.key === 'u' && !e.shiftKey && !e.ctrlKey && !e.metaKey) {
      e.preventDefault();
      unstageSelectedFile();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="flex flex-col flex-1 min-h-0">
  {#if $selectedFileDiff}
    <!-- Header -->
    <div
      class="flex items-center justify-between px-4 py-2 border-b border-border bg-card shrink-0"
    >
      <div class="flex items-center gap-2 min-w-0">
        <span
          class="text-[10px] font-bold w-4 h-4 flex items-center justify-center rounded-sm shrink-0 {statusColor($selectedFileDiff.status)}"
        >
          {$selectedFileDiff.status[0]}
        </span>
        <span class="text-sm font-mono truncate">{$selectedFileDiff.path}</span>
        {#if $selectedFile}
          <span class="text-[10px] text-muted-foreground/60 uppercase">{$selectedFile.section}</span>
        {/if}
      </div>
      <button
        onclick={clearFileSelection}
        class="text-muted-foreground hover:text-foreground p-1 shrink-0 cursor-pointer"
        aria-label="Close file preview"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <line x1="18" y1="6" x2="6" y2="18"></line>
          <line x1="6" y1="6" x2="18" y2="18"></line>
        </svg>
      </button>
    </div>

    <!-- Diff content -->
    <div class="flex-1 overflow-auto min-h-0 font-mono leading-5" style="font-size: var(--editor-font-size)">
      {#if $selectedFileDiff.is_binary}
        <p class="text-muted-foreground text-sm text-center p-8">Binary file — cannot display diff</p>
      {:else if $selectedFileDiff.hunks.length === 0}
        <p class="text-muted-foreground text-sm text-center p-8">No changes</p>
      {:else}
        {#each $selectedFileDiff.hunks as hunk, i}
          {#if hunk.lines.length > 0}
            <!-- Hunk separator -->
            {#if i > 0}
              {@const gap = skippedLines($selectedFileDiff.hunks, i)}
              <div class="flex items-center gap-3 px-4 py-3 select-none bg-accent/30">
                <div class="flex-1 border-t border-dashed border-muted-foreground/50"></div>
                <span class="text-[11px] text-muted-foreground">{gap ? `${gap} lines` : '···'}</span>
                <div class="flex-1 border-t border-dashed border-muted-foreground/50"></div>
              </div>
            {/if}
            <!-- Lines -->
            {#each hunk.lines as line}
              <div class="flex {lineBackground(line.origin)} min-w-fit">
                <!-- Gutter -->
                <div class="flex shrink-0 bg-card/50 border-r border-border/50 select-none">
                  <span class="w-12 text-right pr-2 text-muted-foreground/30">
                    {line.new_lineno ?? ''}
                  </span>
                  <span class="w-6 text-center {originColor(line.origin)}">
                    {originChar(line.origin)}
                  </span>
                </div>
                <!-- Syntax-highlighted content -->
                <span class="whitespace-pre pl-2 pr-4"
                  >{#each tokenizeLine(highlighter, line.content, language) as token}<span
                      style:color={token.color}>{token.content}</span
                    >{/each}</span
                >
              </div>
            {/each}
          {/if}
        {/each}
      {/if}
    </div>
  {:else if $selectedFile}
    <!-- Loading state -->
    <div class="flex items-center justify-center flex-1">
      <p class="text-muted-foreground text-sm">Loading diff...</p>
    </div>
  {/if}
</div>
