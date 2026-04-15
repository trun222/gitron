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

export interface RemoteTagInfo {
  name: string;
  oid: string;
}

export interface MergedBranch {
  name: string;
  is_remote: boolean;
  remote: string | null;
  short_name: string;
}

export interface CheckpointRef {
  refname: string;
  short_oid: string;
  source: string;
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

export type RepoState =
  | 'Clean'
  | 'Merging'
  | 'Rebasing'
  | 'RebasingInteractive'
  | 'CherryPicking'
  | 'Reverting';

export interface RepoStatus {
  staged: FileStatus[];
  unstaged: FileStatus[];
  untracked: string[];
  conflicted: string[];
  state: RepoState;
  operation_step: number | null;
  operation_total: number | null;
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

export interface OpenRepoResult {
  info: RepoInfo;
  status: RepoStatus;
  graph: CommitGraph;
  remotes: Remote[];
  tracking: TrackingStatus | null;
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

export interface RebaseResult {
  success: boolean;
  conflicted: boolean;
  output: OperationOutput;
}

export interface MergeResult {
  success: boolean;
  conflicted: boolean;
  output: OperationOutput;
}

// Conflict types (matching crates/gitron-core/src/git/types.rs)

export interface ConflictSection {
  ours: string[];
  theirs: string[];
  ours_label: string;
  theirs_label: string;
  start_line: number;
  end_line: number;
}

export interface ConflictedFileContent {
  path: string;
  raw_content: string;
  lines: string[];
  conflict_sections: ConflictSection[];
  is_binary: boolean;
}

// Worktree types (matching crates/gitron-core/src/git/types.rs)

export interface WorktreeInfo {
  name: string;
  path: string;
  branch: string | null;
  head_oid: string | null;
  head_short_oid: string | null;
  is_main: boolean;
  is_locked: boolean;
  lock_reason: string | null;
  is_valid: boolean;
}

export interface WorktreeCreateResult {
  worktree: WorktreeInfo;
  output: OperationOutput;
}

export interface WorktreeRemoveResult {
  success: boolean;
  output: OperationOutput;
}

export interface WorktreePruneResult {
  pruned: string[];
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

// AI types (matching src-tauri/src/ai/types.rs)

export interface AIProvider {
  id: string;
  name: string;
  has_key: boolean;
  models: AIModel[];
  base_url: string | null;
}

export interface AIModel {
  id: string;
  name: string;
}

export interface GenerateResult {
  title: string;
  body: string;
}

export interface AISettings {
  selected_provider: string | null;
  selected_model: string | null;
  /** Remembers the last selected model per provider so switching back restores it. */
  selected_models: Record<string, string>;
  custom_base_urls: Record<string, string>;
  /** Max output tokens for AI generation. Default: 1500. */
  max_tokens: number;
}

// Watcher event payloads (matching Rust event structs)

export interface StatusChangedPayload {
  status: RepoStatus;
}

export interface RefsChangedPayload {
  graph: CommitGraph;
  status: RepoStatus;
}

// Settings types
export type ChangesViewMode = 'file' | 'tree';
export type ThemeMode = 'dark' | 'light' | 'system' | 'tron' | 'tron-enhanced' | 'synthwave';
export type AutoFetchInterval = 0 | 15 | 30 | 60 | 300 | 900; // seconds; 0 = off
export type FileWatcherInterval = 0 | 1000 | 2000 | 3000 | 5000; // ms; 0 = native only
export type ZoomLevel = 0.8 | 0.9 | 1.0 | 1.1 | 1.25 | 1.5;
export type EditorFontSize = 12 | 13 | 14 | 16;
export type MonoFont = 'default' | 'fira-code' | 'jetbrains-mono' | 'cascadia-code' | 'sf-mono' | 'menlo';
export type TerminalCursorStyle = 'block' | 'underline' | 'bar';

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

export interface GraphColumnVisibility {
  graph: boolean;
  message: boolean;
  author: boolean;
  date: boolean;
  sha: boolean;
}

export interface AppSettings {
  lastActiveRepo: string | null;
  recentRepos: RecentRepo[];
  graphColumnWidths?: GraphColumnWidths;
  sidebarCollapsed?: boolean;
  theme?: ThemeMode;
  autoFetchInterval?: AutoFetchInterval;
  autoShowOutput?: boolean;
  fileWatcherInterval?: FileWatcherInterval;
  zoomLevel?: ZoomLevel;
  highContrast?: boolean;
  editorFontSize?: EditorFontSize;
  monoFont?: MonoFont;
  showTagsList?: boolean;
  changesViewMode?: ChangesViewMode;
  verboseGitErrors?: boolean;
  terminalApp?: string;
  terminalShell?: string;
  terminalFontSize?: number;
  terminalFontFamily?: string;
  terminalCursorStyle?: TerminalCursorStyle;
  terminalScrollback?: number;
  showWorktreesList?: boolean;
  treeExpandedByDefault?: boolean;
  excludedAuthors?: string[];
  protectedBranches?: string[];
  tagsExpanded?: boolean;
  worktreesExpanded?: boolean;
  outputPanelOpen?: boolean;
  conflictedExpanded?: boolean;
  stagedExpanded?: boolean;
  unstagedExpanded?: boolean;
  untrackedExpanded?: boolean;
  committedExpanded?: boolean;
  graphColumnVisibility?: GraphColumnVisibility;
}
