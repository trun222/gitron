<script lang="ts">
  import { onMount } from 'svelte';
  import { hasRepo, currentBranch, aheadCount, behindCount, networkOperation, pullFromRemote, pushToRemote, forcePushConfirmOpen } from '$lib/stores/repo';
  import { base } from '$app/paths';
  import { CommandBar } from '$lib/components/ui/command';
  import { ShortcutsModal } from '$lib/components/ui/shortcuts';
  import { SettingsModal } from '$lib/components/ui/settings';
  import { GitHubUserMenu } from '$lib/components/ui/dialog';

  let commandBar: CommandBar | undefined = $state();
  let shortcutsOpen = $state(false);
  let settingsOpen = $state(false);
  let pushDropdownOpen = $state(false);

  onMount(() => {
    function handleKeydown(e: KeyboardEvent) {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        commandBar?.focus();
      }
      if (e.key === '?' && !(e.target instanceof HTMLInputElement) && !(e.target instanceof HTMLTextAreaElement)) {
        e.preventDefault();
        shortcutsOpen = !shortcutsOpen;
      }
      if ((e.metaKey || e.ctrlKey) && e.key === ',') {
        e.preventDefault();
        settingsOpen = !settingsOpen;
      }
      if (e.key === '/' && !(e.target instanceof HTMLInputElement) && !(e.target instanceof HTMLTextAreaElement)) {
        e.preventDefault();
        commandBar?.focusCommitSearch();
      }
    }
    document.addEventListener('keydown', handleKeydown);
    return () => document.removeEventListener('keydown', handleKeydown);
  });
</script>

