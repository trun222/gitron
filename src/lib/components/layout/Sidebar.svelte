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
  } from '$lib/stores/repo';
  import type { FileSection } from '$lib/stores/repo';
  import { sidebarCollapsed, toggleSidebar, showTagsList } from '$lib/stores/settings';
  import {
    aiGenerating,
    aiError,
    hasConfiguredProvider,
    generateCommitMessage,
    initAI,
  } from '$lib/stores/ai';

  let commitTitle = $state('');
  let commitBody = $state('');
  let commitError = $state<string | null>(null);
  let committing = $state(false);
  let tagsExpanded = $state(true);

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
      <span class="text-xs font-medium text-foreground flex-1">Changes</span>
      {#if totalChanges > 0}
        <span class="{bubbleColor} text-[10px] px-1.5 rounded-full min-w-[18px] text-center">
          {totalChanges}
        </span>
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
