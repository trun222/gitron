<script lang="ts">
  import { Dialog } from 'bits-ui';
  import { commitGraph } from '$lib/stores/repo';
  import {
    releaseNotesDialog,
    releaseNotesRange,
    releaseNotesRangeLoading,
    releaseNotesRangeError,
    releaseNotesResult,
    releaseNotesGenerating,
    releaseNotesError,
    closeReleaseNotes,
    previewReleaseNotesRange,
    generateReleaseNotes,
    hasConfiguredProvider,
  } from '$lib/stores/ai';
  import { addToast } from '$lib/stores/toast';

  let from = $state('');
  let to = $state('HEAD');
  let markdown = $state('');
  let commitsExpanded = $state(false);
  let previewTimer: ReturnType<typeof setTimeout> | undefined;

  // Seed inputs from the store each time the dialog opens
  $effect(() => {
    if ($releaseNotesDialog.open) {
      from = $releaseNotesDialog.from;
      to = $releaseNotesDialog.to;
      markdown = '';
      commitsExpanded = false;
    }
  });

  // Debounced range preview whenever either ref changes
  $effect(() => {
    if (!$releaseNotesDialog.open) return;
    const f = from;
    const t = to;
    clearTimeout(previewTimer);
    previewTimer = setTimeout(() => previewReleaseNotesRange(f, t), 300);
    return () => clearTimeout(previewTimer);
  });

  // Mirror generated markdown into the editable textarea
  $effect(() => {
    if ($releaseNotesResult) markdown = $releaseNotesResult.markdown;
  });

  // Ref suggestions: tags first (newest first), then local branches, then HEAD
  let refSuggestions = $derived.by(() => {
    const graph = $commitGraph;
    if (!graph) return ['HEAD'];
    const commitIndex = new Map(graph.commits.map((c, i) => [c.oid, i]));
    const tags = [...graph.tags]
      .sort((a, b) => (commitIndex.get(a.target_oid) ?? Infinity) - (commitIndex.get(b.target_oid) ?? Infinity))
      .map((t) => t.name);
    const branches = graph.branches.filter((b) => !b.is_remote).map((b) => b.name);
    return ['HEAD', ...tags, ...branches];
  });

  let canGenerate = $derived(
    !!from.trim() &&
    !!to.trim() &&
    !$releaseNotesGenerating &&
    !$releaseNotesRangeLoading &&
    !$releaseNotesRangeError &&
    ($releaseNotesRange?.commits.length ?? 0) > 0
  );

  function swapRefs() {
    const f = from;
    from = to;
    to = f;
  }

  async function handleGenerate() {
    if (!canGenerate) return;
    await generateReleaseNotes(from, to);
  }

  async function handleCopy() {
    if (!markdown.trim()) return;
    try {
      await navigator.clipboard.writeText(markdown);
      addToast('Release notes copied to clipboard', 'success');
    } catch {
      addToast('Failed to copy to clipboard', 'error');
    }
  }

  function handleOpenChange(open: boolean) {
    if (!open) closeReleaseNotes();
  }
</script>

