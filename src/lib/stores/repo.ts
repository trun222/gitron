import { writable, derived, get } from 'svelte/store';
import type {
  RepoInfo,
  RepoStatus,
  CommitGraph,
  Commit,
  Branch,
  FileDiff,
  FileStatus,
  ConflictedFileContent,
  Remote,
  TrackingStatus,
  StashEntry,
  MergedBranch,
  CheckpointRef,
} from '$lib/api/types';
import * as api from '$lib/api/repo';
import { trackRepoOpen, excludedAuthors, protectedBranches } from '$lib/stores/settings';
import { addOutput } from '$lib/stores/output';
import { addToast, updateToast } from '$lib/stores/toast';
import { startWatcherListeners, stopWatcherListeners } from '$lib/stores/watcher';

// File selection types
export type FileSection = 'staged' | 'unstaged' | 'untracked' | 'conflicted';

export interface SelectedFileInfo {
  path: string;
  section: FileSection;
}

// Core repo state
export const repoPath = writable<string | null>(null);
export const repoInfo = writable<RepoInfo | null>(null);
export const repoStatus = writable<RepoStatus | null>(null);
export const commitGraph = writable<CommitGraph | null>(null);
export const selectedCommit = writable<Commit | null>(null);
export const selectedFileDiff = writable<FileDiff | null>(null);
export const selectedConflictFile = writable<ConflictedFileContent | null>(null);
export const selectedFile = writable<SelectedFileInfo | null>(null);
export const loading = writable(false);
export const error = writable<string | null>(null);

// Remote state
export const remotes = writable<Remote[]>([]);
export const trackingStatus = writable<TrackingStatus | null>(null);
export const networkOperation = writable<string | null>(null);

// Remote tag tracking (name → remote OID)
export const remoteTagMap = writable<Map<string, string>>(new Map());

// Scroll-to-commit (used by tags list to jump to a commit in the graph)
export const scrollToCommitOid = writable<string | null>(null);

// Commit search state
export const commitSearchActive = writable<boolean>(false);
export const commitSearchQuery = writable<string>('');
export const commitSearchMatchOids = writable<Set<string>>(new Set());
export const commitSearchLoading = writable<boolean>(false);
export const commitSearchDiffs = writable<boolean>(false);

// Commit file inspection state
export const commitFiles = writable<FileDiff[] | null>(null);
export const selectedCommitFile = writable<string | null>(null);

// Discard all confirmation
export const discardConfirmOpen = writable(false);

// Force push confirmation
export const forcePushConfirmOpen = writable(false);

// Delete branch confirmation
export interface DeleteBranchConfirmInfo {
  open: boolean;
  branchName: string;
  isRemote: boolean;
}
export const deleteBranchConfirm = writable<DeleteBranchConfirmInfo>({
  open: false,
  branchName: '',
  isRemote: false,
});

// Cleanup merged branches dialog
export const cleanupBranchesOpen = writable(false);
export const cleanupBranchesList = writable<MergedBranch[]>([]);
export const cleanupBranchesLoading = writable(false);

// Delete all branches dialog
export const deleteAllBranchesOpen = writable(false);

// Purge checkpoint refs dialog
export const purgeCheckpointsOpen = writable(false);
export const purgeCheckpointsList = writable<CheckpointRef[]>([]);
export const purgeCheckpointsLoading = writable(false);

// Branch conflict prompt (shown when checking out a remote branch with existing local)
export interface BranchConflictInfo {
  localName: string;
  remoteBranchName: string;
}
export const branchConflictPrompt = writable<BranchConflictInfo | null>(null);

// Derived stores
export const hasRepo = derived(repoInfo, ($info) => $info !== null);
export const isFileSelected = derived(selectedFile, ($f) => $f !== null);
export const isCommitFileSelected = derived(selectedCommitFile, ($f) => $f !== null);
export const currentBranch = derived(repoInfo, ($info) => $info?.head_branch ?? null);
export const localBranches = derived(commitGraph, ($graph) =>
  $graph?.branches.filter((b) => !b.is_remote) ?? []
);
export const remoteBranches = derived(commitGraph, ($graph) =>
  $graph?.branches.filter((b) => b.is_remote) ?? []
);
export const defaultRemote = derived(remotes, ($remotes) => {
  return $remotes.find((r) => r.name === 'origin') ?? $remotes[0] ?? null;
});
export const aheadCount = derived(trackingStatus, ($ts) => $ts?.ahead ?? 0);
export const behindCount = derived(trackingStatus, ($ts) => $ts?.behind ?? 0);
export const stagedCount = derived(repoStatus, ($status) => $status?.staged.length ?? 0);
export const unstagedCount = derived(
  repoStatus,
  ($status) => ($status?.unstaged.length ?? 0) + ($status?.untracked.length ?? 0)
);
export const conflictedCount = derived(repoStatus, ($status) => $status?.conflicted.length ?? 0);
export const isConflictState = derived(
  repoStatus,
  ($status) => $status !== null && $status.state !== 'Clean'
);

