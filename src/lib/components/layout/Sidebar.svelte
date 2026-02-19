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
    remotes,
    networkOperation,
    fetchFromRemote,
    removeRemote,
  } from '$lib/stores/repo';
  import type { FileSection } from '$lib/stores/repo';
  import { sidebarCollapsed, toggleSidebar } from '$lib/stores/settings';

  let commitTitle = $state('');
  let commitBody = $state('');
  let commitError = $state<string | null>(null);
  let committing = $state(false);

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
  let remotesExpanded = $state(true);
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

        <!-- REMOTES -->
        {#if $remotes.length > 0}
          <div class="border-t border-border mt-2 pt-1">
            <button
              class="flex items-center gap-1 w-full px-3 py-1.5 text-[11px] font-semibold text-muted-foreground uppercase tracking-wide hover:text-foreground transition-colors cursor-pointer"
              onclick={() => remotesExpanded = !remotesExpanded}
            >
              <svg
                xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24"
                fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
                class="transition-transform shrink-0"
                class:rotate-90={remotesExpanded}
              >
                <polyline points="9 18 15 12 9 6"></polyline>
              </svg>
              Remotes ({$remotes.length})
            </button>
            {#if remotesExpanded}
              <ul class="list-none">
                {#each $remotes as remote (remote.name)}
                  <li class="group flex items-center gap-2 px-3 py-1 text-xs hover:bg-accent transition-colors">
                    <div class="flex flex-col min-w-0 flex-1">
                      <span class="font-medium text-foreground truncate">{remote.name}</span>
                      <span class="text-[10px] text-muted-foreground truncate">{remote.url}</span>
                    </div>
                    <button
                      class="opacity-0 group-hover:opacity-100 w-5 h-5 flex items-center justify-center rounded text-muted-foreground hover:text-foreground hover:bg-accent transition-colors cursor-pointer shrink-0"
                      onclick={() => fetchFromRemote(remote.name)}
                      disabled={!!$networkOperation}
                      aria-label="Fetch {remote.name}"
                      title="Fetch {remote.name}"
                    >
                      <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="8 17 12 21 16 17"></polyline><line x1="12" y1="12" x2="12" y2="21"></line><path d="M20.88 18.09A5 5 0 0 0 18 9h-1.26A8 8 0 1 0 3 16.29"></path></svg>
                    </button>
                    <button
                      class="opacity-0 group-hover:opacity-100 w-5 h-5 flex items-center justify-center rounded text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-colors cursor-pointer shrink-0"
                      onclick={() => removeRemote(remote.name)}
                      aria-label="Remove {remote.name}"
                      title="Remove {remote.name}"
                    >
                      <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
                    </button>
                  </li>
                {/each}
              </ul>
            {/if}
          </div>
        {/if}
    </div>

    <!-- Commit Panel -->
    {#if $stagedCount > 0}
      <div class="border-t border-border px-3 py-2 flex flex-col gap-1.5">
        <input
          type="text"
          class="w-full bg-input text-foreground text-xs rounded-md border border-border px-2 py-1.5 placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-primary"
          placeholder="Commit title"
          bind:value={commitTitle}
          onkeydown={handleCommitKeydown}
          disabled={committing}
        />
        <textarea
          class="w-full bg-input text-foreground text-xs rounded-md border border-border px-2 py-1.5 resize-none placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-primary"
          rows="3"
          placeholder="Description (optional)"
          bind:value={commitBody}
          onkeydown={handleCommitKeydown}
          disabled={committing}
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
      </div>
    {/if}
  {:else}
    <div class="flex items-center justify-center h-full">
      <p class="text-muted-foreground text-sm text-center p-4">Open a repository to get started</p>
    </div>
  {/if}
</aside>
