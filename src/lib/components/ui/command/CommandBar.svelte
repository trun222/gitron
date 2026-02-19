<script lang="ts">
  import { Command } from 'bits-ui';
  import { open } from '@tauri-apps/plugin-dialog';
  import { sortedRecentRepos } from '$lib/stores/settings';
  import {
    openRepo, hasRepo, localBranches, remoteBranches, currentBranch,
    checkoutBranch, createAndCheckoutBranch,
    remotes, networkOperation,
    fetchFromRemote, pushToRemote, pullFromRemote,
    addRemote, removeRemote,
    discardConfirmOpen,
  } from '$lib/stores/repo';

  let { onShowShortcuts, onShowSettings }: { onShowShortcuts?: () => void; onShowSettings?: () => void } = $props();

  let search = $state('');
  let isOpen = $state(false);
  let inputRef = $state<HTMLInputElement | null>(null);
  let blurTimeout: ReturnType<typeof setTimeout> | undefined;
  let addRemoteMode = $state(false);

  export function focus() {
    inputRef?.focus();
  }

  function handleFocus() {
    clearTimeout(blurTimeout);
    isOpen = true;
  }

  function handleBlur() {
    blurTimeout = setTimeout(() => {
      isOpen = false;
      search = '';
    }, 150);
  }

  async function handleSelectRepo(path: string) {
    isOpen = false;
    search = '';
    inputRef?.blur();
    await openRepo(path);
  }

  async function handleSelectBranch(name: string) {
    isOpen = false;
    search = '';
    inputRef?.blur();
    await checkoutBranch(name);
  }

  function handleShowShortcuts() {
    isOpen = false;
    search = '';
    inputRef?.blur();
    onShowShortcuts?.();
  }

  function handleShowSettings() {
    isOpen = false;
    search = '';
    inputRef?.blur();
    onShowSettings?.();
  }

  async function handleOpenFolder() {
    isOpen = false;
    search = '';
    inputRef?.blur();
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Open Git Repository',
    });
    if (selected) {
      await openRepo(selected);
    }
  }

  // Show "Create branch" when search text doesn't match any existing branch
  let showCreateBranch = $derived.by(() => {
    const name = search.trim();
    if (!name || !$hasRepo) return false;
    const allBranches = [...$localBranches, ...$remoteBranches];
    return !allBranches.some((b) => b.name.toLowerCase() === name.toLowerCase());
  });

  let createBranchName = $derived(search.trim());

  async function handleCreateBranch() {
    const name = createBranchName;
    if (!name || !showCreateBranch) return;
    isOpen = false;
    search = '';
    inputRef?.blur();
    await createAndCheckoutBranch(name);
  }

  function handleDiscardAll() {
    isOpen = false;
    search = '';
    inputRef?.blur();
    discardConfirmOpen.set(true);
  }

  async function handleGitAction(action: 'fetch' | 'push' | 'pull') {
    isOpen = false;
    search = '';
    inputRef?.blur();
    if (action === 'fetch') await fetchFromRemote();
    else if (action === 'push') await pushToRemote();
    else if (action === 'pull') await pullFromRemote();
  }

  function handleEnterAddRemoteMode() {
    addRemoteMode = true;
    search = '';
  }

  async function handleAddRemote() {
    const parts = search.trim().split(/\s+/);
    if (parts.length < 2) return;
    const [name, url] = parts;
    isOpen = false;
    search = '';
    addRemoteMode = false;
    inputRef?.blur();
    await addRemote(name, url);
  }

  async function handleRemoveRemote(name: string) {
    isOpen = false;
    search = '';
    inputRef?.blur();
    await removeRemote(name);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      if (addRemoteMode) {
        addRemoteMode = false;
        search = '';
        return;
      }
      isOpen = false;
      search = '';
      inputRef?.blur();
    }
    if (e.key === 'Enter' && addRemoteMode) {
      e.preventDefault();
      handleAddRemote();
    }
  }
</script>