// Helpers
function getAllFiles(): SelectedFileInfo[] {
  const status = get(repoStatus);
  if (!status) return [];
  return [
    ...status.conflicted.map((p) => ({ path: p, section: 'conflicted' as const })),
    ...status.staged.map((f) => ({ path: f.path, section: 'staged' as const })),
    ...status.unstaged.map((f) => ({ path: f.path, section: 'unstaged' as const })),
    ...status.untracked.map((p) => ({ path: p, section: 'untracked' as const })),
  ];
}

// Actions
export async function openRepo(path: string) {
  loading.set(true);
  error.set(null);
  try {
    const result = await api.openRepo(path);
    repoPath.set(path);
    repoInfo.set(result.info);
    repoStatus.set(result.status);
    commitGraph.set(result.graph);
    remotes.set(result.remotes);
    trackingStatus.set(result.tracking);
    refreshRemoteTags(); // fire-and-forget (needs defaultRemote populated above)
    await trackRepoOpen(path);
    await startWatcherListeners();
    // Re-fetch graph with excluded authors if any are set.
    // The initial open_repo uses GraphOptions::default() (no exclusions).
    const excluded = get(excludedAuthors);
    if (excluded.length > 0) {
      const graph = await api.getCommitGraph(path, undefined, undefined, excluded);
      commitGraph.set(graph);
    }
  } catch (e) {
    error.set(String(e));
  } finally {
    loading.set(false);
  }
}

export async function closeRepo() {
  await stopWatcherListeners();
  try {
    await api.closeRepo();
  } catch {
    // Ignore errors on close
  }
  repoPath.set(null);
  repoInfo.set(null);
  repoStatus.set(null);
  commitGraph.set(null);
  selectedCommit.set(null);
  selectedFileDiff.set(null);
  selectedFile.set(null);
  commitFiles.set(null);
  selectedCommitFile.set(null);
  remotes.set([]);
  remoteTagMap.set(new Map());
  trackingStatus.set(null);
  clearCommitSearch();
  error.set(null);
}

export async function refreshAll(path: string) {
  try {
    const [status, graph] = await Promise.all([
      api.getStatus(path),
      api.getCommitGraph(path, undefined, undefined, get(excludedAuthors)),
    ]);
    repoStatus.set(status);
    commitGraph.set(graph);
    await refreshTrackingStatus();
  } catch (e) {
    error.set(String(e));
  }
}

export async function refreshStatus(path: string) {
  try {
    const status = await api.getStatus(path);
    repoStatus.set(status);
  } catch (e) {
    error.set(String(e));
  }
}

export async function stageFile(path: string, filePath: string) {
  try {
    const status = await api.stageFile(path, filePath);
    repoStatus.set(status);
  } catch (e) {
    error.set(String(e));
  }
}

export async function unstageFile(path: string, filePath: string) {
  try {
    const status = await api.unstageFile(path, filePath);
    repoStatus.set(status);
  } catch (e) {
    error.set(String(e));
  }
}

export async function stageAllFiles(path: string) {
  try {
    const status = await api.stageAll(path);
    repoStatus.set(status);
  } catch (e) {
    error.set(String(e));
  }
}

export async function stageFiles(path: string, filePaths: string[]) {
  try {
    const status = await api.stageFiles(path, filePaths);
    repoStatus.set(status);
  } catch (e) {
    error.set(String(e));
  }
}

export async function unstageAllFiles(path: string) {
  try {
    const status = await api.unstageAll(path);
    repoStatus.set(status);
  } catch (e) {
    error.set(String(e));
  }
}

export async function selectCommit(commit: Commit) {
  selectedCommit.set(commit);
  selectedFileDiff.set(null);
  selectedFile.set(null);
  selectedCommitFile.set(null);
  // Fetch files changed in this commit
  const repoPathVal = get(repoPath);
  if (repoPathVal) {
    try {
      const files = await api.getCommitDiff(repoPathVal, commit.oid);
      commitFiles.set(files);
    } catch {
      commitFiles.set(null);
    }
  }
}

export function selectCommitFile(filePath: string) {
  const files = get(commitFiles);
  if (!files) return;
  const diff = files.find((f) => f.path === filePath);
  if (diff) {
    selectedCommitFile.set(filePath);
    selectedFileDiff.set(diff);
    selectedFile.set(null);
  }
}

export function selectNextCommitFile() {
  const current = get(selectedCommitFile);
  const files = get(commitFiles);
  if (!current || !files) return;
  const idx = files.findIndex((f) => f.path === current);
  if (idx >= 0 && idx < files.length - 1) {
    selectCommitFile(files[idx + 1].path);
  }
}

export function selectPrevCommitFile() {
  const current = get(selectedCommitFile);
  const files = get(commitFiles);
  if (!current || !files) return;
  const idx = files.findIndex((f) => f.path === current);
  if (idx > 0) {
    selectCommitFile(files[idx - 1].path);
  }
}

export function clearCommitFileSelection() {
  selectedCommitFile.set(null);
  selectedFileDiff.set(null);
}

export async function viewFileDiff(path: string, filePath: string) {
  try {
    const diff = await api.getFileDiff(path, filePath);
    selectedFileDiff.set(diff);
  } catch (e) {
    error.set(String(e));
  }
}

