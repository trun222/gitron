<script lang="ts">
  import { outputEntries, clearOutput } from '$lib/stores/output';
  import { hasEntries } from '$lib/stores/output';
  import {
    activeBottomTab,
    bottomPanelOpen,
    terminalSessionId,
    killTerminal,
  } from '$lib/stores/terminal';
  import { tick } from 'svelte';
  import TerminalPanel from './TerminalPanel.svelte';
  import type { BottomTab } from '$lib/stores/terminal';

  let scrollContainer: HTMLDivElement | undefined = $state();
  let prevCount = $state(0);

  // Auto-scroll output to bottom when new entries are added
  $effect(() => {
    const count = $outputEntries.length;
    if (count > prevCount && scrollContainer && $activeBottomTab === 'output') {
      tick().then(() => {
        if (scrollContainer) {
          scrollContainer.scrollTop = scrollContainer.scrollHeight;
        }
      });
    }
    prevCount = count;
  });

  function formatTime(date: Date): string {
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
  }

  function switchTab(tab: BottomTab) {
    activeBottomTab.set(tab);
  }

  function closePanel() {
    bottomPanelOpen.set(false);
  }
</script>

{#if $bottomPanelOpen}
  <div class="bottom-panel">
    <!-- Tab bar -->
    <div class="tab-bar">
      <div class="tabs">
        <button
          class="tab"
          class:active={$activeBottomTab === 'output'}
          onclick={() => switchTab('output')}
        >
          Output
          {#if $hasEntries}
            <span class="tab-badge"></span>
          {/if}
        </button>
        <button
          class="tab"
          class:active={$activeBottomTab === 'terminal'}
          onclick={() => switchTab('terminal')}
        >
          Terminal
          {#if $terminalSessionId}
            <span class="tab-badge active-badge"></span>
          {/if}
        </button>
      </div>
      <div class="tab-actions">
        {#if $activeBottomTab === 'output'}
          <button
            class="action-btn"
            onclick={clearOutput}
            title="Clear output"
          >
            Clear
          </button>
        {/if}
        {#if $activeBottomTab === 'terminal' && $terminalSessionId}
          <button
            class="action-btn"
            onclick={killTerminal}
            title="Kill terminal"
          >
            Kill
          </button>
        {/if}
        <button
          class="close-btn"
          onclick={closePanel}
          aria-label="Close panel"
        >
          <svg viewBox="0 0 16 16" width="14" height="14">
            <path fill="currentColor" d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Tab content -->
    <div class="tab-content">
      {#if $activeBottomTab === 'output'}
        <div
          bind:this={scrollContainer}
          class="output-scroll"
        >
          {#if $outputEntries.length === 0}
            <div class="empty-state">
              <span class="text-muted-foreground text-xs">No output yet</span>
            </div>
          {:else}
            {#each $outputEntries as entry (entry.id)}
              <div class="output-entry">
                <div class="output-meta">
                  <span>{formatTime(entry.timestamp)}</span>
                  <span class="output-op">{entry.operation}</span>
                  {#if entry.success}
                    <span class="text-git-added">OK</span>
                  {:else}
                    <span class="text-destructive">FAILED</span>
                  {/if}
                </div>
                {#if entry.stdout}
                  <pre class="output-text">{entry.stdout}</pre>
                {/if}
                {#if entry.stderr}
                  <pre class="output-text {entry.success ? 'text-muted-foreground' : 'text-destructive'}">{entry.stderr}</pre>
                {/if}
              </div>
            {/each}
          {/if}
        </div>
      {:else}
        <TerminalPanel />
      {/if}
    </div>
  </div>
{/if}

<style>
  .bottom-panel {
    display: flex;
    flex-direction: column;
    height: 250px;
    border-top: 1px solid var(--border);
    background: var(--card);
    flex-shrink: 0;
  }

  .tab-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 8px;
    border-bottom: 1px solid var(--border);
    height: 32px;
    flex-shrink: 0;
  }

  .tabs {
    display: flex;
    align-items: center;
    gap: 0;
    height: 100%;
  }

  .tab {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 0 12px;
    height: 100%;
    font-size: 11px;
    font-weight: 500;
    color: var(--muted-foreground);
    border-bottom: 2px solid transparent;
    cursor: pointer;
    transition: color 0.15s, border-color 0.15s;
    margin-bottom: -1px;
  }
  .tab:hover {
    color: var(--foreground);
  }
  .tab.active {
    color: var(--foreground);
    border-bottom-color: var(--primary);
  }

  .tab-badge {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--muted-foreground);
  }
  .tab-badge.active-badge {
    background: var(--primary);
  }

  .tab-actions {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .action-btn {
    font-size: 11px;
    color: var(--muted-foreground);
    padding: 2px 8px;
    border-radius: 4px;
    cursor: pointer;
    transition: color 0.15s, background 0.15s;
  }
  .action-btn:hover {
    color: var(--foreground);
    background: var(--accent);
  }

  .close-btn {
    padding: 2px;
    border-radius: 4px;
    color: var(--muted-foreground);
    cursor: pointer;
    transition: color 0.15s;
  }
  .close-btn:hover {
    color: var(--foreground);
  }

  .tab-content {
    flex: 1;
    min-height: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .output-scroll {
    flex: 1;
    overflow-y: auto;
    padding: 8px 12px;
    font-family: var(--font-mono, monospace);
    font-size: var(--editor-font-size, 12px);
    line-height: 1.6;
  }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 16px;
    flex: 1;
  }

  .output-entry {
    margin-bottom: 8px;
  }

  .output-meta {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--muted-foreground);
    font-size: 10px;
    margin-bottom: 2px;
  }

  .output-op {
    font-weight: 600;
    text-transform: uppercase;
  }

  .output-text {
    white-space: pre-wrap;
    margin: 0;
    color: var(--foreground);
  }
</style>
