use std::collections::HashMap;

use chrono::{TimeZone, Utc};
use git2::Repository;

use super::error::GitResult;
use super::types::*;

/// Build the commit graph for visualization
pub fn build_commit_graph(repo: &Repository, options: &GraphOptions) -> GitResult<CommitGraph> {
    let max_commits = options.max_commits.unwrap_or(500);
    let mut commits = Vec::new();

    // Set up the revwalk
    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)?;

    // Start from all references to get the full graph
    if let Some(ref from_oid) = options.from_oid {
        let oid = git2::Oid::from_str(from_oid)?;
        revwalk.push(oid)?;
    } else {
        revwalk.push_head().ok(); // May fail on empty repo

        if options.include_remotes {
            // Include all branch tips for a complete graph
            if let Ok(references) = repo.references() {
                for reference in references.flatten() {
                    if let Some(oid) = reference.target() {
                        revwalk.push(oid).ok();
                    }
                }
            }
        }
    }

    // Walk commits
    for oid_result in revwalk {
        if commits.len() >= max_commits {
            break;
        }

        let oid = oid_result?;
        let commit = repo.find_commit(oid)?;
        commits.push(commit_to_type(&commit));
    }

    // Collect branches
    let branches = super::repository::list_branches(repo)?;

    // Collect tags
    let tags = collect_tags(repo)?;

    // HEAD info
    let head_oid = repo
        .head()
        .ok()
        .and_then(|r| r.target().map(|o| o.to_string()));

    let head_branch = repo
        .head()
        .ok()
        .and_then(|r| {
            if r.is_branch() {
                r.shorthand().map(|s| s.to_string())
            } else {
                None
            }
        });

    let layout = compute_graph_layout(&commits, &branches, &head_branch);

    Ok(CommitGraph {
        commits,
        branches,
        tags,
        head_oid,
        head_branch,
        layout: Some(layout),
    })
}

/// Get detailed information about a single commit
pub fn get_commit_detail(repo: &Repository, oid_str: &str) -> GitResult<Commit> {
    let oid = git2::Oid::from_str(oid_str)?;
    let commit = repo.find_commit(oid)?;
    Ok(commit_to_type(&commit))
}

/// Convert a git2 commit to our Commit type
fn commit_to_type(commit: &git2::Commit) -> Commit {
    let oid = commit.id().to_string();
    let short_oid = oid[..7.min(oid.len())].to_string();

    let message = commit
        .message()
        .unwrap_or("")
        .to_string();

    let summary = commit
        .summary()
        .unwrap_or("")
        .to_string();

    let author = Signature {
        name: commit.author().name().unwrap_or("").to_string(),
        email: commit.author().email().unwrap_or("").to_string(),
    };

    let committer = Signature {
        name: commit.committer().name().unwrap_or("").to_string(),
        email: commit.committer().email().unwrap_or("").to_string(),
    };

    let parents = commit
        .parent_ids()
        .map(|oid| oid.to_string())
        .collect();

    let time = commit.time();
    let timestamp = Utc
        .timestamp_opt(time.seconds(), 0)
        .single()
        .unwrap_or_else(Utc::now);

    Commit {
        oid,
        short_oid,
        message,
        summary,
        author,
        committer,
        parents,
        timestamp,
    }
}

