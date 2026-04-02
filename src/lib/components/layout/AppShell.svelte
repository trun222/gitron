<script lang="ts">
  import { onMount } from 'svelte';
  import Sidebar from './Sidebar.svelte';
  import Toolbar from './Toolbar.svelte';
  import StatusBar from './StatusBar.svelte';
  import BottomPanel from './BottomPanel.svelte';
  import {
    error, repoPath, hasRepo,
    pullFromRemote, pushToRemote, fetchFromRemote,
    stageAllAndClear, unstageAllAndClear, discardAllChanges,
    refreshAll,
  } from '$lib/stores/repo';
  import { toggleOutputPanel } from '$lib/stores/output';
  import { toggleTerminalPanel, bottomPanelOpen, activeBottomTab } from '$lib/stores/terminal';
  import { verboseGitErrors } from '$lib/stores/settings';
  import Toast from '$lib/components/ui/toast/Toast.svelte';
  import { get } from 'svelte/store';
  import { BranchConflictDialog, CleanupBranchesDialog, CloneDialog, DeleteBranchDialog, DiscardChangesDialog, ForcePushDialog, GitHubLoginDialog, PurgeCheckpointsDialog } from '$lib/components/ui/dialog';
  import { initAuth } from '$lib/stores/github';
  import type { Snippet } from 'svelte';

  let { children }: { children: Snippet } = $props();

  let errorExpanded = $state(false);

  let errorFirstLine = $derived.by(() => {
    if (!$error) return '';
    const firstLine = $error.split('\n')[0];
    if ($verboseGitErrors || !/^CLI error \(\d+\): `[^`]+`/.test(firstLine)) return firstLine;
    // Non-verbose: skip the technical prefix line, show first non-empty stderr line
    const stderrLine = $error.split('\n').slice(1).find((l) => l.trim());
    return stderrLine?.trim() || firstLine;
  });
  let errorIsMultiline = $derived(($error?.split('\n').length ?? 0) > 1);

  onMount(() => {
    initAuth();

    function handleKeydown(e: KeyboardEvent) {
      const target = e.target as HTMLElement;
      const isInput = target.tagName === 'INPUT' || target.tagName === 'TEXTAREA';
      if (isInput) return;
      if (!get(hasRepo)) return;

      const mod = e.metaKey || e.ctrlKey;

      // Cmd+Shift+P — Push
      if (mod && e.shiftKey && e.key === 'p') {
        e.preventDefault();
        pushToRemote();
        return;
      }

      // Cmd+Shift+L — Pull
      if (mod && e.shiftKey && e.key === 'l') {
        e.preventDefault();
        pullFromRemote();
        return;
      }

      // Cmd+Shift+F — Fetch
      if (mod && e.shiftKey && e.key === 'f') {
        e.preventDefault();
        fetchFromRemote();
        return;
      }

      // Cmd+Shift+A — Stage all
      if (mod && e.shiftKey && e.key === 'a') {
        e.preventDefault();
        stageAllAndClear();
        return;
      }

      // Cmd+Shift+U — Unstage all
      if (mod && e.shiftKey && e.key === 'u') {
        e.preventDefault();
        unstageAllAndClear();
        return;
      }

      // Cmd+Shift+D — Discard all changes
      if (mod && e.shiftKey && e.key === 'd') {
        e.preventDefault();
        discardAllChanges();
        return;
      }

      // Cmd+R — Refresh
      if (mod && !e.shiftKey && e.key === 'r') {
        e.preventDefault();
        const p = get(repoPath);
        if (p) refreshAll(p);
        return;
      }

      // Cmd+` — Toggle output panel (in bottom panel)
      if (e.metaKey && !e.ctrlKey && e.key === '`') {
        e.preventDefault();
        toggleOutputPanel();
        return;
      }

      // Ctrl+` — Toggle terminal panel
      if (e.ctrlKey && !e.metaKey && e.key === '`') {
        e.preventDefault();
        toggleTerminalPanel();
        return;
      }
    }

    document.addEventListener('keydown', handleKeydown);
    return () => document.removeEventListener('keydown', handleKeydown);
  });
</script>

<div class="flex flex-col h-screen overflow-hidden bg-background text-foreground">
  <Toolbar />
  {#if $error}
    <div class="flex flex-col border-b border-destructive/30 bg-destructive/15 text-destructive text-sm">
      <div class="flex items-center gap-2 px-4 py-2">
        {#if errorIsMultiline}
          <button
            type="button"
            class="shrink-0 text-xs transition-transform"
            class:rotate-90={errorExpanded}
            onclick={() => errorExpanded = !errorExpanded}
            title="Toggle error details"
          >
            <svg viewBox="0 0 16 16" width="12" height="12">
              <path fill="currentColor" d="M6.22 3.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.75.75 0 0 1-1.06-1.06L9.94 8 6.22 4.28a.75.75 0 0 1 0-1.06Z" />
            </svg>
          </button>
        {/if}
        <span class="flex-1 truncate">{errorFirstLine}</span>
        {#if errorIsMultiline && !errorExpanded}
          <button
            type="button"
            class="shrink-0 text-xs px-2 py-0.5 rounded hover:bg-destructive/20 transition-colors"
            onclick={() => errorExpanded = true}
          >
            Details
          </button>
        {/if}
        <button
          type="button"
          class="shrink-0 text-xs px-2 py-0.5 rounded hover:bg-destructive/20 transition-colors"
          onclick={() => { error.set(null); errorExpanded = false; }}
        >
          Dismiss
        </button>
      </div>
      {#if errorExpanded && errorIsMultiline}
        <pre class="px-4 pb-3 text-xs whitespace-pre-wrap font-mono overflow-auto max-h-[200px]">{$error}</pre>
      {/if}
    </div>
  {/if}
  <div class="flex flex-1 overflow-hidden">
    <Sidebar />
    <main class="flex-1 overflow-auto flex flex-col">
      {@render children()}
    </main>
  </div>
  <BottomPanel />
  <StatusBar />
</div>

<Toast />
<BranchConflictDialog />
<CleanupBranchesDialog />
<CloneDialog />
<DeleteBranchDialog />
<DiscardChangesDialog />
<ForcePushDialog />
<GitHubLoginDialog />
<PurgeCheckpointsDialog />
