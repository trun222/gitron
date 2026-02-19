import { get } from 'svelte/store';
import { hasRepo, networkOperation, fetchFromRemote } from '$lib/stores/repo';

let timer: ReturnType<typeof setInterval> | null = null;

export function startAutoFetch(intervalSeconds: number): void {
  stopAutoFetch();
  if (intervalSeconds <= 0) return;

  timer = setInterval(() => {
    if (get(hasRepo) && !get(networkOperation)) {
      fetchFromRemote();
    }
  }, intervalSeconds * 1000);
}

export function stopAutoFetch(): void {
  if (timer !== null) {
    clearInterval(timer);
    timer = null;
  }
}
