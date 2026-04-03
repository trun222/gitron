import { get } from 'svelte/store';
import { getTransport } from '$lib/api';
import type { StatusChangedPayload, RefsChangedPayload } from '$lib/api/types';
import { repoStatus, repoPath, refreshAll } from '$lib/stores/repo';
import { refreshTrackingStatus, refreshRemoteTags } from '$lib/stores/repo';
import { refreshWorktrees } from '$lib/stores/worktree';

let unlisteners: (() => void)[] = [];

export async function startWatcherListeners(): Promise<void> {
  // Clean up any existing listeners
  await stopWatcherListeners();

  const transport = getTransport();

  const unlistenStatus = await transport.listen<StatusChangedPayload>(
    'repo:status-changed',
    (payload) => {
      repoStatus.set(payload.status);
    }
  );

  const unlistenRefs = await transport.listen<RefsChangedPayload>(
    'repo:refs-changed',
    (payload) => {
      // Set status immediately from the payload (always correct)
      repoStatus.set(payload.status);
      // Re-fetch graph via refreshAll so excluded authors are applied
      const path = get(repoPath);
      if (path) refreshAll(path);
      refreshTrackingStatus();
      refreshRemoteTags();
      refreshWorktrees();
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
