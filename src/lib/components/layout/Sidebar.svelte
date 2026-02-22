<script lang="ts">
  import {
    hasRepo,
    repoStatus,
    stagedCount,
    unstagedCount,
    repoPath,
    selectedFile,
    selectFile,
    stageFile,
    unstageFile,
    stageUnstagedAndClear,
    stageUntrackedAndClear,
    unstageAllAndClear,
    commitAndRefresh,
    discardConfirmOpen,
    commitGraph,
    jumpToTag,
    remoteTagNames,
    addToGitignore,
  } from '$lib/stores/repo';
  import type { FileSection } from '$lib/stores/repo';
  import { sidebarCollapsed, toggleSidebar, showTagsList, changesViewMode, setChangesViewMode } from '$lib/stores/settings';
  import {
    aiGenerating,
    aiError,
    hasConfiguredProvider,
    generateCommitMessage,
    initAI,
  } from '$lib/stores/ai';

  import { buildTree, flattenTree, collectDirPaths } from './changes-tree';

  let commitTitle = $state('');
  let commitBody = $state('');
  let commitError = $state<string | null>(null);
  let committing = $state(false);
  let tagsExpanded = $state(true);
  let expandedDirs = $state(new Set<string>());

  // --- Context menu ---
  interface ContextMenuItem {
    id: string;
    label: string;
  }
  interface ContextMenuState {
    x: number;
    y: number;
    items: ContextMenuItem[];
  }

  let contextMenu: ContextMenuState | null = $state(null);

  function closeContextMenu() {
    contextMenu = null;
  }

  function buildFileMenuItems(filePath: string): ContextMenuItem[] {
    const fileName = filePath.split('/').pop() ?? filePath;
    const items: ContextMenuItem[] = [
      { id: `ignore-file:${filePath}`, label: `Add ${fileName} to .gitignore` },
    ];
    const dotIdx = fileName.lastIndexOf('.');
    if (dotIdx > 0) {
      const ext = fileName.slice(dotIdx + 1);
      items.push({ id: `ignore-ext:*.${ext}`, label: `Add *.${ext} to .gitignore` });
    }
    return items;
  }

  function handleFileContextMenu(e: MouseEvent, filePath: string) {
    e.preventDefault();
    e.stopPropagation();
    contextMenu = {
      x: e.clientX,
      y: e.clientY,
      items: buildFileMenuItems(filePath),
    };
  }

  function handleDirContextMenu(e: MouseEvent, dirPath: string) {
    e.preventDefault();
    e.stopPropagation();
    const dirName = dirPath.endsWith('/') ? dirPath : dirPath + '/';
    contextMenu = {
      x: e.clientX,
      y: e.clientY,
      items: [
        { id: `ignore-dir:${dirName}`, label: `Add ${dirName} to .gitignore` },
      ],
    };
  }

  function executeMenuAction(actionId: string) {
    closeContextMenu();
    const colonIdx = actionId.indexOf(':');
    if (colonIdx < 0) return;
    const pattern = actionId.slice(colonIdx + 1);
    addToGitignore(pattern);
  }

  // Load AI state on mount
  $effect(() => {
    initAI();
  });

  async function handleAIGenerate() {
    if ($aiGenerating) return;
    const result = await generateCommitMessage();
    if (result) {
      commitTitle = result.title;
      commitBody = result.body;
    }
  }

  function handleFileClick(path: string, section: FileSection) {
    selectFile(path, section);
  }

  function handleStageClick(e: MouseEvent, filePath: string) {
    e.stopPropagation();
    const path = $repoPath;
    if (path) stageFile(path, filePath);
  }

  function handleUnstageClick(e: MouseEvent, filePath: string) {
    e.stopPropagation();
    const path = $repoPath;
    if (path) unstageFile(path, filePath);
  }

  function isSelected(path: string, section: FileSection): boolean {
    return $selectedFile?.path === path && $selectedFile?.section === section;
  }

  async function handleCommit() {
    if (!commitTitle.trim() || $stagedCount === 0 || committing) return;
    const message = commitBody.trim()
      ? `${commitTitle.trim()}\n\n${commitBody.trim()}`
      : commitTitle.trim();
    committing = true;
    commitError = null;
    const oid = await commitAndRefresh(message);
    committing = false;
    if (oid) {
      commitTitle = '';
      commitBody = '';
    } else {
      commitError = 'Commit failed. Check your git config (user.name / user.email).';
    }
  }

  function handleCommitKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
      e.preventDefault();
      handleCommit();
    }
  }

  const bubbleColor = 'bg-primary text-primary-foreground';
  const totalChanges = $derived($stagedCount + $unstagedCount);

  // Raw trees — pure derivation, no side effects
  let rawTrees = $derived.by(() => {
    if ($changesViewMode !== 'tree' || !$repoStatus) return null;
    const staged = $repoStatus.staged.map((f) => ({ path: f.path, status: f.status, section: 'staged' as const }));
    const unstaged = $repoStatus.unstaged.map((f) => ({ path: f.path, status: f.status, section: 'unstaged' as const }));
    const untracked = $repoStatus.untracked.map((p) => ({ path: p, section: 'untracked' as const }));
    return {
      staged: buildTree(staged),
      unstaged: buildTree(unstaged),
      untracked: buildTree(untracked),
    };
  });

  // Auto-expand directories we haven't seen yet
  let knownDirs = new Set<string>();
  $effect(() => {
    if (!rawTrees) return;
    const allDirs = [
      ...collectDirPaths(rawTrees.staged),
      ...collectDirPaths(rawTrees.unstaged),
      ...collectDirPaths(rawTrees.untracked),
    ];
    let added = false;
    const next = new Set(expandedDirs);
    for (const d of allDirs) {
      if (!knownDirs.has(d)) {
        knownDirs.add(d);
        next.add(d);
        added = true;
      }
    }
    if (added) {
      expandedDirs = next;
    }
  });

  // Flattened tree for rendering — reads expandedDirs without mutating it
  let treeSections = $derived.by(() => {
    if (!rawTrees) return null;
    return {
      staged: flattenTree(rawTrees.staged, expandedDirs),
      unstaged: flattenTree(rawTrees.unstaged, expandedDirs),
      untracked: flattenTree(rawTrees.untracked, expandedDirs),
    };
  });

  function toggleDir(path: string) {
    const next = new Set(expandedDirs);
    if (next.has(path)) {
      next.delete(path);
    } else {
      next.add(path);
    }
    expandedDirs = next;
  }


  // Tags sorted by commit position in graph (newest first)
  let sortedTags = $derived.by(() => {
    const graph = $commitGraph;
    if (!graph || graph.tags.length === 0) return [];
    const oidIndex = new Map(graph.commits.map((c, i) => [c.oid, i]));
    return [...graph.tags].sort((a, b) => {
      const ai = oidIndex.get(a.target_oid) ?? Infinity;
      const bi = oidIndex.get(b.target_oid) ?? Infinity;
      return ai - bi;
    });
  });
