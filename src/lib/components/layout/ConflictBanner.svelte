<script lang="ts">
  import {
    repoStatus,
    isConflictState,
    rebaseContinue,
    rebaseAbort,
    mergeAbort,
    cherryPickAbort,
  } from '$lib/stores/repo';

  let operationLabel = $derived.by(() => {
    const state = $repoStatus?.state;
    if (state === 'Rebasing' || state === 'RebasingInteractive') return 'Rebase';
    if (state === 'Merging') return 'Merge';
    if (state === 'CherryPicking') return 'Cherry-pick';
    if (state === 'Reverting') return 'Revert';
    return 'Operation';
  });

  let conflictCount = $derived($repoStatus?.conflicted.length ?? 0);
  let hasConflicts = $derived(conflictCount > 0);

  function handleContinue() {
    const state = $repoStatus?.state;
    if (state === 'Rebasing' || state === 'RebasingInteractive') {
      rebaseContinue();
    }
    // Merge continue = just commit (handled elsewhere)
  }

  function handleAbort() {
    const state = $repoStatus?.state;
    if (state === 'Rebasing' || state === 'RebasingInteractive') {
      rebaseAbort();
    } else if (state === 'Merging') {
      mergeAbort();
    } else if (state === 'CherryPicking') {
      cherryPickAbort();
    }
  }

  let canContinue = $derived.by(() => {
    const state = $repoStatus?.state;
    // Can continue rebase when no conflicts remain
    if (state === 'Rebasing' || state === 'RebasingInteractive') return !hasConflicts;
    // Merge "continue" is done via commit, not a separate action
    return false;
  });

  let showContinue = $derived.by(() => {
    const state = $repoStatus?.state;
    return state === 'Rebasing' || state === 'RebasingInteractive';
  });
</script>

{#if $isConflictState}
  <div
    class="flex items-center gap-3 px-4 py-2 border-b text-sm"
    style="border-color: var(--color-git-conflict); background: var(--color-git-conflict-bg); color: var(--color-git-conflict);"
  >
    <svg viewBox="0 0 16 16" width="16" height="16" class="shrink-0" fill="currentColor">
      <path d="M6.457 1.047c.659-1.234 2.427-1.234 3.086 0l6.082 11.378A1.75 1.75 0 0 1 14.082 15H1.918a1.75 1.75 0 0 1-1.543-2.575Zm1.763.707a.25.25 0 0 0-.44 0L1.698 13.132a.25.25 0 0 0 .22.368h12.164a.25.25 0 0 0 .22-.368Zm.53 3.996v2.5a.75.75 0 0 1-1.5 0v-2.5a.75.75 0 0 1 1.5 0ZM9 11a1 1 0 1 1-2 0 1 1 0 0 1 2 0Z" />
    </svg>
    <span class="flex-1">
      <strong>{operationLabel}</strong> in progress
      {#if hasConflicts}
        — {conflictCount} conflicted {conflictCount === 1 ? 'file' : 'files'}
      {/if}
    </span>
    <div class="flex items-center gap-2">
      {#if showContinue}
        <button
          type="button"
          class="px-3 py-1 rounded text-xs font-medium transition-colors"
          style="background: var(--color-git-conflict); color: var(--background);"
          class:opacity-50={!canContinue}
          disabled={!canContinue}
          onclick={handleContinue}
          title={hasConflicts ? 'Resolve all conflicts first' : `Continue ${operationLabel.toLowerCase()}`}
        >
          Continue {operationLabel}
        </button>
      {/if}
      <button
        type="button"
        class="px-3 py-1 rounded text-xs font-medium border transition-colors hover:opacity-80"
        style="border-color: var(--color-git-conflict); color: var(--color-git-conflict);"
        onclick={handleAbort}
      >
        Abort {operationLabel}
      </button>
    </div>
  </div>
{/if}
