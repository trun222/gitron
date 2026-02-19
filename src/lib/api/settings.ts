import { LazyStore } from '@tauri-apps/plugin-store';
import type { AppSettings, AutoFetchInterval, GraphColumnWidths, RecentRepo, ThemeMode } from './types';

const store = new LazyStore('settings.json');

const MAX_RECENT_REPOS = 20;

const DEFAULT_SETTINGS: AppSettings = {
  lastActiveRepo: null,
  recentRepos: [],
};

export async function getSettings(): Promise<AppSettings> {
  const settings = await store.get<AppSettings>('app');
  return settings ?? DEFAULT_SETTINGS;
}

async function saveSettings(settings: AppSettings): Promise<void> {
  await store.set('app', settings);
}

export async function addRecentRepo(path: string): Promise<AppSettings> {
  const settings = await getSettings();
  const name = path.split('/').pop() ?? path;
  const now = new Date().toISOString();

  // Remove existing entry for this path (if any)
  const filtered = settings.recentRepos.filter((r) => r.path !== path);

  // Add at front
  const entry: RecentRepo = {
    path,
    name,
    lastOpened: now,
    pinned: settings.recentRepos.find((r) => r.path === path)?.pinned ?? false,
  };

  settings.recentRepos = [entry, ...filtered].slice(0, MAX_RECENT_REPOS);
  settings.lastActiveRepo = path;

  await saveSettings(settings);
  return settings;
}

export async function removeRecentRepo(path: string): Promise<AppSettings> {
  const settings = await getSettings();
  settings.recentRepos = settings.recentRepos.filter((r) => r.path !== path);
  if (settings.lastActiveRepo === path) {
    settings.lastActiveRepo = null;
  }
  await saveSettings(settings);
  return settings;
}

export async function togglePinRepo(path: string): Promise<AppSettings> {
  const settings = await getSettings();
  const repo = settings.recentRepos.find((r) => r.path === path);
  if (repo) {
    repo.pinned = !repo.pinned;
    await saveSettings(settings);
  }
  return settings;
}

export async function getLastActiveRepo(): Promise<string | null> {
  const settings = await getSettings();
  return settings.lastActiveRepo;
}

export async function getColumnWidths(): Promise<GraphColumnWidths | null> {
  const settings = await getSettings();
  return settings.graphColumnWidths ?? null;
}

export async function saveColumnWidths(widths: GraphColumnWidths): Promise<void> {
  const settings = await getSettings();
  settings.graphColumnWidths = widths;
  await saveSettings(settings);
}

export async function saveSidebarCollapsed(collapsed: boolean): Promise<void> {
  const settings = await getSettings();
  settings.sidebarCollapsed = collapsed;
  await saveSettings(settings);
}

export async function saveTheme(theme: ThemeMode): Promise<void> {
  const settings = await getSettings();
  settings.theme = theme;
  await saveSettings(settings);
}

export async function saveAutoFetchInterval(interval: AutoFetchInterval): Promise<void> {
  const settings = await getSettings();
  settings.autoFetchInterval = interval;
  await saveSettings(settings);
}

export async function saveAutoShowOutput(enabled: boolean): Promise<void> {
  const settings = await getSettings();
  settings.autoShowOutput = enabled;
  await saveSettings(settings);
}
