<script lang="ts">
  import {
    selectedFileDiff,
    selectedFile,
    selectedCommitFile,
    clearFileSelection,
    selectNextFile,
    selectPrevFile,
    selectNextCommitFile,
    selectPrevCommitFile,
    stageSelectedFile,
    unstageSelectedFile,
  } from '$lib/stores/repo';
  import { getHighlighter, detectLanguage, tokenizeLine } from '$lib/highlight';
  import type { Highlighter } from 'shiki';
  import type { DiffLineType, FileStatusType } from '$lib/api/types';

  let { onClose }: { onClose?: () => void } = $props();

  function handleClose() {
    if (onClose) {
      onClose();
    } else {
      clearFileSelection();
    }
  }

  let displayPath = $derived($selectedFile?.path ?? $selectedCommitFile ?? '');

  let highlighter: Highlighter | null = $state(null);

  getHighlighter().then((h) => {
    highlighter = h;
  });

  let language = $derived(
    displayPath ? detectLanguage(displayPath) : 'text',
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

  let isCommitView = $derived($selectedCommitFile !== null);

  function handleKeydown(e: KeyboardEvent) {
    const target = e.target as HTMLElement;
    if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') return;

    if (isCommitView) {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        selectNextCommitFile();
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        selectPrevCommitFile();
      } else if (e.key === 'Escape') {
        e.preventDefault();
        handleClose();
      }
      return;
    }

    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectNextFile();
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectPrevFile();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      handleClose();
    } else if (e.key === 's' && !e.shiftKey && !e.ctrlKey && !e.metaKey) {
      e.preventDefault();
      stageSelectedFile();
    } else if (e.key === 'u' && !e.shiftKey && !e.ctrlKey && !e.metaKey) {
      e.preventDefault();
      unstageSelectedFile();
    }
  }

  // --- Virtualization ---
  const LINE_HEIGHT = 20; // matches leading-5
  const SEPARATOR_HEIGHT = 36; // hunk separator row height
  const OVERSCAN = 20; // extra rows rendered above/below viewport
  // Threshold: only virtualise when total items exceed this count
  const VIRTUAL_THRESHOLD = 200;

  // Flatten hunks into a linear list of renderable items
  interface DiffLine {
    kind: 'line';
    origin: DiffLineType;
    content: string;
    new_lineno: number | null;
  }
  interface DiffSeparator {
    kind: 'separator';
    gap: number | null;
  }
  type DiffItem = DiffLine | DiffSeparator;

  let flatItems = $derived.by((): DiffItem[] => {
    const diff = $selectedFileDiff;
    if (!diff || diff.is_binary || diff.hunks.length === 0) return [];
    const items: DiffItem[] = [];
    for (let i = 0; i < diff.hunks.length; i++) {
      const hunk = diff.hunks[i];
      if (hunk.lines.length === 0) continue;
      if (i > 0) {
        items.push({ kind: 'separator', gap: skippedLines(diff.hunks, i) });
      }
      for (const line of hunk.lines) {
        items.push({ kind: 'line', origin: line.origin, content: line.content, new_lineno: line.new_lineno });
      }
    }
    return items;
  });

  let useVirtual = $derived(flatItems.length > VIRTUAL_THRESHOLD);

  // Total height for the virtual scroller
  let totalHeight = $derived.by(() => {
    if (!useVirtual) return 0;
    let h = 0;
    for (const item of flatItems) {
      h += item.kind === 'separator' ? SEPARATOR_HEIGHT : LINE_HEIGHT;
    }
    return h;
  });

  // Precompute cumulative offsets for each item
  let itemOffsets = $derived.by((): number[] => {
    if (!useVirtual) return [];
    const offsets: number[] = new Array(flatItems.length);
    let y = 0;
    for (let i = 0; i < flatItems.length; i++) {
      offsets[i] = y;
      y += flatItems[i].kind === 'separator' ? SEPARATOR_HEIGHT : LINE_HEIGHT;
    }
    return offsets;
  });

  let scrollContainer: HTMLDivElement | undefined = $state();
  let scrollTop = $state(0);
  let viewportHeight = $state(0);

  // Reset scroll when diff changes
  $effect(() => {
    void $selectedFileDiff;
    scrollTop = 0;
    if (scrollContainer) scrollContainer.scrollTop = 0;
  });

  function handleScroll() {
    if (scrollContainer) {
      scrollTop = scrollContainer.scrollTop;
      viewportHeight = scrollContainer.clientHeight;
    }
  }

  // Binary search to find the first visible item
  function findFirstVisible(top: number): number {
    let lo = 0, hi = itemOffsets.length - 1;
    while (lo < hi) {
      const mid = (lo + hi) >>> 1;
      const itemBottom = itemOffsets[mid] + (flatItems[mid].kind === 'separator' ? SEPARATOR_HEIGHT : LINE_HEIGHT);
      if (itemBottom <= top) lo = mid + 1;
      else hi = mid;
    }
    return lo;
  }

  let visibleRange = $derived.by((): { start: number; end: number; offsetY: number } => {
    if (!useVirtual || flatItems.length === 0) return { start: 0, end: 0, offsetY: 0 };
    const first = Math.max(0, findFirstVisible(scrollTop) - OVERSCAN);
    const bottom = scrollTop + viewportHeight;
    let last = first;
    while (last < flatItems.length) {
      if (itemOffsets[last] > bottom) break;
      last++;
    }
    last = Math.min(flatItems.length, last + OVERSCAN);
    return { start: first, end: last, offsetY: itemOffsets[first] ?? 0 };
  });

  let visibleItems = $derived(
    useVirtual ? flatItems.slice(visibleRange.start, visibleRange.end) : flatItems,
  );

  // Memoize syntax highlighting results
  let highlightCache = new Map<string, { content: string; color?: string }[]>();
  let lastDiffPath: string | null = null;

  // Clear cache when file changes
  $effect(() => {
    const path = displayPath;
    if (path !== lastDiffPath) {
      highlightCache = new Map();
      lastDiffPath = path;
    }
  });

  function tokenizeLineCached(
    hl: Highlighter | null,
    content: string,
    lang: string,
  ): { content: string; color?: string }[] {
    const cached = highlightCache.get(content);
    if (cached) return cached;
    const result = tokenizeLine(hl, content, lang);
    // Cap cache size to prevent unbounded growth
    if (highlightCache.size > 5000) highlightCache.clear();
    highlightCache.set(content, result);
    return result;
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
        {:else if $selectedCommitFile}
          <span class="text-[10px] text-muted-foreground/60 uppercase">commit</span>
        {/if}
      </div>
      <button
        onclick={handleClose}
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
    <div
      bind:this={scrollContainer}
      onscroll={handleScroll}
      class="flex-1 overflow-auto min-h-0 font-mono leading-5"
      style="font-size: var(--editor-font-size)"
    >
      {#if $selectedFileDiff.is_binary}
        <p class="text-muted-foreground text-sm text-center p-8">Binary file — cannot display diff</p>
      {:else if $selectedFileDiff.hunks.length === 0}
        <p class="text-muted-foreground text-sm text-center p-8">No changes</p>
      {:else if useVirtual}
        <!-- Virtualised rendering for large diffs -->
        <div style="height: {totalHeight}px; position: relative;">
          <div style="position: absolute; top: {visibleRange.offsetY}px; left: 0; right: 0;">
            {#each visibleItems as item}
              {#if item.kind === 'separator'}
                <div class="flex items-center gap-3 px-4 select-none bg-accent/30" style="height: {SEPARATOR_HEIGHT}px;">
                  <div class="flex-1 border-t border-dashed border-muted-foreground/50"></div>
                  <span class="text-[11px] text-muted-foreground">{item.gap ? `${item.gap} lines` : '···'}</span>
                  <div class="flex-1 border-t border-dashed border-muted-foreground/50"></div>
                </div>
              {:else}
                <div class="flex {lineBackground(item.origin)} min-w-fit" style="height: {LINE_HEIGHT}px;">
                  <!-- Gutter -->
                  <div class="flex shrink-0 bg-card/50 border-r border-border/50 select-none">
                    <span class="w-12 text-right pr-2 text-muted-foreground/30">
                      {item.new_lineno ?? ''}
                    </span>
                    <span class="w-6 text-center {originColor(item.origin)}">
                      {originChar(item.origin)}
                    </span>
                  </div>
                  <!-- Syntax-highlighted content -->
                  <span class="whitespace-pre pl-2 pr-4"
                    >{#each tokenizeLineCached(highlighter, item.content, language) as token}<span
                        style:color={token.color}>{token.content}</span
                      >{/each}</span
                  >
                </div>
              {/if}
            {/each}
          </div>
        </div>
      {:else}
        <!-- Small diffs: render all lines directly (no virtualisation overhead) -->
        {#each flatItems as item}
          {#if item.kind === 'separator'}
            <div class="flex items-center gap-3 px-4 py-3 select-none bg-accent/30">
              <div class="flex-1 border-t border-dashed border-muted-foreground/50"></div>
              <span class="text-[11px] text-muted-foreground">{item.gap ? `${item.gap} lines` : '···'}</span>
              <div class="flex-1 border-t border-dashed border-muted-foreground/50"></div>
            </div>
          {:else}
            <div class="flex {lineBackground(item.origin)} min-w-fit">
              <!-- Gutter -->
              <div class="flex shrink-0 bg-card/50 border-r border-border/50 select-none">
                <span class="w-12 text-right pr-2 text-muted-foreground/30">
                  {item.new_lineno ?? ''}
                </span>
                <span class="w-6 text-center {originColor(item.origin)}">
                  {originChar(item.origin)}
                </span>
              </div>
              <!-- Syntax-highlighted content -->
              <span class="whitespace-pre pl-2 pr-4"
                >{#each tokenizeLineCached(highlighter, item.content, language) as token}<span
                    style:color={token.color}>{token.content}</span
                  >{/each}</span
              >
            </div>
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
