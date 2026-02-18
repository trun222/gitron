<script lang="ts">
  import { Command } from 'bits-ui';
  import { open } from '@tauri-apps/plugin-dialog';
  import { sortedRecentRepos } from '$lib/stores/settings';
  import { openRepo, hasRepo, localBranches, remoteBranches, currentBranch, checkoutBranch, createAndCheckoutBranch } from '$lib/stores/repo';

  let { onShowShortcuts }: { onShowShortcuts?: () => void } = $props();

  let search = $state('');
  let isOpen = $state(false);
  let inputRef = $state<HTMLInputElement | null>(null);
  let blurTimeout: ReturnType<typeof setTimeout> | undefined;

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
    return !allBranches.some((b) => b.name === name);
  });

  let createBranchName = $derived(search.trim());

  async function handleCreateBranch() {
    const name = createBranchName;
    if (!name) return;
    isOpen = false;
    search = '';
    inputRef?.blur();
    await createAndCheckoutBranch(name);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      isOpen = false;
      search = '';
      inputRef?.blur();
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
      placeholder="Search... (Cmd+K)"
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
            </Command.GroupItems>
          </Command.Group>
        </Command.Viewport>
      </Command.List>
    {/if}
  </Command.Root>
</div>