export async function viewStagedFileDiff(path: string, filePath: string) {
  try {
    const diff = await api.getStagedFileDiff(path, filePath);
    selectedFileDiff.set(diff);
  } catch (e) {
    error.set(String(e));
  }
}

// File selection — tracks current request to avoid races
let selectFileSeq = 0;

export async function selectFile(path: string, section: FileSection) {
  const repoPathVal = get(repoPath);
  if (!repoPathVal) return;
  const seq = ++selectFileSeq;
  selectedFile.set({ path, section });
  selectedCommit.set(null);
  selectedFileDiff.set(null);
  selectedConflictFile.set(null);
  if (section === 'conflicted') {
    try {
      const content = await api.getConflictedFile(repoPathVal, path);
      if (seq !== selectFileSeq) return; // superseded by newer selection
      selectedConflictFile.set(content);
    } catch (e) {
      if (seq !== selectFileSeq) return;
      error.set(String(e));
      selectedFile.set(null);
    }
  } else if (section === 'staged') {
    try {
      const diff = await api.getStagedFileDiff(repoPathVal, path);
      if (seq !== selectFileSeq) return;
      selectedFileDiff.set(diff);
    } catch (e) {
      if (seq !== selectFileSeq) return;
      error.set(String(e));
      selectedFile.set(null);
    }
  } else {
    try {
      const diff = await api.getFileDiff(repoPathVal, path);
      if (seq !== selectFileSeq) return;
      selectedFileDiff.set(diff);
    } catch (e) {
      if (seq !== selectFileSeq) return;
      error.set(String(e));
      selectedFile.set(null);
    }
  }
}

export function clearFileSelection() {
  selectedFile.set(null);
  selectedFileDiff.set(null);
  selectedConflictFile.set(null);
}

export function selectNextFile() {
  const current = get(selectedFile);
  if (!current) return;
  const files = getAllFiles();
  const idx = files.findIndex((f) => f.path === current.path && f.section === current.section);
  if (idx >= 0 && idx < files.length - 1) {
    const next = files[idx + 1];
    selectFile(next.path, next.section);
  }
}

export function selectPrevFile() {
  const current = get(selectedFile);
  if (!current) return;
  const files = getAllFiles();
  const idx = files.findIndex((f) => f.path === current.path && f.section === current.section);
  if (idx > 0) {
    const prev = files[idx - 1];
    selectFile(prev.path, prev.section);
  }
}

// Staging actions with selection management
export async function stageSelectedFile() {
  const current = get(selectedFile);
  const repoPathVal = get(repoPath);
  if (!current || !repoPathVal || current.section === 'staged') return;

  // Find position before staging
  const unstaged = getAllFiles().filter((f) => f.section !== 'staged');
  const idx = unstaged.findIndex((f) => f.path === current.path);

  await stageFile(repoPathVal, current.path);

  // Select next file at same position in unstaged+untracked
  const remaining = getAllFiles().filter((f) => f.section !== 'staged');
  if (remaining.length > 0) {
    const newIdx = Math.min(idx, remaining.length - 1);
    selectFile(remaining[newIdx].path, remaining[newIdx].section);
  } else {
    clearFileSelection();
  }
}

export async function unstageSelectedFile() {
  const current = get(selectedFile);
  const repoPathVal = get(repoPath);
  if (!current || !repoPathVal || current.section !== 'staged') return;

  await unstageFile(repoPathVal, current.path);
  // File moves to unstaged — follow it
  selectFile(current.path, 'unstaged');
}

export async function stageAllAndClear() {
  const repoPathVal = get(repoPath);
  if (!repoPathVal) return;
  await stageAllFiles(repoPathVal);
  clearFileSelection();
}

export async function stageUnstagedAndClear() {
  const repoPathVal = get(repoPath);
  const status = get(repoStatus);
  if (!repoPathVal || !status) return;
  const paths = status.unstaged.map((f) => f.path);
  if (paths.length === 0) return;
  await stageFiles(repoPathVal, paths);
  clearFileSelection();
}

export async function stageUntrackedAndClear() {
  const repoPathVal = get(repoPath);
  const status = get(repoStatus);
  if (!repoPathVal || !status) return;
  const paths = status.untracked;
  if (paths.length === 0) return;
  await stageFiles(repoPathVal, paths);
  clearFileSelection();
}

export async function unstageAllAndClear() {
  const repoPathVal = get(repoPath);
  if (!repoPathVal) return;
  await unstageAllFiles(repoPathVal);
  clearFileSelection();
}

export async function addToGitignore(pattern: string) {
  const path = get(repoPath);
  if (!path) return;
  try {
    const status = await api.addToGitignore(path, pattern);
    repoStatus.set(status);
    clearFileSelection();
  } catch (e) {
    error.set(String(e));
  }
}

export async function discardFiles(
  staged: string[],
  unstaged: string[],
  untracked: string[],
) {
  const path = get(repoPath);
  if (!path) return;
  try {
    const status = await api.discardFiles(path, staged, unstaged, untracked);
    repoStatus.set(status);
    clearFileSelection();
  } catch (e) {
    error.set(String(e));
  }
}

