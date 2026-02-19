// Types matching the Rust backend types (src-tauri/src/git/types.rs)

export interface Commit {
  oid: string;
  short_oid: string;
  message: string;
  summary: string;
  author: Signature;
  committer: Signature;
  parents: string[];
  timestamp: string;
}

export interface Signature {
  name: string;
  email: string;
}

export interface Branch {
  name: string;
  is_head: boolean;
  is_remote: boolean;
  upstream: string | null;
  target_oid: string | null;
}

export interface Tag {
  name: string;
  target_oid: string;
  is_annotated: boolean;
  message: string | null;
}

export interface StashEntry {
  index: number;
  oid: string;
  short_oid: string;
  message: string;
  base_oid: string;
}

export interface CommitGraph {
  commits: Commit[];
  branches: Branch[];
  tags: Tag[];
  stashes: StashEntry[];
  head_oid: string | null;
  head_branch: string | null;
  layout: GraphLayout | null;
}

export interface GraphLayout {
  nodes: GraphNode[];
  max_lanes: number;
  branch_colors: BranchColorEntry[];
}

export interface GraphNode {
  oid: string;
  lane: number;
  color_index: number;
  branch_name: string | null;
  edges: GraphEdge[];
}

export interface GraphEdge {
  from_lane: number;
  to_lane: number;
  to_row: number;
  color_index: number;
  edge_type: GraphEdgeType;
}

export type GraphEdgeType = 'Straight' | 'MergeIn' | 'ForkOut';

export interface BranchColorEntry {
  name: string;
  color_index: number;
}

export interface RepoStatus {
  staged: FileStatus[];
  unstaged: FileStatus[];
  untracked: string[];
  conflicted: string[];
}

export interface FileStatus {
  path: string;
  status: FileStatusType;
}

export type FileStatusType =
  | 'Added'
  | 'Modified'
  | 'Deleted'
  | 'Renamed'
  | 'Copied'
  | 'TypeChanged';

export interface FileDiff {
  path: string;
  old_path: string | null;
  hunks: DiffHunk[];
  is_binary: boolean;
  status: FileStatusType;
}

export interface DiffHunk {
  header: string;
  old_start: number;
  old_lines: number;
  new_start: number;
  new_lines: number;
  lines: DiffLine[];
}

export interface DiffLine {
  origin: DiffLineType;
  content: string;
  old_lineno: number | null;
  new_lineno: number | null;
}

export type DiffLineType = 'Context' | 'Addition' | 'Deletion' | 'Header';

export interface RepoInfo {
  path: string;
  workdir: string;
  head_branch: string | null;
  head_oid: string | null;
  is_bare: boolean;
  is_empty: boolean;
}

export interface Remote {
  name: string;
  url: string;
  push_url: string | null;
}

export interface TrackingStatus {
  ahead: number;
  behind: number;
  upstream: string | null;
}

export interface OperationOutput {
  stdout: string;
  stderr: string;
}

export interface CommitResult {
  oid: string;
  success: boolean;
  output: OperationOutput;
}

export interface FetchResult {
  remote: string;
  updated_refs: string[];
  summary: string;
  output: OperationOutput;
}

export interface PushResult {
  remote: string;
  branch: string;
  summary: string;
  output: OperationOutput;
}

export interface PullResult {
  remote: string;
  branch: string;
  summary: string;
  merge_conflicts: boolean;
  output: OperationOutput;
}

// GitHub OAuth types (matching src-tauri/src/github/types.rs)

export interface GitHubUser {
  login: string;
  id: number;
  name: string | null;
  email: string | null;
  avatar_url: string;
}

export type GitHubAuthStatus =
  | { type: 'NotAuthenticated' }
  | { type: 'AwaitingUserCode' }
  | { type: 'Authenticated'; user: GitHubUser }
  | { type: 'TokenExpired' }
  | { type: 'Failed'; message: string };

export interface GitHubAuthInfo {
  status: GitHubAuthStatus;
}

export interface DeviceCodeResponse {
  device_code: string;
  user_code: string;
  verification_uri: string;
  expires_in: number;
  interval: number;
}

export interface CloneResult {
  path: string;
  repo_info: RepoInfo;
  output: OperationOutput;
}

export interface GitHubRepo {
  id: number;
  full_name: string;
  name: string;
  description: string | null;
  private: boolean;
  clone_url: string;
  updated_at: string;
  owner: GitHubRepoOwner;
}

export interface GitHubRepoOwner {
  login: string;
  avatar_url: string;
}

// Settings types
export type ThemeMode = 'dark' | 'light' | 'system' | 'tron' | 'tron-enhanced';
export type AutoFetchInterval = 0 | 60 | 300 | 900; // seconds; 0 = off

// Persistence types (frontend-only, used by tauri-plugin-store)

export interface RecentRepo {
  path: string;
  name: string;
  lastOpened: string; // ISO 8601
  pinned: boolean;
}

export interface GraphColumnWidths {
  graph: number;
  author: number;
  date: number;
  sha: number;
}

export interface AppSettings {
  lastActiveRepo: string | null;
  recentRepos: RecentRepo[];
  graphColumnWidths?: GraphColumnWidths;
  sidebarCollapsed?: boolean;
  theme?: ThemeMode;
  autoFetchInterval?: AutoFetchInterval;
}
