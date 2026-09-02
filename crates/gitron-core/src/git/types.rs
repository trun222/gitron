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

/// A branch that has been fully merged into the current HEAD
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergedBranch {
    pub name: String,
    pub is_remote: bool,
    /// For remote branches: the remote name (e.g. "origin")
    pub remote: Option<String>,
    /// The short branch name without remote prefix (e.g. "feature/foo")
    pub short_name: String,
}

/// A tag reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub target_oid: String,
    pub is_annotated: bool,
    pub message: Option<String>,
}

/// A remote tag reference (name + OID from ls-remote)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteTagInfo {
    pub name: String,
    pub oid: String,
}

/// A checkpoint ref created by AI coding tools (Claude Code, T3 Code, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointRef {
    pub refname: String,
    pub short_oid: String,
    /// The tool/prefix group (e.g. "claude", "t3")
    pub source: String,
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

/// The current state of the repository (e.g., mid-rebase, mid-merge)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RepoState {
    Clean,
    Merging,
    Rebasing,
    RebasingInteractive,
    CherryPicking,
    Reverting,
}

/// Repository status summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoStatus {
    pub staged: Vec<FileStatus>,
    pub unstaged: Vec<FileStatus>,
    pub untracked: Vec<String>,
    pub conflicted: Vec<String>,
    pub state: RepoState,
    /// Current step in a rebase/cherry-pick sequence (1-based), if any
    pub operation_step: Option<u32>,
    /// Total steps in a rebase/cherry-pick sequence, if any
    pub operation_total: Option<u32>,
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

/// Result of creating and checking out a branch in one step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBranchResult {
    pub info: RepoInfo,
    /// Uncommitted changes were stashed before switching branches
    pub auto_stashed: bool,
    /// The auto-stash was successfully re-applied on the new branch
    /// (false means it was left in the stash list for manual recovery)
    pub stash_restored: bool,
}

/// Consolidated result from opening a repository (all data in one round-trip)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRepoResult {
    pub info: RepoInfo,
    pub status: RepoStatus,
    pub graph: CommitGraph,
    pub remotes: Vec<Remote>,
    pub tracking: Option<TrackingStatus>,
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

/// A git worktree entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeInfo {
    /// Name of the worktree (directory basename for linked, "main" for the primary)
    pub name: String,
    /// Absolute path to the worktree's working directory
    pub path: String,
    /// Branch currently checked out (None if detached HEAD)
    pub branch: Option<String>,
    /// HEAD commit OID
    pub head_oid: Option<String>,
    /// Short HEAD commit OID
    pub head_short_oid: Option<String>,
    /// Whether this is the main (non-linked) worktree
    pub is_main: bool,
    /// Whether the worktree directory is locked (prevents pruning)
    pub is_locked: bool,
    /// Lock reason, if locked
    pub lock_reason: Option<String>,
    /// Whether the worktree path is valid (directory exists)
    pub is_valid: bool,
}

/// Result of creating a worktree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeCreateResult {
    pub worktree: WorktreeInfo,
    pub output: OperationOutput,
}

/// Result of removing a worktree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeRemoveResult {
    pub success: bool,
    pub output: OperationOutput,
}

/// Result of pruning worktrees
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreePruneResult {
    pub pruned: Vec<String>,
    pub output: OperationOutput,
}

/// A conflict section within a file (between <<<<<<< and >>>>>>>)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictSection {
    pub ours: Vec<String>,
    pub theirs: Vec<String>,
    pub ours_label: String,
    pub theirs_label: String,
    pub start_line: u32,
    pub end_line: u32,
}

/// Content of a conflicted file with parsed conflict sections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictedFileContent {
    pub path: String,
    pub raw_content: String,
    pub lines: Vec<String>,
    pub conflict_sections: Vec<ConflictSection>,
    pub is_binary: bool,
}

/// Options for graph queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphOptions {
    pub max_commits: Option<usize>,
    pub from_oid: Option<String>,
    pub include_remotes: bool,
    #[serde(default)]
    pub excluded_authors: Vec<String>,
}

impl Default for GraphOptions {
    fn default() -> Self {
        Self {
            max_commits: Some(500),
            from_oid: None,
            include_remotes: true,
            excluded_authors: Vec::new(),
        }
    }
}
