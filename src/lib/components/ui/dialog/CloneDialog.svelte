<script lang="ts">
  import { Dialog } from 'bits-ui';
  import { getTransport } from '$lib/api';
  import {
    cloneDialogOpen,
    closeCloneDialog,
    filteredRepos,
    githubReposLoading,
    cloning,
    cloneError,
    repoSearchQuery,
    cloneRepository,
    loadGitHubRepos,
  } from '$lib/stores/clone';
  import { isAuthenticated } from '$lib/stores/github';

  let activeTab = $state<'github' | 'url'>('github');
  let cloneUrl = $state('');
  let destPath = $state('');

  function selectGitHubRepo(cloneUrlVal: string) {
    cloneUrl = cloneUrlVal;
    // Auto-switch to show the clone URL is populated
  }

  function repoNameFromUrl(url: string): string {
    try {
      const parts = url.replace(/\.git$/, '').split('/');
      return parts[parts.length - 1] || '';
    } catch {
      return '';
    }
  }

  async function handleBrowse() {
    const selected = await getTransport().pickDirectory('Choose clone destination');
    if (selected) {
      const name = repoNameFromUrl(cloneUrl);
      destPath = name ? `${selected}/${name}` : selected;
    }
  }

  async function handleClone() {
    if (!cloneUrl.trim() || !destPath.trim() || $cloning) return;
    await cloneRepository(cloneUrl.trim(), destPath.trim());
  }

  function handleClose(open: boolean) {
    if (!open) {
      closeCloneDialog();
      activeTab = 'github';
      cloneUrl = '';
      destPath = '';
    }
  }

  // When cloneUrl changes and destPath has a repo name, update the repo name portion
  let prevUrl = '';
  $effect(() => {
    if (cloneUrl && cloneUrl !== prevUrl && destPath) {
      const oldName = repoNameFromUrl(prevUrl);
      const newName = repoNameFromUrl(cloneUrl);
      if (oldName && destPath.endsWith(`/${oldName}`)) {
        destPath = destPath.slice(0, -oldName.length) + newName;
      }
    }
    prevUrl = cloneUrl;
  });
</script>

<Dialog.Root
  open={$cloneDialogOpen}
  onOpenChange={handleClose}
