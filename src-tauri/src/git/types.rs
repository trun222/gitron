use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a commit in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub oid: String,
    pub short_oid: String,
    pub message: String,
    pub summary: String,
    pub author: Signature,
    pub committer: Signature,
    pub parents: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

/// Author/committer signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub name: String,
    pub email: String,
}

/// A branch reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    pub is_head: bool,
    pub is_remote: bool,
    pub upstream: Option<String>,
    pub target_oid: Option<String>,
}

/// A tag reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub target_oid: String,
    pub is_annotated: bool,
    pub message: Option<String>,
}

/// A stash entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StashEntry {
    pub index: usize,
    pub oid: String,
    pub short_oid: String,
    pub message: String,
    pub base_oid: String,
}

/// The commit graph — nodes and their relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitGraph {
    pub commits: Vec<Commit>,
    pub branches: Vec<Branch>,
    pub tags: Vec<Tag>,
    pub stashes: Vec<StashEntry>,
    pub head_oid: Option<String>,
    pub head_branch: Option<String>,
    pub layout: Option<GraphLayout>,
}

/// Graph layout data for visualization (1:1 with commits)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphLayout {
    pub nodes: Vec<GraphNode>,
    pub max_lanes: usize,
    pub branch_colors: Vec<BranchColorEntry>,
}

/// A single node in the graph layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub oid: String,
    pub lane: usize,
    pub color_index: usize,
    pub branch_name: Option<String>,
    pub edges: Vec<GraphEdge>,
}

/// An edge connecting a commit to a parent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from_lane: usize,
    pub to_lane: usize,
    pub to_row: usize,
    pub color_index: usize,
    pub edge_type: GraphEdgeType,
}

/// Type of edge connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphEdgeType {
    Straight,
    MergeIn,
    ForkOut,
}

/// Maps a branch name to its assigned color index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchColorEntry {
    pub name: String,
    pub color_index: usize,
}

/// Repository status summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoStatus {
    pub staged: Vec<FileStatus>,
    pub unstaged: Vec<FileStatus>,
    pub untracked: Vec<String>,
    pub conflicted: Vec<String>,
}

/// Status of a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStatus {
    pub path: String,
    pub status: FileStatusType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileStatusType {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    TypeChanged,
}

/// Diff for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    pub path: String,
    pub old_path: Option<String>,
    pub hunks: Vec<DiffHunk>,
    pub is_binary: bool,
    pub status: FileStatusType,
}

/// A diff hunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub header: String,
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub lines: Vec<DiffLine>,
}

/// A single line in a diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub origin: DiffLineType,
    pub content: String,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiffLineType {
    Context,
    Addition,
    Deletion,
    Header,
}

/// Basic repo information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoInfo {
    pub path: String,
    pub workdir: String,
    pub head_branch: Option<String>,
    pub head_oid: Option<String>,
    pub is_bare: bool,
    pub is_empty: bool,
}

/// A configured remote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Remote {
    pub name: String,
    pub url: String,
    pub push_url: Option<String>,
}

/// Tracking status (ahead/behind upstream)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingStatus {
    pub ahead: usize,
    pub behind: usize,
    pub upstream: Option<String>,
}

/// Output captured from a git CLI operation (stdout + stderr)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationOutput {
    pub stdout: String,
    pub stderr: String,
}

/// Result of a commit operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitResult {
    pub oid: String,
    pub success: bool,
    pub output: OperationOutput,
}

/// Result of a fetch operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchResult {
    pub remote: String,
    pub updated_refs: Vec<String>,
    pub summary: String,
    pub output: OperationOutput,
}

/// Result of a push operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushResult {
    pub remote: String,
    pub branch: String,
    pub summary: String,
    pub output: OperationOutput,
}

/// Result of a pull operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullResult {
    pub remote: String,
    pub branch: String,
    pub summary: String,
    pub merge_conflicts: bool,
    pub output: OperationOutput,
}

/// Result of a rebase operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebaseResult {
    pub success: bool,
    pub conflicted: bool,
    pub output: OperationOutput,
}

/// Result of a merge operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    pub success: bool,
    pub conflicted: bool,
    pub output: OperationOutput,
}

/// Result of a clone operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneResult {
    pub path: String,
    pub repo_info: RepoInfo,
    pub output: OperationOutput,
}

/// Payload emitted when file status changes (workdir or staging)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusChangedPayload {
    pub status: RepoStatus,
}

/// Payload emitted when refs change (branches, tags, HEAD)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefsChangedPayload {
    pub graph: CommitGraph,
    pub status: RepoStatus,
}

/// Options for graph queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphOptions {
    pub max_commits: Option<usize>,
    pub from_oid: Option<String>,
    pub include_remotes: bool,
}

impl Default for GraphOptions {
    fn default() -> Self {
        Self {
            max_commits: Some(500),
            from_oid: None,
            include_remotes: true,
        }
    }
}
