import { invoke } from '@tauri-apps/api/core';
import type {
  RepoInfo,
  RepoStatus,
  CommitGraph,
  Branch,
  FileDiff,
  Remote,
  TrackingStatus,
  FetchResult,
  PushResult,
  PullResult,
} from './types';

export async function openRepo(path: string): Promise<RepoInfo> {
  return invoke('open_repo', { path });
}

export async function getStatus(path: string): Promise<RepoStatus> {
  return invoke('get_status', { path });
}

export async function getRepoInfo(path: string): Promise<RepoInfo> {
  return invoke('get_repo_info', { path });
}

export async function getCommitGraph(
  path: string,
  maxCommits?: number,
  includeRemotes?: boolean
): Promise<CommitGraph> {
  return invoke('get_commit_graph', {
    path,
    maxCommits: maxCommits ?? 500,
    includeRemotes: includeRemotes ?? true,
  });
}

export async function getCommitDetail(path: string, oid: string) {
  return invoke('get_commit_detail', { path, oid });
}

export async function getWorkdirDiff(path: string): Promise<FileDiff[]> {
  return invoke('get_workdir_diff', { path });
}

export async function getFileDiff(path: string, filePath: string): Promise<FileDiff> {
  return invoke('get_file_diff', { path, filePath });
}

export async function getStagedFileDiff(path: string, filePath: string): Promise<FileDiff> {
  return invoke('get_staged_file_diff', { path, filePath });
}

export async function stageFile(path: string, filePath: string): Promise<RepoStatus> {
  return invoke('stage_file', { path, filePath });
}

export async function unstageFile(path: string, filePath: string): Promise<RepoStatus> {
  return invoke('unstage_file', { path, filePath });
}

export async function stageFiles(path: string, filePaths: string[]): Promise<RepoStatus> {
  return invoke('stage_files', { path, filePaths });
}

export async function stageAll(path: string): Promise<RepoStatus> {
  return invoke('stage_all', { path });
}

export async function unstageAll(path: string): Promise<RepoStatus> {
  return invoke('unstage_all', { path });
}

export async function discardAllChanges(path: string): Promise<RepoStatus> {
  return invoke('discard_all_changes', { path });
}

export async function listBranches(path: string): Promise<Branch[]> {
  return invoke('list_branches', { path });
}

export async function createBranch(
  path: string,
  name: string,
  target?: string
): Promise<Branch> {
  return invoke('create_branch', { path, name, target });
}

export async function checkoutBranch(path: string, name: string): Promise<RepoInfo> {
  return invoke('checkout_branch', { path, name });
}

export async function deleteBranch(path: string, name: string): Promise<Branch[]> {
  return invoke('delete_branch', { path, name });
}

export async function resetToCommit(
  path: string,
  commitOid: string,
  resetType: 'soft' | 'mixed' | 'hard'
): Promise<RepoInfo> {
  return invoke('reset_to_commit', { path, commitOid, resetType });
}

export async function createCommit(path: string, message: string): Promise<string> {
  return invoke('create_commit', { path, message });
}

export async function applyStash(path: string, index: number): Promise<RepoStatus> {
  return invoke('apply_stash', { path, index });
}

export async function popStash(path: string, index: number): Promise<RepoStatus> {
  return invoke('pop_stash', { path, index });
}

export async function dropStash(path: string, index: number): Promise<RepoStatus> {
  return invoke('drop_stash', { path, index });
}

// Remote operations

export async function listRemotes(path: string): Promise<Remote[]> {
  return invoke('list_remotes', { path });
}

export async function addRemote(path: string, name: string, url: string): Promise<Remote[]> {
  return invoke('add_remote', { path, name, url });
}

export async function removeRemote(path: string, name: string): Promise<Remote[]> {
  return invoke('remove_remote', { path, name });
}

export async function getTrackingStatus(
  path: string,
  branchName: string
): Promise<TrackingStatus> {
  return invoke('get_tracking_status', { path, branchName });
}

export async function fetchRemote(
  path: string,
  remoteName: string,
  branch?: string
): Promise<FetchResult> {
  return invoke('fetch_remote', { path, remoteName, branch });
}

export async function fetchAllRemotes(path: string): Promise<FetchResult> {
  return invoke('fetch_all_remotes', { path });
}

export async function pushToRemote(
  path: string,
  remoteName: string,
  branch?: string,
  force?: boolean,
  setUpstream?: boolean
): Promise<PushResult> {
  return invoke('push_to_remote', { path, remoteName, branch, force, setUpstream });
}

export async function pullFromRemote(
  path: string,
  remoteName: string,
  branch?: string
): Promise<PullResult> {
  return invoke('pull_from_remote', { path, remoteName, branch });
}

export async function checkoutRemoteBranch(
  path: string,
  remoteBranchName: string
): Promise<RepoInfo> {
  return invoke('checkout_remote_branch', { path, remoteBranchName });
}
