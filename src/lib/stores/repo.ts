import { writable, derived } from 'svelte/store';
import type { RepoInfo, RepoStatus, CommitGraph, Commit, Branch, FileDiff } from '$lib/api/types';
import * as api from '$lib/api/repo';
import { trackRepoOpen } from '$lib/stores/settings';

// Core repo state
export const repoPath = writable<string | null>(null);
export const repoInfo = writable<RepoInfo | null>(null);
export const repoStatus = writable<RepoStatus | null>(null);
export const commitGraph = writable<CommitGraph | null>(null);
export const selectedCommit = writable<Commit | null>(null);
export const selectedFileDiff = writable<FileDiff | null>(null);
export const loading = writable(false);
export const error = writable<string | null>(null);

// Derived stores
export const hasRepo = derived(repoInfo, ($info) => $info !== null);
export const currentBranch = derived(repoInfo, ($info) => $info?.head_branch ?? null);
export const localBranches = derived(commitGraph, ($graph) =>
  $graph?.branches.filter((b) => !b.is_remote) ?? []
);
export const remoteBranches = derived(commitGraph, ($graph) =>
  $graph?.branches.filter((b) => b.is_remote) ?? []
);
export const stagedCount = derived(repoStatus, ($status) => $status?.staged.length ?? 0);
export const unstagedCount = derived(
  repoStatus,
  ($status) => ($status?.unstaged.length ?? 0) + ($status?.untracked.length ?? 0)
);

// Actions
export async function openRepo(path: string) {
  loading.set(true);
  error.set(null);
  try {
    const info = await api.openRepo(path);
    repoPath.set(path);
    repoInfo.set(info);
    await refreshAll(path);
    await trackRepoOpen(path);
  } catch (e) {
    error.set(String(e));
  } finally {
    loading.set(false);
  }
}

export async function refreshAll(path: string) {
  try {
    const [status, graph] = await Promise.all([
      api.getStatus(path),
      api.getCommitGraph(path),
    ]);
    repoStatus.set(status);
    commitGraph.set(graph);
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
