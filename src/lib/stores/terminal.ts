/**
 * Terminal store — manages the integrated PTY terminal session.
 *
 * In Tauri mode: uses `invoke()` commands + Tauri event listeners.
 * In web mode:   uses a WebSocket to `/api/terminal`.
 */
import { writable, get } from 'svelte/store';
import { isTauri } from '$lib/api';
import { terminalShell } from '$lib/stores/settings';
import { repoPath } from '$lib/stores/repo';

export type BottomTab = 'output' | 'terminal';

/** Which tab is active in the bottom panel */
export const activeBottomTab = writable<BottomTab>('output');

/** Whether the bottom panel is visible (replaces raw outputPanelOpen for the tabbed panel) */
export const bottomPanelOpen = writable(false);

/** The PTY session ID (non-null while a session is alive) */
export const terminalSessionId = writable<string | null>(null);

/** Whether the terminal session is connecting */
export const terminalConnecting = writable(false);

// Internal references for cleanup
let _tauriUnlisten: (() => void) | null = null;
let _websocket: WebSocket | null = null;

/** Callback set by TerminalPanel to receive raw PTY data */
let _onData: ((data: Uint8Array) => void) | null = null;
let _onExit: ((code: number) => void) | null = null;

export function setTerminalCallbacks(
  onData: (data: Uint8Array) => void,
  onExit: (code: number) => void,
) {
  _onData = onData;
  _onExit = onExit;
}

export function clearTerminalCallbacks() {
  _onData = null;
  _onExit = null;
}

/**
 * Spawn a new integrated terminal session.
 */
export async function spawnTerminal(cols: number, rows: number): Promise<void> {
  // Kill any existing session first.
  await killTerminal();

  terminalConnecting.set(true);
  const shell = get(terminalShell) || undefined;
  const cwd = get(repoPath) || undefined;

  try {
    if (isTauri()) {
      await spawnTauri(shell, cwd, cols, rows);
    } else {
      await spawnWebSocket(shell, cwd, cols, rows);
    }
  } catch (e) {
    console.error('Failed to spawn terminal:', e);
    terminalSessionId.set(null);
  } finally {
    terminalConnecting.set(false);
  }
}

/**
 * Write raw bytes to the terminal (user input from xterm.js).
 */
export async function writeTerminal(data: string): Promise<void> {
  const id = get(terminalSessionId);
  if (!id) return;

  if (isTauri()) {
    const { invoke } = await import('@tauri-apps/api/core');
    const bytes = new TextEncoder().encode(data);
    await invoke('terminal_write', { id, data: Array.from(bytes) });
  } else if (_websocket && _websocket.readyState === WebSocket.OPEN) {
    const bytes = new TextEncoder().encode(data);
    _websocket.send(bytes);
  }
}

/**
 * Resize the terminal PTY.
 */
export async function resizeTerminal(cols: number, rows: number): Promise<void> {
  const id = get(terminalSessionId);
  if (!id) return;

  if (isTauri()) {
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('terminal_resize', { id, cols, rows });
  } else if (_websocket && _websocket.readyState === WebSocket.OPEN) {
    _websocket.send(JSON.stringify({ type: 'resize', cols, rows }));
  }
}

/**
 * Kill the terminal session.
 */
export async function killTerminal(): Promise<void> {
  const id = get(terminalSessionId);

  if (isTauri() && id) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('terminal_kill', { id });
    } catch {
      // Session may already be dead
    }
  }

  if (_tauriUnlisten) {
    _tauriUnlisten();
    _tauriUnlisten = null;
  }

  if (_websocket) {
    _websocket.close();
    _websocket = null;
  }

  terminalSessionId.set(null);
}

/**
 * Toggle the bottom panel open/closed, switching to the terminal tab.
 */
export function toggleTerminalPanel(): void {
  const isOpen = get(bottomPanelOpen);
  const tab = get(activeBottomTab);

  if (isOpen && tab === 'terminal') {
    // Already showing terminal — close the panel
    bottomPanelOpen.set(false);
  } else {
    // Open the panel and switch to terminal tab
    bottomPanelOpen.set(true);
    activeBottomTab.set('terminal');
  }
}

// --- Tauri transport ---

async function spawnTauri(
  shell: string | undefined,
  cwd: string | undefined,
  cols: number,
  rows: number,
): Promise<void> {
  const { invoke } = await import('@tauri-apps/api/core');
  const { listen } = await import('@tauri-apps/api/event');

  const id: string = await invoke('terminal_spawn', {
    shell: shell || null,
    cwd: cwd || null,
    cols,
    rows,
  });

  terminalSessionId.set(id);

  // Listen for PTY output events
  const unlistenData = await listen<number[]>(`terminal:data:${id}`, (event) => {
    if (_onData) {
      _onData(new Uint8Array(event.payload));
    }
  });

  const unlistenExit = await listen<number>(`terminal:exit:${id}`, (event) => {
    if (_onExit) _onExit(event.payload);
    terminalSessionId.set(null);
    _tauriUnlisten = null;
  });

  _tauriUnlisten = () => {
    unlistenData();
    unlistenExit();
  };
}

// --- WebSocket transport ---

async function spawnWebSocket(
  shell: string | undefined,
  cwd: string | undefined,
  cols: number,
  rows: number,
): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    const params = new URLSearchParams();
    if (shell) params.set('shell', shell);
    if (cwd) params.set('cwd', cwd);
    params.set('cols', String(cols));
    params.set('rows', String(rows));

    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    // Derive base path the same way HttpTransport does
    let basePath = '';
    const match = window.location.pathname.match(/^\/t\/\d+\/p\/\d+/);
    if (match) basePath = match[0];

    const url = `${protocol}//${window.location.host}${basePath}/api/terminal?${params}`;
    const ws = new WebSocket(url);
    ws.binaryType = 'arraybuffer';

    ws.onopen = () => {
      _websocket = ws;
      // Use a synthetic session ID for web mode
      const id = `ws-${Date.now()}`;
      terminalSessionId.set(id);
      resolve();
    };

    ws.onmessage = (event) => {
      if (event.data instanceof ArrayBuffer) {
        if (_onData) _onData(new Uint8Array(event.data));
      } else if (typeof event.data === 'string') {
        try {
          const msg = JSON.parse(event.data);
          if ('exit' in msg) {
            if (_onExit) _onExit(msg.exit);
            terminalSessionId.set(null);
            _websocket = null;
          }
        } catch {
          // Ignore
        }
      }
    };

    ws.onerror = () => {
      reject(new Error('WebSocket connection failed'));
    };

    ws.onclose = () => {
      terminalSessionId.set(null);
      _websocket = null;
    };
  });
}