export async function discardAllChanges() {
  const path = get(repoPath);
  if (!path) return;
  discardConfirmOpen.set(false);
  try {
    const status = await api.discardAllChanges(path);
    repoStatus.set(status);
    clearFileSelection();
    await refreshAll(path);
  } catch (e) {
    error.set(String(e));
  }
}

export async function commitAndRefresh(message: string): Promise<string | null> {
  const path = get(repoPath);
  if (!path) return null;
  try {
    const result = await api.createCommit(path, message);
    addOutput('commit', result.output.stdout, result.output.stderr, result.success);
    if (!result.success) {
      const hookOutput = result.output.stderr || result.output.stdout;
      error.set(`Commit failed (hook or validation):\n${hookOutput}`);
      return null;
    }
    await refreshAll(path);
    clearFileSelection();
    return result.oid;
  } catch (e) {
    error.set(String(e));
    return null;
  }
}

export async function checkoutBranch(name: string) {
  const path = get(repoPath);
  if (!path) return;

  // Check if this is a remote branch with an existing local counterpart
  const graph = get(commitGraph);
  if (graph) {
    const isRemote = graph.branches.some((b) => b.is_remote && b.name === name);
    if (isRemote) {
      // Pull down the latest tip first so the checkout lands on current state
      if (!(await syncRemoteBranch(name))) return;
      const localName = name.includes('/') ? name.split('/').slice(1).join('/') : name;
      const localExists = graph.branches.some((b) => !b.is_remote && b.name === localName);
      if (localExists) {
        branchConflictPrompt.set({ localName, remoteBranchName: name });
        return;
      }
    }
  }

  try {
    const info = await api.checkoutBranch(path, name);
    repoInfo.set(info);
    await refreshAll(path);
  } catch (e) {
    error.set(String(e));
  }
}

export async function resetLocalToRemote(remoteBranchName: string) {
  const path = get(repoPath);
  if (!path) return;
  branchConflictPrompt.set(null);
  try {
    const info = await api.checkoutRemoteBranch(path, remoteBranchName);
    repoInfo.set(info);
    await refreshAll(path);
  } catch (e) {
    error.set(String(e));
  }
}

export async function checkoutLocalInstead(localName: string) {
  branchConflictPrompt.set(null);
  const path = get(repoPath);
  if (!path) return;
  try {
    const info = await api.checkoutBranch(path, localName);
    repoInfo.set(info);
    await refreshAll(path);
  } catch (e) {
    error.set(String(e));
  }
}

export function dismissBranchConflict() {
  branchConflictPrompt.set(null);
}

/// Create a branch and switch to it. Uncommitted changes are auto-stashed
/// before the switch and re-applied afterwards (see core create_and_checkout_branch).
async function createAndSwitch(name: string, target?: string) {
  const path = get(repoPath);
  if (!path) return;
  try {
    const result = await api.createAndCheckoutBranch(path, name, target);
    repoInfo.set(result.info);
    await refreshAll(path);
    if (result.auto_stashed) {
      if (result.stash_restored) {
        addToast(`Uncommitted changes carried over to '${name}'`, 'info');
      } else {
        addToast(
          `Changes were stashed before switching to '${name}' but could not be re-applied. Pop the stash to recover them.`,
          'error',
          8000
        );
      }
    }
  } catch (e) {
    error.set(String(e));
  }
}

export async function createAndCheckoutBranch(name: string) {
  await createAndSwitch(name);
}

export async function createBranchAtCommit(name: string, targetOid: string) {
  await createAndSwitch(name, targetOid);
}

export async function resetToCommit(commitOid: string, resetType: 'soft' | 'mixed' | 'hard') {
  const path = get(repoPath);
  if (!path) return;
  try {
    const info = await api.resetToCommit(path, commitOid, resetType);
    repoInfo.set(info);
    await refreshAll(path);
  } catch (e) {
    error.set(String(e));
  }
}

/**
 * Split a remote-tracking branch name ("origin/feature/x") into its remote and
 * branch parts, preferring a match against the configured remote names so that
 * branches containing slashes are handled correctly.
 */
function splitRemoteBranch(name: string): { remote: string; branch: string } | null {
  for (const r of get(remotes)) {
    if (name.startsWith(`${r.name}/`)) {
      return { remote: r.name, branch: name.slice(r.name.length + 1) };
    }
  }
  const slash = name.indexOf('/');
  if (slash <= 0) return null;
  return { remote: name.slice(0, slash), branch: name.slice(slash + 1) };
}

/**
 * Fetch the latest tip of a remote-tracking branch so an operation targeting it
 * runs against up-to-date state instead of a stale ref. Returns false when the
 * fetch failed, in which case the caller should not run the operation.
 */
