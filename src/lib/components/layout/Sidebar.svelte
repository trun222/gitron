<script lang="ts">
  import {
    hasRepo,
    repoStatus,
    commitGraph,
    localBranches,
    remoteBranches,
    stagedCount,
    unstagedCount,
    repoPath,
  } from '$lib/stores/repo';

  let activeTab: 'changes' | 'branches' = $state('changes');
</script>

<aside class="sidebar">
  {#if $hasRepo}
    <div class="sidebar-tabs">
      <button
        class="tab"
        class:active={activeTab === 'changes'}
        onclick={() => (activeTab = 'changes')}
      >
        Changes
        {#if $stagedCount + $unstagedCount > 0}
          <span class="badge">{$stagedCount + $unstagedCount}</span>
        {/if}
      </button>
      <button
        class="tab"
        class:active={activeTab === 'branches'}
        onclick={() => (activeTab = 'branches')}
      >
        Branches
      </button>
    </div>

    <div class="sidebar-content">
      {#if activeTab === 'changes'}
        <div class="section">
          {#if $repoStatus}
            {#if $repoStatus.staged.length > 0}
              <div class="section-header">
                <span>Staged ({$repoStatus.staged.length})</span>
              </div>
              <ul class="file-list">
                {#each $repoStatus.staged as file}
                  <li class="file-entry">
                    <span class="status-badge staged">{file.status[0]}</span>
                    <span class="file-path">{file.path}</span>
                  </li>
                {/each}
              </ul>
            {/if}

            {#if $repoStatus.unstaged.length > 0}
              <div class="section-header">
                <span>Unstaged ({$repoStatus.unstaged.length})</span>
              </div>
              <ul class="file-list">
                {#each $repoStatus.unstaged as file}
                  <li class="file-entry">
                    <span class="status-badge unstaged">{file.status[0]}</span>
                    <span class="file-path">{file.path}</span>
                  </li>
                {/each}
              </ul>
            {/if}

            {#if $repoStatus.untracked.length > 0}
              <div class="section-header">
                <span>Untracked ({$repoStatus.untracked.length})</span>
              </div>
              <ul class="file-list">
                {#each $repoStatus.untracked as file}
                  <li class="file-entry">
                    <span class="status-badge untracked">?</span>
                    <span class="file-path">{file}</span>
                  </li>
                {/each}
              </ul>
            {/if}

            {#if $repoStatus.staged.length === 0 && $repoStatus.unstaged.length === 0 && $repoStatus.untracked.length === 0}
              <p class="empty-state">Working tree clean</p>
            {/if}
          {/if}
        </div>
      {:else if activeTab === 'branches'}
        <div class="section">
          <div class="section-header">Local</div>
          <ul class="file-list">
            {#each $localBranches as branch}
              <li class="file-entry" class:active-branch={branch.is_head}>
                {#if branch.is_head}
                  <span class="head-indicator">*</span>
                {/if}
                <span class="file-path">{branch.name}</span>
              </li>
            {/each}
          </ul>

          {#if $remoteBranches.length > 0}
            <div class="section-header">Remote</div>
            <ul class="file-list">
              {#each $remoteBranches as branch}
                <li class="file-entry remote">
                  <span class="file-path">{branch.name}</span>
                </li>
              {/each}
            </ul>
          {/if}
        </div>
      {/if}
    </div>
  {:else}
    <div class="empty-state-container">
      <p class="empty-state">Open a repository to get started</p>
    </div>
  {/if}
</aside>

<style>
  .sidebar {
    width: 260px;
    min-width: 200px;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .sidebar-tabs {
    display: flex;
    border-bottom: 1px solid var(--border-color);
  }

  .tab {
    flex: 1;
    padding: 8px;
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    border-bottom: 2px solid transparent;
    transition: all 0.15s;
  }

  .tab:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .tab.active {
    color: var(--text-accent);
    border-bottom-color: var(--accent-color);
  }

  .badge {
    background: var(--accent-color);
    color: white;
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 10px;
    min-width: 18px;
    text-align: center;
  }

  .sidebar-content {
    flex: 1;
    overflow-y: auto;
    padding: 8px 0;
  }

  .section-header {
    padding: 6px 12px;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .file-list {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .file-entry {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 12px;
    font-size: 12px;
    cursor: pointer;
    transition: background-color 0.1s;
  }

  .file-entry:hover {
    background: var(--bg-hover);
  }

  .file-path {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-primary);
  }

  .status-badge {
    font-size: 10px;
    font-weight: 700;
    width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 3px;
    flex-shrink: 0;
  }

  .status-badge.staged {
    color: var(--color-added);
    background: var(--color-added-bg);
  }

  .status-badge.unstaged {
    color: var(--color-modified);
    background: var(--color-modified-bg);
  }

  .status-badge.untracked {
    color: var(--text-muted);
    background: var(--bg-hover);
  }

  .active-branch {
    color: var(--text-accent);
    font-weight: 500;
  }

  .head-indicator {
    color: var(--accent-color);
    font-weight: 700;
  }

  .remote {
    opacity: 0.7;
  }

  .empty-state-container {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
  }

  .empty-state {
    color: var(--text-muted);
    font-size: 13px;
    text-align: center;
    padding: 16px;
  }
</style>
