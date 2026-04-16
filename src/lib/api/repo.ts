import { getTransport } from '$lib/api';
import type {
  RepoInfo,
  RepoStatus,
  CommitGraph,
  Branch,
  Tag,
  FileDiff,
  Remote,
  TrackingStatus,
  FetchResult,
  PushResult,
  PullResult,
  CommitResult,
  CloneResult,
  RebaseResult,
  MergeResult,
  OperationOutput,
  ConflictedFileContent,
  RemoteTagInfo,
  OpenRepoResult,
  StashEntry,
  MergedBranch,
  CheckpointRef,
} from './types';

export async function openRepo(path: string): Promise<OpenRepoResult> {
  return getTransport().invoke('open_repo', { path });
}

export async function closeRepo(): Promise<void> {
  return getTransport().invoke('close_repo');
}

export async function setWatcherInterval(intervalMs: number): Promise<void> {
  return getTransport().invoke('set_watcher_interval', { intervalMs });
}

export async function getStatus(path: string): Promise<RepoStatus> {
  return getTransport().invoke('get_status', { path });
}

export async function getRepoInfo(path: string): Promise<RepoInfo> {
  return getTransport().invoke('get_repo_info', { path });
}

export async function getCommitGraph(
  path: string,
  maxCommits?: number,
  includeRemotes?: boolean,
  excludedAuthors?: string[]
): Promise<CommitGraph> {
  return getTransport().invoke('get_commit_graph', {
    path,
    maxCommits: maxCommits ?? 500,
    includeRemotes: includeRemotes ?? true,
    excludedAuthors: excludedAuthors ?? [],
  });
}

export async function searchCommits(
  path: string,
  query: string,
  searchDiffs?: boolean,
  maxCommits?: number,
  includeRemotes?: boolean
): Promise<string[]> {
  return getTransport().invoke('search_commits', {
    path,
    query,
    searchDiffs: searchDiffs ?? false,
    maxCommits: maxCommits ?? 500,
    includeRemotes: includeRemotes ?? true,
  });
}

export async function getCommitDetail(path: string, oid: string) {
  return getTransport().invoke('get_commit_detail', { path, oid });
}

export async function getCommitDiff(path: string, oid: string): Promise<FileDiff[]> {
  return getTransport().invoke('get_commit_diff', { path, oid });
}

export async function getWorkdirDiff(path: string): Promise<FileDiff[]> {
  return getTransport().invoke('get_workdir_diff', { path });
}

export async function getFileDiff(path: string, filePath: string): Promise<FileDiff> {
  return getTransport().invoke('get_file_diff', { path, filePath });
}

export async function getStagedFileDiff(path: string, filePath: string): Promise<FileDiff> {
  return getTransport().invoke('get_staged_file_diff', { path, filePath });
}

export async function stageFile(path: string, filePath: string): Promise<RepoStatus> {
  return getTransport().invoke('stage_file', { path, filePath });
}

export async function unstageFile(path: string, filePath: string): Promise<RepoStatus> {
  return getTransport().invoke('unstage_file', { path, filePath });
}

export async function stageFiles(path: string, filePaths: string[]): Promise<RepoStatus> {
  return getTransport().invoke('stage_files', { path, filePaths });
}

export async function stageAll(path: string): Promise<RepoStatus> {
  return getTransport().invoke('stage_all', { path });
}

export async function unstageAll(path: string): Promise<RepoStatus> {
  return getTransport().invoke('unstage_all', { path });
}

export async function discardAllChanges(path: string): Promise<RepoStatus> {
  return getTransport().invoke('discard_all_changes', { path });
}

export async function discardFiles(
  path: string,
  staged: string[],
  unstaged: string[],
  untracked: string[],
): Promise<RepoStatus> {
  return getTransport().invoke('discard_files', { path, staged, unstaged, untracked });
}

export async function addToGitignore(path: string, pattern: string): Promise<RepoStatus> {
  return getTransport().invoke('add_to_gitignore', { path, pattern });
}

export async function listBranches(path: string): Promise<Branch[]> {
  return getTransport().invoke('list_branches', { path });
}

export async function createBranch(
  path: string,
  name: string,
  target?: string
): Promise<Branch> {
  return getTransport().invoke('create_branch', { path, name, target });
}

export async function checkoutBranch(path: string, name: string): Promise<RepoInfo> {
  return getTransport().invoke('checkout_branch', { path, name });
}

export async function deleteBranch(path: string, name: string): Promise<Branch[]> {
  return getTransport().invoke('delete_branch', { path, name });
}

export async function findMergedBranches(path: string): Promise<MergedBranch[]> {
  return getTransport().invoke('find_merged_branches', { path });
}

export async function cleanupMergedBranches(path: string, branches: MergedBranch[]): Promise<string[]> {
  return getTransport().invoke('cleanup_merged_branches', { path, branches });
}

export async function resetToCommit(
  path: string,
  commitOid: string,
  resetType: 'soft' | 'mixed' | 'hard'
): Promise<RepoInfo> {
  return getTransport().invoke('reset_to_commit', { path, commitOid, resetType });
}

export async function createCommit(path: string, message: string): Promise<CommitResult> {
  return getTransport().invoke('create_commit', { path, message });
}

export async function saveStash(path: string, message?: string): Promise<RepoStatus> {
  return getTransport().invoke('save_stash', { path, message: message || null });
}