export async function syncRemoteBranch(remoteBranchName: string): Promise<boolean> {
  const path = get(repoPath);
  if (!path) return false;
  const parts = splitRemoteBranch(remoteBranchName);
  if (!parts) {
    error.set(`Cannot determine the remote for "${remoteBranchName}"`);
    return false;
  }
  const label = `fetch ${parts.remote} ${parts.branch}`;
  networkOperation.set('fetching');
  try {
    const result = await api.fetchRemote(path, parts.remote, parts.branch);
    addOutput(label, result.output.stdout, result.output.stderr, true);
    return true;
  } catch (e) {
    const msg = String(e);
    addOutput(label, '', msg, false);
    error.set(`Failed to fetch ${remoteBranchName}:\n${msg}`);
    return false;
  } finally {
    networkOperation.set(null);
  }
}

export async function rebaseOnto(ontoBranch: string, options?: { syncRemote?: boolean }) {
  const path = get(repoPath);
  if (!path) return;
  const status = get(repoStatus);
  if (status && status.state !== 'Clean') {
    error.set(`Cannot rebase while a ${status.state.toLowerCase()} is in progress. Resolve conflicts or abort first.`);
    return;
  }
  if (options?.syncRemote && !(await syncRemoteBranch(ontoBranch))) return;
  try {
    const result = await api.rebaseOnto(path, ontoBranch);
    addOutput('rebase', result.output.stdout, result.output.stderr, result.success);
    if (result.conflicted) {
      error.set(`Rebase resulted in conflicts. Resolve conflicts and run \`git rebase --continue\`.`);
    } else if (!result.success) {
      error.set(`Rebase failed:\n${result.output.stderr || result.output.stdout}`);
    }
    const info = await api.getRepoInfo(path);
    repoInfo.set(info);
    await refreshAll(path);
  } catch (e) {
    error.set(String(e));
  }
}

export async function mergeInto(branchName: string, options?: { syncRemote?: boolean }) {
  const path = get(repoPath);
  if (!path) return;
  const status = get(repoStatus);
  if (status && status.state !== 'Clean') {
    error.set(`Cannot merge while a ${status.state.toLowerCase()} is in progress. Resolve conflicts or abort first.`);
    return;
  }
  if (options?.syncRemote && !(await syncRemoteBranch(branchName))) return;
  try {
    const result = await api.mergeInto(path, branchName);
    addOutput('merge', result.output.stdout, result.output.stderr, result.success);
    if (result.conflicted) {
      error.set(`Merge resulted in conflicts. Resolve conflicts and commit.`);
    } else if (!result.success) {
      error.set(`Merge failed:\n${result.output.stderr || result.output.stdout}`);
    }
    const info = await api.getRepoInfo(path);
    repoInfo.set(info);
    await refreshAll(path);
  } catch (e) {
    error.set(String(e));
  }
}

// Conflict resolution actions

export async function rebaseContinue() {
  const path = get(repoPath);
  if (!path) return;
  try {
    const result = await api.rebaseContinue(path);
    addOutput('rebase --continue', result.output.stdout, result.output.stderr, result.success);
    if (result.conflicted) {
      // Don't show error banner — the conflict banner already indicates this
      error.set(null);
    } else if (!result.success) {
      error.set(`Rebase continue failed:\n${result.output.stderr || result.output.stdout}`);
    } else {
      error.set(null);
      addToast('Rebase completed successfully', 'success');
    }
    const info = await api.getRepoInfo(path);
    repoInfo.set(info);
    await refreshAll(path);
  } catch (e) {
    error.set(String(e));
  }
}

export async function rebaseAbort() {
  const path = get(repoPath);
  if (!path) return;
  try {
    const result = await api.rebaseAbort(path);
    addOutput('rebase --abort', result.output.stdout, result.output.stderr, result.success);
    if (!result.success) {
      error.set(`Rebase abort failed:\n${result.output.stderr || result.output.stdout}`);
    } else {
      error.set(null);
      addToast('Rebase aborted', 'info');
    }
    const info = await api.getRepoInfo(path);
    repoInfo.set(info);
    await refreshAll(path);
  } catch (e) {
    error.set(String(e));
  }
}

export async function mergeContinue() {
  const path = get(repoPath);
  if (!path) return;
  try {
    const result = await api.mergeContinue(path);
    addOutput('merge --continue', result.output.stdout, result.output.stderr, result.success);
    if (result.conflicted) {
      error.set(null);
    } else if (!result.success) {
      error.set(`Merge continue failed:\n${result.output.stderr || result.output.stdout}`);
    } else {
      error.set(null);
      addToast('Merge completed successfully', 'success');
    }
    const info = await api.getRepoInfo(path);
    repoInfo.set(info);
    await refreshAll(path);
  } catch (e) {
    error.set(String(e));
  }
}

export async function mergeAbort() {
  const path = get(repoPath);
  if (!path) return;
  try {
    const result = await api.mergeAbort(path);
    addOutput('merge --abort', result.output.stdout, result.output.stderr, result.success);
    if (!result.success) {
      error.set(`Merge abort failed:\n${result.output.stderr || result.output.stdout}`);
    } else {
      error.set(null);
      addToast('Merge aborted', 'info');
    }
    const info = await api.getRepoInfo(path);
    repoInfo.set(info);
    await refreshAll(path);
  } catch (e) {
    error.set(String(e));
  }
}

