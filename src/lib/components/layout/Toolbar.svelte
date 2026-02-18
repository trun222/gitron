<script lang="ts">
  import { open } from '@tauri-apps/plugin-opener';
  import { repoInfo, currentBranch, hasRepo } from '$lib/stores/repo';
  import { openRepo } from '$lib/stores/repo';

  let folderPath = $state('');

  async function handleOpenRepo() {
    if (folderPath.trim()) {
      await openRepo(folderPath.trim());
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      handleOpenRepo();
    }
  }
</script>

<header class="toolbar">
  <div class="toolbar-left">
    <span class="app-title">Gitron</span>
  </div>

  <div class="toolbar-center">
    <div class="repo-opener">
      <input
        type="text"
        placeholder="Open repository path..."
        bind:value={folderPath}
        onkeydown={handleKeydown}
        class="repo-input"
      />
      <button onclick={handleOpenRepo} class="btn btn-primary">Open</button>
    </div>
  </div>

  <div class="toolbar-right">
    {#if $hasRepo && $currentBranch}
      <span class="branch-badge">
        <svg class="icon" viewBox="0 0 16 16" width="14" height="14">
          <path fill="currentColor" d="M9.5 3.25a2.25 2.25 0 1 1 3 2.122V6A2.5 2.5 0 0 1 10 8.5H6a1 1 0 0 0-1 1v1.128a2.251 2.251 0 1 1-1.5 0V5.372a2.25 2.25 0 1 1 1.5 0v1.836A2.5 2.5 0 0 1 6 7h4a1 1 0 0 0 1-1v-.628A2.25 2.25 0 0 1 9.5 3.25Z" />
        </svg>
        {$currentBranch}
      </span>
    {/if}
  </div>
</header>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 48px;
    padding: 0 16px;
    background-color: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    -webkit-app-region: drag;
    user-select: none;
  }

  .toolbar-left,
  .toolbar-right {
    display: flex;
    align-items: center;
    min-width: 200px;
  }

  .toolbar-right {
    justify-content: flex-end;
  }

  .toolbar-center {
    flex: 1;
    display: flex;
    justify-content: center;
    -webkit-app-region: no-drag;
  }

  .app-title {
    font-weight: 700;
    font-size: 14px;
    letter-spacing: 0.5px;
    color: var(--text-accent);
  }

  .repo-opener {
    display: flex;
    gap: 8px;
    max-width: 500px;
    width: 100%;
  }

  .repo-input {
    flex: 1;
    padding: 6px 12px;
    border-radius: 6px;
    border: 1px solid var(--border-color);
    background: var(--bg-input);
    color: var(--text-primary);
    font-size: 13px;
    outline: none;
  }

  .repo-input:focus {
    border-color: var(--accent-color);
  }

  .btn {
    padding: 6px 16px;
    border-radius: 6px;
    border: none;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: background-color 0.15s;
  }

  .btn-primary {
    background: var(--accent-color);
    color: white;
  }

  .btn-primary:hover {
    background: var(--accent-hover);
  }

  .branch-badge {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    border-radius: 4px;
    background: var(--bg-badge);
    font-size: 12px;
    font-weight: 500;
    color: var(--text-secondary);
  }

  .icon {
    flex-shrink: 0;
  }
</style>
