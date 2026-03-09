import { getTransport } from '$lib/api';
import type {
  WorktreeInfo,
  WorktreeCreateResult,
  WorktreeRemoveResult,
  WorktreePruneResult,
} from '$lib/api/types';

export async function listWorktrees(path: string): Promise<WorktreeInfo[]> {
  return getTransport().invoke('list_worktrees', { path });
}

export async function addWorktree(
  path: string,
  worktreePath: string,
  branch: string | null,
  newBranch: boolean,
): Promise<WorktreeCreateResult> {
  return getTransport().invoke('add_worktree', {
    path,
    worktreePath,
    branch,
    newBranch,
  });
}

export async function removeWorktree(
  path: string,
  worktreePath: string,
  force: boolean,
): Promise<WorktreeRemoveResult> {
  return getTransport().invoke('remove_worktree', {
    path,
    worktreePath,
    force,
  });
}

export async function lockWorktree(
  path: string,
  worktreePath: string,
  reason?: string,
): Promise<WorktreeInfo[]> {
  return getTransport().invoke('lock_worktree', {
    path,
    worktreePath,
    reason: reason ?? null,
  });
}

export async function unlockWorktree(
  path: string,
  worktreePath: string,
): Promise<WorktreeInfo[]> {
  return getTransport().invoke('unlock_worktree', {
    path,
    worktreePath,
  });
}

export async function pruneWorktrees(
  path: string,
  dryRun: boolean,
): Promise<WorktreePruneResult> {
  return getTransport().invoke('prune_worktrees', {
    path,
    dryRun,
  });
}