<header class="flex items-center justify-between h-12 px-4 bg-card border-b border-border select-none" style="-webkit-app-region: drag;">
  <div class="flex items-center gap-1.5 min-w-[200px]">
    <img src="{base}/gitron-logo.png" alt="Gitron" width="20" height="20" class="shrink-0" />
    <span class="font-bold text-sm tracking-wide text-primary">Gitron</span>
  </div>

  <div class="flex-1 flex justify-center" style="-webkit-app-region: no-drag;">
    <CommandBar bind:this={commandBar} onShowShortcuts={() => shortcutsOpen = true} onShowSettings={() => settingsOpen = true} />
  </div>

  <div class="flex items-center justify-end min-w-[200px] gap-1">
    {#if $hasRepo && $currentBranch}
      <button
        class="toolbar-btn"
        onclick={() => pullFromRemote()}
        disabled={!!$networkOperation}
        title="Pull"
      >
        {#if $networkOperation === 'pulling'}
          <svg class="spinner" viewBox="0 0 16 16" width="12" height="12"><circle cx="8" cy="8" r="6" fill="none" stroke="currentColor" stroke-width="2" stroke-dasharray="28" stroke-dashoffset="8" stroke-linecap="round"/></svg>
        {:else}
          <svg class="shrink-0" viewBox="0 0 16 16" width="12" height="12">
            <path fill="currentColor" d="M8 14a.75.75 0 0 1-.53-.22l-3-3a.75.75 0 1 1 1.06-1.06L7.25 11.44V2.75a.75.75 0 0 1 1.5 0v8.69l1.72-1.72a.75.75 0 1 1 1.06 1.06l-3 3A.75.75 0 0 1 8 14Z" />
          </svg>
        {/if}
        {#if $behindCount > 0}
          <span class="text-[10px] bg-accent rounded px-1">{$behindCount}</span>
        {/if}
      </button>
      <div class="push-split-btn">
        <button
          class="push-main"
          onclick={() => pushToRemote()}
          disabled={!!$networkOperation}
          title="Push"
        >
          {#if $networkOperation === 'pushing'}
            <svg class="spinner" viewBox="0 0 16 16" width="12" height="12"><circle cx="8" cy="8" r="6" fill="none" stroke="currentColor" stroke-width="2" stroke-dasharray="28" stroke-dashoffset="8" stroke-linecap="round"/></svg>
          {:else}
            <svg class="shrink-0" viewBox="0 0 16 16" width="12" height="12">
              <path fill="currentColor" d="M8 2a.75.75 0 0 1 .53.22l3 3a.75.75 0 0 1-1.06 1.06L8.75 4.56v8.69a.75.75 0 0 1-1.5 0V4.56L5.53 6.28a.75.75 0 0 1-1.06-1.06l3-3A.75.75 0 0 1 8 2Z" />
            </svg>
          {/if}
          {#if $aheadCount > 0}
            <span class="text-[10px] bg-accent rounded px-1">{$aheadCount}</span>
          {/if}
        </button>
        <span class="push-divider"></span>
        <div class="relative flex">
          <button
            class="push-chevron"
            onclick={() => pushDropdownOpen = !pushDropdownOpen}
            disabled={!!$networkOperation}
            title="Push options"
          >
            <svg class="shrink-0" viewBox="0 0 16 16" width="10" height="10">
              <path fill="currentColor" d="M4.427 7.427l3.396 3.396a.25.25 0 0 0 .354 0l3.396-3.396A.25.25 0 0 0 11.396 7H4.604a.25.25 0 0 0-.177.427Z" />
            </svg>
          </button>
          {#if pushDropdownOpen}
            <button
              type="button"
              class="fixed inset-0 z-40"
              tabindex="-1"
              onclick={() => pushDropdownOpen = false}
            ></button>
            <div class="absolute right-0 top-full mt-1 min-w-[160px] rounded-md border border-border bg-popover shadow-lg z-50 p-1">
              <button
                type="button"
                class="flex items-center gap-2 w-full rounded-sm px-2 py-1.5 text-sm text-destructive hover:bg-accent transition-colors"
                onclick={() => { pushDropdownOpen = false; forcePushConfirmOpen.set(true); }}
              >
                <svg class="shrink-0" viewBox="0 0 16 16" width="14" height="14">
                  <path fill="currentColor" d="M8 2a.75.75 0 0 1 .53.22l3 3a.75.75 0 0 1-1.06 1.06L8.75 4.56v8.69a.75.75 0 0 1-1.5 0V4.56L5.53 6.28a.75.75 0 0 1-1.06-1.06l3-3A.75.75 0 0 1 8 2Z" />
                </svg>
                Force Push
              </button>
            </div>
          {/if}
        </div>
      </div>
      <button
        class="toolbar-btn !gap-1.5 !px-2.5"
        onclick={() => commandBar?.focus()}
      >
        <svg class="shrink-0" viewBox="0 0 16 16" width="14" height="14">
          <path fill="currentColor" d="M9.5 3.25a2.25 2.25 0 1 1 3 2.122V6A2.5 2.5 0 0 1 10 8.5H6a1 1 0 0 0-1 1v1.128a2.251 2.251 0 1 1-1.5 0V5.372a2.25 2.25 0 1 1 1.5 0v1.836A2.5 2.5 0 0 1 6 7h4a1 1 0 0 0 1-1v-.628A2.25 2.25 0 0 1 9.5 3.25Z" />
        </svg>
        {$currentBranch}
      </button>
    {/if}
    <GitHubUserMenu />
  </div>
</header>

<ShortcutsModal bind:open={shortcutsOpen} />
<SettingsModal bind:open={settingsOpen} />

<style>
  .toolbar-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    border-radius: 6px;
    background: var(--secondary);
    font-size: 12px;
    font-weight: 500;
    color: var(--muted-foreground);
    cursor: pointer;
    transition: background 150ms, color 150ms;
  }
  .toolbar-btn:hover:not(:disabled) {
    background: var(--accent);
    color: var(--foreground);
  }
  .toolbar-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .push-split-btn {
    display: flex;
    align-items: stretch;
    background: var(--secondary);
    border-radius: 6px;
  }
  .push-split-btn:has(button:disabled) {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .push-main,
  .push-chevron {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    font-weight: 500;
    color: var(--muted-foreground);
    background: transparent;
    cursor: pointer;
    transition: background 150ms, color 150ms;
  }
  .push-main {
    padding: 4px 8px;
  }
  .push-chevron {
    padding: 4px 4px;
  }
  .push-main:hover:not(:disabled),
  .push-chevron:hover:not(:disabled) {
    background: var(--accent);
    color: var(--foreground);
  }
  .push-main:disabled,
  .push-chevron:disabled {
    cursor: not-allowed;
  }
  .push-divider {
    width: 1px;
    align-self: stretch;
    margin: 4px 0;
    background: var(--border);
  }

  .spinner {
    flex-shrink: 0;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
