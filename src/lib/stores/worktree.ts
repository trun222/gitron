import { writable, derived, get } from 'svelte/store';
import { repoPath } from '$lib/stores/repo';
import { addOutput } from '$lib/stores/output';
import * as api from '$lib/api/worktree';
import type { WorktreeInfo } from '$lib/api/types';

// State
export const worktrees = writable<WorktreeInfo[]>([]);
export const worktreeLoading = writable(false);
export const worktreeError = writable<string | null>(null);
export const showAddWorktreeDialog = writable(false);

// Derived
export const linkedWorktrees = derived(worktrees, ($wt) =>
  $wt.filter((w) => !w.is_main)
);
export const worktreeCount = derived(worktrees, ($wt) => $wt.length);

// Actions

export async function refreshWorktrees(): Promise<void> {
  const path = get(repoPath);
  if (!path) return;

  worktreeLoading.set(true);
  worktreeError.set(null);

  try {
    const list = await api.listWorktrees(path);
    worktrees.set(list);
  } catch (e) {
    worktreeError.set(e instanceof Error ? e.message : String(e));
  } finally {
    worktreeLoading.set(false);
  }
}

export async function createWorktree(
  worktreePath: string,
  branch: string | null,
  newBranch: boolean,
): Promise<boolean> {
  const path = get(repoPath);
  if (!path) return false;

  try {
    const result = await api.addWorktree(path, worktreePath, branch, newBranch);
    addOutput(
      'Add worktree',
      `Created worktree: ${result.worktree.path} (${result.worktree.branch ?? 'detached'})`,
      result.output.stderr,
      true,
    );
    await refreshWorktrees();
    return true;
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    worktreeError.set(msg);
    addOutput('Add worktree', '', msg, false);
    return false;
  }
}

export async function deleteWorktree(
  worktreePath: string,
  force: boolean,
): Promise<boolean> {
  const path = get(repoPath);
  if (!path) return false;

  try {
    const result = await api.removeWorktree(path, worktreePath, force);
    if (result.success) {
      addOutput('Remove worktree', `Removed: ${worktreePath}`, result.output.stderr, true);
    } else {
      addOutput('Remove worktree', '', result.output.stderr, false);
      worktreeError.set(result.output.stderr);
    }
    await refreshWorktrees();
    return result.success;
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    worktreeError.set(msg);
    addOutput('Remove worktree', '', msg, false);
    return false;
  }
}

export async function toggleWorktreeLock(
  worktreePath: string,
  currentlyLocked: boolean,
  reason?: string,
): Promise<void> {
  const path = get(repoPath);
  if (!path) return;

  try {
    let list: WorktreeInfo[];
    if (currentlyLocked) {
      list = await api.unlockWorktree(path, worktreePath);
    } else {
      list = await api.lockWorktree(path, worktreePath, reason);
    }
    worktrees.set(list);
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    worktreeError.set(msg);
    addOutput(
      currentlyLocked ? 'Unlock worktree' : 'Lock worktree',
      '',
      msg,
      false,
    );
  }
}

export async function deleteAllWorktrees(force: boolean): Promise<boolean> {
  const path = get(repoPath);
  if (!path) return false;

  const linked = get(linkedWorktrees);
  if (linked.length === 0) return true;

  let allOk = true;
  for (const wt of linked) {
    const ok = await deleteWorktree(wt.path, force);
    if (!ok) allOk = false;
  }
  return allOk;
}

export async function pruneStaleWorktrees(): Promise<void> {
  const path = get(repoPath);
  if (!path) return;

  try {
    const result = await api.pruneWorktrees(path, false);
    if (result.pruned.length > 0) {
      addOutput('Prune worktrees', result.pruned.join('\n'), result.output.stderr, true);
    }
    await refreshWorktrees();
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    worktreeError.set(msg);
    addOutput('Prune worktrees', '', msg, false);
  }
}