export async function cherryPickAbort() {
  const path = get(repoPath);
  if (!path) return;
  try {
    await api.cherryPickAbort(path);
    error.set(null);
    addToast('Cherry-pick aborted', 'info');
    const info = await api.getRepoInfo(path);
    repoInfo.set(info);
    await refreshAll(path);
  } catch (e) {
    error.set(String(e));
  }
}

export async function writeResolvedFile(filePath: string, content: string) {
  const path = get(repoPath);
  if (!path) return;
  try {
    const status = await api.writeResolvedFile(path, filePath, content);
    repoStatus.set(status);
    selectedConflictFile.set(null);
    selectedFileDiff.set(null);
    // Move selection to next conflicted file if any
    if (status.conflicted.length > 0) {
      await selectFile(status.conflicted[0], 'conflicted');
    } else {
      clearFileSelection();
    }
    addToast(`Resolved: ${filePath}`, 'success');
  } catch (e) {
    error.set(String(e));
  }
}

// Jump to a specific commit in the graph (select + scroll)
export function jumpToCommit(targetOid: string) {
  const graph = get(commitGraph);
  if (!graph) return;
  const commit = graph.commits.find((c) => c.oid === targetOid);
  if (commit) {
    selectCommit(commit);
  }
  scrollToCommitOid.set(targetOid);
}

// Tag actions

export async function createTagAtCommit(name: string, targetOid: string, message?: string) {
  const path = get(repoPath);
  if (!path) return;
  try {
    await api.createTag(path, name, targetOid, message);
    await refreshAll(path);
  } catch (e) {
    error.set(String(e));
  }
}

export async function deleteTag(name: string) {
  const path = get(repoPath);
  if (!path) return;
  try {
    await api.deleteTag(path, name);
    await refreshAll(path);
    addToast(`Deleted tag '${name}'`, 'success');
  } catch (e) {
    error.set(String(e));
  }
}

export async function moveTag(name: string, targetOid: string) {
  const path = get(repoPath);
  if (!path) return;
  try {
    await api.moveTag(path, name, targetOid);
    await refreshAll(path);
  } catch (e) {
    error.set(String(e));
  }
}

export async function pushTag(tagName: string, force?: boolean) {
  const path = get(repoPath);
  if (!path) return;
  const remote = get(defaultRemote);
  if (!remote) {
    error.set('No remote configured');
    return;
  }
  try {
    const result = await api.pushTag(path, remote.name, tagName, force);
    addOutput('push-tag', result.output.stdout, result.output.stderr, true);
    addToast(`Pushed tag '${tagName}' to ${remote.name}`, 'success');
    refreshRemoteTags(); // fire-and-forget
  } catch (e) {
    error.set(String(e));
    addOutput('push-tag', '', String(e), false);
    addToast(`Failed to push tag '${tagName}'`, 'error');
  }
}

export async function deleteRemoteTag(tagName: string) {
  const path = get(repoPath);
  if (!path) return;
  const remote = get(defaultRemote);
  if (!remote) {
    error.set('No remote configured');
    return;
  }
  try {
    await api.deleteRemoteTag(path, remote.name, tagName);
    addOutput('delete-remote-tag', `Deleted remote tag '${tagName}'`, '', true);
    addToast(`Deleted remote tag '${tagName}' from ${remote.name}`, 'success');
    refreshRemoteTags(); // fire-and-forget
  } catch (e) {
    error.set(String(e));
    addOutput('delete-remote-tag', '', String(e), false);
    addToast(`Failed to delete remote tag '${tagName}'`, 'error');
  }
}

export async function deleteBranch(name: string) {
  const path = get(repoPath);
  if (!path) return;
  deleteBranchConfirm.set({ open: false, branchName: '', isRemote: false });
  try {
    await api.deleteBranch(path, name);
    await refreshAll(path);
  } catch (e) {
    error.set(String(e));
  }
}

export async function deleteRemoteBranch(remoteName: string, branch: string) {
  const path = get(repoPath);
  if (!path) return;
  deleteBranchConfirm.set({ open: false, branchName: '', isRemote: false });
  try {
    await api.deleteRemoteBranch(path, remoteName, branch);
    await refreshAll(path);
  } catch (e) {
    error.set(String(e));
  }
}

export async function openCleanupBranches() {
  const path = get(repoPath);
  if (!path) return;
  cleanupBranchesLoading.set(true);
  cleanupBranchesOpen.set(true);
  try {
    const merged = await api.findMergedBranches(path);
    const protected_ = get(protectedBranches);
    const filtered = merged.filter((b) =>
      !protected_.includes(b.name) && !protected_.includes(b.short_name)
    );
    cleanupBranchesList.set(filtered);
  } catch (e) {
    error.set(String(e));
    addToast(String(e), 'error');
    cleanupBranchesOpen.set(false);
  } finally {
    cleanupBranchesLoading.set(false);
  }
}

export async function confirmCleanupBranches(branches: MergedBranch[]) {
  const path = get(repoPath);
  if (!path) return;
  cleanupBranchesOpen.set(false);
  try {
    const deleted = await api.cleanupMergedBranches(path, branches);
    await refreshAll(path);
    addToast(`Deleted ${deleted.length} branch${deleted.length === 1 ? '' : 'es'}`, 'success');
  } catch (e) {
    error.set(String(e));
    addToast(String(e), 'error');
  }
}

