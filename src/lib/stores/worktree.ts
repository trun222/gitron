import { writable, derived, get } from 'svelte/store';
import { repoPath } from '$lib/stores/repo';
import { addToast, updateToast } from '$lib/stores/toast';
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
    addToast(`Created worktree "${result.worktree.name}" (${result.worktree.branch ?? 'detached'})`, 'success');
    await refreshWorktrees();
    return true;
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    worktreeError.set(msg);
    addToast(`Failed to create worktree: ${msg}`, 'error');
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
    const name = worktreePath.split('/').pop() ?? worktreePath;
    if (result.success) {
      addToast(`Removed worktree "${name}"`, 'success');
    } else {
      const detail = result.output.stderr || 'Unknown error';
      worktreeError.set(detail);
      addToast(`Failed to remove worktree "${name}": ${detail}`, 'error');
    }
    await refreshWorktrees();
    return result.success;
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    worktreeError.set(msg);
    addToast(`Failed to remove worktree: ${msg}`, 'error');
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
    const op = currentlyLocked ? 'unlock' : 'lock';
    addToast(`Failed to ${op} worktree: ${msg}`, 'error');
  }
}

export async function deleteAllWorktrees(force: boolean): Promise<boolean> {
  const path = get(repoPath);
  if (!path) return false;

  const linked = get(linkedWorktrees);
  if (linked.length === 0) return true;

  const total = linked.length;
  const toastId = addToast(`Removing worktrees (0/${total})...`, 'info', 0);

  let removed = 0;
  let failed = 0;
  const errors: string[] = [];

  for (const wt of linked) {
    const name = wt.path.split('/').pop() ?? wt.path;
    updateToast(toastId, `Removing "${name}" (${removed + failed + 1}/${total})...`);

    try {
      const result = await api.removeWorktree(path, wt.path, force);
      if (result.success) {
        removed++;
      } else {
        failed++;
        errors.push(`${name}: ${result.output.stderr || 'Unknown error'}`);
      }
    } catch (e) {
      failed++;
      errors.push(`${name}: ${e instanceof Error ? e.message : String(e)}`);
    }
    await refreshWorktrees();
  }

  if (failed === 0) {
    updateToast(toastId, `Removed all ${removed} worktree(s)`, 'success', 4000);
  } else {
    updateToast(toastId, `Removed ${removed}/${total} worktree(s), ${failed} failed`, 'error', 6000);
    addOutput('Remove all worktrees', `Removed ${removed}/${total}`, errors.join('\n'), false);
  }

  return failed === 0;
}

export async function pruneStaleWorktrees(): Promise<void> {
  const path = get(repoPath);
  if (!path) return;

  try {
    const result = await api.pruneWorktrees(path, false);
    if (result.pruned.length > 0) {
      addToast(`Pruned ${result.pruned.length} stale worktree(s)`, 'success');
    } else {
      addToast('No stale worktrees to prune', 'info');
    }
    await refreshWorktrees();
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    worktreeError.set(msg);
    addToast(`Failed to prune worktrees: ${msg}`, 'error');
  }
}
