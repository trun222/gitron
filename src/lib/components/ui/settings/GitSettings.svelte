<script lang="ts">
  import { autoFetchInterval, setAutoFetchInterval, fileWatcherInterval, setFileWatcherInterval, verboseGitErrors, setVerboseGitErrors, terminalApp, setTerminalApp } from '$lib/stores/settings';
  import { isTauri } from '$lib/api';
  import type { AutoFetchInterval, FileWatcherInterval } from '$lib/api/types';

  const fetchOptions: { value: AutoFetchInterval; label: string }[] = [
    { value: 0, label: 'Off' },
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

{#if isTauri()}
<div class="section">
  <h3 class="section-title">Terminal</h3>
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
</style>
