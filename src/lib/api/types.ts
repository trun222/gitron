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

export interface CommitGraph {
  commits: Commit[];
  branches: Branch[];
  tags: Tag[];
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
}
