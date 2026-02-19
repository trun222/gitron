import { writable, derived } from 'svelte/store';
import type { AutoFetchInterval, GraphColumnWidths, RecentRepo, ThemeMode } from '$lib/api/types';
import * as settingsApi from '$lib/api/settings';
import { startAutoFetch, stopAutoFetch } from '$lib/stores/autofetch';

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
export const theme = writable<ThemeMode>('tron');
export const autoFetchInterval = writable<AutoFetchInterval>(0);
export const autoShowOutput = writable(true);

// Derived: pinned first, then by lastOpened descending
export const sortedRecentRepos = derived(recentRepos, ($repos) => {
  const pinned = $repos.filter((r) => r.pinned);
  const unpinned = $repos.filter((r) => !r.pinned);
  const byDate = (a: RecentRepo, b: RecentRepo) =>
    new Date(b.lastOpened).getTime() - new Date(a.lastOpened).getTime();
  return [...pinned.sort(byDate), ...unpinned.sort(byDate)];
});

// Theme management
let systemThemeQuery: MediaQueryList | null = null;
let systemThemeHandler: ((e: MediaQueryListEvent) => void) | null = null;

export function applyTheme(mode: ThemeMode): void {
  // Clean up previous system listener
  if (systemThemeQuery && systemThemeHandler) {
    systemThemeQuery.removeEventListener('change', systemThemeHandler);
    systemThemeHandler = null;
  }

  // Always remove tron classes first, then re-add if needed
  document.documentElement.classList.remove('tron', 'tron-enhanced');

  if (mode === 'system') {
    systemThemeQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const applySystemTheme = () => {
      if (systemThemeQuery!.matches) {
        document.documentElement.classList.add('dark');
      } else {
        document.documentElement.classList.remove('dark');
      }
    };
    applySystemTheme();
    systemThemeHandler = () => applySystemTheme();
    systemThemeQuery.addEventListener('change', systemThemeHandler);
  } else if (mode === 'tron-enhanced') {
    document.documentElement.classList.add('dark');
    document.documentElement.classList.add('tron');
    document.documentElement.classList.add('tron-enhanced');
  } else if (mode === 'tron') {
    document.documentElement.classList.add('dark');
    document.documentElement.classList.add('tron');
  } else if (mode === 'dark') {
    document.documentElement.classList.add('dark');
  } else {
    document.documentElement.classList.remove('dark');
  }
}

export async function setTheme(mode: ThemeMode): Promise<void> {
  theme.set(mode);
  applyTheme(mode);
  await settingsApi.saveTheme(mode);
}

export async function setAutoFetchInterval(interval: AutoFetchInterval): Promise<void> {
  autoFetchInterval.set(interval);
  stopAutoFetch();
  startAutoFetch(interval);
  await settingsApi.saveAutoFetchInterval(interval);
}

export async function setAutoShowOutput(enabled: boolean): Promise<void> {
  autoShowOutput.set(enabled);
  await settingsApi.saveAutoShowOutput(enabled);
}

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

  // Theme
  const savedTheme = settings.theme ?? 'tron';
  theme.set(savedTheme);
  applyTheme(savedTheme);

  // Auto-fetch
  const savedInterval = settings.autoFetchInterval ?? 0;
  autoFetchInterval.set(savedInterval);
  startAutoFetch(savedInterval);

  // Auto-show output panel
  autoShowOutput.set(settings.autoShowOutput ?? true);

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
