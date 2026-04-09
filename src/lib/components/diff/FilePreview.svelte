<script lang="ts">
  import { tick } from 'svelte';
  import {
    selectedFileDiff,
    selectedFile,
    selectedCommitFile,
    selectedConflictFile,
    clearFileSelection,
    selectNextFile,
    selectPrevFile,
    selectNextCommitFile,
    selectPrevCommitFile,
    stageSelectedFile,
    unstageSelectedFile,
    writeResolvedFile,
  } from '$lib/stores/repo';
  import { getHighlighter, detectLanguage, tokenizeLine } from '$lib/highlight';
  import type { Highlighter } from 'shiki';
  import type { DiffLineType, FileStatusType, ConflictSection } from '$lib/api/types';

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
  let isConflictView = $derived($selectedConflictFile !== null);

  // Conflict resolution state: per-line inclusion tracking
  interface SectionResolution {
    oursIncluded: boolean[];
    theirsIncluded: boolean[];
    resolved: boolean; // explicitly marked as resolved (handles empty sides)
  }
  let resolutions = $state<SectionResolution[]>([]);

  // Reset resolutions when conflict file changes
  $effect(() => {
    if ($selectedConflictFile) {
      resolutions = $selectedConflictFile.conflict_sections.map((s) => ({
        oursIncluded: new Array(s.ours.length).fill(false),
        theirsIncluded: new Array(s.theirs.length).fill(false),
        resolved: false,
      }));
    }
  });

  let allResolved = $derived(
    $selectedConflictFile !== null &&
    $selectedConflictFile.conflict_sections.length > 0 &&
    resolutions.length === $selectedConflictFile.conflict_sections.length &&
    resolutions.every((r) => r.resolved)
  );

  function toggleOursLine(sectionIdx: number, lineIdx: number) {
    resolutions = resolutions.map((r, i) => {
      if (i !== sectionIdx) return r;
      const next = [...r.oursIncluded];
      next[lineIdx] = !next[lineIdx];
      return { ...r, oursIncluded: next, resolved: true };
    });
  }

  function toggleTheirsLine(sectionIdx: number, lineIdx: number) {
    resolutions = resolutions.map((r, i) => {
      if (i !== sectionIdx) return r;
      const next = [...r.theirsIncluded];
      next[lineIdx] = !next[lineIdx];
      return { ...r, theirsIncluded: next, resolved: true };
    });
  }

  function acceptCurrent(sectionIdx: number) {
    const section = $selectedConflictFile?.conflict_sections[sectionIdx];
    if (!section) return;
    resolutions = resolutions.map((r, i) => {
      if (i !== sectionIdx) return r;
      return {
        oursIncluded: new Array(section.ours.length).fill(true),
        theirsIncluded: new Array(section.theirs.length).fill(false),
        resolved: true,
      };
    });
  }

  function acceptIncoming(sectionIdx: number) {
    const section = $selectedConflictFile?.conflict_sections[sectionIdx];
    if (!section) return;
    resolutions = resolutions.map((r, i) => {
      if (i !== sectionIdx) return r;
      return {
        oursIncluded: new Array(section.ours.length).fill(false),
        theirsIncluded: new Array(section.theirs.length).fill(true),
        resolved: true,
      };
    });
  }

  function acceptBoth(sectionIdx: number) {
    const section = $selectedConflictFile?.conflict_sections[sectionIdx];
    if (!section) return;
    resolutions = resolutions.map((r, i) => {
      if (i !== sectionIdx) return r;
      return {
        oursIncluded: new Array(section.ours.length).fill(true),
        theirsIncluded: new Array(section.theirs.length).fill(true),
        resolved: true,
      };
    });
  }

  function acceptAllCurrent() {
    const sections = $selectedConflictFile?.conflict_sections;
    if (!sections) return;
    resolutions = sections.map((section) => ({
      oursIncluded: new Array(section.ours.length).fill(true),
      theirsIncluded: new Array(section.theirs.length).fill(false),
      resolved: true,
    }));
  }

  function acceptAllIncoming() {
    const sections = $selectedConflictFile?.conflict_sections;
    if (!sections) return;
    resolutions = sections.map((section) => ({
      oursIncluded: new Array(section.ours.length).fill(false),
      theirsIncluded: new Array(section.theirs.length).fill(true),
      resolved: true,
    }));
  }

  // Track which dropdown is open (per conflict section, per side)
  let openDropdown = $state<string | null>(null);

  function toggleDropdown(id: string, e: MouseEvent) {
    e.stopPropagation();
    openDropdown = openDropdown === id ? null : id;
  }

  function closeDropdowns() {
    openDropdown = null;
  }

  type PreviewLineKind = 'context' | 'ours' | 'theirs';
  interface PreviewLine {
    text: string;
    kind: PreviewLineKind;
  }

  function buildResolvedLines(): PreviewLine[] {
    const file = $selectedConflictFile;
    if (!file) return [];
    const lines = file.lines;
    const sections = file.conflict_sections;
    const result: PreviewLine[] = [];
    let lineIdx = 0;

    for (const [sectionIdx, section] of sections.entries()) {
      // Add lines before this conflict section (start_line is 1-based)
      while (lineIdx < section.start_line - 1) {
        result.push({ text: lines[lineIdx], kind: 'context' });
        lineIdx++;
      }

      // Add included lines: ours first, then theirs
      const res = resolutions[sectionIdx];
      if (res) {
        for (let i = 0; i < section.ours.length; i++) {
          if (res.oursIncluded[i]) result.push({ text: section.ours[i], kind: 'ours' });
        }
        for (let i = 0; i < section.theirs.length; i++) {
          if (res.theirsIncluded[i]) result.push({ text: section.theirs[i], kind: 'theirs' });
        }
      }

      // Skip past the conflict marker lines (end_line is 1-based, inclusive)
      lineIdx = section.end_line;
    }

    // Add remaining lines after the last conflict
    while (lineIdx < lines.length) {
      result.push({ text: lines[lineIdx], kind: 'context' });
      lineIdx++;
    }

    return result;
  }

  function buildResolvedContent(): string {
    return buildResolvedLines().map((l) => l.text).join('\n') + '\n';
  }

  // Live preview of resolved content
  let previewContent = $derived.by(() => {
    if (!$selectedConflictFile) return '';
    return buildResolvedContent();
  });

  let previewLineData = $derived.by(() => {
    if (!$selectedConflictFile) return [];
    return buildResolvedLines();
  });

  // Bracket balance warning
  let bracketWarning = $derived.by((): string | null => {
    if (!previewContent) return null;
    const openBrace = (previewContent.match(/\{/g) || []).length;
    const closeBrace = (previewContent.match(/\}/g) || []).length;
    if (openBrace !== closeBrace) return `Unbalanced braces: ${openBrace} opening, ${closeBrace} closing`;
    const openParen = (previewContent.match(/\(/g) || []).length;
    const closeParen = (previewContent.match(/\)/g) || []).length;
    if (openParen !== closeParen) return `Unbalanced parentheses: ${openParen} opening, ${closeParen} closing`;
    return null;
  });

  let previewOpen = $state(false);

  // Sync preview to current conflict when resolutions change (user accepts/toggles lines)
  $effect(() => {
    void resolutions; // track changes
    if (previewOpen) {
      tick().then(() => {
        if (currentConflictIdx < previewRegions.length) {
          scrollToPreviewRegion(currentConflictIdx);
        }
      });
    }
  });

  // Preview change navigation — groups contiguous resolved lines into regions
  let previewRegions = $derived.by(() => {
    const regions: number[] = []; // line indices where each region starts
    const data = previewLineData;
    for (let i = 0; i < data.length; i++) {
      if (data[i].kind !== 'context' && (i === 0 || data[i - 1].kind === 'context')) {
        regions.push(i);
      }
    }
    return regions;
  });

  let previewScrollContainer: HTMLDivElement | undefined = $state();
  let currentPreviewRegionIdx = $state(0);
  let previewScrollLocked = false;

  // Reset when preview data changes
  $effect(() => {
    void previewLineData;
    currentPreviewRegionIdx = 0;
  });

  function scrollToPreviewRegion(idx: number) {
    const el = previewScrollContainer?.querySelector(`[data-preview-region="${idx}"]`);
    if (el) {
      previewScrollLocked = true;
      el.scrollIntoView({ behavior: 'smooth', block: 'center' });
      setTimeout(() => { previewScrollLocked = false; }, 400);
    }
    currentPreviewRegionIdx = idx;
  }

  function nextPreviewRegion() {
    if (previewRegions.length === 0) return;
    const next = currentPreviewRegionIdx + 1 < previewRegions.length ? currentPreviewRegionIdx + 1 : 0;
    scrollToPreviewRegion(next);
  }

  function prevPreviewRegion() {
    if (previewRegions.length === 0) return;
    const prev = currentPreviewRegionIdx - 1 >= 0 ? currentPreviewRegionIdx - 1 : previewRegions.length - 1;
    scrollToPreviewRegion(prev);
  }

  function updateCurrentPreviewRegionFromScroll() {
    if (previewScrollLocked) return;
    if (!previewScrollContainer || previewRegions.length === 0) return;
    const container = previewScrollContainer;
    const threshold = container.scrollTop + container.clientHeight / 3;
    let closest = 0;
    for (let i = 0; i < previewRegions.length; i++) {
      const el = container.querySelector(`[data-preview-region="${i}"]`) as HTMLElement | null;
      if (el && el.offsetTop <= threshold) closest = i;
    }
    currentPreviewRegionIdx = closest;
  }

  function handleMarkResolved() {
    const file = $selectedConflictFile;
    if (!file || !allResolved) return;
    const content = buildResolvedContent();
    writeResolvedFile(file.path, content);
  }

  // Conflict navigation
  let conflictScrollContainer: HTMLDivElement | undefined = $state();
  let currentConflictIdx = $state(0);
  let conflictScrollLocked = false;

  // Reset index and scroll to first conflict when file changes
  $effect(() => {
    if ($selectedConflictFile) {
      currentConflictIdx = 0;
      tick().then(() => scrollToConflict(0));
    }
  });

  function scrollToConflict(idx: number) {
    const el = conflictScrollContainer?.querySelector(`[data-conflict-idx="${idx}"]`);
    if (el) {
      conflictScrollLocked = true;
      el.scrollIntoView({ behavior: 'smooth', block: 'start' });
      setTimeout(() => { conflictScrollLocked = false; }, 400);
    }
    currentConflictIdx = idx;
    // Keep preview in sync if open
    if (previewOpen && idx < previewRegions.length) {
      scrollToPreviewRegion(idx);
    }
  }

  let totalConflicts = $derived($selectedConflictFile?.conflict_sections.length ?? 0);

  // Update current conflict index based on scroll position
  function updateCurrentConflictFromScroll() {
    if (conflictScrollLocked) return;
    if (!conflictScrollContainer || totalConflicts === 0) return;
    const container = conflictScrollContainer;
    const scrollY = container.scrollTop;
    const containerHeight = container.clientHeight;
    // Find the conflict section closest to the top third of the viewport
    const threshold = scrollY + containerHeight / 3;
    let closest = 0;
    for (let i = 0; i < totalConflicts; i++) {
      const el = container.querySelector(`[data-conflict-idx="${i}"]`) as HTMLElement | null;
      if (el && el.offsetTop <= threshold) closest = i;
    }
    currentConflictIdx = closest;
  }

  function nextConflict() {
    if (!conflictScrollContainer || totalConflicts === 0) return;
    const next = currentConflictIdx + 1 < totalConflicts ? currentConflictIdx + 1 : 0;
    scrollToConflict(next);
  }

  function prevConflict() {
    if (!conflictScrollContainer || totalConflicts === 0) return;
    const prev = currentConflictIdx - 1 >= 0 ? currentConflictIdx - 1 : totalConflicts - 1;
    scrollToConflict(prev);
  }

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
    } else if (isConflictView) {
      // Skip stage/unstage shortcuts in conflict view
      return;
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

<svelte:window onkeydown={handleKeydown} onclick={closeDropdowns} />

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
  {:else if $selectedConflictFile}
    <!-- Conflict file header -->
    <div
      class="flex items-center justify-between px-4 py-2 border-b border-border bg-card shrink-0"
    >
      <div class="flex items-center gap-2 min-w-0">
        <span
          class="text-[10px] font-bold w-4 h-4 flex items-center justify-center rounded-sm shrink-0"
          style="color: var(--color-git-conflict); background: var(--color-git-conflict-bg);"
        >
          C
        </span>
        <span class="text-sm font-mono truncate">{$selectedConflictFile.path}</span>
        <span class="text-[10px] uppercase tracking-wide" style="color: var(--color-git-conflict);">conflicted</span>
      </div>
      <div class="flex items-center gap-2">
        {#if totalConflicts > 0}
          <div class="flex items-center gap-1 text-muted-foreground">
            <button
              class="p-1 hover:text-foreground transition-colors cursor-pointer"
              onclick={prevConflict}
              title="Previous conflict"
              aria-label="Previous conflict"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="18 15 12 9 6 15"></polyline>
              </svg>
            </button>
            <span class="text-[10px] tabular-nums">{currentConflictIdx + 1}/{totalConflicts}</span>
            <button
              class="p-1 hover:text-foreground transition-colors cursor-pointer"
              onclick={nextConflict}
              title="Next conflict"
              aria-label="Next conflict"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="6 9 12 15 18 9"></polyline>
              </svg>
            </button>
          </div>
        {/if}
        {#if bracketWarning}
          <span
            class="shrink-0"
            style="color: var(--color-git-modified);"
            title={bracketWarning}
          >
            <svg viewBox="0 0 16 16" width="14" height="14" fill="currentColor">
              <path d="M6.457 1.047c.659-1.234 2.427-1.234 3.086 0l6.082 11.378A1.75 1.75 0 0 1 14.082 15H1.918a1.75 1.75 0 0 1-1.543-2.575Zm1.763.707a.25.25 0 0 0-.44 0L1.698 13.132a.25.25 0 0 0 .22.368h12.164a.25.25 0 0 0 .22-.368Zm.53 3.996v2.5a.75.75 0 0 1-1.5 0v-2.5a.75.75 0 0 1 1.5 0ZM9 11a1 1 0 1 1-2 0 1 1 0 0 1 2 0Z" />
            </svg>
          </span>
        {/if}
        <button
          class="text-[11px] px-2 py-0.5 rounded transition-colors font-medium cursor-pointer"
          class:text-foreground={previewOpen}
          class:text-muted-foreground={!previewOpen}
          onclick={() => previewOpen = !previewOpen}
          title="Toggle resolved file preview"
        >
          Preview
        </button>
        <button
          onclick={handleMarkResolved}
          class="text-xs px-3 py-1 rounded font-medium transition-colors"
          style="background: var(--color-git-conflict); color: var(--background);"
          class:opacity-50={!allResolved}
          disabled={!allResolved}
          title={bracketWarning && allResolved ? bracketWarning : allResolved ? 'Write resolved content and stage file' : 'Resolve all conflict sections first'}
        >
          Mark as Resolved
        </button>
        <button
          onclick={handleClose}
          class="text-muted-foreground hover:text-foreground p-1 shrink-0 cursor-pointer"
          aria-label="Close file preview"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>
    </div>

    <!-- Conflict file content -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div bind:this={conflictScrollContainer} onscroll={updateCurrentConflictFromScroll} class="flex-1 overflow-auto min-h-0 font-mono leading-5" style="font-size: var(--editor-font-size)">
      {#if $selectedConflictFile.is_binary}
        <p class="text-muted-foreground text-sm text-center p-8">Binary file conflict — resolve externally or choose a version</p>
      {:else}
        {@const sections = $selectedConflictFile.conflict_sections}
        {@const lines = $selectedConflictFile.lines}
        {@const sectionStarts = new Set(sections.map(s => s.start_line - 1))}
        {#each lines as line, lineIdx}
          {@const sectionIdx = sections.findIndex(s => lineIdx >= s.start_line - 1 && lineIdx <= s.end_line - 1)}
          {#if sectionIdx >= 0}
            {@const section = sections[sectionIdx]}
            {@const isStart = lineIdx === section.start_line - 1}
            {#if isStart}
              {@const res = resolutions[sectionIdx]}
              <!-- Conflict section block -->
              <div class="border-y" style="border-color: var(--color-git-conflict);" data-conflict-idx={sectionIdx}>
                <!-- Action bar -->
                <div class="flex items-center gap-2 px-4 py-1.5" style="background: var(--color-git-conflict-bg);">
                  <span class="text-[11px] font-medium" style="color: var(--color-git-conflict);">
                    Conflict {sectionIdx + 1}/{sections.length}
                  </span>
                  <span class="flex-1"></span>
                  <!-- Accept Current with dropdown -->
                  <div class="relative">
                    <div class="flex items-center">
                      <button
                        class="text-[11px] px-2 py-0.5 rounded-l transition-colors font-medium hover:opacity-80 cursor-pointer"
                        style="color: var(--color-git-conflict-ours);"
                        onclick={() => { acceptCurrent(sectionIdx); closeDropdowns(); }}
                      >
                        Accept Current
                      </button>
                      <button
                        class="text-[11px] px-1 py-0.5 rounded-r transition-colors hover:opacity-80 cursor-pointer border-l"
                        style="color: var(--color-git-conflict-ours); border-color: color-mix(in srgb, var(--color-git-conflict-ours) 30%, transparent);"
                        onclick={(e) => toggleDropdown(`ours-${sectionIdx}`, e)}
                        aria-label="Accept all current"
                      >
                        <svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                          <polyline points="6 9 12 15 18 9"></polyline>
                        </svg>
                      </button>
                    </div>
                    {#if openDropdown === `ours-${sectionIdx}`}
                      <div class="absolute right-0 top-full mt-1 z-10 rounded shadow-lg border border-border bg-card py-1 whitespace-nowrap">
                        <button
                          class="text-[11px] px-3 py-1.5 w-full text-left hover:bg-muted/50 transition-colors cursor-pointer font-medium"
                          style="color: var(--color-git-conflict-ours);"
                          onclick={() => { acceptAllCurrent(); closeDropdowns(); }}
                        >
                          Accept All Current
                        </button>
                      </div>
                    {/if}
                  </div>
                  <!-- Accept Incoming with dropdown -->
                  <div class="relative">
                    <div class="flex items-center">
                      <button
                        class="text-[11px] px-2 py-0.5 rounded-l transition-colors font-medium hover:opacity-80 cursor-pointer"
                        style="color: var(--color-git-conflict-theirs);"
                        onclick={() => { acceptIncoming(sectionIdx); closeDropdowns(); }}
                      >
                        Accept Incoming
                      </button>
                      <button
                        class="text-[11px] px-1 py-0.5 rounded-r transition-colors hover:opacity-80 cursor-pointer border-l"
                        style="color: var(--color-git-conflict-theirs); border-color: color-mix(in srgb, var(--color-git-conflict-theirs) 30%, transparent);"
                        onclick={(e) => toggleDropdown(`theirs-${sectionIdx}`, e)}
                        aria-label="Accept all incoming"
                      >
                        <svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                          <polyline points="6 9 12 15 18 9"></polyline>
                        </svg>
                      </button>
                    </div>
                    {#if openDropdown === `theirs-${sectionIdx}`}
                      <div class="absolute right-0 top-full mt-1 z-10 rounded shadow-lg border border-border bg-card py-1 whitespace-nowrap">
                        <button
                          class="text-[11px] px-3 py-1.5 w-full text-left hover:bg-muted/50 transition-colors cursor-pointer font-medium"
                          style="color: var(--color-git-conflict-theirs);"
                          onclick={() => { acceptAllIncoming(); closeDropdowns(); }}
                        >
                          Accept All Incoming
                        </button>
                      </div>
                    {/if}
                  </div>
                  <button
                    class="text-[11px] px-2 py-0.5 rounded transition-colors font-medium hover:opacity-80 cursor-pointer"
                    style="color: var(--color-git-conflict);"
                    onclick={() => acceptBoth(sectionIdx)}
                  >
                    Accept Both
                  </button>
                </div>
                <!-- Current (ours) label + lines with checkboxes -->
                <div class="text-[10px] px-4 py-0.5 font-medium select-none" style="color: var(--color-git-conflict-ours); background: color-mix(in srgb, var(--color-git-conflict-ours) 8%, transparent);">
                  Current: {section.ours_label || 'HEAD'}
                </div>
                {#if section.ours.length === 0}
                  <div
                    class="flex items-center gap-2 px-4 py-1.5 text-[11px] italic select-none"
                    style="background: color-mix(in srgb, var(--color-git-conflict-ours) 8%, transparent); color: var(--color-git-conflict-ours); opacity: {res?.resolved ? 1 : 0.4};"
                  >
                    <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
                      {#if res?.resolved}
                        <path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.75.75 0 0 1 1.06-1.06L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z" />
                      {:else}
                        <path d="M8 2a6 6 0 1 0 0 12A6 6 0 0 0 8 2Z" opacity="0.15" />
                      {/if}
                    </svg>
                    (empty — no changes on this side)
                  </div>
                {:else}
                {#each section.ours as oursLine, oursIdx}
                  {@const included = res?.oursIncluded[oursIdx] ?? false}
                  <div
                    class="flex min-w-fit cursor-pointer transition-opacity"
                    style="background: color-mix(in srgb, var(--color-git-conflict-ours) 8%, transparent);"
                    class:opacity-40={!included}
                    onclick={() => toggleOursLine(sectionIdx, oursIdx)}
                    role="checkbox"
                    aria-checked={included}
                  >
                    <div class="w-1 shrink-0" style="background: var(--color-git-conflict-ours);"></div>
                    <div class="w-6 flex items-center justify-center shrink-0 select-none" style="color: var(--color-git-conflict-ours);">
                      <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
                        {#if included}
                          <path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.75.75 0 0 1 1.06-1.06L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z" />
                        {:else}
                          <path d="M8 2a6 6 0 1 0 0 12A6 6 0 0 0 8 2Z" opacity="0.15" />
                        {/if}
                      </svg>
                    </div>
                    <span class="whitespace-pre pr-4"
                      >{#each tokenizeLineCached(highlighter, oursLine, language) as token}<span
                          style:color={token.color}>{token.content}</span
                        >{/each}</span
                    >
                  </div>
                {/each}
                {/if}
                <!-- Incoming (theirs) label + lines with checkboxes -->
                <div class="text-[10px] px-4 py-0.5 font-medium select-none" style="color: var(--color-git-conflict-theirs); background: color-mix(in srgb, var(--color-git-conflict-theirs) 8%, transparent);">
                  Incoming: {section.theirs_label || 'incoming'}
                </div>
                {#if section.theirs.length === 0}
                  <div
                    class="flex items-center gap-2 px-4 py-1.5 text-[11px] italic select-none"
                    style="background: color-mix(in srgb, var(--color-git-conflict-theirs) 8%, transparent); color: var(--color-git-conflict-theirs); opacity: {res?.resolved ? 1 : 0.4};"
                  >
                    <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
                      {#if res?.resolved}
                        <path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.75.75 0 0 1 1.06-1.06L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z" />
                      {:else}
                        <path d="M8 2a6 6 0 1 0 0 12A6 6 0 0 0 8 2Z" opacity="0.15" />
                      {/if}
                    </svg>
                    (empty — no changes on this side)
                  </div>
                {:else}
                {#each section.theirs as theirsLine, theirsIdx}
                  {@const included = res?.theirsIncluded[theirsIdx] ?? false}
                  <div
                    class="flex min-w-fit cursor-pointer transition-opacity"
                    style="background: color-mix(in srgb, var(--color-git-conflict-theirs) 8%, transparent);"
                    class:opacity-40={!included}
                    onclick={() => toggleTheirsLine(sectionIdx, theirsIdx)}
                    role="checkbox"
                    aria-checked={included}
                  >
                    <div class="w-1 shrink-0" style="background: var(--color-git-conflict-theirs);"></div>
                    <div class="w-6 flex items-center justify-center shrink-0 select-none" style="color: var(--color-git-conflict-theirs);">
                      <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
                        {#if included}
                          <path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.75.75 0 0 1 1.06-1.06L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z" />
                        {:else}
                          <path d="M8 2a6 6 0 1 0 0 12A6 6 0 0 0 8 2Z" opacity="0.15" />
                        {/if}
                      </svg>
                    </div>
                    <span class="whitespace-pre pr-4"
                      >{#each tokenizeLineCached(highlighter, theirsLine, language) as token}<span
                          style:color={token.color}>{token.content}</span
                        >{/each}</span
                    >
                  </div>
                {/each}
                {/if}
              </div>
            {/if}
            <!-- Lines within the conflict marker range are rendered as part of the block above -->
          {:else}
            <!-- Normal (non-conflict) line -->
            <div class="flex min-w-fit">
              <div class="flex shrink-0 bg-card/50 border-r border-border/50 select-none">
                <span class="w-12 text-right pr-2 text-muted-foreground/30">
                  {lineIdx + 1}
                </span>
                <span class="w-6 text-center text-muted-foreground/30"> </span>
              </div>
              <span class="whitespace-pre pl-2 pr-4"
                >{#each tokenizeLineCached(highlighter, line, language) as token}<span
                    style:color={token.color}>{token.content}</span
                  >{/each}</span
              >
            </div>
          {/if}
        {/each}
      {/if}
    </div>
    <!-- Resolved preview pane -->
    {#if previewOpen && previewLineData.length > 0}
      <div class="border-t border-border flex flex-col" style="max-height: 40%; min-height: 120px;">
        <div class="flex items-center justify-between px-4 py-1.5 bg-card border-b border-border shrink-0">
          <div class="flex items-center gap-2">
            <span class="text-[11px] font-semibold text-muted-foreground uppercase tracking-wide">Resolved Preview</span>
            {#if bracketWarning}
              <span class="text-[11px] flex items-center gap-1" style="color: var(--color-git-modified);">
                <svg viewBox="0 0 16 16" width="10" height="10" fill="currentColor">
                  <path d="M6.457 1.047c.659-1.234 2.427-1.234 3.086 0l6.082 11.378A1.75 1.75 0 0 1 14.082 15H1.918a1.75 1.75 0 0 1-1.543-2.575Zm1.763.707a.25.25 0 0 0-.44 0L1.698 13.132a.25.25 0 0 0 .22.368h12.164a.25.25 0 0 0 .22-.368Zm.53 3.996v2.5a.75.75 0 0 1-1.5 0v-2.5a.75.75 0 0 1 1.5 0ZM9 11a1 1 0 1 1-2 0 1 1 0 0 1 2 0Z" />
                </svg>
                {bracketWarning}
              </span>
            {/if}
          </div>
          {#if previewRegions.length > 0}
            <div class="flex items-center gap-1 text-muted-foreground">
              <button
                class="p-1 hover:text-foreground transition-colors cursor-pointer"
                onclick={prevPreviewRegion}
                title="Previous change"
                aria-label="Previous change"
              >
                <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <polyline points="18 15 12 9 6 15"></polyline>
                </svg>
              </button>
              <span class="text-[10px] tabular-nums">{currentPreviewRegionIdx + 1}/{previewRegions.length}</span>
              <button
                class="p-1 hover:text-foreground transition-colors cursor-pointer"
                onclick={nextPreviewRegion}
                title="Next change"
                aria-label="Next change"
              >
                <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <polyline points="6 9 12 15 18 9"></polyline>
                </svg>
              </button>
            </div>
          {/if}
          <button
            class="text-muted-foreground hover:text-foreground p-1 cursor-pointer"
            onclick={() => previewOpen = false}
            aria-label="Close preview"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </button>
        </div>
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div bind:this={previewScrollContainer} onscroll={updateCurrentPreviewRegionFromScroll} class="flex-1 overflow-auto font-mono leading-5" style="font-size: var(--editor-font-size)">
          {#each previewLineData as pline, idx}
            {@const regionIdx = previewRegions.indexOf(idx)}
            <div
              class="flex min-w-fit"
              data-preview-region={regionIdx >= 0 ? regionIdx : undefined}
              style:background={pline.kind === 'ours'
                ? 'color-mix(in srgb, var(--color-git-conflict-ours) 10%, transparent)'
                : pline.kind === 'theirs'
                  ? 'color-mix(in srgb, var(--color-git-conflict-theirs) 10%, transparent)'
                  : undefined}
            >
              <div class="flex shrink-0 bg-card/50 border-r border-border/50 select-none">
                <span class="w-12 text-right pr-2 text-muted-foreground/30">
                  {idx + 1}
                </span>
                <span class="w-6 flex items-center justify-center">
                  {#if pline.kind !== 'context'}
                    <span
                      class="w-1 h-3 rounded-full"
                      style:background={pline.kind === 'ours' ? 'var(--color-git-conflict-ours)' : 'var(--color-git-conflict-theirs)'}
                    ></span>
                  {/if}
                </span>
              </div>
              <span class="whitespace-pre pl-2 pr-4"
                >{#each tokenizeLineCached(highlighter, pline.text, language) as token}<span
                    style:color={token.color}>{token.content}</span
                  >{/each}</span
              >
            </div>
          {/each}
        </div>
      </div>
    {/if}
  {:else if $selectedFile}
    <!-- Loading state -->
    <div class="flex items-center justify-center flex-1">
      <p class="text-muted-foreground text-sm">Loading diff...</p>
    </div>
  {/if}
</div>
