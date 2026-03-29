import { writable, derived } from 'svelte/store';
import type { AutoFetchInterval, ChangesViewMode, EditorFontSize, FileWatcherInterval, GraphColumnWidths, MonoFont, RecentRepo, ThemeMode, ZoomLevel } from '$lib/api/types';
import * as settingsApi from '$lib/api/settings';
import * as repoApi from '$lib/api/repo';
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
export const autoFetchInterval = writable<AutoFetchInterval>(15);
export const autoShowOutput = writable(true);
export const fileWatcherInterval = writable<FileWatcherInterval>(0);
export const zoomLevel = writable<ZoomLevel>(1.0);
export const highContrast = writable(false);
export const editorFontSize = writable<EditorFontSize>(12);
export const monoFont = writable<MonoFont>('default');
export const showTagsList = writable(true);
export const showWorktreesList = writable(false);
export const changesViewMode = writable<ChangesViewMode>('file');
export const verboseGitErrors = writable(false);
export const terminalApp = writable<string>('');
export const treeExpandedByDefault = writable(false);
export const excludedAuthors = writable<string[]>([]);

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

// Zoom management
const MONO_FONT_STACKS: Record<MonoFont, string> = {
  'default': '"SF Mono", "Fira Code", "Cascadia Code", monospace',
  'fira-code': '"Fira Code", "SF Mono", monospace',
  'jetbrains-mono': '"JetBrains Mono", "SF Mono", monospace',
  'cascadia-code': '"Cascadia Code", "SF Mono", monospace',
  'sf-mono': '"SF Mono", monospace',
  'menlo': 'Menlo, "SF Mono", monospace',
};

async function applyZoom(level: ZoomLevel): Promise<void> {
  const { isTauri } = await import('$lib/api');
  if (isTauri()) {
    const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow');
    await getCurrentWebviewWindow().setZoom(level);
  } else {
    document.documentElement.style.transform = level === 1 ? '' : `scale(${level})`;
    document.documentElement.style.transformOrigin = 'top left';
  }
}

function applyHighContrast(enabled: boolean): void {
  if (enabled) {
    document.documentElement.classList.add('high-contrast');
  } else {
    document.documentElement.classList.remove('high-contrast');
  }
}

function applyEditorFontSize(size: EditorFontSize): void {
  document.documentElement.style.setProperty('--editor-font-size', `${size}px`);
}

function applyMonoFont(font: MonoFont): void {
  const stack = MONO_FONT_STACKS[font];
  document.documentElement.style.setProperty('--font-mono', stack);
}

export async function setZoomLevel(level: ZoomLevel): Promise<void> {
  zoomLevel.set(level);
  await applyZoom(level);
  await settingsApi.saveZoomLevel(level);
}

export async function setHighContrast(enabled: boolean): Promise<void> {
  highContrast.set(enabled);
  applyHighContrast(enabled);
  await settingsApi.saveHighContrast(enabled);
}

export async function setEditorFontSize(size: EditorFontSize): Promise<void> {
  editorFontSize.set(size);
  applyEditorFontSize(size);
  await settingsApi.saveEditorFontSize(size);
}

export async function setMonoFont(font: MonoFont): Promise<void> {
  monoFont.set(font);
  applyMonoFont(font);
  await settingsApi.saveMonoFont(font);
}

export async function setShowTagsList(enabled: boolean): Promise<void> {
  showTagsList.set(enabled);
  await settingsApi.saveShowTagsList(enabled);
}

export async function setShowWorktreesList(enabled: boolean): Promise<void> {
  showWorktreesList.set(enabled);
  await settingsApi.saveShowWorktreesList(enabled);
}

export async function setChangesViewMode(mode: ChangesViewMode): Promise<void> {
  changesViewMode.set(mode);
  await settingsApi.saveChangesViewMode(mode);
}

export async function setAutoFetchInterval(interval: AutoFetchInterval): Promise<void> {
  autoFetchInterval.set(interval);
  stopAutoFetch();
  startAutoFetch(interval);
  await settingsApi.saveAutoFetchInterval(interval);
}

export async function setFileWatcherInterval(interval: FileWatcherInterval): Promise<void> {
  fileWatcherInterval.set(interval);
  await repoApi.setWatcherInterval(interval);
  await settingsApi.saveFileWatcherInterval(interval);
}

export async function setAutoShowOutput(enabled: boolean): Promise<void> {
  autoShowOutput.set(enabled);
  await settingsApi.saveAutoShowOutput(enabled);
}

export async function setVerboseGitErrors(enabled: boolean): Promise<void> {
  verboseGitErrors.set(enabled);
  await settingsApi.saveVerboseGitErrors(enabled);
}

export async function setTerminalApp(app: string): Promise<void> {
  terminalApp.set(app);
  await settingsApi.saveTerminalApp(app);
}

export async function setTreeExpandedByDefault(enabled: boolean): Promise<void> {
  treeExpandedByDefault.set(enabled);
  await settingsApi.saveTreeExpandedByDefault(enabled);
}

export async function setExcludedAuthors(authors: string[]): Promise<void> {
  excludedAuthors.set(authors);
  await settingsApi.saveExcludedAuthors(authors);
}

export async function addExcludedAuthor(author: string): Promise<void> {
  excludedAuthors.update((current) => {
    if (current.includes(author)) return current;
    const next = [...current, author];
    settingsApi.saveExcludedAuthors(next);
    return next;
  });
}

export async function removeExcludedAuthor(author: string): Promise<void> {
  excludedAuthors.update((current) => {
    const next = current.filter((a) => a !== author);
    settingsApi.saveExcludedAuthors(next);
    return next;
  });
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
  const savedInterval = settings.autoFetchInterval ?? 15;
  autoFetchInterval.set(savedInterval);
  startAutoFetch(savedInterval);

  // Auto-show output panel
  autoShowOutput.set(settings.autoShowOutput ?? true);

  // File watcher interval
  const savedWatcherInterval = settings.fileWatcherInterval ?? 0;
  fileWatcherInterval.set(savedWatcherInterval);
  if (savedWatcherInterval > 0) {
    repoApi.setWatcherInterval(savedWatcherInterval);
  }

  // Zoom level
  const savedZoom = settings.zoomLevel ?? 1.0;
  zoomLevel.set(savedZoom);
  if (savedZoom !== 1.0) {
    applyZoom(savedZoom);
  }

  // High contrast
  const savedHighContrast = settings.highContrast ?? false;
  highContrast.set(savedHighContrast);
  applyHighContrast(savedHighContrast);

  // Editor font size
  const savedFontSize = settings.editorFontSize ?? 12;
  editorFontSize.set(savedFontSize);
  applyEditorFontSize(savedFontSize);

  // Monospace font
  const savedMonoFont = settings.monoFont ?? 'default';
  monoFont.set(savedMonoFont);
  if (savedMonoFont !== 'default') {
    applyMonoFont(savedMonoFont);
  }

  // Show tags list
  showTagsList.set(settings.showTagsList ?? true);
  showWorktreesList.set(settings.showWorktreesList ?? false);

  // Changes view mode
  changesViewMode.set(settings.changesViewMode ?? 'file');

  // Verbose git errors
  verboseGitErrors.set(settings.verboseGitErrors ?? false);

  // Terminal app
  terminalApp.set(settings.terminalApp ?? '');

  // Tree expanded by default
  treeExpandedByDefault.set(settings.treeExpandedByDefault ?? false);

  // Excluded authors
  excludedAuthors.set(settings.excludedAuthors ?? []);

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
