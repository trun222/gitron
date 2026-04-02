import { writable, derived, get } from 'svelte/store';
import { autoShowOutput } from '$lib/stores/settings';
import { bottomPanelOpen, activeBottomTab } from '$lib/stores/terminal';

export interface OutputEntry {
  id: number;
  timestamp: Date;
  operation: string;
  stdout: string;
  stderr: string;
  success: boolean;
}

let nextId = 0;

export const outputEntries = writable<OutputEntry[]>([]);

export const hasEntries = derived(outputEntries, ($entries) => $entries.length > 0);

export function addOutput(
  operation: string,
  stdout: string,
  stderr: string,
  success: boolean
) {
  const hasContent = stdout.trim() || stderr.trim();
  if (!hasContent) return;

  const entry: OutputEntry = {
    id: nextId++,
    timestamp: new Date(),
    operation,
    stdout: stdout.trim(),
    stderr: stderr.trim(),
    success,
  };

  outputEntries.update((entries) => [...entries, entry]);
  if (get(autoShowOutput)) {
    bottomPanelOpen.set(true);
    activeBottomTab.set('output');
  }
}

export function clearOutput() {
  outputEntries.set([]);
}

export function toggleOutputPanel() {
  const isOpen = get(bottomPanelOpen);
  const tab = get(activeBottomTab);

  if (isOpen && tab === 'output') {
    // Already showing output — close the panel
    bottomPanelOpen.set(false);
  } else {
    // Open the panel and switch to output tab
    bottomPanelOpen.set(true);
    activeBottomTab.set('output');
  }
}

// Re-export for backward compatibility with StatusBar etc.
export { bottomPanelOpen as outputPanelOpen };
