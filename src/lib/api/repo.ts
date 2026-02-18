import { invoke } from '@tauri-apps/api/core';
import type { RepoInfo, RepoStatus, CommitGraph, Branch, FileDiff } from './types';

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

export async function createCommit(path: string, message: string): Promise<string> {
  return invoke('create_commit', { path, message });
}
