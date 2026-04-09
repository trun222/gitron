import { get } from 'svelte/store';
import { hasRepo, networkOperation, fetchFromRemote, isConflictState } from '$lib/stores/repo';

let timer: ReturnType<typeof setInterval> | null = null;

export function startAutoFetch(intervalSeconds: number): void {
  stopAutoFetch();
  if (intervalSeconds <= 0) return;

  timer = setInterval(() => {
    // Skip auto-fetch during rebase/merge/cherry-pick to avoid ref lock races
    if (get(hasRepo) && !get(networkOperation) && !get(isConflictState)) {
      fetchFromRemote(undefined, { silent: true });
    }
  }, intervalSeconds * 1000);
}

export function stopAutoFetch(): void {
  if (timer !== null) {
    clearInterval(timer);
    timer = null;
  }
}
