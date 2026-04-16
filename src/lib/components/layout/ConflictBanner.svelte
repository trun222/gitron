<script lang="ts">
  import {
    repoStatus,
    isConflictState,
    rebaseContinue,
    rebaseAbort,
    mergeContinue,
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

  let operationStep = $derived($repoStatus?.operation_step ?? null);
  let operationTotal = $derived($repoStatus?.operation_total ?? null);
  let hasProgress = $derived(operationStep !== null && operationTotal !== null);
  let progressPercent = $derived(
    hasProgress ? Math.round(((operationStep! - 1) / operationTotal!) * 100) : 0
  );

  function handleContinue() {
    const state = $repoStatus?.state;
    if (state === 'Rebasing' || state === 'RebasingInteractive') {
      rebaseContinue();
    } else if (state === 'Merging') {
      mergeContinue();
    }
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
    if (state === 'Rebasing' || state === 'RebasingInteractive') return !hasConflicts;
    if (state === 'Merging') return !hasConflicts;
    return false;
  });

  let showContinue = $derived.by(() => {
    const state = $repoStatus?.state;
    return state === 'Rebasing' || state === 'RebasingInteractive' || state === 'Merging';
  });

  let continueLabel = $derived($repoStatus?.state === 'Merging' ? 'Complete Merge' : `Continue ${operationLabel}`);
</script>

{#if $isConflictState}
  <div
    class="flex items-center gap-3 px-4 py-2 border-b text-sm"
    style="border-color: var(--color-git-conflict); background: var(--color-git-conflict-bg); color: var(--color-git-conflict);"
  >
    <svg viewBox="0 0 16 16" width="16" height="16" class="shrink-0" fill="currentColor">
      <path d="M6.457 1.047c.659-1.234 2.427-1.234 3.086 0l6.082 11.378A1.75 1.75 0 0 1 14.082 15H1.918a1.75 1.75 0 0 1-1.543-2.575Zm1.763.707a.25.25 0 0 0-.44 0L1.698 13.132a.25.25 0 0 0 .22.368h12.164a.25.25 0 0 0 .22-.368Zm.53 3.996v2.5a.75.75 0 0 1-1.5 0v-2.5a.75.75 0 0 1 1.5 0ZM9 11a1 1 0 1 1-2 0 1 1 0 0 1 2 0Z" />
    </svg>
    <span class="flex-1 flex items-center gap-3">
      <span>
        <strong>{operationLabel}</strong> in progress
        {#if hasProgress}
          — step {operationStep}/{operationTotal}
        {/if}
        {#if hasConflicts}
          — {conflictCount} conflicted {conflictCount === 1 ? 'file' : 'files'}
        {/if}
      </span>
      {#if hasProgress}
        <span class="flex items-center gap-2 text-[11px] tabular-nums" style="min-width: 120px;">
          <span class="flex-1 h-1.5 rounded-full overflow-hidden" style="background: color-mix(in srgb, var(--color-git-conflict) 25%, transparent);">
            <span
              class="block h-full rounded-full transition-all duration-300"
              style="width: {progressPercent}%; background: var(--color-git-conflict);"
            ></span>
          </span>
          <span>{progressPercent}%</span>
        </span>
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
          title={hasConflicts ? 'Resolve all conflicts first' : continueLabel}
        >
          {continueLabel}
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
