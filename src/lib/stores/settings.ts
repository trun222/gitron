import { writable, derived } from 'svelte/store';
import type { GraphColumnWidths, RecentRepo } from '$lib/api/types';
import * as settingsApi from '$lib/api/settings';

// Core settings state
export const recentRepos = writable<RecentRepo[]>([]);
export const lastActiveRepo = writable<string | null>(null);
export const settingsLoaded = writable(false);
export const graphColumnWidths = writable<GraphColumnWidths>({
  graph: 40,
  author: 140,
  date: 80,
  sha: 70,
});
export const sidebarCollapsed = writable(false);

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
  if (settings.graphColumnWidths) {
    graphColumnWidths.set(settings.graphColumnWidths);
  }
  if (settings.sidebarCollapsed != null) {
    sidebarCollapsed.set(settings.sidebarCollapsed);
  }
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

export async function saveGraphColumnWidths(widths: GraphColumnWidths): Promise<void> {
  graphColumnWidths.set(widths);
  await settingsApi.saveColumnWidths(widths);
}

export async function toggleSidebar(): Promise<void> {
  sidebarCollapsed.update((v) => {
    const next = !v;
    settingsApi.saveSidebarCollapsed(next);
    return next;
  });
}
