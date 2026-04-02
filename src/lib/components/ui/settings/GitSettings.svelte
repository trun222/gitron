<script lang="ts">
  import {
    autoFetchInterval, setAutoFetchInterval,
    fileWatcherInterval, setFileWatcherInterval,
    verboseGitErrors, setVerboseGitErrors,
    terminalApp, setTerminalApp,
    terminalShell, setTerminalShell,
    terminalFontSize, setTerminalFontSize,
    terminalFontFamily, setTerminalFontFamily,
    terminalCursorStyle, setTerminalCursorStyle,
    terminalScrollback, setTerminalScrollback,
    excludedAuthors, removeExcludedAuthor, addExcludedAuthor, setExcludedAuthors,
  } from '$lib/stores/settings';
  import { isTauri } from '$lib/api';
  import type { AutoFetchInterval, FileWatcherInterval, TerminalCursorStyle } from '$lib/api/types';

  const fetchOptions: { value: AutoFetchInterval; label: string }[] = [
    { value: 0, label: 'Off' },
    { value: 15, label: '15 seconds' },
    { value: 30, label: '30 seconds' },
    { value: 60, label: '1 minute' },
    { value: 300, label: '5 minutes' },
    { value: 900, label: '15 minutes' },
  ];

  const watcherOptions: { value: FileWatcherInterval; label: string }[] = [
    { value: 0, label: 'Native only' },
    { value: 1000, label: '1 second' },
    { value: 2000, label: '2 seconds' },
    { value: 3000, label: '3 seconds' },
    { value: 5000, label: '5 seconds' },
  ];

  let newAuthorInput = $state('');

  function handleAddAuthor() {
    const trimmed = newAuthorInput.trim();
    if (trimmed) {
      addExcludedAuthor(trimmed);
      newAuthorInput = '';
    }
  }

  const knownTerminals = ['', 'iTerm', 'Warp', 'Ghostty', 'Alacritty', 'Kitty', 'Hyper'];
  let isCustomTerminal = $derived(!knownTerminals.includes($terminalApp));
  let showCustomInput = $state(false);

  function handleTerminalSelect(value: string) {
    if (value === '__custom__') {
      showCustomInput = true;
    } else {
      showCustomInput = false;
      setTerminalApp(value);
    }
  }
</script>

