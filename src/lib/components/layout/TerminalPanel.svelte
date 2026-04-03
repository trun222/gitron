<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
  import { get } from 'svelte/store';
  import {
    terminalSessionId,
    terminalConnecting,
    spawnTerminal,
    writeTerminal,
    resizeTerminal,
    killTerminal,
    setTerminalCallbacks,
    clearTerminalCallbacks,
    bottomPanelOpen,
    activeBottomTab,
  } from '$lib/stores/terminal';
  import {
    terminalFontSize,
    terminalFontFamily,
    terminalCursorStyle,
    terminalScrollback,
  } from '$lib/stores/settings';
  import '@xterm/xterm/css/xterm.css';

  let containerRef: HTMLDivElement | undefined = $state();
  let terminal: import('@xterm/xterm').Terminal | undefined;
  let fitAddon: import('@xterm/addon-fit').FitAddon | undefined;
  let initialized = $state(false);

  // Auto-start terminal when the component mounts (i.e. terminal tab becomes active)
  onMount(() => {
    initAndSpawn();
  });

  // Reactive: refit when panel becomes visible
  $effect(() => {
    if ($bottomPanelOpen && $activeBottomTab === 'terminal' && terminal && fitAddon) {
      tick().then(() => {
        try { fitAddon?.fit(); } catch { /* ignore */ }
      });
    }
  });

  onDestroy(() => {
    clearTerminalCallbacks();
    terminal?.dispose();
    terminal = undefined;
    fitAddon = undefined;
    initialized = false;
  });

  /** Initialize xterm.js and spawn a PTY session. */
  async function initAndSpawn() {
    if (!containerRef) return;
    if (get(terminalConnecting)) return;

    // If xterm not initialized yet, create it
    if (!terminal) {
      const { Terminal } = await import('@xterm/xterm');
      const { FitAddon } = await import('@xterm/addon-fit');
      const { WebLinksAddon } = await import('@xterm/addon-web-links');

      // Resolve font family
      const customFont = get(terminalFontFamily);
      const fontFamily = customFont || getComputedStyle(document.documentElement).getPropertyValue('--font-mono').trim() || 'monospace';

      terminal = new Terminal({
        fontSize: get(terminalFontSize),
        fontFamily,
        cursorStyle: get(terminalCursorStyle),
        scrollback: get(terminalScrollback),
        theme: getThemeColors(),
        convertEol: false,
      });

      fitAddon = new FitAddon();
      terminal.loadAddon(fitAddon);
      terminal.loadAddon(new WebLinksAddon());

      terminal.open(containerRef);

      // Forward user input to the PTY
      terminal.onData((data: string) => {
        writeTerminal(data);
      });

      // Handle resize
      const resizeObserver = new ResizeObserver(() => {
        if (!fitAddon || !terminal) return;
        try {
          fitAddon.fit();
          const dims = fitAddon.proposeDimensions();
          if (dims && get(terminalSessionId)) {
            resizeTerminal(dims.cols, dims.rows);
          }
        } catch { /* ignore fit errors during transitions */ }
      });
      resizeObserver.observe(containerRef);

      // Clean up resize observer on destroy
      const origDispose = terminal.dispose.bind(terminal);
      terminal.dispose = () => {
        resizeObserver.disconnect();
        origDispose();
      };

      initialized = true;
    }

    // Set up callbacks from the store to receive PTY output
    setTerminalCallbacks(
      (data: Uint8Array) => {
        terminal?.write(data);
      },
      (_code: number) => {
        terminal?.writeln('\r\n\x1b[90m[Process exited]\x1b[0m');
      },
    );

    // Fit after a tick so the container has layout dimensions
    await tick();
    try { fitAddon?.fit(); } catch { /* ignore */ }

    // Spawn the PTY session
    const dims = fitAddon?.proposeDimensions();
    const cols = dims?.cols ?? 80;
    const rows = dims?.rows ?? 24;
    await spawnTerminal(cols, rows);

    // Focus the terminal
    terminal?.focus();
  }

  function getThemeColors(): import('@xterm/xterm').ITheme {
    const style = getComputedStyle(document.documentElement);
    const cssVar = (prop: string) => style.getPropertyValue(prop).trim();
    return {
      background: cssVar('--card') || '#1a1a2e',
      foreground: cssVar('--foreground') || '#e0e0e0',
      cursor: cssVar('--primary') || '#00d4ff',
      cursorAccent: cssVar('--card') || '#1a1a2e',
      selectionBackground: cssVar('--accent') || '#333355',
    };
  }

  async function handleRestart() {
    await killTerminal();
    clearTerminalCallbacks();
    if (terminal) {
      terminal.clear();
      terminal.reset();
    }
    await tick();
    await initAndSpawn();
  }
</script>

<div class="terminal-panel">
  {#if $terminalConnecting && !initialized}
    <div class="terminal-status">
      <span class="text-muted-foreground text-xs">Starting terminal...</span>
    </div>
  {:else if !$terminalSessionId && initialized}
    <div class="terminal-status-overlay">
      <button
        class="restart-btn"
        onclick={handleRestart}
      >
        Restart Terminal
      </button>
    </div>
  {/if}
  <div
    bind:this={containerRef}
    class="terminal-container"
    class:visible={initialized}
  ></div>
</div>

<style>
  .terminal-panel {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    overflow: hidden;
    position: relative;
  }

  .terminal-container {
    flex: 1;
    min-height: 0;
    overflow: hidden;
    padding: 4px 0 0 8px;
    display: none;
  }

  .terminal-container.visible {
    display: block;
  }

  .terminal-container :global(.xterm) {
    height: 100%;
  }

  .terminal-container :global(.xterm-viewport) {
    overflow-y: auto !important;
  }

  .terminal-status {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 16px;
    flex: 1;
  }

  .terminal-status-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--card);
    z-index: 1;
  }

  .restart-btn {
    padding: 6px 16px;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--secondary);
    color: var(--foreground);
    font-size: 12px;
    cursor: pointer;
    transition: background 0.15s;
  }
  .restart-btn:hover {
    background: var(--accent);
  }
</style>
