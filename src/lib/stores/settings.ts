import { writable, derived } from 'svelte/store';
import type { RecentRepo } from '$lib/api/types';
import * as settingsApi from '$lib/api/settings';

// Core settings state
export const recentRepos = writable<RecentRepo[]>([]);
export const lastActiveRepo = writable<string | null>(null);
export const settingsLoaded = writable(false);

// Derived: pinned first, then by lastOpened descending
export const sortedRecentRepos = derived(recentRepos, ($repos) => {
  const pinned = $repos.filter((r) => r.pinned);
  const unpinned = $repos.filter((r) => !r.pinned);
  const byDate = (a: RecentRepo, b: RecentRepo) =>
    new Date(b.lastOpened).getTime() - new Date(a.lastOpened).getTime();
  return [...pinned.sort(byDate), ...unpinned.sort(byDate)];
});

// Actions
export async function loadSettings(): Promise<void> {
  const settings = await settingsApi.getSettings();
  recentRepos.set(settings.recentRepos);
  lastActiveRepo.set(settings.lastActiveRepo);
  settingsLoaded.set(true);
}

export async function trackRepoOpen(path: string): Promise<void> {
  const settings = await settingsApi.addRecentRepo(path);
  recentRepos.set(settings.recentRepos);
  lastActiveRepo.set(settings.lastActiveRepo);
}

export async function removeRepo(path: string): Promise<void> {
  const settings = await settingsApi.removeRecentRepo(path);
  recentRepos.set(settings.recentRepos);
  lastActiveRepo.set(settings.lastActiveRepo);
}

export async function togglePin(path: string): Promise<void> {
  const settings = await settingsApi.togglePinRepo(path);
  recentRepos.set(settings.recentRepos);
}