<div class="section">
  <h3 class="section-title">Fetch</h3>
  <div class="setting-row">
    <div class="setting-label">
      <span class="label-text">Auto-fetch interval</span>
      <span class="label-description">Automatically fetch from remotes at a regular interval</span>
    </div>
    <select
      class="select-input"
      value={$autoFetchInterval}
      onchange={(e) => setAutoFetchInterval(Number((e.target as HTMLSelectElement).value) as AutoFetchInterval)}
    >
      {#each fetchOptions as opt (opt.value)}
        <option value={opt.value}>{opt.label}</option>
      {/each}
    </select>
  </div>
</div>

<div class="section">
  <h3 class="section-title">File Watcher</h3>
  <div class="setting-row">
    <div class="setting-label">
      <span class="label-text">Poll fallback interval</span>
      <span class="label-description">Use polling if native file watching fails (e.g. Linux inotify limits). "Native only" disables the fallback.</span>
    </div>
    <select
      class="select-input"
      value={$fileWatcherInterval}
      onchange={(e) => setFileWatcherInterval(Number((e.target as HTMLSelectElement).value) as FileWatcherInterval)}
    >
      {#each watcherOptions as opt (opt.value)}
        <option value={opt.value}>{opt.label}</option>
      {/each}
    </select>
  </div>
</div>

<div class="section">
  <h3 class="section-title">Integrated Terminal</h3>
  <div class="setting-row">
    <div class="setting-label">
      <span class="label-text">Shell</span>
      <span class="label-description">Shell to use in the integrated terminal. Leave empty for system default ($SHELL).</span>
    </div>
    <input
      type="text"
      class="text-input"
      placeholder="e.g. /bin/zsh, /bin/bash"
      value={$terminalShell}
      onchange={(e) => setTerminalShell((e.target as HTMLInputElement).value)}
    />
  </div>
  <div class="setting-row">
    <div class="setting-label">
      <span class="label-text">Font size</span>
      <span class="label-description">Font size in the terminal panel</span>
    </div>
    <select
      class="select-input"
      value={$terminalFontSize}
      onchange={(e) => setTerminalFontSize(Number((e.target as HTMLSelectElement).value))}
    >
      <option value={10}>10</option>
      <option value={11}>11</option>
      <option value={12}>12</option>
      <option value={13}>13</option>
      <option value={14}>14</option>
      <option value={16}>16</option>
      <option value={18}>18</option>
      <option value={20}>20</option>
    </select>
  </div>
  <div class="setting-row">
    <div class="setting-label">
      <span class="label-text">Font family</span>
      <span class="label-description">Custom font family. Leave empty to use the monospace font setting.</span>
    </div>
    <input
      type="text"
      class="text-input"
      placeholder="e.g. Fira Code, JetBrains Mono"
      value={$terminalFontFamily}
      onchange={(e) => setTerminalFontFamily((e.target as HTMLInputElement).value)}
    />
  </div>
  <div class="setting-row">
    <div class="setting-label">
      <span class="label-text">Cursor style</span>
      <span class="label-description">Shape of the cursor in the terminal</span>
    </div>
    <select
      class="select-input"
      value={$terminalCursorStyle}
      onchange={(e) => setTerminalCursorStyle((e.target as HTMLSelectElement).value as TerminalCursorStyle)}
    >
      <option value="block">Block</option>
      <option value="underline">Underline</option>
      <option value="bar">Bar</option>
    </select>
  </div>
  <div class="setting-row">
    <div class="setting-label">
      <span class="label-text">Scrollback buffer</span>
      <span class="label-description">Number of lines kept in scroll history</span>
    </div>
    <select
      class="select-input"
      value={$terminalScrollback}
      onchange={(e) => setTerminalScrollback(Number((e.target as HTMLSelectElement).value))}
    >
      <option value={1000}>1,000</option>
      <option value={5000}>5,000</option>
      <option value={10000}>10,000</option>
      <option value={50000}>50,000</option>
    </select>
  </div>
</div>

{#if isTauri()}
<div class="section">
  <h3 class="section-title">External Terminal</h3>
  <div class="setting-row">
    <div class="setting-label">
      <span class="label-text">Terminal application</span>
      <span class="label-description">Used for "Open in Terminal" on worktrees</span>
    </div>
    <div class="terminal-controls">
      <select
        class="select-input"
        value={isCustomTerminal || showCustomInput ? '__custom__' : $terminalApp}
        onchange={(e) => handleTerminalSelect((e.target as HTMLSelectElement).value)}
      >
        <option value="">System Default</option>
        <option value="iTerm">iTerm2</option>
        <option value="Warp">Warp</option>
        <option value="Ghostty">Ghostty</option>
        <option value="Alacritty">Alacritty</option>
        <option value="Kitty">Kitty</option>
        <option value="Hyper">Hyper</option>
        <option value="__custom__">Custom...</option>
      </select>
      {#if isCustomTerminal || showCustomInput}
        <input
          type="text"
          class="text-input"
          placeholder="App name or path"
          value={$terminalApp}
          onchange={(e) => setTerminalApp((e.target as HTMLInputElement).value)}
        />
      {/if}
    </div>
  </div>
</div>
{/if}

<div class="section">
  <h3 class="section-title">Hidden Authors</h3>
  <div class="setting-row" style="flex-direction: column; align-items: stretch; gap: 8px;">
    <div class="setting-label">
      <span class="label-text">Excluded authors</span>
      <span class="label-description">Commits by these authors will be hidden from the graph. Right-click a commit to hide an author.</span>
    </div>
    {#if $excludedAuthors.length > 0}
      <div class="excluded-authors-list">
        {#each $excludedAuthors as author}
          <span class="excluded-author-chip">
            {author}
            <button
              class="chip-remove"
              title="Remove"
              onclick={() => removeExcludedAuthor(author)}
            >
              <svg viewBox="0 0 16 16" width="10" height="10" fill="currentColor"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"/></svg>
            </button>
          </span>
        {/each}
        <button class="clear-all-btn" onclick={() => setExcludedAuthors([])}>Clear all</button>
      </div>
    {:else}
      <span class="no-authors-text">No authors hidden</span>
    {/if}
    <div class="add-author-row">
      <input
        type="text"
        class="text-input"
        placeholder="Author name to hide..."
        bind:value={newAuthorInput}
        onkeydown={(e) => { if (e.key === 'Enter') handleAddAuthor(); }}
      />
      <button class="add-author-btn" onclick={handleAddAuthor} disabled={!newAuthorInput.trim()}>Add</button>
    </div>
  </div>
</div>

<div class="section">
  <h3 class="section-title">Errors</h3>
  <div class="setting-row">
    <div class="setting-label">
      <span class="label-text">Verbose git errors</span>
      <span class="label-description">Show full command details and exit codes in error messages</span>
    </div>
    <button
      class="toggle"
      class:on={$verboseGitErrors}
      onclick={() => setVerboseGitErrors(!$verboseGitErrors)}
      role="switch"
      aria-checked={$verboseGitErrors}
      aria-label="Toggle verbose git errors"
    >
      <span class="toggle-thumb"></span>
    </button>
  </div>
</div>

<style>
  .section {
    margin-bottom: 24px;
  }

  .section-title {
    font-size: 11px;
    font-weight: 500;
    color: var(--muted-foreground);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 12px;
  }

  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 8px 0;
  }

  .setting-label {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .label-text {
    font-size: 13px;
    color: var(--foreground);
  }

  .label-description {
    font-size: 11px;
    color: var(--muted-foreground);
  }

  .select-input {
    padding: 4px 8px;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--secondary);
    color: var(--foreground);
    font-size: 12px;
    cursor: pointer;
    flex-shrink: 0;
  }
  .select-input:focus {
    outline: none;
    border-color: var(--primary);
  }

  .terminal-controls {
    display: flex;
    flex-direction: column;
    gap: 6px;
    align-items: flex-end;
    flex-shrink: 0;
  }

  .text-input {
    padding: 4px 8px;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--secondary);
    color: var(--foreground);
    font-size: 12px;
    flex-shrink: 0;
    width: 160px;
  }
  .text-input::placeholder {
    color: var(--muted-foreground);
  }
  .text-input:focus {
    outline: none;
    border-color: var(--primary);
  }

  .toggle {
    position: relative;
    width: 36px;
    height: 20px;
    border-radius: 10px;
    background: var(--input);
    cursor: pointer;
    transition: background 0.2s;
    flex-shrink: 0;
  }
  .toggle.on {
    background: var(--primary);
  }

  .toggle-thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--foreground);
    transition: transform 0.2s;
  }
  .toggle.on .toggle-thumb {
    transform: translateX(16px);
    background: var(--primary-foreground);
  }

  .excluded-authors-list {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: center;
  }

  .excluded-author-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 8px;
    border-radius: 4px;
    background: var(--secondary);
    border: 1px solid var(--border);
    color: var(--foreground);
    font-size: 12px;
  }

  .chip-remove {
    display: inline-flex;
    align-items: center;
    cursor: pointer;
    color: var(--muted-foreground);
    transition: color 0.15s;
  }
  .chip-remove:hover {
    color: var(--destructive);
  }

  .no-authors-text {
    font-size: 12px;
    color: var(--muted-foreground);
    font-style: italic;
  }

  .add-author-row {
    display: flex;
    gap: 6px;
    align-items: center;
  }

  .add-author-btn {
    padding: 4px 12px;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--secondary);
    color: var(--foreground);
    font-size: 12px;
    cursor: pointer;
    transition: background 0.15s;
    flex-shrink: 0;
  }
  .add-author-btn:hover:not(:disabled) {
    background: var(--accent);
  }
  .add-author-btn:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .clear-all-btn {
    font-size: 11px;
    color: var(--muted-foreground);
    cursor: pointer;
    text-decoration: underline;
  }
  .clear-all-btn:hover {
    color: var(--foreground);
  }
</style>