<Dialog.Root open={$releaseNotesDialog.open} onOpenChange={handleOpenChange}>
  <Dialog.Portal>
    <Dialog.Overlay class="fixed inset-0 bg-black/50 z-50" />
    <Dialog.Content
      class="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 z-50 w-full max-w-2xl max-h-[85vh] flex flex-col rounded-lg border border-border bg-card p-6 shadow-lg"
    >
      <Dialog.Title class="text-sm font-semibold text-foreground mb-1">
        Generate Release Notes
      </Dialog.Title>
      <Dialog.Description class="text-xs text-muted-foreground mb-4">
        Notes cover every commit after <em>From</em>, up to and including <em>To</em>. Accepts tags, branches, commit SHAs, or any revision like <code class="font-mono">HEAD~5</code>.
      </Dialog.Description>

      <datalist id="release-notes-refs">
        {#each refSuggestions as ref (ref)}
          <option value={ref}></option>
        {/each}
      </datalist>

      <div class="flex items-end gap-2 mb-2">
        <label class="flex-1 flex flex-col gap-1">
          <span class="text-[11px] uppercase tracking-wide text-muted-foreground">From <span class="normal-case">(exclusive)</span></span>
          <input
            type="text"
            class="ref-input"
            list="release-notes-refs"
            placeholder="v1.0.0"
            bind:value={from}
            disabled={$releaseNotesGenerating}
            spellcheck="false"
          />
        </label>
        <button
          type="button"
          class="swap-btn"
          title="Swap from and to"
          onclick={swapRefs}
          disabled={$releaseNotesGenerating}
        >
          <svg viewBox="0 0 16 16" width="14" height="14"><path fill="currentColor" d="M5.22 14.78a.75.75 0 0 0 1.06-1.06L4.56 12h8.69a.75.75 0 0 0 0-1.5H4.56l1.72-1.72a.75.75 0 0 0-1.06-1.06l-3 3a.75.75 0 0 0 0 1.06l3 3Zm5.56-6.5a.75.75 0 1 1-1.06-1.06l1.72-1.72H2.75a.75.75 0 0 1 0-1.5h8.69L9.72 2.28a.75.75 0 0 1 1.06-1.06l3 3a.75.75 0 0 1 0 1.06l-3 3Z" /></svg>
        </button>
        <label class="flex-1 flex flex-col gap-1">
          <span class="text-[11px] uppercase tracking-wide text-muted-foreground">To <span class="normal-case">(inclusive)</span></span>
          <input
            type="text"
            class="ref-input"
            list="release-notes-refs"
            placeholder="HEAD"
            bind:value={to}
            disabled={$releaseNotesGenerating}
            spellcheck="false"
          />
        </label>
      </div>

      <!-- Range preview -->
      <div class="text-xs mb-4 min-h-[1.25rem]">
        {#if $releaseNotesRangeLoading}
          <span class="text-muted-foreground">Resolving range…</span>
        {:else if $releaseNotesRangeError}
          <span class="text-destructive">{$releaseNotesRangeError}</span>
        {:else if $releaseNotesRange}
          {@const r = $releaseNotesRange}
          {#if r.commits.length === 0}
            <span class="text-muted-foreground">No commits between these revisions.</span>
          {:else}
            <button
              type="button"
              class="inline-flex items-center gap-1 text-muted-foreground hover:text-foreground transition-colors"
              onclick={() => commitsExpanded = !commitsExpanded}
            >
              <svg class="transition-transform {commitsExpanded ? 'rotate-90' : ''}" viewBox="0 0 16 16" width="10" height="10"><path fill="currentColor" d="M6.22 3.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.75.75 0 0 1-1.06-1.06L9.94 8 6.22 4.28a.75.75 0 0 1 0-1.06Z" /></svg>
              <span class="text-foreground font-medium">{r.commits.length} {r.commits.length === 1 ? 'commit' : 'commits'}</span>
              <span>·</span>
              <span>{r.files_changed} {r.files_changed === 1 ? 'file' : 'files'}</span>
              <span class="text-green-500">+{r.insertions}</span>
              <span class="text-red-500">−{r.deletions}</span>
            </button>
            {#if commitsExpanded}
              <ul class="mt-2 max-h-40 overflow-y-auto rounded-md border border-border bg-background p-2 font-mono text-[11px] space-y-0.5">
                {#each r.commits as c (c.oid)}
                  <li class="flex gap-2 truncate">
                    <span class="text-muted-foreground shrink-0">{c.short_oid}</span>
                    <span class="truncate text-foreground">{c.summary}</span>
                  </li>
                {/each}
              </ul>
            {/if}
          {/if}
        {/if}
      </div>

      <!-- Output -->
      <div class="flex-1 min-h-0 flex flex-col">
        <div class="flex items-center justify-between mb-1">
          <span class="text-[11px] uppercase tracking-wide text-muted-foreground">Release notes (Markdown)</span>
          {#if markdown.trim()}
            <button type="button" class="text-xs text-muted-foreground hover:text-foreground transition-colors" onclick={handleCopy}>
              Copy
            </button>
          {/if}
        </div>
        <textarea
          class="notes-output"
          rows="12"
          bind:value={markdown}
          placeholder={$releaseNotesGenerating ? 'Generating…' : 'Generated notes will appear here. You can edit them before copying.'}
          readonly={$releaseNotesGenerating}
          spellcheck="false"
        ></textarea>
      </div>

      {#if $releaseNotesError}
        <p class="mt-2 text-xs text-destructive">{$releaseNotesError}</p>
      {:else if !$hasConfiguredProvider}
        <p class="mt-2 text-xs text-muted-foreground">No AI provider configured. Open Settings → AI to set one up.</p>
      {/if}

      <div class="flex items-center gap-2 justify-end mt-4">
        <button
          type="button"
          class="px-3 py-1.5 text-sm rounded-md border border-border bg-background text-foreground hover:bg-accent transition-colors"
          onclick={closeReleaseNotes}
        >
          Close
        </button>
        <button
          type="button"
          class="px-3 py-1.5 text-sm rounded-md bg-primary text-primary-foreground hover:bg-primary/90 transition-colors font-medium disabled:opacity-50 disabled:cursor-not-allowed inline-flex items-center gap-2"
          onclick={handleGenerate}
          disabled={!canGenerate}
        >
          {#if $releaseNotesGenerating}
            <svg class="spinner" viewBox="0 0 16 16" width="12" height="12"><circle cx="8" cy="8" r="6" fill="none" stroke="currentColor" stroke-width="2" stroke-dasharray="28" stroke-dashoffset="8" stroke-linecap="round"/></svg>
            Generating…
          {:else if $releaseNotesResult}
            Regenerate
          {:else}
            Generate
          {/if}
        </button>
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>

<style>
  .ref-input {
    width: 100%;
    padding: 6px 8px;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--background);
    color: var(--foreground);
    font-family: var(--font-mono, ui-monospace, monospace);
    font-size: 12px;
    outline: none;
    transition: border-color 150ms;
  }
  .ref-input:focus {
    border-color: var(--primary);
  }
  .ref-input:disabled {
    opacity: 0.6;
  }

  .swap-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    border-radius: 6px;
    color: var(--muted-foreground);
    cursor: pointer;
    transition: background 150ms, color 150ms;
  }
  .swap-btn:hover:not(:disabled) {
    background: var(--accent);
    color: var(--foreground);
  }
  .swap-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .notes-output {
    width: 100%;
    flex: 1;
    min-height: 200px;
    resize: vertical;
    padding: 8px 10px;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--background);
    color: var(--foreground);
    font-family: var(--font-mono, ui-monospace, monospace);
    font-size: 12px;
    line-height: 1.5;
    outline: none;
    transition: border-color 150ms;
  }
  .notes-output:focus {
    border-color: var(--primary);
  }

  .spinner {
    flex-shrink: 0;
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
