import { writable, derived } from 'svelte/store';

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
export const outputPanelOpen = writable(false);

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
  outputPanelOpen.set(true);
}

export function clearOutput() {
  outputEntries.set([]);
}

export function toggleOutputPanel() {
  outputPanelOpen.update((open) => !open);
}
