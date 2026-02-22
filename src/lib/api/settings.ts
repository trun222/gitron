import { isTauri } from '$lib/api';
import type { AppSettings, AutoFetchInterval, ChangesViewMode, EditorFontSize, FileWatcherInterval, GraphColumnWidths, MonoFont, RecentRepo, ThemeMode, ZoomLevel } from './types';

const MAX_RECENT_REPOS = 20;

const DEFAULT_SETTINGS: AppSettings = {
  lastActiveRepo: null,
  recentRepos: [],
};

// --- Storage abstraction ---

interface SettingsStore {
  get<T>(key: string): Promise<T | null>;
  set(key: string, value: unknown): Promise<void>;
}

class TauriSettingsStore implements SettingsStore {
  private storePromise: Promise<InstanceType<typeof import('@tauri-apps/plugin-store').LazyStore>> | null = null;

  private getStore() {
    if (!this.storePromise) {
      this.storePromise = import('@tauri-apps/plugin-store').then(
        ({ LazyStore }) => new LazyStore('settings.json')
      );
    }
    return this.storePromise;
  }

  async get<T>(key: string): Promise<T | null> {
    const store = await this.getStore();
    return (await store.get<T>(key)) ?? null;
  }

  async set(key: string, value: unknown): Promise<void> {
    const store = await this.getStore();
    await store.set(key, value);
  }
}

class WebSettingsStore implements SettingsStore {
  private prefix = 'gitron:';

  async get<T>(key: string): Promise<T | null> {
    const raw = localStorage.getItem(`${this.prefix}${key}`);
    if (!raw) return null;
    try {
      return JSON.parse(raw) as T;
    } catch {
      return null;
    }
  }

  async set(key: string, value: unknown): Promise<void> {
    localStorage.setItem(`${this.prefix}${key}`, JSON.stringify(value));
  }
}

let _store: SettingsStore | null = null;

function getStore(): SettingsStore {
  if (!_store) {
    _store = isTauri() ? new TauriSettingsStore() : new WebSettingsStore();
  }
  return _store;
}

// --- Public API (unchanged signatures) ---

export async function getSettings(): Promise<AppSettings> {
  const settings = await getStore().get<AppSettings>('app');
  return settings ?? DEFAULT_SETTINGS;
}

async function saveSettings(settings: AppSettings): Promise<void> {
  await getStore().set('app', settings);
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

export async function saveFileWatcherInterval(interval: FileWatcherInterval): Promise<void> {
  const settings = await getSettings();
  settings.fileWatcherInterval = interval;
  await saveSettings(settings);
}

export async function saveZoomLevel(level: ZoomLevel): Promise<void> {
  const settings = await getSettings();
  settings.zoomLevel = level;
  await saveSettings(settings);
}

export async function saveHighContrast(enabled: boolean): Promise<void> {
  const settings = await getSettings();
  settings.highContrast = enabled;
  await saveSettings(settings);
}

export async function saveEditorFontSize(size: EditorFontSize): Promise<void> {
  const settings = await getSettings();
  settings.editorFontSize = size;
  await saveSettings(settings);
}

export async function saveMonoFont(font: MonoFont): Promise<void> {
  const settings = await getSettings();
  settings.monoFont = font;
  await saveSettings(settings);
}

export async function saveShowTagsList(enabled: boolean): Promise<void> {
  const settings = await getSettings();
  settings.showTagsList = enabled;
  await saveSettings(settings);
}

export async function saveChangesViewMode(mode: ChangesViewMode): Promise<void> {
  const settings = await getSettings();
  settings.changesViewMode = mode;
  await saveSettings(settings);
}