</script>

<aside
  class="bg-card border-r border-border flex flex-col overflow-hidden transition-[width] duration-200 ease-in-out"
  class:w-[260px]={!$sidebarCollapsed}
  class:min-w-[200px]={!$sidebarCollapsed}
  class:w-10={$sidebarCollapsed}
  class:min-w-10={$sidebarCollapsed}
>
  {#if $sidebarCollapsed}
    <!-- Collapsed strip -->
    <div class="flex flex-col items-center py-2 gap-2 h-full">
      <button
        class="w-7 h-7 flex items-center justify-center rounded-md text-muted-foreground hover:text-foreground hover:bg-accent transition-colors cursor-pointer"
        onclick={toggleSidebar}
        aria-label="Expand sidebar"
        title="Expand sidebar"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 18 15 12 9 6"></polyline></svg>
      </button>
      {#if totalChanges > 0}
        <span class="{bubbleColor} text-[10px] w-5 h-5 rounded-full flex items-center justify-center" title="{totalChanges} changes">
          {totalChanges}
        </span>
      {/if}
    </div>
  {:else if $hasRepo}
    <div class="flex items-center gap-1.5 px-3 py-2 border-b border-border">
      <span class="text-xs font-medium text-foreground">Changes</span>
      {#if totalChanges > 0}
        <span class="{bubbleColor} text-[10px] px-1.5 rounded-full min-w-[18px] text-center">
          {totalChanges}
        </span>
      {/if}
      <span class="flex-1"></span>
      <!-- View mode toggles -->
      <button
        class="w-5 h-5 flex items-center justify-center rounded transition-colors cursor-pointer {$changesViewMode === 'file' ? 'text-foreground' : 'text-muted-foreground hover:text-foreground hover:bg-accent'}"
        onclick={() => setChangesViewMode('file')}
        aria-label="File view"
        title="File view"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="8" y1="6" x2="21" y2="6"></line><line x1="8" y1="12" x2="21" y2="12"></line><line x1="8" y1="18" x2="21" y2="18"></line><line x1="3" y1="6" x2="3.01" y2="6"></line><line x1="3" y1="12" x2="3.01" y2="12"></line><line x1="3" y1="18" x2="3.01" y2="18"></line></svg>
      </button>
      <button
        class="w-5 h-5 flex items-center justify-center rounded transition-colors cursor-pointer {$changesViewMode === 'tree' ? 'text-foreground' : 'text-muted-foreground hover:text-foreground hover:bg-accent'}"
        onclick={() => setChangesViewMode('tree')}
        aria-label="Tree view"
        title="Tree view"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg>
      </button>
      {#if totalChanges > 0}
        <button
          class="w-5 h-5 flex items-center justify-center rounded text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-colors cursor-pointer"
          onclick={() => discardConfirmOpen.set(true)}
          aria-label="Discard all changes"
          title="Discard all changes"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>
        </button>
      {/if}
      <button
        class="w-5 h-5 flex items-center justify-center rounded text-muted-foreground hover:text-foreground hover:bg-accent transition-colors cursor-pointer"
        onclick={toggleSidebar}
        aria-label="Collapse sidebar"
        title="Collapse sidebar"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="15 18 9 12 15 6"></polyline></svg>
      </button>
    </div>

    <div
      class="flex-1 overflow-y-auto py-2"
      role="listbox"
      aria-label="Changed files"
    >
        {#if $repoStatus}
          {#if $changesViewMode === 'tree' && treeSections}
            <!-- TREE VIEW -->
            <!-- STAGED (tree) -->
            {#if $repoStatus.staged.length > 0}
              <div class="flex items-center justify-between px-3 py-1.5">
                <span class="text-[11px] font-semibold text-git-added uppercase tracking-wide">
                  Staged ({$repoStatus.staged.length})
                </span>
                <button
                  class="text-[10px] text-muted-foreground hover:text-foreground cursor-pointer"
                  onclick={() => unstageAllAndClear()}
                >
                  Unstage All
                </button>
              </div>
              <ul class="list-none">
                {#each treeSections.staged as entry (entry.path + ':' + entry.type)}
                  {#if entry.type === 'dir'}
                    <li
                      class="flex items-center gap-1.5 px-3 py-1 text-xs cursor-pointer hover:bg-accent/50 transition-colors text-muted-foreground"
                      style="padding-left: {12 + entry.depth * 12}px"
                      onclick={() => toggleDir(entry.path)}
                      oncontextmenu={(e) => handleDirContextMenu(e, entry.path)}
                    >
                      <svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="shrink-0 transition-transform duration-100" style="transform: rotate({entry.expanded ? '90deg' : '0deg'})"><polyline points="9 18 15 12 9 6"></polyline></svg>
                      <svg xmlns="http://www.w3.org/2000/svg" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="shrink-0"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg>
                      <span class="truncate">{entry.name}</span>
                    </li>
                  {:else}
                    <li
                      class="group flex items-center gap-2 py-1 text-xs cursor-pointer hover:bg-accent transition-colors {isSelected(entry.path, 'staged') ? 'bg-accent ring-1 ring-primary/30' : ''}"
                      style="padding-left: {12 + entry.depth * 12}px; padding-right: 12px"
                      onclick={() => handleFileClick(entry.path, 'staged')}
                      oncontextmenu={(e) => handleFileContextMenu(e, entry.path)}
                      role="option"
                      aria-selected={isSelected(entry.path, 'staged')}
                    >
                      <span class="text-[10px] font-bold w-4 h-4 flex items-center justify-center rounded-sm shrink-0 text-[var(--color-git-added)] bg-[var(--color-git-added-bg)]">
                        {entry.fileStatus?.[0] ?? '?'}
                      </span>
                      <span class="truncate text-foreground flex-1">{entry.name}</span>
                      <button
                        class="opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-foreground shrink-0 cursor-pointer text-sm leading-none"
                        onclick={(e) => handleUnstageClick(e, entry.path)}
                        aria-label="Unstage {entry.path}"
                      >
                        −
                      </button>
                    </li>
                  {/if}
                {/each}
              </ul>
            {/if}

            <!-- UNSTAGED (tree) -->
            {#if $repoStatus.unstaged.length > 0}
              <div class="flex items-center justify-between px-3 py-1.5">
                <span class="text-[11px] font-semibold text-git-modified uppercase tracking-wide">
                  Unstaged ({$repoStatus.unstaged.length})
                </span>
                <button
                  class="text-[10px] text-muted-foreground hover:text-foreground cursor-pointer"
                  onclick={() => stageUnstagedAndClear()}
                >
                  Stage All
                </button>
              </div>
              <ul class="list-none">
                {#each treeSections.unstaged as entry (entry.path + ':' + entry.type)}
                  {#if entry.type === 'dir'}
                    <li
                      class="flex items-center gap-1.5 px-3 py-1 text-xs cursor-pointer hover:bg-accent/50 transition-colors text-muted-foreground"
                      style="padding-left: {12 + entry.depth * 12}px"
                      onclick={() => toggleDir(entry.path)}
                      oncontextmenu={(e) => handleDirContextMenu(e, entry.path)}
                    >
                      <svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="shrink-0 transition-transform duration-100" style="transform: rotate({entry.expanded ? '90deg' : '0deg'})"><polyline points="9 18 15 12 9 6"></polyline></svg>
                      <svg xmlns="http://www.w3.org/2000/svg" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="shrink-0"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg>
                      <span class="truncate">{entry.name}</span>
                    </li>
                  {:else}
                    <li
                      class="group flex items-center gap-2 py-1 text-xs cursor-pointer hover:bg-accent transition-colors {isSelected(entry.path, 'unstaged') ? 'bg-accent ring-1 ring-primary/30' : ''}"
                      style="padding-left: {12 + entry.depth * 12}px; padding-right: 12px"
                      onclick={() => handleFileClick(entry.path, 'unstaged')}
                      oncontextmenu={(e) => handleFileContextMenu(e, entry.path)}
                      role="option"
                      aria-selected={isSelected(entry.path, 'unstaged')}
                    >
                      <span class="text-[10px] font-bold w-4 h-4 flex items-center justify-center rounded-sm shrink-0 text-[var(--color-git-modified)] bg-[var(--color-git-modified-bg)]">
                        {entry.fileStatus?.[0] ?? '?'}
                      </span>
                      <span class="truncate text-foreground flex-1">{entry.name}</span>
                      <button
                        class="opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-foreground shrink-0 cursor-pointer text-sm leading-none"
                        onclick={(e) => handleStageClick(e, entry.path)}
                        aria-label="Stage {entry.path}"
                      >
                        +
                      </button>
                    </li>
                  {/if}
                {/each}
              </ul>
            {/if}

            <!-- UNTRACKED (tree) -->
            {#if $repoStatus.untracked.length > 0}
              <div class="flex items-center justify-between px-3 py-1.5">
                <span class="text-[11px] font-semibold text-muted-foreground uppercase tracking-wide">
                  Untracked ({$repoStatus.untracked.length})
                </span>
                <button
                  class="text-[10px] text-muted-foreground hover:text-foreground cursor-pointer"
                  onclick={() => stageUntrackedAndClear()}
                >
                  Stage All
                </button>
              </div>
              <ul class="list-none">
                {#each treeSections.untracked as entry (entry.path + ':' + entry.type)}
                  {#if entry.type === 'dir'}
                    <li
                      class="flex items-center gap-1.5 px-3 py-1 text-xs cursor-pointer hover:bg-accent/50 transition-colors text-muted-foreground"
                      style="padding-left: {12 + entry.depth * 12}px"
                      onclick={() => toggleDir(entry.path)}
                      oncontextmenu={(e) => handleDirContextMenu(e, entry.path)}
                    >
                      <svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="shrink-0 transition-transform duration-100" style="transform: rotate({entry.expanded ? '90deg' : '0deg'})"><polyline points="9 18 15 12 9 6"></polyline></svg>
                      <svg xmlns="http://www.w3.org/2000/svg" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="shrink-0"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg>
                      <span class="truncate">{entry.name}</span>
                    </li>
                  {:else}
                    <li
                      class="group flex items-center gap-2 py-1 text-xs cursor-pointer hover:bg-accent transition-colors {isSelected(entry.path, 'untracked') ? 'bg-accent ring-1 ring-primary/30' : ''}"
                      style="padding-left: {12 + entry.depth * 12}px; padding-right: 12px"
                      onclick={() => handleFileClick(entry.path, 'untracked')}
                      oncontextmenu={(e) => handleFileContextMenu(e, entry.path)}
                      role="option"
                      aria-selected={isSelected(entry.path, 'untracked')}
                    >
                      <span class="text-[10px] font-bold w-4 h-4 flex items-center justify-center rounded-sm shrink-0 text-muted-foreground bg-accent">
                        ?
                      </span>
                      <span class="truncate text-foreground flex-1">{entry.name}</span>
                      <button
                        class="opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-foreground shrink-0 cursor-pointer text-sm leading-none"
                        onclick={(e) => handleStageClick(e, entry.path)}
                        aria-label="Stage {entry.path}"
                      >
                        +
                      </button>
                    </li>
                  {/if}
                {/each}
              </ul>
            {/if}

            {#if $repoStatus.staged.length === 0 && $repoStatus.unstaged.length === 0 && $repoStatus.untracked.length === 0}
              <p class="text-muted-foreground text-sm text-center p-4">Working tree clean</p>
            {/if}
          {:else}
            <!-- FLAT FILE VIEW -->
            <!-- STAGED -->
            {#if $repoStatus.staged.length > 0}
              <div class="flex items-center justify-between px-3 py-1.5">
                <span class="text-[11px] font-semibold text-git-added uppercase tracking-wide">
                  Staged ({$repoStatus.staged.length})
                </span>
                <button
                  class="text-[10px] text-muted-foreground hover:text-foreground cursor-pointer"
                  onclick={() => unstageAllAndClear()}
                >
                  Unstage All
                </button>
              </div>
              <ul class="list-none">
                {#each $repoStatus.staged as file}
                  <li
                    class="group flex items-center gap-2 px-3 py-1 text-xs cursor-pointer hover:bg-accent transition-colors {isSelected(file.path, 'staged') ? 'bg-accent ring-1 ring-primary/30' : ''}"
                    onclick={() => handleFileClick(file.path, 'staged')}
                    oncontextmenu={(e) => handleFileContextMenu(e, file.path)}
                    role="option"
                    aria-selected={isSelected(file.path, 'staged')}
                  >
                    <span class="text-[10px] font-bold w-4 h-4 flex items-center justify-center rounded-sm shrink-0 text-[var(--color-git-added)] bg-[var(--color-git-added-bg)]">
                      {file.status[0]}
                    </span>
                    <span class="truncate text-foreground flex-1">{file.path}</span>
                    <button
                      class="opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-foreground shrink-0 cursor-pointer text-sm leading-none"
                      onclick={(e) => handleUnstageClick(e, file.path)}
                      aria-label="Unstage {file.path}"
                    >
                      −
                    </button>
                  </li>
                {/each}
              </ul>
            {/if}

            <!-- UNSTAGED -->
            {#if $repoStatus.unstaged.length > 0}
              <div class="flex items-center justify-between px-3 py-1.5">
                <span class="text-[11px] font-semibold text-git-modified uppercase tracking-wide">
                  Unstaged ({$repoStatus.unstaged.length})
                </span>
                <button
                  class="text-[10px] text-muted-foreground hover:text-foreground cursor-pointer"
                  onclick={() => stageUnstagedAndClear()}
                >
                  Stage All
                </button>
              </div>
              <ul class="list-none">
                {#each $repoStatus.unstaged as file}
                  <li
                    class="group flex items-center gap-2 px-3 py-1 text-xs cursor-pointer hover:bg-accent transition-colors {isSelected(file.path, 'unstaged') ? 'bg-accent ring-1 ring-primary/30' : ''}"
                    onclick={() => handleFileClick(file.path, 'unstaged')}
                    oncontextmenu={(e) => handleFileContextMenu(e, file.path)}
                    role="option"
                    aria-selected={isSelected(file.path, 'unstaged')}
                  >
                    <span class="text-[10px] font-bold w-4 h-4 flex items-center justify-center rounded-sm shrink-0 text-[var(--color-git-modified)] bg-[var(--color-git-modified-bg)]">
                      {file.status[0]}
                    </span>
                    <span class="truncate text-foreground flex-1">{file.path}</span>
                    <button
                      class="opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-foreground shrink-0 cursor-pointer text-sm leading-none"
                      onclick={(e) => handleStageClick(e, file.path)}
                      aria-label="Stage {file.path}"
                    >
                      +
                    </button>
                  </li>
                {/each}
              </ul>
            {/if}

            <!-- UNTRACKED -->
            {#if $repoStatus.untracked.length > 0}
              <div class="flex items-center justify-between px-3 py-1.5">
                <span class="text-[11px] font-semibold text-muted-foreground uppercase tracking-wide">
                  Untracked ({$repoStatus.untracked.length})
                </span>
                <button
                  class="text-[10px] text-muted-foreground hover:text-foreground cursor-pointer"
                  onclick={() => stageUntrackedAndClear()}
                >
                  Stage All
                </button>
              </div>
              <ul class="list-none">
                {#each $repoStatus.untracked as file}
                  <li
                    class="group flex items-center gap-2 px-3 py-1 text-xs cursor-pointer hover:bg-accent transition-colors {isSelected(file, 'untracked') ? 'bg-accent ring-1 ring-primary/30' : ''}"
                    onclick={() => handleFileClick(file, 'untracked')}
                    oncontextmenu={(e) => handleFileContextMenu(e, file)}
                    role="option"
                    aria-selected={isSelected(file, 'untracked')}
                  >
                    <span class="text-[10px] font-bold w-4 h-4 flex items-center justify-center rounded-sm shrink-0 text-muted-foreground bg-accent">
                      ?
                    </span>
                    <span class="truncate text-foreground flex-1">{file}</span>
                    <button
                      class="opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-foreground shrink-0 cursor-pointer text-sm leading-none"
                      onclick={(e) => handleStageClick(e, file)}
                      aria-label="Stage {file}"
                    >
                      +
                    </button>
                  </li>
                {/each}
              </ul>
            {/if}

            {#if $repoStatus.staged.length === 0 && $repoStatus.unstaged.length === 0 && $repoStatus.untracked.length === 0}
              <p class="text-muted-foreground text-sm text-center p-4">Working tree clean</p>
            {/if}
          {/if}
        {/if}
    </div>

    <!-- TAGS LIST (bottom-docked) -->
    {#if $showTagsList && $commitGraph && $commitGraph.tags.length > 0}
      <div class="border-t border-border">
        <button
          class="flex items-center justify-between w-full px-3 py-1.5 cursor-pointer hover:bg-accent/50 transition-colors"
          onclick={() => tagsExpanded = !tagsExpanded}
          aria-expanded={tagsExpanded}
        >
          <span class="text-[11px] font-semibold text-muted-foreground uppercase tracking-wide">
            Tags ({$commitGraph.tags.length})
          </span>
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="12"
            height="12"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class="text-muted-foreground transition-transform duration-150"
            style="transform: rotate({tagsExpanded ? '90deg' : '0deg'})"
          >
            <polyline points="9 18 15 12 9 6"></polyline>
          </svg>
        </button>
        {#if tagsExpanded}
          <ul class="list-none overflow-y-auto max-h-[40vh]">
            {#each sortedTags as tag}
              <li>
                <button
                  class="flex items-center gap-2 w-full px-3 py-1 text-xs cursor-pointer hover:bg-accent transition-colors text-left"
                  onclick={() => jumpToTag(tag.target_oid)}
                  title={tag.message ? `${tag.name}\n${tag.message}` : tag.name}
                >
                  <svg class="w-3 h-3 shrink-0 text-muted-foreground" viewBox="0 0 16 16" fill="currentColor"><path d="M1 7.775V2.75C1 1.784 1.784 1 2.75 1h5.025c.464 0 .91.184 1.238.513l6.25 6.25a1.75 1.75 0 0 1 0 2.474l-5.026 5.026a1.75 1.75 0 0 1-2.474 0l-6.25-6.25A1.752 1.752 0 0 1 1 7.775Zm1.5 0c0 .066.026.13.073.177l6.25 6.25a.25.25 0 0 0 .354 0l5.025-5.025a.25.25 0 0 0 0-.354l-6.25-6.25a.25.25 0 0 0-.177-.073H2.75a.25.25 0 0 0-.25.25ZM6 5a1 1 0 1 1 0 2 1 1 0 0 1 0-2Z"/></svg>
                  <span class="truncate text-foreground flex-1">{tag.name}</span>
                  {#if $remoteTagNames.has(tag.name)}
                    <span title="On remote">
                      <svg class="w-3 h-3 shrink-0 text-muted-foreground/60" viewBox="0 0 16 16" fill="currentColor"><path d="M4.5 11a.5.5 0 0 1 .5-.5h6a.5.5 0 0 1 0 1H5a.5.5 0 0 1-.5-.5Zm-.4-3.8A3.5 3.5 0 0 1 11 5.5a.5.5 0 0 0 .5.5 2.5 2.5 0 0 1 0 5h-7a3 3 0 0 1-.4-5.8ZM8 3a4.5 4.5 0 0 0-4.38 3.48A4 4 0 0 0 4.5 14h7a3.5 3.5 0 0 0 .83-6.9A4.49 4.49 0 0 0 8 3Z"/></svg>
                    </span>
                  {/if}
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      </div>
    {/if}

    <!-- Commit Panel -->
    {#if $stagedCount > 0}
      <div class="border-t border-border px-3 py-2 flex flex-col gap-1.5">
        <div class="flex items-center gap-1">
          <input
            type="text"
            class="flex-1 min-w-0 bg-input text-foreground text-xs rounded-md border border-border px-2 py-1.5 placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-primary"
            placeholder="Commit title"
            bind:value={commitTitle}
            onkeydown={handleCommitKeydown}
            disabled={committing || $aiGenerating}
          />
          {#if $hasConfiguredProvider}
            <button
              class="w-7 h-7 flex items-center justify-center rounded-md transition-colors cursor-pointer shrink-0 {$aiGenerating ? 'text-primary animate-pulse' : 'text-muted-foreground hover:text-primary hover:bg-accent'}"
              onclick={handleAIGenerate}
              disabled={$aiGenerating || committing}
              aria-label="Generate commit message with AI"
              title="Generate commit message with AI"
            >
              {#if $aiGenerating}
                <svg class="animate-spin" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12a9 9 0 1 1-6.219-8.56" /></svg>
              {:else}
                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
                  <path d="M7.657 6.247c.11-.33.576-.33.686 0l.645 1.937a2.89 2.89 0 0 0 1.829 1.828l1.936.645c.33.11.33.576 0 .686l-1.937.645a2.89 2.89 0 0 0-1.828 1.829l-.645 1.936a.361.361 0 0 1-.686 0l-.645-1.937a2.89 2.89 0 0 0-1.828-1.828l-1.937-.645a.361.361 0 0 1 0-.686l1.937-.645a2.89 2.89 0 0 0 1.828-1.829l.645-1.936ZM3.794 1.148a.217.217 0 0 1 .412 0l.387 1.162c.173.518.579.924 1.097 1.097l1.162.387a.217.217 0 0 1 0 .412l-1.162.387A1.73 1.73 0 0 0 4.593 5.69l-.387 1.162a.217.217 0 0 1-.412 0L3.407 5.69a1.73 1.73 0 0 0-1.097-1.097l-1.162-.387a.217.217 0 0 1 0-.412l1.162-.387A1.73 1.73 0 0 0 3.407 2.31l.387-1.162Z" />
                </svg>
              {/if}
            </button>
          {/if}
        </div>
        <textarea
          class="w-full bg-input text-foreground text-xs rounded-md border border-border px-2 py-1.5 resize-none placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-primary"
          rows="3"
          placeholder="Description (optional)"
          bind:value={commitBody}
          onkeydown={handleCommitKeydown}
          disabled={committing || $aiGenerating}
        ></textarea>
        <button
          class="w-full text-xs font-medium py-1.5 rounded-md transition-colors cursor-pointer {commitTitle.trim() && !committing ? 'bg-primary text-primary-foreground hover:bg-primary/90' : 'bg-muted text-muted-foreground cursor-not-allowed'}"
          onclick={handleCommit}
          disabled={!commitTitle.trim() || committing}
        >
          {committing ? 'Committing...' : `Commit (${$stagedCount})`}
        </button>
        {#if commitError}
          <p class="text-[11px] text-destructive">{commitError}</p>
        {/if}
        {#if $aiError}
          <p class="text-[11px] text-destructive">{$aiError}</p>
        {/if}
      </div>
    {/if}
  {:else}
    <div class="flex items-center justify-center h-full">
      <p class="text-muted-foreground text-sm text-center p-4">Open a repository to get started</p>
    </div>
  {/if}
</aside>

<!-- Context menu overlay -->
{#if contextMenu}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="fixed inset-0 z-40" onclick={closeContextMenu} oncontextmenu={(e) => { e.preventDefault(); closeContextMenu(); }}></div>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed z-50 min-w-[180px] rounded-lg border border-border bg-popover shadow-lg py-1"
    style="left: {contextMenu.x}px; top: {contextMenu.y}px"
    onclick={(e) => e.stopPropagation()}
  >
    {#each contextMenu.items as item}
      <button
        type="button"
        class="context-menu-item w-full text-left px-3 py-1.5 text-sm outline-none transition-colors text-popover-foreground hover:bg-accent cursor-pointer"
        onclick={() => executeMenuAction(item.id)}
      >
        {item.label}
      </button>
    {/each}
  </div>
{/if}
