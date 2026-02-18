# Gitron — Agent Gateway Design

This document defines how the Agent Gateway works — the system that makes Gitron an AI-native git interface. It covers the MCP server, permission model, action queue, event stream, agent lifecycle, and visualization. This system is designed in Phase 1 and implemented in Phase 5.

---

## Table of Contents

1. [Overview](#overview)
2. [Design Principles](#design-principles)
3. [Agent Gateway Architecture](#agent-gateway-architecture)
4. [MCP Server](#mcp-server)
5. [Agent Lifecycle](#agent-lifecycle)
6. [Permission System](#permission-system)
7. [Action Queue](#action-queue)
8. [Event Stream](#event-stream)
9. [Agent Visualization](#agent-visualization)
10. [Built-in AI Workflows](#built-in-ai-workflows)
11. [Autonomous Agent Flows](#autonomous-agent-flows)
12. [Configuration](#configuration)

---

## 1. Overview

The Agent Gateway is a subsystem within Gitron that allows AI agents to connect, read repository state, and propose or execute git operations. It provides:

- A **MCP (Model Context Protocol) server** that exposes repo state as structured resources and tools
- A **permission system** that controls what each agent can do
- An **action queue** where agent-proposed actions await human review (or auto-execute based on policy)
- An **event stream** that notifies agents of repo changes in real time
- **Visual integration** in the commit graph showing agent activity

Agents are external processes. They connect to Gitron over MCP. Gitron does not embed or run agents — it provides the interface for agents to interact with git repositories.

---

## 2. Design Principles

1. **Agents are participants, not controllers**: Agents propose actions. Humans approve or reject. The default mode is human-in-the-loop. Autonomous mode is opt-in per agent.

2. **Structured data, not CLI scraping**: Agents receive typed, structured data (JSON) — not raw git CLI output. This makes agent logic more reliable and less brittle.

3. **Observable**: Every agent action is visible in the UI. Users can see what agents are doing, what they've proposed, and what's been executed. No hidden operations.

4. **Scoped access**: Agents operate within defined permission boundaries. A read-only agent cannot stage files. A branch-scoped agent cannot modify branches outside its scope.

5. **Protocol-standard**: MCP is an open protocol. Any MCP-compatible agent can connect to Gitron. No proprietary integration required.

---

## 3. Agent Gateway Architecture

```
┌──────────────────────────────────────────────────┐
│                 Agent Gateway                     │
│                                                   │
│  ┌─────────────┐  ┌──────────────────────────┐   │
│  │             │  │                          │   │
│  │  MCP Server │  │  Agent Registry          │   │
│  │  (stdio/    │  │  - Connected agents      │   │
│  │   SSE)      │  │  - Permissions per agent │   │
│  │             │  │  - Session state         │   │
│  └──────┬──────┘  └────────────┬─────────────┘   │
│         │                      │                  │
│  ┌──────▼──────────────────────▼──────────────┐   │
│  │                                            │   │
│  │           Gateway Core                     │   │
│  │  - Route MCP requests to git operations    │   │
│  │  - Enforce permissions on every call       │   │
│  │  - Log all agent actions                   │   │
│  │                                            │   │
│  └──────┬──────────────────────┬──────────────┘   │
│         │                      │                  │
│  ┌──────▼──────┐  ┌───────────▼───────────────┐   │
│  │             │  │                           │   │
│  │  Action     │  │  Event Stream             │   │
│  │  Queue      │  │  - Push repo events       │   │
│  │  - Pending  │  │  - Agent subscribes to    │   │
│  │  - Approved │  │    specific event types   │   │
│  │  - Rejected │  │                           │   │
│  │  - Executed │  │                           │   │
│  └─────────────┘  └───────────────────────────┘   │
│                                                   │
└───────────────────────┬───────────────────────────┘
                        │
                        ▼
              ┌─────────────────┐
              │  Core Git API   │
              │  (git/ module)  │
              └─────────────────┘
```

### Module Structure

```
src-tauri/src/agents/
├── mod.rs              — module declarations, gateway struct
├── gateway.rs          — main gateway logic, request routing
├── mcp.rs              — MCP server implementation (resources + tools)
├── permissions.rs      — permission types, enforcement logic
├── events.rs           — event stream management
└── queue.rs            — action queue (propose, approve, reject, execute)
```

---

## 4. MCP Server

The MCP server exposes repository state as resources (read) and tools (write).

### Resources (Read Operations)

Resources are data the agent can read. They return JSON.

| Resource URI | Returns | Description |
|-------------|---------|-------------|
| `repo://info` | `RepoInfo` | Basic repo metadata (path, head, branch) |
| `repo://status` | `RepoStatus` | Staged, unstaged, untracked, conflicted files |
| `repo://graph` | `CommitGraph` | Full commit graph (commits, branches, tags) |
| `repo://graph?max=100` | `CommitGraph` | Commit graph limited to N commits |
| `repo://branches` | `Branch[]` | All branches (local + remote) |
| `repo://tags` | `Tag[]` | All tags |
| `repo://commit/{oid}` | `Commit` | Details of a specific commit |
| `repo://diff/workdir` | `FileDiff[]` | All unstaged diffs |
| `repo://diff/staged` | `FileDiff[]` | All staged diffs |
| `repo://diff/file/{path}` | `FileDiff` | Diff for a specific file |
| `repo://file/{path}` | `string` | File content at HEAD |
| `repo://file/{path}?ref={oid}` | `string` | File content at a specific commit |
| `repo://blame/{path}` | `BlameLine[]` | Blame annotations for a file |
| `repo://log/{path}` | `Commit[]` | Commit history for a specific file |
| `repo://stashes` | `Stash[]` | List of stashes |
| `repo://config` | `GitConfig` | Git configuration values |

### Tools (Write Operations)

Tools are actions the agent can execute. All write tools go through the permission system and (depending on configuration) the action queue.

| Tool Name | Parameters | Description |
|-----------|-----------|-------------|
| `stage_files` | `paths: string[]` | Stage one or more files |
| `unstage_files` | `paths: string[]` | Unstage one or more files |
| `stage_all` | — | Stage all changes |
| `create_commit` | `message: string` | Create a commit with staged changes |
| `amend_commit` | `message: string` | Amend the last commit |
| `create_branch` | `name: string, target?: string` | Create a new branch |
| `checkout` | `ref: string` | Check out a branch or commit |
| `delete_branch` | `name: string` | Delete a local branch |
| `merge` | `source: string` | Merge a branch into current |
| `fetch` | `remote?: string` | Fetch from remote |
| `pull` | `remote?: string, branch?: string` | Pull from remote |
| `push` | `remote?: string, branch?: string` | Push to remote |
| `stash_create` | `message?: string` | Create a stash |
| `stash_apply` | `index?: number` | Apply a stash |
| `propose_action` | `action: ActionProposal` | Submit an action for human review |

### Tool Execution Flow

```
Agent calls tool (e.g., create_commit)
  │
  ├── Permission check
  │   ├── Denied → Return error: "Permission denied: {reason}"
  │   └── Allowed → Continue
  │
  ├── Approval mode check
  │   ├── HumanInTheLoop → Add to action queue, return "Pending approval"
  │   ├── AutoApprove (if action type matches) → Execute immediately
  │   └── FullyAutonomous → Execute immediately
  │
  ├── Execute git operation via core API
  │   ├── Success → Return result, log action as Executed
  │   └── Failure → Return error, log action as Failed
  │
  └── Emit event: agent:action-executed or agent:action-failed
```

---

## 5. Agent Lifecycle

### Connection

```
1. Agent process starts (external to Gitron)
2. Agent connects to Gitron's MCP server (via stdio or SSE transport)
3. MCP handshake: agent identifies itself (name, version, capabilities)
4. Gateway looks up agent in registry:
   a. Known agent → apply configured permissions
   b. Unknown agent → apply default permissions (ReadOnly + HumanInTheLoop)
5. Agent is now connected and can read resources / call tools
```

### Session

```
- Agent can read resources at any time
- Agent can call tools (subject to permissions)
- Agent receives events via the event stream (if subscribed)
- Agent can propose actions for human review
- Agent session persists until disconnection
```

### Disconnection

```
1. Agent process exits or connection drops
2. Gateway detects disconnection
3. Agent's pending actions remain in the queue (not automatically cancelled)
4. Agent's event subscriptions are cleaned up
5. UI updates to show agent as disconnected
```

---

## 6. Permission System

### Permission Levels

```rust
pub enum AgentPermission {
    /// Can only read repo state. Cannot modify anything.
    ReadOnly,

    /// Can read everything. Can write only to specified branches.
    /// Prevents accidental modifications to main/production branches.
    BranchScoped(Vec<String>),

    /// Can read and write anything. Full git access.
    FullAccess,
}
```

### Approval Modes

```rust
pub enum ApprovalMode {
    /// Every write action requires explicit human approval in the UI.
    /// The action is added to the queue and the user must click Approve.
    HumanInTheLoop,

    /// Specific action types are auto-approved. Others require human approval.
    /// Example: auto-approve stage/unstage, but require approval for commit/push.
    AutoApprove(Vec<ActionType>),

    /// All actions execute immediately without approval.
    /// Use with caution. Every action is still logged.
    FullyAutonomous,
}
```

### Action Types for Auto-Approve

```rust
pub enum ActionType {
    StageFile,
    UnstageFile,
    CreateCommit,
    AmendCommit,
    CreateBranch,
    DeleteBranch,
    Checkout,
    Merge,
    Fetch,
    Pull,
    Push,
    Stash,
}
```

### Permission Enforcement

Permissions are checked at the Gateway Core level, BEFORE the action reaches the git layer:

```
Agent calls create_commit("fix: typo")
  → Gateway checks agent's permission
    → ReadOnly: DENIED ("Agent does not have write permission")
    → BranchScoped(["feature/*"]):
        → Current branch is "feature/login": ALLOWED
        → Current branch is "main": DENIED ("Agent cannot write to branch: main")
    → FullAccess: ALLOWED
```

### Default Permissions

When an unknown agent connects, it receives:
- Permission: `ReadOnly`
- Approval: `HumanInTheLoop`

The user must explicitly upgrade an agent's permissions in the settings.

---

## 7. Action Queue

The action queue is the human review interface for agent-proposed actions.

### Action Structure

```rust
pub struct AgentAction {
    /// Unique action ID
    pub id: Uuid,

    /// Which agent proposed this action
    pub agent_id: String,

    /// What type of action (commit, branch, push, etc.)
    pub action_type: ActionType,

    /// Human-readable description of what the action does
    pub description: String,

    /// The action parameters (serialized)
    pub payload: serde_json::Value,

    /// Current status
    pub status: ActionStatus,

    /// When the action was proposed
    pub created_at: DateTime<Utc>,

    /// When the action was resolved (approved/rejected/executed)
    pub resolved_at: Option<DateTime<Utc>>,

    /// Who resolved it (user or auto-approve policy)
    pub resolved_by: Option<String>,
}
```

### Action Statuses

```rust
pub enum ActionStatus {
    /// Waiting for human review
    Pending,

    /// Human approved, executing
    Approved,

    /// Human rejected
    Rejected,

    /// Successfully executed
    Executed,

    /// Execution failed
    Failed(String),

    /// Cancelled (agent disconnected or timeout)
    Cancelled,
}
```

### Queue Flow

```
Agent proposes action
  → Action created with Pending status
  → Tauri event: agent:action-proposed
  → Frontend AgentPanel shows the pending action
  → User reviews:
      ├── Approve → status = Approved → execute → status = Executed or Failed
      ├── Reject → status = Rejected → agent notified
      └── Modify → user edits payload → Approve modified version

Auto-approved actions skip the queue:
  → Permission checked
  → Action created with Approved status
  → Execute immediately
  → Status = Executed or Failed
  → Action still logged for audit
```

### Queue UI

The AgentPanel (sidebar tab or bottom panel) shows:

```
┌─────────────────────────────────────────┐
│ Agent Actions                           │
├─────────────────────────────────────────┤
│ ⏳ claude-code wants to:               │
│    Create commit: "fix: resolve merge   │
│    conflict in auth module"             │
│    [Approve] [Reject] [View Diff]       │
├─────────────────────────────────────────┤
│ ✅ claude-code committed:              │
│    "feat: add login validation"         │
│    2 minutes ago                        │
├─────────────────────────────────────────┤
│ ❌ Rejected:                           │
│    "refactor: rewrite auth module"      │
│    5 minutes ago                        │
└─────────────────────────────────────────┘
```

---

## 8. Event Stream

Agents can subscribe to real-time events to react to repository changes.

### Available Events

| Event | Payload | When |
|-------|---------|------|
| `repo:opened` | `RepoInfo` | Repository is opened in Gitron |
| `repo:closed` | `{}` | Repository is closed |
| `repo:status-changed` | `RepoStatus` | Working directory or index changed |
| `repo:head-changed` | `{ oid, branch }` | HEAD moved (commit, checkout, reset) |
| `repo:refs-changed` | `{ branches, tags }` | Branches or tags changed |
| `repo:commit-created` | `Commit` | New commit was created |
| `repo:branch-created` | `Branch` | New branch was created |
| `repo:branch-deleted` | `{ name }` | Branch was deleted |
| `repo:merge-completed` | `{ source, target, oid }` | Merge completed |
| `repo:conflict-detected` | `{ paths }` | Merge conflict detected |
| `repo:push-completed` | `{ remote, branch }` | Push completed |
| `repo:pull-completed` | `{ remote, branch }` | Pull completed |

### Subscription Model

Agents subscribe to events during their session:

```
Agent → MCP: subscribe to ["repo:status-changed", "repo:commit-created"]
Gateway → registers subscriptions
File change detected → Gateway pushes event to subscribed agents
Agent receives event → can read resources or call tools in response
```

### Event Filtering

Agents can filter events:
- By type: subscribe only to specific event types
- By path: receive status changes only for specific file paths (future)
- By branch: receive ref changes only for specific branches (future)

---

## 9. Agent Visualization

Agents are visible in the Gitron UI. Users can always see what agents are doing.

### Commit Graph Integration

- Agent-authored commits are visually distinguished in the commit graph
- Different icon, color, or badge on the graph node
- Tooltip shows which agent created the commit
- Filter option: show/hide agent commits

### How Agent Commits Are Detected

```
Method 1: Commit message trailer
  Agent adds a trailer: "Agent: claude-code/1.0"
  Gitron parser detects this and marks the commit

Method 2: Git notes
  After creating a commit, the agent adds a git note:
  git notes add -m "agent:claude-code" <oid>
  Gitron reads notes and marks the commit

Method 3: Action log
  The action queue records which commits were created by agents
  Cross-reference OIDs to mark agent commits in the graph
```

### Agent Activity Panel

A dedicated panel showing:
- Connected agents (name, version, permission level)
- Recent agent actions (with status: pending, executed, rejected)
- Agent event subscriptions
- Agent statistics (commits created, actions taken)

### Agent Timeline (Future)

A parallel timeline alongside the commit graph showing agent actions:
```
Time    Git Graph              Agent Timeline
─────   ─────────              ──────────────
12:00   ● fix: typo
12:01                          🤖 claude-code: analyzing diff
12:02                          🤖 claude-code: proposed commit
12:03   ● feat: add login      🤖 claude-code: commit approved
12:05                          🤖 claude-code: running tests
```

---

## 10. Built-in AI Workflows

These are AI-powered features built into Gitron, powered by the agent gateway.

### Smart Commit Messages

```
Trigger: User clicks "Generate Message" in commit panel
Flow:
  1. Frontend reads staged diff (already available in status)
  2. Sends diff to a configured AI provider (Claude API, OpenAI, local model)
  3. AI generates commit message following repo conventions
  4. Message appears in commit input, user can edit before committing
```

### PR Description Generation

```
Trigger: User clicks "Generate PR Description"
Flow:
  1. Diff current branch against base branch (e.g., main)
  2. Collect all commit messages in the branch
  3. Send to AI: "Summarize these changes for a PR description"
  4. Formatted PR description shown to user
  5. User can copy to clipboard or push to GitHub/GitLab (via plugin)
```

### Conflict Resolution Suggestions

```
Trigger: Merge conflict detected
Flow:
  1. Read both sides of the conflict
  2. Read surrounding code context
  3. Send to AI: "Suggest a resolution for this merge conflict"
  4. AI produces a proposed merge
  5. Shown inline in the merge conflict editor with accept/reject
```

### Code Review in Diff Viewer

```
Trigger: User clicks "Review" on a diff
Flow:
  1. Send diff to AI: "Review this code change"
  2. AI returns comments attached to specific lines
  3. Comments shown inline in the diff viewer
  4. User can dismiss, reply, or create issues from comments
```

### Explain History

```
Trigger: User selects a range of commits, clicks "Explain"
Flow:
  1. Collect commit messages and diffs for the range
  2. Send to AI: "Explain what happened in this series of changes"
  3. AI produces a narrative summary
  4. Shown in a modal or panel
```

---

## 11. Autonomous Agent Flows

These are fully autonomous agent behaviors, running continuously.

### Auto-Branching Agent

```
Purpose: Automatically create feature branches based on issue assignments
Trigger: Webhook or poll from issue tracker
Flow:
  1. Agent detects new issue assigned to the repo
  2. Agent creates branch: feature/{issue-number}-{slug}
  3. Agent checks out the branch
  4. Agent notifies via event stream
```

### CI-Aware Agent

```
Purpose: React to CI/CD results
Trigger: CI webhook or poll
Flow:
  1. Agent detects CI failure on a branch
  2. Agent reads the failing test output
  3. Agent analyzes the code change that caused the failure
  4. Agent proposes a fix commit (through action queue)
  5. User reviews and approves/rejects
```

### Release Management Agent

```
Purpose: Automate release workflows
Trigger: User command or schedule
Flow:
  1. Agent reads commit history since last release
  2. Agent generates changelog from conventional commits
  3. Agent proposes version bump (semver)
  4. Agent creates release branch, updates version files
  5. Agent creates tag
  6. All via action queue for human approval
```

### Multi-Repo Orchestration Agent

```
Purpose: Coordinate changes across multiple repositories
Trigger: Agent-initiated
Flow:
  1. Agent connects to multiple Gitron instances (one per repo)
  2. Agent reads state across all repos
  3. Agent proposes coordinated changes (e.g., update dependency version in all repos)
  4. Actions queued in each repo's Gitron instance
  5. User approves per-repo or approves the batch
```

---

## 12. Configuration

### Agent Configuration File

Stored in the Gitron config directory:

```toml
# ~/.config/gitron/agents.toml

[defaults]
permission = "ReadOnly"
approval = "HumanInTheLoop"

[agents.claude-code]
permission = "BranchScoped"
allowed_branches = ["feature/*", "fix/*"]
approval = "AutoApprove"
auto_approve_actions = ["StageFile", "UnstageFile", "CreateCommit"]

[agents.release-bot]
permission = "FullAccess"
approval = "HumanInTheLoop"

[agents.ci-watcher]
permission = "ReadOnly"
approval = "HumanInTheLoop"
```

### Per-Repo Overrides

Stored in `.gitron/agents.toml` within the repository:

```toml
# .gitron/agents.toml (repo-level overrides)

[agents.claude-code]
permission = "FullAccess"           # Override: full access in this repo
approval = "FullyAutonomous"        # Override: no approval needed
```

Repo-level config takes precedence over global config.

### AI Provider Configuration

```toml
# ~/.config/gitron/ai.toml

[provider]
type = "anthropic"                  # or "openai", "local", "custom"
api_key_env = "ANTHROPIC_API_KEY"   # environment variable name
model = "claude-sonnet-4-5-20250514"

[provider.local]
endpoint = "http://localhost:11434" # Ollama or similar
model = "llama3"
```
