import { writable, derived, get } from 'svelte/store';
import type {
  RepoInfo,
  RepoStatus,
  CommitGraph,
  Commit,
  Branch,
  FileDiff,
  FileStatus,
  Remote,
  TrackingStatus,
} from '$lib/api/types';
import * as api from '$lib/api/repo';
import { trackRepoOpen } from '$lib/stores/settings';
import { addOutput } from '$lib/stores/output';
import { startWatcherListeners, stopWatcherListeners } from '$lib/stores/watcher';

// File selection types
export type FileSection = 'staged' | 'unstaged' | 'untracked';

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
export const selectedFile = writable<SelectedFileInfo | null>(null);
export const loading = writable(false);
export const error = writable<string | null>(null);

// Remote state
export const remotes = writable<Remote[]>([]);
export const trackingStatus = writable<TrackingStatus | null>(null);
export const networkOperation = writable<string | null>(null);

// Remote tag tracking
export const remoteTagNames = writable<Set<string>>(new Set());

// Scroll-to-commit (used by tags list to jump to a commit in the graph)
export const scrollToCommitOid = writable<string | null>(null);

// Commit search state
export const commitSearchActive = writable<boolean>(false);
export const commitSearchQuery = writable<string>('');
export const commitSearchMatchOids = writable<Set<string>>(new Set());
export const commitSearchLoading = writable<boolean>(false);
export const commitSearchDiffs = writable<boolean>(false);

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

// Branch conflict prompt (shown when checking out a remote branch with existing local)
export interface BranchConflictInfo {
  localName: string;
  remoteBranchName: string;
}
export const branchConflictPrompt = writable<BranchConflictInfo | null>(null);

// Derived stores
export const hasRepo = derived(repoInfo, ($info) => $info !== null);
export const isFileSelected = derived(selectedFile, ($f) => $f !== null);
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

// Helpers
function getAllFiles(): SelectedFileInfo[] {
  const status = get(repoStatus);
  if (!status) return [];
  return [
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
    const info = await api.openRepo(path);
    repoPath.set(path);
    repoInfo.set(info);
    await refreshAll(path);
    await refreshRemotes(path);
    await refreshTrackingStatus();
    refreshRemoteTags(); // fire-and-forget (needs defaultRemote populated)
    await trackRepoOpen(path);
    await startWatcherListeners();
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
  remotes.set([]);
  remoteTagNames.set(new Set());
  trackingStatus.set(null);
  clearCommitSearch();
  error.set(null);
}

export async function refreshAll(path: string) {
  try {
    const [status, graph] = await Promise.all([
      api.getStatus(path),
      api.getCommitGraph(path),
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

// File selection
export async function selectFile(path: string, section: FileSection) {
  const repoPathVal = get(repoPath);
  if (!repoPathVal) return;
  selectedFile.set({ path, section });
  selectedCommit.set(null);
  selectedFileDiff.set(null);
  if (section === 'staged') {
    await viewStagedFileDiff(repoPathVal, path);
  } else {
    await viewFileDiff(repoPathVal, path);
  }
}

export function clearFileSelection() {
  selectedFile.set(null);
  selectedFileDiff.set(null);
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

export async function createAndCheckoutBranch(name: string) {
  const path = get(repoPath);
  if (!path) return;
  try {
    await api.createBranch(path, name);
    const info = await api.checkoutBranch(path, name);
    repoInfo.set(info);
    await refreshAll(path);
  } catch (e) {
    error.set(String(e));
  }
}

export async function createBranchAtCommit(name: string, targetOid: string) {
  const path = get(repoPath);
  if (!path) return;
  try {
    await api.createBranch(path, name, targetOid);
    const info = await api.checkoutBranch(path, name);
    repoInfo.set(info);
    await refreshAll(path);
  } catch (e) {
    error.set(String(e));
  }
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

export async function rebaseOnto(ontoBranch: string) {
  const path = get(repoPath);
  if (!path) return;
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

export async function mergeInto(branchName: string) {
  const path = get(repoPath);
  if (!path) return;
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

// Jump to a tag's commit in the graph
export function jumpToTag(targetOid: string) {
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
  } catch (e) {
    error.set(String(e));
  }
}

export async function pushTag(tagName: string) {
  const path = get(repoPath);
  if (!path) return;
  const remote = get(defaultRemote);
  if (!remote) {
    error.set('No remote configured');
    return;
  }
  try {
    const result = await api.pushTag(path, remote.name, tagName);
    addOutput('push-tag', result.output.stdout, result.output.stderr, true);
    refreshRemoteTags(); // fire-and-forget
  } catch (e) {
    error.set(String(e));
    addOutput('push-tag', '', String(e), false);
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
    refreshRemoteTags(); // fire-and-forget
  } catch (e) {
    error.set(String(e));
    addOutput('delete-remote-tag', '', String(e), false);
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

export async function applyStash(index: number) {
  const path = get(repoPath);
  if (!path) return;
  try {
    const status = await api.applyStash(path, index);
    repoStatus.set(status);
    await refreshAll(path);
  } catch (e) {
    error.set(String(e));
  }
}

export async function popStash(index: number) {
  const path = get(repoPath);
  if (!path) return;
  try {
    const status = await api.popStash(path, index);
    repoStatus.set(status);
    await refreshAll(path);
  } catch (e) {
    error.set(String(e));
  }
}

export async function dropStash(index: number) {
  const path = get(repoPath);
  if (!path) return;
  try {
    const status = await api.dropStash(path, index);
    repoStatus.set(status);
    await refreshAll(path);
  } catch (e) {
    error.set(String(e));
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
    remoteTagNames.set(new Set());
    return;
  }
  try {
    const names = await api.listRemoteTags(path, remote.name);
    remoteTagNames.set(new Set(names));
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

export async function fetchFromRemote(remoteName?: string) {
  const path = get(repoPath);
  if (!path) return;
  if (get(networkOperation)) return;
  networkOperation.set('fetching');
  try {
    const result = remoteName
      ? await api.fetchRemote(path, remoteName)
      : await api.fetchAllRemotes(path);
    addOutput('fetch', result.output.stdout, result.output.stderr, true);
    await refreshAll(path);
    await refreshRemotes(path);
    refreshRemoteTags(); // fire-and-forget
  } catch (e) {
    error.set(String(e));
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
    await refreshAll(path);
  } catch (e) {
    error.set(String(e));
  } finally {
    networkOperation.set(null);
  }
}

// Commit search actions

export async function searchCommitsAction(query: string) {
  const path = get(repoPath);
  if (!path || !query.trim()) {
    commitSearchMatchOids.set(new Set());
    return;
  }
  commitSearchLoading.set(true);
  try {
    const diffs = get(commitSearchDiffs);
    const oids = await api.searchCommits(path, query, diffs);
    commitSearchMatchOids.set(new Set(oids));
  } catch {
    commitSearchMatchOids.set(new Set());
  } finally {
    commitSearchLoading.set(false);
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
    }
    await refreshAll(path);
  } catch (e) {
    error.set(String(e));
  } finally {
    networkOperation.set(null);
  }
}