<div class="relative w-full max-w-[600px]">
  <Command.Root
    shouldFilter={true}
    class="w-full"
  >
    <Command.Input
      bind:ref={inputRef}
      bind:value={search}
      onfocus={handleFocus}
      onblur={handleBlur}
      onkeydown={handleKeydown}
      placeholder={addRemoteMode ? "remote-name https://url.git" : "Type a command... (Cmd+K)"}
      class="w-full px-3 py-1.5 rounded-md border border-input bg-background text-foreground text-sm outline-none focus:border-primary transition-colors"
    />
    {#if isOpen}
      <Command.List
        class="absolute top-full left-0 right-0 mt-1 max-h-[300px] overflow-y-auto rounded-lg border border-border bg-popover shadow-lg z-50"
      >
        <Command.Viewport class="p-1">
          <Command.Empty
            class="flex items-center justify-center py-6 text-sm text-muted-foreground"
          >
            No repositories found
          </Command.Empty>

          {#if $sortedRecentRepos.length > 0}
            <Command.Group>
              <Command.GroupHeading class="px-2 pb-1.5 pt-2 text-xs text-muted-foreground">
                Recent Repositories
              </Command.GroupHeading>
              <Command.GroupItems>
                {#each $sortedRecentRepos as repo (repo.path)}
                  <Command.Item
                    value={repo.path}
                    keywords={[repo.name, repo.path]}
                    onSelect={() => handleSelectRepo(repo.path)}
                    class="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm cursor-pointer outline-none data-[selected]:bg-accent"
                  >
                    <svg class="shrink-0 text-muted-foreground" viewBox="0 0 16 16" width="14" height="14">
                      <path fill="currentColor" d="M2 2.5A2.5 2.5 0 0 1 4.5 0h8.75a.75.75 0 0 1 .75.75v12.5a.75.75 0 0 1-.75.75h-2.5a.75.75 0 0 1 0-1.5h1.75v-2h-8a1 1 0 0 0-.714 1.7.75.75 0 1 1-1.072 1.05A2.495 2.495 0 0 1 2 11.5Zm10.5-1h-8a1 1 0 0 0-1 1v6.708A2.486 2.486 0 0 1 4.5 9h8ZM5 12.25a.25.25 0 0 1 .25-.25h3.5a.25.25 0 0 1 .25.25v3.25a.25.25 0 0 1-.4.2l-1.45-1.087a.25.25 0 0 0-.3 0L5.4 15.7a.25.25 0 0 1-.4-.2Z" />
                    </svg>
                    <div class="flex flex-col min-w-0">
                      <span class="truncate font-medium">
                        {#if repo.pinned}
                          <span class="text-primary mr-1">*</span>
                        {/if}
                        {repo.name}
                      </span>
                      <span class="truncate text-xs text-muted-foreground">{repo.path}</span>
                    </div>
                  </Command.Item>
                {/each}
              </Command.GroupItems>
            </Command.Group>
            <Command.Separator class="my-1 h-px bg-border" />
          {/if}

          <Command.Group>
            <Command.GroupItems>
              <Command.Item
                value="open-repository"
                keywords={['open', 'folder', 'browse', 'directory']}
                onSelect={handleOpenFolder}
                class="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm cursor-pointer outline-none data-[selected]:bg-accent"
              >
                <svg class="shrink-0 text-muted-foreground" viewBox="0 0 16 16" width="14" height="14">
                  <path fill="currentColor" d="M.513 1.513A1.75 1.75 0 0 1 1.75 1h3.5c.55 0 1.07.26 1.4.7l.9 1.2a.25.25 0 0 0 .2.1H13a1 1 0 0 1 1 1v.5H2.75a.75.75 0 0 0 0 1.5h11.978a1 1 0 0 1 .994 1.117L15 13.25A1.75 1.75 0 0 1 13.25 15H1.75A1.75 1.75 0 0 1 0 13.25V2.75c0-.464.184-.91.513-1.237Z" />
                </svg>
                <span>Open Repository...</span>
              </Command.Item>
            </Command.GroupItems>
          </Command.Group>

          {#if $hasRepo && ($localBranches.length > 0 || showCreateBranch)}
            <Command.Separator class="my-1 h-px bg-border" />
            <Command.Group>
              <Command.GroupHeading class="px-2 pb-1.5 pt-2 text-xs text-muted-foreground">
                Branches
              </Command.GroupHeading>
              <Command.GroupItems>
                {#if showCreateBranch}
                  <Command.Item
                    value={`create:${createBranchName}`}
                    keywords={[createBranchName, 'create', 'new', 'branch']}
                    onSelect={handleCreateBranch}
                    class="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm cursor-pointer outline-none data-[selected]:bg-accent"
                  >
                    <svg class="shrink-0 text-primary" viewBox="0 0 16 16" width="14" height="14">
                      <path fill="currentColor" d="M7.75 2a.75.75 0 0 1 .75.75V7h4.25a.75.75 0 0 1 0 1.5H8.5v4.25a.75.75 0 0 1-1.5 0V8.5H2.75a.75.75 0 0 1 0-1.5H7V2.75A.75.75 0 0 1 7.75 2Z" />
                    </svg>
                    <span>Create branch "<strong>{createBranchName}</strong>"</span>
                  </Command.Item>
                {/if}
                {#each $localBranches as branch (branch.name)}
                  <Command.Item
                    value={`branch:${branch.name}`}
                    keywords={[branch.name, 'checkout', 'switch', 'branch']}
                    onSelect={() => handleSelectBranch(branch.name)}
                    class="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm cursor-pointer outline-none data-[selected]:bg-accent"
                  >
                    {#if branch.is_head}
                      <svg class="shrink-0 text-primary" viewBox="0 0 16 16" width="14" height="14">
                        <path fill="currentColor" d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.75.75 0 0 1 1.06-1.06L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z" />
                      </svg>
                    {:else}
                      <svg class="shrink-0 text-muted-foreground" viewBox="0 0 16 16" width="14" height="14">
                        <path fill="currentColor" d="M9.5 3.25a2.25 2.25 0 1 1 3 2.122V6A2.5 2.5 0 0 1 10 8.5H6a1 1 0 0 0-1 1v1.128a2.251 2.251 0 1 1-1.5 0V5.372a2.25 2.25 0 1 1 1.5 0v1.836A2.5 2.5 0 0 1 6 7h4a1 1 0 0 0 1-1v-.628A2.25 2.25 0 0 1 9.5 3.25Z" />
                      </svg>
                    {/if}
                    <span class={branch.is_head ? 'font-medium text-primary' : ''}>{branch.name}</span>
                  </Command.Item>
                {/each}
              </Command.GroupItems>
            </Command.Group>
          {/if}

          {#if $hasRepo && $remoteBranches.length > 0}
            <Command.Group>
              <Command.GroupHeading class="px-2 pb-1.5 pt-2 text-xs text-muted-foreground">
                Remote Branches
              </Command.GroupHeading>
              <Command.GroupItems>
                {#each $remoteBranches as branch (branch.name)}
                  <Command.Item
                    value={`remote:${branch.name}`}
                    keywords={[branch.name, 'remote', 'checkout', 'branch']}
                    onSelect={() => handleSelectBranch(branch.name)}
                    class="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm cursor-pointer outline-none data-[selected]:bg-accent opacity-70"
                  >
                    <svg class="shrink-0 text-muted-foreground" viewBox="0 0 16 16" width="14" height="14">
                      <path fill="currentColor" d="M9.5 3.25a2.25 2.25 0 1 1 3 2.122V6A2.5 2.5 0 0 1 10 8.5H6a1 1 0 0 0-1 1v1.128a2.251 2.251 0 1 1-1.5 0V5.372a2.25 2.25 0 1 1 1.5 0v1.836A2.5 2.5 0 0 1 6 7h4a1 1 0 0 0 1-1v-.628A2.25 2.25 0 0 1 9.5 3.25Z" />
                    </svg>
                    <span>{branch.name}</span>
                  </Command.Item>
                {/each}
              </Command.GroupItems>
            </Command.Group>
          {/if}

          {#if $hasRepo}
            <Command.Separator class="my-1 h-px bg-border" />
            <Command.Group>
              <Command.GroupHeading class="px-2 pb-1.5 pt-2 text-xs text-muted-foreground">
                Git Actions
              </Command.GroupHeading>
              <Command.GroupItems>
                <Command.Item
                  value="fetch-all-remotes"
                  keywords={['fetch', 'download', 'sync', 'remote']}
                  onSelect={() => handleGitAction('fetch')}
                  disabled={!!$networkOperation}
                  class="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm cursor-pointer outline-none data-[selected]:bg-accent disabled:opacity-50"
                >
                  <svg class="shrink-0 text-muted-foreground" viewBox="0 0 16 16" width="14" height="14">
                    <path fill="currentColor" d="M8 2a.75.75 0 0 1 .75.75v6.69l1.72-1.72a.75.75 0 1 1 1.06 1.06l-3 3a.75.75 0 0 1-1.06 0l-3-3a.75.75 0 0 1 1.06-1.06l1.72 1.72V2.75A.75.75 0 0 1 8 2Z" />
                    <path fill="currentColor" d="M2.5 13.25a.75.75 0 0 1 .75-.75h9.5a.75.75 0 0 1 0 1.5h-9.5a.75.75 0 0 1-.75-.75Z" />
                  </svg>
                  <span>Fetch All Remotes</span>
                </Command.Item>
                <Command.Item
                  value="pull-from-remote"
                  keywords={['pull', 'merge', 'update', 'download']}
                  onSelect={() => handleGitAction('pull')}
                  disabled={!!$networkOperation}
                  class="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm cursor-pointer outline-none data-[selected]:bg-accent disabled:opacity-50"
                >
                  <svg class="shrink-0 text-muted-foreground" viewBox="0 0 16 16" width="14" height="14">
                    <path fill="currentColor" d="M4.75 0a.75.75 0 0 1 .75.75v5.69l1.72-1.72a.75.75 0 1 1 1.06 1.06l-3 3a.75.75 0 0 1-1.06 0l-3-3a.75.75 0 0 1 1.06-1.06L3 6.44V.75A.75.75 0 0 1 3.75 0ZM8 12h3.75a.75.75 0 0 0 0-1.5H8.5v-2h3.75a2.25 2.25 0 0 1 0 4.5H8Z" />
                  </svg>
                  <span>Pull</span>
                </Command.Item>
                <Command.Item
                  value="push-to-remote"
                  keywords={['push', 'upload', 'publish']}
                  onSelect={() => handleGitAction('push')}
                  disabled={!!$networkOperation}
                  class="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm cursor-pointer outline-none data-[selected]:bg-accent disabled:opacity-50"
                >
                  <svg class="shrink-0 text-muted-foreground" viewBox="0 0 16 16" width="14" height="14">
                    <path fill="currentColor" d="M8 14a.75.75 0 0 1-.75-.75V6.56L5.53 8.28a.75.75 0 0 1-1.06-1.06l3-3a.75.75 0 0 1 1.06 0l3 3a.75.75 0 1 1-1.06 1.06L8.75 6.56v6.69A.75.75 0 0 1 8 14Z" />
                    <path fill="currentColor" d="M2.5 2.75a.75.75 0 0 1 .75-.75h9.5a.75.75 0 0 1 0 1.5h-9.5a.75.75 0 0 1-.75-.75Z" />
                  </svg>
                  <span>Push</span>
                </Command.Item>
                <Command.Item
                  value="discard-all-changes"
                  keywords={['discard', 'reset', 'clean', 'revert', 'undo', 'changes']}
                  onSelect={handleDiscardAll}
                  class="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm cursor-pointer outline-none data-[selected]:bg-accent text-destructive"
                >
                  <svg class="shrink-0" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>
                  <span>Discard All Changes</span>
                </Command.Item>
              </Command.GroupItems>
            </Command.Group>

            <Command.Separator class="my-1 h-px bg-border" />
            <Command.Group>
              <Command.GroupHeading class="px-2 pb-1.5 pt-2 text-xs text-muted-foreground">
                Remotes
              </Command.GroupHeading>
              <Command.GroupItems>
                {#each $remotes as r (r.name)}
                  <Command.Item
                    value={`remote-config:${r.name}`}
                    keywords={[r.name, r.url, 'remote', 'remove', 'delete']}
                    onSelect={() => handleRemoveRemote(r.name)}
                    class="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm cursor-pointer outline-none data-[selected]:bg-accent"
                  >
                    <svg class="shrink-0 text-muted-foreground" viewBox="0 0 16 16" width="14" height="14">
                      <path fill="currentColor" d="M2 2.5A2.5 2.5 0 0 1 4.5 0h8.75a.75.75 0 0 1 .75.75v12.5a.75.75 0 0 1-.75.75h-2.5a.75.75 0 0 1 0-1.5h1.75v-2h-8a1 1 0 0 0-.714 1.7.75.75 0 1 1-1.072 1.05A2.495 2.495 0 0 1 2 11.5Zm10.5-1h-8a1 1 0 0 0-1 1v6.708A2.486 2.486 0 0 1 4.5 9h8Z" />
                    </svg>
                    <div class="flex flex-col min-w-0">
                      <span class="truncate font-medium">{r.name}</span>
                      <span class="truncate text-xs text-muted-foreground">{r.url}</span>
                    </div>
                    <span class="ml-auto text-xs text-muted-foreground">Remove</span>
                  </Command.Item>
                {/each}
                <Command.Item
                  value="add-remote"
                  keywords={['add', 'remote', 'new', 'create']}
                  onSelect={handleEnterAddRemoteMode}
                  class="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm cursor-pointer outline-none data-[selected]:bg-accent"
                >
                  <svg class="shrink-0 text-primary" viewBox="0 0 16 16" width="14" height="14">
                    <path fill="currentColor" d="M7.75 2a.75.75 0 0 1 .75.75V7h4.25a.75.75 0 0 1 0 1.5H8.5v4.25a.75.75 0 0 1-1.5 0V8.5H2.75a.75.75 0 0 1 0-1.5H7V2.75A.75.75 0 0 1 7.75 2Z" />
                  </svg>
                  <span>Add Remote...</span>
                </Command.Item>
              </Command.GroupItems>
            </Command.Group>
          {/if}

          <Command.Separator class="my-1 h-px bg-border" />
          <Command.Group>
            <Command.GroupItems>
              <Command.Item
                value="keyboard-shortcuts"
                keywords={['shortcuts', 'keyboard', 'keys', 'hotkeys', 'help']}
                onSelect={handleShowShortcuts}
                class="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm cursor-pointer outline-none data-[selected]:bg-accent"
              >
                <svg class="shrink-0 text-muted-foreground" viewBox="0 0 16 16" width="14" height="14">
                  <path fill="currentColor" d="M0 2.75C0 1.784.784 1 1.75 1h12.5c.966 0 1.75.784 1.75 1.75v8.5A1.75 1.75 0 0 1 14.25 13H1.75A1.75 1.75 0 0 1 0 11.25Zm1.75-.25a.25.25 0 0 0-.25.25v8.5c0 .138.112.25.25.25h12.5a.25.25 0 0 0 .25-.25v-8.5a.25.25 0 0 0-.25-.25ZM3.5 4a.75.75 0 0 0 0 1.5h1A.75.75 0 0 0 4.5 4Zm3.25.75A.75.75 0 0 1 7.5 4h1a.75.75 0 0 1 0 1.5h-1a.75.75 0 0 1-.75-.75ZM11.5 4a.75.75 0 0 0 0 1.5h1a.75.75 0 0 0 0-1.5ZM3.5 7a.75.75 0 0 0 0 1.5h1A.75.75 0 0 0 4.5 7Zm3.25.75A.75.75 0 0 1 7.5 7h1a.75.75 0 0 1 0 1.5h-1a.75.75 0 0 1-.75-.75ZM11.5 7a.75.75 0 0 0 0 1.5h1a.75.75 0 0 0 0-1.5ZM5 10a.75.75 0 0 0 0 1.5h6a.75.75 0 0 0 0-1.5Z" />
                </svg>
                <span>Keyboard Shortcuts</span>
                <kbd class="ml-auto text-xs text-muted-foreground border border-border rounded px-1 py-0.5 font-mono">?</kbd>
              </Command.Item>
              <Command.Item
                value="settings"
                keywords={['settings', 'preferences', 'options', 'config', 'theme']}
                onSelect={handleShowSettings}
                class="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm cursor-pointer outline-none data-[selected]:bg-accent"
              >
                <svg class="shrink-0 text-muted-foreground" viewBox="0 0 16 16" width="14" height="14">
                  <path fill="currentColor" d="M8 0a8.2 8.2 0 0 1 .701.031C9.444.095 9.99.645 10.16 1.29l.288 1.107c.018.066.079.158.212.224.231.114.454.243.668.386.123.082.233.09.3.071L12.727 2.8c.63-.186 1.345.018 1.737.631A8 8 0 0 1 15.63 5.57c.174.647-.134 1.318-.683 1.637l-.97.567a.4.4 0 0 0-.177.288 6.8 6.8 0 0 1 0 .776c.005.09.076.186.177.288l.97.567c.549.319.857.99.683 1.637a8 8 0 0 1-1.166 2.14c-.392.612-1.108.816-1.737.63l-1.1-.278c-.066-.018-.177-.011-.3.071a5.7 5.7 0 0 1-.668.386c-.133.066-.194.158-.212.224l-.288 1.107c-.17.645-.716 1.195-1.459 1.26a8.1 8.1 0 0 1-1.402 0c-.743-.065-1.289-.615-1.459-1.26l-.288-1.107a.37.37 0 0 0-.212-.224 5.7 5.7 0 0 1-.668-.386c-.123-.082-.233-.09-.3-.071l-1.1.278c-.63.186-1.345-.018-1.737-.631A8 8 0 0 1 .37 10.43c-.174-.647.134-1.318.683-1.637l.97-.567c.1-.102.171-.198.177-.288a6.8 6.8 0 0 1 0-.776.4.4 0 0 0-.177-.288l-.97-.567C.504 5.988.196 5.317.37 4.67a8 8 0 0 1 1.166-2.14c.392-.612 1.108-.816 1.737-.63l1.1.278c.066.018.177.011.3-.071.214-.143.437-.272.668-.386a.37.37 0 0 0 .212-.224l.288-1.107C6.01.645 6.556.095 7.299.03 7.53.01 7.764 0 8 0Zm-.571 1.525c-.036.003-.108.036-.137.146l-.289 1.105c-.147.56-.55.967-.997 1.189a4.2 4.2 0 0 0-.488.282c-.4.266-.881.395-1.437.223l-1.1-.278c-.11-.03-.175.016-.195.046a6.5 6.5 0 0 0-.9 1.652c-.03.11.004.186.077.228l.97.567c.477.278.79.754.816 1.297.015.315.015.635 0 .95-.027.543-.34 1.019-.816 1.297l-.97.567c-.073.042-.107.118-.077.228.203.724.508 1.305.9 1.652.02.03.085.076.195.046l1.1-.278c.556-.172 1.037-.043 1.437.223.155.104.318.197.488.283.448.222.85.629.997 1.189l.289 1.105c.029.109.101.143.137.146a6.6 6.6 0 0 0 1.142 0c.036-.003.108-.036.137-.146l.289-1.105c.147-.56.55-.967.997-1.189.17-.086.333-.179.488-.283.4-.266.881-.395 1.437-.223l1.1.278c.11.03.175-.016.195-.046.392-.347.697-.928.9-1.652.03-.11-.004-.186-.077-.228l-.97-.567c-.477-.278-.79-.754-.816-1.297a5.3 5.3 0 0 1 0-.95c.027-.543.34-1.019.816-1.297l.97-.567c.073-.042.107-.118.077-.228a6.5 6.5 0 0 0-.9-1.652c-.02-.03-.085-.076-.195-.046l-1.1.278c-.556.172-1.037.043-1.437-.223a4.2 4.2 0 0 0-.488-.282c-.448-.222-.85-.629-.997-1.189l-.289-1.105c-.029-.11-.101-.143-.137-.146a6.6 6.6 0 0 0-1.142 0ZM11 8a3 3 0 1 1-6 0 3 3 0 0 1 6 0ZM9.5 8a1.5 1.5 0 1 0-3.001.001A1.5 1.5 0 0 0 9.5 8Z" />
                </svg>
                <span>Settings</span>
                <kbd class="ml-auto text-xs text-muted-foreground border border-border rounded px-1 py-0.5 font-mono">Cmd+,</kbd>
              </Command.Item>
            </Command.GroupItems>
          </Command.Group>
        </Command.Viewport>
      </Command.List>
    {/if}
  </Command.Root>
</div>