export async function listStashes(path: string): Promise<StashEntry[]> {
  return getTransport().invoke('list_stashes', { path });
}

export async function applyStash(path: string, index: number): Promise<RepoStatus> {
  return getTransport().invoke('apply_stash', { path, index });
}

export async function popStash(path: string, index: number): Promise<RepoStatus> {
  return getTransport().invoke('pop_stash', { path, index });
}

export async function dropStash(path: string, index: number): Promise<RepoStatus> {
  return getTransport().invoke('drop_stash', { path, index });
}

// Checkpoint operations

export async function findCheckpointRefs(path: string): Promise<CheckpointRef[]> {
  return getTransport().invoke('find_checkpoint_refs', { path });
}

export async function purgeCheckpointRefs(path: string, refs: CheckpointRef[]): Promise<number> {
  return getTransport().invoke('purge_checkpoint_refs', { path, refs });
}

// Remote operations

export async function listRemotes(path: string): Promise<Remote[]> {
  return getTransport().invoke('list_remotes', { path });
}

export async function addRemote(path: string, name: string, url: string): Promise<Remote[]> {
  return getTransport().invoke('add_remote', { path, name, url });
}

export async function removeRemote(path: string, name: string): Promise<Remote[]> {
  return getTransport().invoke('remove_remote', { path, name });
}

export async function getTrackingStatus(
  path: string,
  branchName: string
): Promise<TrackingStatus> {
  return getTransport().invoke('get_tracking_status', { path, branchName });
}

export async function fetchRemote(
  path: string,
  remoteName: string,
  branch?: string
): Promise<FetchResult> {
  return getTransport().invoke('fetch_remote', { path, remoteName, branch });
}

export async function fetchAllRemotes(path: string): Promise<FetchResult> {
  return getTransport().invoke('fetch_all_remotes', { path });
}

export async function pushToRemote(
  path: string,
  remoteName: string,
  branch?: string,
  force?: boolean,
  setUpstream?: boolean
): Promise<PushResult> {
  return getTransport().invoke('push_to_remote', { path, remoteName, branch, force, setUpstream });
}

export async function pullFromRemote(
  path: string,
  remoteName: string,
  branch?: string
): Promise<PullResult> {
  return getTransport().invoke('pull_from_remote', { path, remoteName, branch });
}

export async function deleteRemoteBranch(
  path: string,
  remoteName: string,
  branch: string
): Promise<Branch[]> {
  return getTransport().invoke('delete_remote_branch', { path, remoteName, branch });
}

export async function cloneRepo(url: string, dest: string): Promise<CloneResult> {
  return getTransport().invoke('clone_repo', { url, dest });
}

export async function rebaseOnto(path: string, ontoBranch: string): Promise<RebaseResult> {
  return getTransport().invoke('rebase_onto', { path, ontoBranch });
}

export async function mergeInto(path: string, branchName: string): Promise<MergeResult> {
  return getTransport().invoke('merge_into', { path, branchName });
}

export async function rebaseContinue(path: string): Promise<RebaseResult> {
  return getTransport().invoke('rebase_continue', { path });
}

export async function rebaseAbort(path: string): Promise<RebaseResult> {
  return getTransport().invoke('rebase_abort', { path });
}

export async function mergeAbort(path: string): Promise<MergeResult> {
  return getTransport().invoke('merge_abort', { path });
}

export async function mergeContinue(path: string): Promise<MergeResult> {
  return getTransport().invoke('merge_continue', { path });
}

export async function cherryPickAbort(path: string): Promise<OperationOutput> {
  return getTransport().invoke('cherry_pick_abort', { path });
}

export async function getConflictedFile(path: string, filePath: string): Promise<ConflictedFileContent> {
  return getTransport().invoke('get_conflicted_file', { path, filePath });
}

export async function writeResolvedFile(path: string, filePath: string, content: string): Promise<RepoStatus> {
  return getTransport().invoke('write_resolved_file', { path, filePath, content });
}

export async function checkoutRemoteBranch(
  path: string,
  remoteBranchName: string
): Promise<RepoInfo> {
  return getTransport().invoke('checkout_remote_branch', { path, remoteBranchName });
}

// Tag operations

export async function createTag(
  path: string,
  name: string,
  targetOid: string,
  message?: string
): Promise<Tag> {
  return getTransport().invoke('create_tag', { path, name, targetOid, message });
}

export async function deleteTag(path: string, name: string): Promise<void> {
  return getTransport().invoke('delete_tag', { path, name });
}

export async function moveTag(path: string, name: string, targetOid: string): Promise<Tag> {
  return getTransport().invoke('move_tag', { path, name, targetOid });
}

export async function pushTag(
  path: string,
  remoteName: string,
  tagName: string,
  force?: boolean
): Promise<PushResult> {
  return getTransport().invoke('push_tag', { path, remoteName, tagName, force: force ?? false });
}

export async function deleteRemoteTag(
  path: string,
  remoteName: string,
  tagName: string
): Promise<void> {
  return getTransport().invoke('delete_remote_tag', { path, remoteName, tagName });
}

export async function listRemoteTags(
  path: string,
  remoteName: string
): Promise<RemoteTagInfo[]> {
  return getTransport().invoke('list_remote_tags', { path, remoteName });
}
