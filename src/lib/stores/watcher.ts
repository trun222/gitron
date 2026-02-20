import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { StatusChangedPayload, RefsChangedPayload } from '$lib/api/types';
import { repoStatus, commitGraph } from '$lib/stores/repo';
import { refreshTrackingStatus } from '$lib/stores/repo';

let unlisteners: UnlistenFn[] = [];

export async function startWatcherListeners(): Promise<void> {
  // Clean up any existing listeners
  await stopWatcherListeners();

  const unlistenStatus = await listen<StatusChangedPayload>(
    'repo:status-changed',
    (event) => {
      repoStatus.set(event.payload.status);
    }
  );

  const unlistenRefs = await listen<RefsChangedPayload>(
    'repo:refs-changed',
    (event) => {
      commitGraph.set(event.payload.graph);
      repoStatus.set(event.payload.status);
      refreshTrackingStatus();
    }
  );

  unlisteners = [unlistenStatus, unlistenRefs];
}

export async function stopWatcherListeners(): Promise<void> {
  for (const unlisten of unlisteners) {
    unlisten();
  }
  unlisteners = [];
}