/// Compute graph layout: branch attribution, color assignment, and lane allocation
fn compute_graph_layout(
    commits: &[Commit],
    branches: &[Branch],
    head_branch: &Option<String>,
) -> GraphLayout {
    // Build oid → index lookup
    let oid_to_index: HashMap<&str, usize> = commits
        .iter()
        .enumerate()
        .map(|(i, c)| (c.oid.as_str(), i))
        .collect();

    // --- Branch attribution (first-parent heuristic) ---
    let mut attribution: Vec<Option<String>> = vec![None; commits.len()];

    // Sort branches by priority: HEAD > main/master/develop > local alpha > remote
    let mut sorted_branches: Vec<&Branch> = branches
        .iter()
        .filter(|b| b.target_oid.is_some())
        .collect();
    sorted_branches.sort_by(|a, b| {
        let tier = |br: &Branch| -> u8 {
            if br.is_head {
                0
            } else if matches!(br.name.as_str(), "main" | "master" | "develop") {
                1
            } else if !br.is_remote {
                2
            } else {
                3
            }
        };
        let ta = tier(a);
        let tb = tier(b);
        ta.cmp(&tb)
            .then_with(|| a.is_remote.cmp(&b.is_remote))
            .then_with(|| a.name.cmp(&b.name))
    });

    for branch in &sorted_branches {
        let tip_oid = match &branch.target_oid {
            Some(oid) => oid,
            None => continue,
        };
        let mut current_oid = tip_oid.clone();
        loop {
            let idx = match oid_to_index.get(current_oid.as_str()) {
                Some(&i) => i,
                None => break,
            };
            if attribution[idx].is_some() {
                break;
            }
            attribution[idx] = Some(branch.name.clone());
            // Walk first parent
            if let Some(first_parent) = commits[idx].parents.first() {
                current_oid = first_parent.clone();
            } else {
                break;
            }
        }
    }

    // Assign orphan names to unattributed commits
    let mut orphan_count = 0;
    for attr in attribution.iter_mut() {
        if attr.is_none() {
            *attr = Some(format!("~orphan-{}", orphan_count));
            orphan_count += 1;
        }
    }

    // --- Color assignment ---
    let mut branch_color_map: HashMap<String, usize> = HashMap::new();
    let mut next_color: usize = 0;

    // HEAD branch gets color 0
    if let Some(ref hb) = head_branch {
        branch_color_map.insert(hb.clone(), 0);
        next_color = 1;
    }

    // Assign colors in branch priority order
    for branch in &sorted_branches {
        if !branch_color_map.contains_key(&branch.name) {
            branch_color_map.insert(branch.name.clone(), next_color);
            next_color += 1;
        }
    }

    // Assign colors to orphan branches
    for attr in &attribution {
        if let Some(ref name) = attr {
            if !branch_color_map.contains_key(name) {
                branch_color_map.insert(name.clone(), next_color);
                next_color += 1;
            }
        }
    }

    // --- Streaming lane allocator ---
    // active_lanes[lane_index] = Some(branch_name) if occupied, None if free
    let mut active_lanes: Vec<Option<String>> = Vec::new();
    // Track which lane each branch is assigned to
    let mut branch_to_lane: HashMap<String, usize> = HashMap::new();
    // Track the last row index where each branch has a commit (to know when to free lanes)
    let mut branch_last_row: HashMap<String, usize> = HashMap::new();

    // Pre-compute last row for each branch
    for (i, attr) in attribution.iter().enumerate() {
        if let Some(ref name) = attr {
            branch_last_row.insert(name.clone(), i);
        }
    }

    // Also track the last row each branch is needed (as a parent target)
    // A branch's lane must stay active until its last commit's parents are all processed
    // We need lanes to stay active for parent edges too
    let mut branch_needed_until: HashMap<String, usize> = branch_last_row.clone();
    for (i, commit) in commits.iter().enumerate() {
        for parent_oid in &commit.parents {
            if let Some(&parent_idx) = oid_to_index.get(parent_oid.as_str()) {
                if let Some(ref parent_branch) = attribution[parent_idx] {
                    let current = branch_needed_until.get(parent_branch).copied().unwrap_or(0);
                    if parent_idx > current {
                        branch_needed_until.insert(parent_branch.clone(), parent_idx);
                    }
                }
                // The current branch needs to stay until this row at minimum for edge drawing
                if let Some(ref curr_branch) = attribution[i] {
                    let current = branch_needed_until.get(curr_branch).copied().unwrap_or(0);
                    if i > current {
                        branch_needed_until.insert(curr_branch.clone(), i);
                    }
                }
            }
        }
    }

    let mut nodes: Vec<GraphNode> = Vec::with_capacity(commits.len());
    let mut max_lanes: usize = 0;

    for (i, commit) in commits.iter().enumerate() {
        let branch_name = attribution[i].clone().unwrap_or_default();
        let color_index = branch_color_map.get(&branch_name).copied().unwrap_or(0);

        // Determine this commit's lane
        let lane = if let Some(&existing_lane) = branch_to_lane.get(&branch_name) {
            existing_lane
        } else {
            // Allocate first free lane
            let free = active_lanes.iter().position(|l| l.is_none());
            let lane = match free {
                Some(l) => {
                    active_lanes[l] = Some(branch_name.clone());
                    l
                }
                None => {
                    active_lanes.push(Some(branch_name.clone()));
                    active_lanes.len() - 1
                }
            };
            branch_to_lane.insert(branch_name.clone(), lane);
            lane
        };

        if lane + 1 > max_lanes {
            max_lanes = lane + 1;
        }

        // Compute edges to parents
        let mut edges = Vec::new();
        for parent_oid in &commit.parents {
            if let Some(&parent_idx) = oid_to_index.get(parent_oid.as_str()) {
                let parent_branch = attribution[parent_idx].clone().unwrap_or_default();
                let parent_color = branch_color_map.get(&parent_branch).copied().unwrap_or(0);

                // Determine parent's lane (allocate if needed)
                let parent_lane = if let Some(&existing) = branch_to_lane.get(&parent_branch) {
                    existing
                } else {
                    // Allocate a lane for the parent's branch
                    let free = active_lanes.iter().position(|l| l.is_none());
                    let pl = match free {
                        Some(l) => {
                            active_lanes[l] = Some(parent_branch.clone());
                            l
                        }
                        None => {
                            active_lanes.push(Some(parent_branch.clone()));
                            active_lanes.len() - 1
                        }
                    };
                    branch_to_lane.insert(parent_branch, pl);
                    pl
                };

                if parent_lane + 1 > max_lanes {
                    max_lanes = parent_lane + 1;
                }

                let edge_type = if lane == parent_lane {
                    GraphEdgeType::Straight
                } else if commit.parents.len() > 1 {
                    GraphEdgeType::MergeIn
                } else {
                    GraphEdgeType::ForkOut
                };

                edges.push(GraphEdge {
                    from_lane: lane,
                    to_lane: parent_lane,
                    to_row: parent_idx,
                    color_index: parent_color,
                    edge_type,
                });
            }
        }

        nodes.push(GraphNode {
            oid: commit.oid.clone(),
            lane,
            color_index,
            branch_name: attribution[i].clone(),
            edges,
        });

        // Free lanes for branches that are done (no more commits or parent references needed)
        // Check all active branches to see if they're fully done at this row
        let mut branches_to_free: Vec<String> = Vec::new();
        for (bname, &needed_until) in &branch_needed_until {
            if i >= needed_until {
                // Check this branch has actually been placed already
                if branch_to_lane.contains_key(bname) {
                    // And all its commits have been processed
                    if let Some(&last_row) = branch_last_row.get(bname) {
                        if i >= last_row {
                            branches_to_free.push(bname.clone());
                        }
                    }
                }
            }
        }
        for bname in branches_to_free {
            if let Some(freed_lane) = branch_to_lane.remove(&bname) {
                if freed_lane < active_lanes.len() {
                    active_lanes[freed_lane] = None;
                }
            }
        }
    }

    // Build branch_colors list
    let mut branch_colors: Vec<BranchColorEntry> = branch_color_map
        .into_iter()
        .filter(|(name, _)| !name.starts_with("~orphan-"))
        .map(|(name, color_index)| BranchColorEntry { name, color_index })
        .collect();
    branch_colors.sort_by_key(|e| e.color_index);

    GraphLayout {
        nodes,
        max_lanes,
        branch_colors,
    }
}

/// Collect all tags in the repository
fn collect_tags(repo: &Repository) -> GitResult<Vec<Tag>> {
    let mut tags = Vec::new();

    repo.tag_foreach(|oid, name| {
        let name = String::from_utf8_lossy(name)
            .trim_start_matches("refs/tags/")
            .to_string();

        // Try to peel to find the target commit
        let (target_oid, is_annotated, message) = if let Ok(obj) = repo.find_object(oid, None) {
            if let Ok(tag) = obj.peel_to_tag() {
                let msg = tag.message().map(|m| m.to_string());
                let target = tag.target_id().to_string();
                (target, true, msg)
            } else {
                (oid.to_string(), false, None)
            }
        } else {
            (oid.to_string(), false, None)
        };

        tags.push(Tag {
            name,
            target_oid,
            is_annotated,
            message,
        });

        true // continue iteration
    })?;

    Ok(tags)
}