>
  <Dialog.Portal>
    <Dialog.Overlay class="fixed inset-0 bg-black/50 z-50" />
    <Dialog.Content
      class="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 z-50 w-full max-w-lg rounded-lg border border-border bg-card shadow-lg flex flex-col max-h-[80vh]"
    >
      <div class="px-5 pt-5 pb-3">
        <Dialog.Title class="text-sm font-semibold text-foreground">
          Clone Repository
        </Dialog.Title>
      </div>

      <!-- Tabs -->
      <div class="flex border-b border-border px-5">
        <button
          class="px-3 py-2 text-xs font-medium transition-colors cursor-pointer {activeTab === 'github' ? 'text-primary border-b-2 border-primary' : 'text-muted-foreground hover:text-foreground'}"
          onclick={() => activeTab = 'github'}
        >
          GitHub
        </button>
        <button
          class="px-3 py-2 text-xs font-medium transition-colors cursor-pointer {activeTab === 'url' ? 'text-primary border-b-2 border-primary' : 'text-muted-foreground hover:text-foreground'}"
          onclick={() => activeTab = 'url'}
        >
          URL
        </button>
      </div>

      <!-- Tab Content -->
      <div class="flex-1 overflow-hidden flex flex-col min-h-0">
        {#if activeTab === 'github'}
          <div class="flex flex-col flex-1 min-h-0 p-4 gap-3">
            {#if $isAuthenticated}
              <input
                type="text"
                class="w-full bg-input text-foreground text-xs rounded-md border border-border px-3 py-2 placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-primary"
                placeholder="Search repositories..."
                bind:value={$repoSearchQuery}
              />
              <div class="flex-1 overflow-y-auto min-h-0 rounded-md border border-border">
                {#if $githubReposLoading}
                  <div class="flex items-center justify-center py-8 text-sm text-muted-foreground">
                    <svg class="animate-spin mr-2 h-4 w-4" viewBox="0 0 24 24" fill="none">
                      <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" />
                      <path d="M4 12a8 8 0 018-8" stroke="currentColor" stroke-width="3" stroke-linecap="round" class="opacity-75" />
                    </svg>
                    Loading repositories...
                  </div>
                {:else if $filteredRepos.length === 0}
                  <div class="flex items-center justify-center py-8 text-sm text-muted-foreground">
                    No repositories found
                  </div>
                {:else}
                  <ul class="list-none divide-y divide-border">
                    {#each $filteredRepos as repo (repo.id)}
                      <li>
                        <button
                          class="w-full text-left px-3 py-2 hover:bg-accent transition-colors cursor-pointer {cloneUrl === repo.clone_url ? 'bg-accent ring-1 ring-primary/30' : ''}"
                          onclick={() => selectGitHubRepo(repo.clone_url)}
                        >
                          <div class="flex items-center gap-2">
                            <span class="text-xs font-medium text-foreground truncate">{repo.full_name}</span>
                            {#if repo.private}
                              <span class="text-[10px] px-1.5 py-0.5 rounded-full bg-amber-500/15 text-amber-400 font-medium shrink-0">
                                Private
                              </span>
                            {/if}
                          </div>
                          {#if repo.description}
                            <p class="text-[11px] text-muted-foreground truncate mt-0.5">{repo.description}</p>
                          {/if}
                        </button>
                      </li>
                    {/each}
                  </ul>
                {/if}
              </div>
              <button
                class="text-xs text-muted-foreground hover:text-foreground transition-colors cursor-pointer self-start"
                onclick={loadGitHubRepos}
                disabled={$githubReposLoading}
              >
                Refresh
              </button>
            {:else}
              <div class="flex items-center justify-center py-8 text-sm text-muted-foreground flex-1">
                Sign in to GitHub to browse your repositories
              </div>
            {/if}
          </div>
        {:else}
          <div class="p-4">
            <label for="clone-url-input" class="block text-xs font-medium text-muted-foreground mb-1.5">Repository URL</label>
            <input
              id="clone-url-input"
              type="text"
              class="w-full bg-input text-foreground text-xs rounded-md border border-border px-3 py-2 placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-primary"
              placeholder="https://github.com/user/repo.git"
              bind:value={cloneUrl}
            />
          </div>
        {/if}
      </div>

      <!-- Bottom: Destination + Clone -->
      <div class="border-t border-border px-5 py-4 flex flex-col gap-3">
        <div>
          <label for="clone-dest-input" class="block text-xs font-medium text-muted-foreground mb-1.5">Destination</label>
          <div class="flex gap-2">
            <input
              id="clone-dest-input"
              type="text"
              class="flex-1 bg-input text-foreground text-xs rounded-md border border-border px-3 py-2 placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-primary"
              placeholder="/path/to/clone"
              bind:value={destPath}
            />
            <button
              class="px-3 py-2 text-xs rounded-md border border-border bg-background text-foreground hover:bg-accent transition-colors cursor-pointer shrink-0"
              onclick={handleBrowse}
            >
              Browse...
            </button>
          </div>
        </div>

        {#if $cloneError}
          <p class="text-[11px] text-destructive">{$cloneError}</p>
        {/if}

        <div class="flex items-center gap-2 justify-end">
          <button
            type="button"
            class="px-3 py-1.5 text-sm rounded-md border border-border bg-background text-foreground hover:bg-accent transition-colors cursor-pointer"
            onclick={() => handleClose(false)}
          >
            Cancel
          </button>
          <button
            type="button"
            class="px-3 py-1.5 text-sm rounded-md font-medium transition-colors cursor-pointer {cloneUrl.trim() && destPath.trim() && !$cloning ? 'bg-primary text-primary-foreground hover:bg-primary/90' : 'bg-muted text-muted-foreground cursor-not-allowed'}"
            onclick={handleClone}
            disabled={!cloneUrl.trim() || !destPath.trim() || $cloning}
          >
            {#if $cloning}
              <span class="flex items-center gap-1.5">
                <svg class="animate-spin h-3.5 w-3.5" viewBox="0 0 24 24" fill="none">
                  <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" />
                  <path d="M4 12a8 8 0 018-8" stroke="currentColor" stroke-width="3" stroke-linecap="round" class="opacity-75" />
                </svg>
                Cloning...
              </span>
            {:else}
              Clone
            {/if}
          </button>
        </div>
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