export async function confirmDeleteAllBranches(branches: Branch[]) {
  const path = get(repoPath);
  if (!path) return;
  deleteAllBranchesOpen.set(false);

  const total = branches.length;
  if (total === 0) return;

  const toastId = addToast(`Deleting branches (0/${total})...`, 'info', 0);

  let deleted = 0;
  let failed = 0;
  const errors: string[] = [];

  for (const branch of branches) {
    const displayName = branch.name;
    updateToast(toastId, `Deleting "${displayName}" (${deleted + failed + 1}/${total})...`);

    try {
      if (branch.is_remote) {
        const slashIdx = branch.name.indexOf('/');
        if (slashIdx > 0) {
          const remote = branch.name.substring(0, slashIdx);
          const branchName = branch.name.substring(slashIdx + 1);
          await api.deleteRemoteBranch(path, remote, branchName);
        } else {
          throw new Error(`Invalid remote branch format: ${branch.name}`);
        }
      } else {
        await api.deleteBranch(path, branch.name);
      }
      deleted++;
    } catch (e) {
      failed++;
      errors.push(`${displayName}: ${e instanceof Error ? e.message : String(e)}`);
    }
  }

  await refreshAll(path);

  if (failed === 0) {
    updateToast(toastId, `Deleted all ${deleted} branch${deleted === 1 ? '' : 'es'}`, 'success', 4000);
  } else {
    updateToast(toastId, `Deleted ${deleted}/${total} branch${total === 1 ? '' : 'es'}, ${failed} failed`, 'error', 6000);
    addOutput('Delete all branches', `Deleted ${deleted}/${total}`, errors.join('\n'), false);
  }
}

export async function openPurgeCheckpoints() {
  const path = get(repoPath);
  if (!path) return;
  purgeCheckpointsLoading.set(true);
  purgeCheckpointsOpen.set(true);
  try {
    const refs = await api.findCheckpointRefs(path);
    purgeCheckpointsList.set(refs);
  } catch (e) {
    error.set(String(e));
    addToast(String(e), 'error');
    purgeCheckpointsOpen.set(false);
  } finally {
    purgeCheckpointsLoading.set(false);
  }
}

export async function confirmPurgeCheckpoints(refs: CheckpointRef[]) {
  const path = get(repoPath);
  if (!path) return;
  purgeCheckpointsOpen.set(false);
  try {
    const deleted = await api.purgeCheckpointRefs(path, refs);
    await refreshAll(path);
    addToast(`Purged ${deleted} checkpoint ref${deleted === 1 ? '' : 's'}`, 'success');
  } catch (e) {
    error.set(String(e));
    addToast(String(e), 'error');
  }
}

export async function saveStash(message?: string) {
  const path = get(repoPath);
  if (!path) return;
  try {
    const status = await api.saveStash(path, message);
    repoStatus.set(status);
    await refreshAll(path);
    addToast('Stash saved', 'success');
  } catch (e) {
    error.set(String(e));
    addToast(String(e), 'error');
  }
}

export async function listStashes(): Promise<StashEntry[]> {
  const path = get(repoPath);
  if (!path) return [];
  try {
    return await api.listStashes(path);
  } catch (e) {
    error.set(String(e));
    return [];
  }
}

export async function applyStash(index: number) {
  const path = get(repoPath);
  if (!path) return;
  try {
    const status = await api.applyStash(path, index);
    repoStatus.set(status);
    await refreshAll(path);
    addToast('Stash applied', 'success');
  } catch (e) {
    error.set(String(e));
    addToast(String(e), 'error');
  }
}

export async function popStash(index: number) {
  const path = get(repoPath);
  if (!path) return;
  try {
    const status = await api.popStash(path, index);
    repoStatus.set(status);
    await refreshAll(path);
    addToast('Stash popped', 'success');
  } catch (e) {
    error.set(String(e));
    addToast(String(e), 'error');
  }
}

export async function dropStash(index: number) {
  const path = get(repoPath);
  if (!path) return;
  try {
    const status = await api.dropStash(path, index);
    repoStatus.set(status);
    await refreshAll(path);
    addToast('Stash dropped', 'success');
  } catch (e) {
    error.set(String(e));
    addToast(String(e), 'error');
  }
}

// Remote actions

export async function refreshRemotes(path: string) {
  try {
    const result = await api.listRemotes(path);
    remotes.set(result);
  } catch (e) {
    error.set(String(e));
  }
}

export async function refreshRemoteTags() {
  const path = get(repoPath);
  const remote = get(defaultRemote);
  if (!path || !remote) {
    remoteTagMap.set(new Map());
    return;
  }
  try {
    const tags = await api.listRemoteTags(path, remote.name);
    remoteTagMap.set(new Map(tags.map((t) => [t.name, t.oid])));
  } catch {
    // Silently ignore — remote may be unreachable
  }
}

export async function refreshTrackingStatus() {
  const path = get(repoPath);
  const info = get(repoInfo);
  if (!path || !info?.head_branch) {
    trackingStatus.set(null);
    return;
  }
  try {
    const status = await api.getTrackingStatus(path, info.head_branch);
    trackingStatus.set(status);
  } catch {
    trackingStatus.set(null);
  }
}

export async function addRemote(name: string, url: string) {
  const path = get(repoPath);
  if (!path) return;
  try {
    const result = await api.addRemote(path, name, url);
    remotes.set(result);
    addOutput('add-remote', `Added remote '${name}' → ${url}`, '', true);
  } catch (e) {
    error.set(String(e));
    addOutput('add-remote', '', String(e), false);
  }
}

export async function removeRemote(name: string) {
  const path = get(repoPath);
  if (!path) return;
  try {
    const result = await api.removeRemote(path, name);
    remotes.set(result);
    addOutput('remove-remote', `Removed remote '${name}'`, '', true);
  } catch (e) {
    error.set(String(e));
    addOutput('remove-remote', '', String(e), false);
  }
}

export async function fetchFromRemote(remoteName?: string, options?: { silent?: boolean }) {
  const path = get(repoPath);
  if (!path) return;
  if (get(networkOperation)) return;
  networkOperation.set('fetching');
  const silent = options?.silent ?? false;
  try {
    const result = remoteName
      ? await api.fetchRemote(path, remoteName)
      : await api.fetchAllRemotes(path);
    addOutput('fetch', result.output.stdout, result.output.stderr, true);
    if (!silent) addToast(`Fetch: ${result.summary}`, 'success');
    await Promise.all([refreshAll(path), refreshRemotes(path)]);
    refreshRemoteTags(); // fire-and-forget
  } catch (e) {
    const msg = String(e);
    error.set(msg);
    addOutput('fetch', '', msg, false);
    if (!silent) addToast('Fetch failed', 'error');
  } finally {
    networkOperation.set(null);
  }
}

export async function pushToRemote(remoteName?: string, force?: boolean) {
  const path = get(repoPath);
  if (!path) return;
  if (get(networkOperation)) return;
  const remote = remoteName ?? get(defaultRemote)?.name;
  if (!remote) {
    error.set('No remote configured');
    return;
  }
  const info = get(repoInfo);
  const branch = info?.head_branch ?? undefined;
  const ts = get(trackingStatus);
  const setUpstream = !ts?.upstream;
  networkOperation.set('pushing');
  try {
    const result = await api.pushToRemote(path, remote, branch, force, setUpstream);
    addOutput('push', result.output.stdout, result.output.stderr, true);
    addToast(`Push: ${result.summary}`, 'success');
    await refreshAll(path);
  } catch (e) {
    const msg = String(e);
    error.set(msg);
    addOutput('push', '', msg, false);
    addToast('Push failed', 'error');
  } finally {
    networkOperation.set(null);
  }
}

// Commit search actions

let searchVersion = 0;

export async function searchCommitsAction(query: string) {
  const path = get(repoPath);
  if (!path || !query.trim()) {
    searchVersion++;
    commitSearchMatchOids.set(new Set());
    commitSearchLoading.set(false);
    return;
  }
  const version = ++searchVersion;
  commitSearchLoading.set(true);
  try {
    const diffs = get(commitSearchDiffs);
    const oids = await api.searchCommits(path, query, diffs);
    // Discard stale results if a newer search was triggered
    if (version !== searchVersion) return;
    commitSearchMatchOids.set(new Set(oids));
  } catch {
    if (version !== searchVersion) return;
    commitSearchMatchOids.set(new Set());
  } finally {
    if (version === searchVersion) {
      commitSearchLoading.set(false);
    }
  }
}

export function clearCommitSearch() {
  commitSearchActive.set(false);
  commitSearchQuery.set('');
  commitSearchMatchOids.set(new Set());
  commitSearchLoading.set(false);
  commitSearchDiffs.set(false);
}

export async function pullFromRemote(remoteName?: string) {
  const path = get(repoPath);
  if (!path) return;
  if (get(networkOperation)) return;
  const status = get(repoStatus);
  if (status && status.state !== 'Clean') {
    error.set(`Cannot pull while a ${status.state.toLowerCase()} is in progress. Resolve conflicts or abort first.`);
    return;
  }
  const remote = remoteName ?? get(defaultRemote)?.name;
  if (!remote) {
    error.set('No remote configured');
    return;
  }
  const info = get(repoInfo);
  const branch = info?.head_branch ?? undefined;
  networkOperation.set('pulling');
  try {
    const result = await api.pullFromRemote(path, remote, branch);
    addOutput('pull', result.output.stdout, result.output.stderr, !result.merge_conflicts);
    if (result.merge_conflicts) {
      error.set('Pull completed with merge conflicts. Resolve conflicts and commit.');
      addToast('Pull completed with merge conflicts', 'error');
    } else {
      addToast(`Pull: ${result.summary}`, 'success');
    }
    await refreshAll(path);
  } catch (e) {
    const msg = String(e);
    error.set(msg);
    addOutput('pull', '', msg, false);
    addToast('Pull failed', 'error');
  } finally {
    networkOperation.set(null);
  }
}
