<script lang="ts">
  import {
    commitGraph, selectedCommit, selectCommit,
    checkoutBranch, deleteBranchConfirm, createBranchAtCommit,
    resetToCommit, currentBranch,
    applyStash, popStash, dropStash,
    rebaseOnto, mergeInto,
  } from '$lib/stores/repo';
  import { graphColumnWidths, saveGraphColumnWidths, theme } from '$lib/stores/settings';
  import { gravatarUrl } from '$lib/utils/gravatar';
  import type { Commit, Branch, StashEntry, GraphColumnWidths, GraphEdge } from '$lib/api/types';

  const GRAPH_COLOR_COUNT = 14;
  const ROW_HEIGHT = 30;
  const LANE_WIDTH = 24;
  const LANE_PADDING = 10;
  const CIRCLE_RADIUS = 5;
  const LINE_WIDTH = 2;

  const MIN_WIDTHS: Record<keyof GraphColumnWidths, number> = {
    graph: 30,
    author: 60,
    date: 60,
    sha: 50,
  };

  let listEl: HTMLDivElement | undefined = $state();

  // Local column widths driven by the store
  let colWidths: GraphColumnWidths = $state({ graph: 40, author: 140, date: 80, sha: 70 });

  $effect(() => {
    const storeVal = $graphColumnWidths;
    colWidths = { ...storeVal };
  });

  // Cache graph colors from CSS custom properties (re-reads on theme change)
  let graphColors: string[] = $state([]);
  $effect(() => {
    // Reference $theme so this effect re-runs when the theme changes
    void $theme;
    // Defer read to next frame so CSS variables have been applied
    requestAnimationFrame(() => {
      const style = getComputedStyle(document.documentElement);
      const colors: string[] = [];
      for (let i = 0; i < GRAPH_COLOR_COUNT; i++) {
        colors.push(style.getPropertyValue(`--color-graph-${i}`).trim() || '#888');
      }
      graphColors = colors;
    });
  });

  let isTronEnhanced = $derived($theme === 'tron-enhanced');

  function getGraphColor(colorIndex: number): string {
    if (graphColors.length === 0) return '#888';
    return graphColors[colorIndex % graphColors.length];
  }

  // Dynamic graph column width based on layout lane count
  let graphColumnWidth = $derived.by(() => {
    const layout = $commitGraph?.layout;
    if (!layout) return colWidths.graph;
    return Math.max(colWidths.graph, layout.max_lanes * LANE_WIDTH + LANE_PADDING * 2);
  });

  function getGridTemplate(): string {
    return `${graphColumnWidth}px 1fr ${colWidths.author}px ${colWidths.date}px ${colWidths.sha}px`;
  }

  // --- Column resize ---
  type DragMode =
    | { kind: 'single'; column: keyof GraphColumnWidths; startWidth: number; inverse: boolean }
    | { kind: 'pair'; left: keyof GraphColumnWidths; right: keyof GraphColumnWidths; startLeft: number; startRight: number };
  let dragMode: DragMode | null = $state(null);
  let dragStartX = 0;

  function startResize(column: keyof GraphColumnWidths, inverse = false) {
    return (e: MouseEvent) => {
      e.preventDefault();
      dragMode = { kind: 'single', column, startWidth: colWidths[column], inverse };
      dragStartX = e.clientX;
      document.addEventListener('mousemove', onMouseMove);
      document.addEventListener('mouseup', onMouseUp);
      document.body.style.cursor = 'col-resize';
      document.body.style.userSelect = 'none';
    };
  }

  function startResizePair(left: keyof GraphColumnWidths, right: keyof GraphColumnWidths) {
    return (e: MouseEvent) => {
      e.preventDefault();
      dragMode = { kind: 'pair', left, right, startLeft: colWidths[left], startRight: colWidths[right] };
      dragStartX = e.clientX;
      document.addEventListener('mousemove', onMouseMove);
      document.addEventListener('mouseup', onMouseUp);
      document.body.style.cursor = 'col-resize';
      document.body.style.userSelect = 'none';
    };
  }

  function onMouseMove(e: MouseEvent) {
    if (!dragMode) return;
    const delta = e.clientX - dragStartX;
    if (dragMode.kind === 'single') {
      const adjusted = dragMode.inverse ? -delta : delta;
      colWidths[dragMode.column] = Math.max(MIN_WIDTHS[dragMode.column], dragMode.startWidth + adjusted);
    } else {
      colWidths[dragMode.left] = Math.max(MIN_WIDTHS[dragMode.left], dragMode.startLeft + delta);
      colWidths[dragMode.right] = Math.max(MIN_WIDTHS[dragMode.right], dragMode.startRight - delta);
    }
  }

  function onMouseUp() {
    if (!dragMode) return;
    document.removeEventListener('mousemove', onMouseMove);
    document.removeEventListener('mouseup', onMouseUp);
    document.body.style.cursor = '';
    document.body.style.userSelect = '';
    dragMode = null;
    saveGraphColumnWidths({ ...colWidths });
  }

  // --- Lane activity precomputation ---
  interface LaneActivity {
    hasTop: boolean;
    hasBottom: boolean;
    colorIndex: number;
  }

  let laneActivities = $derived.by(() => {
    const layout = $commitGraph?.layout;
    if (!layout) return [] as Map<number, LaneActivity>[];

    const result: Map<number, LaneActivity>[] = [];
    for (let i = 0; i < layout.nodes.length; i++) {
      result.push(new Map());
    }

    function ensure(row: number, lane: number, colorIndex: number): LaneActivity {
      if (row < 0 || row >= result.length) return { hasTop: false, hasBottom: false, colorIndex };
      const rowMap = result[row];
      if (!rowMap.has(lane)) rowMap.set(lane, { hasTop: false, hasBottom: false, colorIndex });
      return rowMap.get(lane)!;
    }

    for (let row = 0; row < layout.nodes.length; row++) {
      const node = layout.nodes[row];
      for (const edge of node.edges) {
        if (edge.from_lane === edge.to_lane) {
          const lane = edge.from_lane;
          ensure(row, lane, edge.color_index).hasBottom = true;
          for (let r = row + 1; r < edge.to_row; r++) {
            const a = ensure(r, lane, edge.color_index);
            a.hasTop = true;
            a.hasBottom = true;
          }
          if (edge.to_row > row) {
            ensure(edge.to_row, lane, edge.color_index).hasTop = true;
          }
        } else {
          const lane = edge.to_lane;
          const startRow = row + 1;
          const endRow = edge.to_row;
          if (startRow <= endRow) {
            const a = ensure(startRow, lane, edge.color_index);
            a.hasTop = true;
            if (startRow < endRow) a.hasBottom = true;
            for (let r = startRow + 1; r < endRow; r++) {
              const b = ensure(r, lane, edge.color_index);
              b.hasTop = true;
              b.hasBottom = true;
            }
            if (endRow > startRow) {
              ensure(endRow, lane, edge.color_index).hasTop = true;
            }
          }
        }
      }
    }

    return result;
  });

  function laneX(lane: number): number {
    return lane * LANE_WIDTH + LANE_PADDING + LANE_WIDTH / 2;
  }

  function getEdgePath(edge: GraphEdge): string {
    const x1 = laneX(edge.from_lane);
    const y1 = ROW_HEIGHT / 2;
    const x2 = laneX(edge.to_lane);
    const y2 = ROW_HEIGHT;
    // Smooth S-curve: control points stay at their lane's X but shift vertically
    const cy1 = y1 + (y2 - y1) * 0.6;
    const cy2 = y2 - (y2 - y1) * 0.6;
    return `M ${x1} ${y1} C ${x1} ${cy1}, ${x2} ${cy2}, ${x2} ${y2}`;
  }

  // Branch color lookup from layout
  let branchColorMap = $derived.by(() => {
    const layout = $commitGraph?.layout;
    if (!layout) return new Map<string, number>();
    return new Map(layout.branch_colors.map((e) => [e.name, e.color_index]));
  });

  // Stash OID lookup from graph
  let stashMap = $derived.by(() => {
    const stashes = $commitGraph?.stashes;
    if (!stashes) return new Map<string, StashEntry>();
    return new Map(stashes.map((s) => [s.oid, s]));
  });

  function formatDate(timestamp: string): string {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return 'just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 30) return `${diffDays}d ago`;
    return date.toLocaleDateString();
  }

  function getBranchesForCommit(oid: string): Branch[] {
    if (!$commitGraph) return [];
    return $commitGraph.branches.filter((b) => b.target_oid === oid);
  }

  function isSelected(commit: Commit): boolean {
    return $selectedCommit?.oid === commit.oid;
  }

  function selectedIndex(): number {
    if (!$commitGraph || !$selectedCommit) return -1;
    return $commitGraph.commits.findIndex((c) => c.oid === $selectedCommit!.oid);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!$commitGraph || $commitGraph.commits.length === 0) return;
    const commits = $commitGraph.commits;
    const idx = selectedIndex();

    if (e.key === 'ArrowDown') {
      e.preventDefault();
      const next = idx < commits.length - 1 ? idx + 1 : idx;
      selectCommit(commits[next]);
      scrollToIndex(next);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      const prev = idx > 0 ? idx - 1 : 0;
      selectCommit(commits[prev]);
      scrollToIndex(prev);
    }
  }

  function scrollToIndex(index: number) {
    if (!listEl) return;
    const row = listEl.children[index] as HTMLElement | undefined;
    row?.scrollIntoView({ block: 'nearest' });
  }

  function getBranchLabelStyle(branch: Branch): string {
    const colorIdx = branchColorMap.get(branch.name);
    if (colorIdx === undefined) return '';
    const color = getGraphColor(colorIdx);
    if (branch.is_head) {
      return `border-color: ${color}; background: ${color}; color: var(--primary-foreground)`;
    }
    if (branch.is_remote) {
      return `border-color: ${color}80; color: ${color}; border-style: dashed; opacity: 0.7`;
    }
    return `border-color: ${color}; color: ${color}; background: ${color}1a`;
  }

  // --- Unified branch grouping (GitKraken-style) ---
  interface UnifiedBranch {
    /** Display name (short, no remote prefix) */
    name: string;
    local: Branch | null;
    remote: Branch | null;
    /** The branch used for color lookup and primary interactions */
    primary: Branch;
  }

  function groupBranches(branches: Branch[]): UnifiedBranch[] {
    const locals = branches.filter((b) => !b.is_remote);
    const remotes = branches.filter((b) => b.is_remote);
    const pairedRemotes = new Set<string>();
    const groups: UnifiedBranch[] = [];

    for (const local of locals) {
      const tracking = local.upstream
        ? remotes.find((r) => r.name === local.upstream)
        : null;
      if (tracking) pairedRemotes.add(tracking.name);
      groups.push({
        name: local.name,
        local,
        remote: tracking ?? null,
        primary: local,
      });
    }

    // Standalone remotes (no local counterpart)
    for (const remote of remotes) {
      if (pairedRemotes.has(remote.name)) continue;
      // Strip remote prefix for display (e.g. "origin/dev" → "dev")
      const shortName = remote.name.replace(/^[^/]+\//, '');
      groups.push({
        name: shortName,
        local: null,
        remote,
        primary: remote,
      });
    }

    return groups;
  }

  function getUnifiedBranchStyle(group: UnifiedBranch): string {
    const colorIdx = branchColorMap.get(group.primary.name);
    if (colorIdx === undefined) return '';
    const color = getGraphColor(colorIdx);
    if (group.primary.is_head) {
      return `border-color: ${color}; background: ${color}; color: var(--primary-foreground)`;
    }
    if (!group.local) {
      // Remote-only
      return `border-color: ${color}80; color: ${color}; border-style: dashed; opacity: 0.7`;
    }
    return `border-color: ${color}; color: ${color}; background: ${color}1a`;
  }

  // --- Context menu (uses action IDs, not closures, to avoid $state proxy issues) ---
  interface MenuAction {
    id: string;
    label: string;
    disabled?: boolean;
    danger?: boolean;
    submenu?: MenuAction[];
  }

  interface ContextMenuState {
    x: number;
    y: number;
    items: (MenuAction | 'separator')[];
    // Target data for action dispatch
    commitOid?: string;
    shortOid?: string;
    commitMessage?: string;
    branchName?: string;
    branchIsRemote?: boolean;
    stashIndex?: number;
  }

  let contextMenu: ContextMenuState | null = $state(null);
  let hoveredSubmenu: string | null = $state(null);

  function closeContextMenu() {
    contextMenu = null;
    hoveredSubmenu = null;
  }

  function handleCommitContextMenu(e: MouseEvent, commit: Commit) {
    e.preventDefault();

    // Stash-specific context menu
    const stash = stashMap.get(commit.oid);
    if (stash) {
      contextMenu = {
        x: e.clientX,
        y: e.clientY,
        commitOid: commit.oid,
        shortOid: commit.short_oid,
        commitMessage: commit.message,
        stashIndex: stash.index,
        items: [
          { id: 'apply-stash', label: 'Apply Stash' },
          { id: 'pop-stash', label: 'Pop Stash' },
          { id: 'drop-stash', label: 'Drop Stash', danger: true },
          'separator',
          { id: 'copy-sha', label: 'Copy commit SHA' },
          { id: 'copy-message', label: 'Copy stash message' },
        ],
      };
      return;
    }

    const branch = $currentBranch;
    const items: (MenuAction | 'separator')[] = [
      { id: 'create-branch', label: 'Create branch here' },
    ];
    if (branch) {
      items.push({
        id: 'reset-submenu',
        label: `Reset ${branch} to this commit`,
        submenu: [
          { id: 'reset-soft', label: 'Soft \u2013 keep all changes staged' },
          { id: 'reset-mixed', label: 'Mixed \u2013 keep changes unstaged' },
          { id: 'reset-hard', label: 'Hard \u2013 discard all changes', danger: true },
        ],
      });
    }
    items.push(
      'separator',
      { id: 'copy-sha', label: 'Copy commit SHA' },
      { id: 'copy-message', label: 'Copy commit message' },
    );
    contextMenu = {
      x: e.clientX,
      y: e.clientY,
      commitOid: commit.oid,
      shortOid: commit.short_oid,
      commitMessage: commit.message,
      items,
    };
  }

  function handleBranchContextMenu(e: MouseEvent, branch: Branch) {
    e.preventDefault();
    e.stopPropagation();
    const current = $currentBranch;
    const items: (MenuAction | 'separator')[] = [
      { id: 'checkout', label: 'Checkout branch', disabled: branch.is_head },
    ];
    if (current && !branch.is_head && !branch.is_remote) {
      items.push(
        { id: 'rebase-onto', label: `Rebase ${current} onto ${branch.name}` },
        { id: 'merge-into', label: `Merge ${current} into ${branch.name}` },
      );
    }
    items.push(
      { id: 'delete-branch', label: 'Delete branch', disabled: branch.is_head, danger: true },
    );
    if (current && branch.target_oid) {
      items.push({
        id: 'reset-submenu',
        label: `Reset ${current} to this commit`,
        submenu: [
          { id: 'reset-soft', label: 'Soft \u2013 keep all changes staged' },
          { id: 'reset-mixed', label: 'Mixed \u2013 keep changes unstaged' },
          { id: 'reset-hard', label: 'Hard \u2013 discard all changes', danger: true },
        ],
      });
    }
    items.push(
      'separator',
      { id: 'copy-name', label: 'Copy branch name' },
    );
    contextMenu = {
      x: e.clientX,
      y: e.clientY,
      branchName: branch.name,
      branchIsRemote: branch.is_remote,
      commitOid: branch.target_oid ?? undefined,
      items,
    };
  }

  function executeMenuAction(actionId: string) {
    if (!contextMenu) return;
    // Snapshot values before closing
    const { x, y, commitOid, shortOid, commitMessage, branchName, branchIsRemote, stashIndex } = contextMenu;
    closeContextMenu();

    switch (actionId) {
      case 'create-branch':
        if (commitOid && shortOid) {
          branchPrompt = { x, y, commitOid, shortOid };
        }
        break;
      case 'copy-sha':
        if (commitOid) navigator.clipboard.writeText(commitOid);
        break;
      case 'copy-message':
        if (commitMessage) navigator.clipboard.writeText(commitMessage);
        break;
      case 'checkout':
        if (branchName) checkoutBranch(branchName);
        break;
      case 'delete-branch':
        if (branchName) {
          deleteBranchConfirm.set({
            open: true,
            branchName,
            isRemote: branchIsRemote ?? false,
          });
        }
        break;
      case 'rebase-onto':
        if (branchName) rebaseOnto(branchName);
        break;
      case 'merge-into':
        if (branchName) {
          const current = $currentBranch;
          if (current) mergeInto(current, branchName);
        }
        break;
      case 'copy-name':
        if (branchName) navigator.clipboard.writeText(branchName);
        break;
      case 'reset-soft':
        if (commitOid) resetToCommit(commitOid, 'soft');
        break;
      case 'reset-mixed':
        if (commitOid) resetToCommit(commitOid, 'mixed');
        break;
      case 'reset-hard':
        if (commitOid) resetToCommit(commitOid, 'hard');
        break;
      case 'apply-stash':
        if (stashIndex !== undefined) applyStash(stashIndex);
        break;
      case 'pop-stash':
        if (stashIndex !== undefined) popStash(stashIndex);
        break;
      case 'drop-stash':
        if (stashIndex !== undefined) dropStash(stashIndex);
        break;
    }
  }

  function handleBranchClick(e: MouseEvent, branch: Branch) {
    e.stopPropagation();
    if (branch.is_head) return;
    checkoutBranch(branch.name);
  }

  // --- Branch creation prompt ---
  let branchPrompt = $state<{
    x: number;
    y: number;
    commitOid: string;
    shortOid: string;
  } | null>(null);

  let newBranchName = $state('');

  function closeBranchPrompt() {
    branchPrompt = null;
    newBranchName = '';
  }

  function handleBranchPromptKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      closeBranchPrompt();
    } else if (e.key === 'Enter') {
      e.preventDefault();
      const name = newBranchName.trim();
      if (name && branchPrompt) {
        const oid = branchPrompt.commitOid;
        closeBranchPrompt();
        createBranchAtCommit(name, oid);
      }
    }
  }

  function autoFocusAction(node: HTMLInputElement) {
    requestAnimationFrame(() => node.focus());
  }

  // Close overlays on scroll
  function handleListScroll() {
    closeContextMenu();
    closeBranchPrompt();
  }

  // Close overlays on Escape
  $effect(() => {
    if (contextMenu || branchPrompt) {
      const handler = (e: KeyboardEvent) => {
        if (e.key === 'Escape') {
          closeContextMenu();
          closeBranchPrompt();
        }
      };
      document.addEventListener('keydown', handler);
      return () => document.removeEventListener('keydown', handler);
    }
  });
</script>

<div class="flex flex-col flex-1 overflow-hidden text-[13px]" style="--grid-cols: {getGridTemplate()}">
  {#if $commitGraph && $commitGraph.commits.length > 0}
    {@const layout = $commitGraph.layout}
    {#if isTronEnhanced}
      <svg aria-hidden="true" style="position: absolute; width: 0; height: 0; overflow: hidden; pointer-events: none;">
        <defs>
          <filter id="tron-glow" x="-30%" y="-30%" width="160%" height="160%">
            <feGaussianBlur in="SourceGraphic" stdDeviation="1.5" result="blur"/>
            <feMerge>
              <feMergeNode in="blur"/>
              <feMergeNode in="SourceGraphic"/>
            </feMerge>
          </filter>
        </defs>
      </svg>
    {/if}
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div class="commit-row px-2 py-1.5 bg-card border-b border-border text-[11px] font-semibold text-muted-foreground uppercase tracking-wide">
      <span class="text-center header-cell">Graph<span class="resize-handle" role="separator" onmousedown={startResize('graph')}></span></span>
      <span class="text-center header-cell">Message<span class="resize-handle" role="separator" onmousedown={startResize('author', true)}></span></span>
      <span class="text-center header-cell">Author<span class="resize-handle" role="separator" onmousedown={startResizePair('author', 'date')}></span></span>
      <span class="text-center header-cell">Date<span class="resize-handle" role="separator" onmousedown={startResizePair('date', 'sha')}></span></span>
      <span class="text-center">SHA</span>
    </div>

    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <div
      class="flex-1 overflow-y-auto outline-none"
      tabindex="0"
      role="listbox"
      aria-label="Commit list"
      bind:this={listEl}
      onkeydown={handleKeydown}
      onscroll={handleListScroll}
    >
      {#each $commitGraph.commits as commit, i}
        {@const node = layout?.nodes[i]}
        {@const branches = getBranchesForCommit(commit.oid)}
        {@const isHead = commit.oid === $commitGraph?.head_oid}
        {@const rowLanes = laneActivities[i] ?? new Map()}
        <button
          role="option"
          aria-selected={isSelected(commit)}
          class="commit-row px-2 border-b border-border/50 w-full text-left cursor-pointer transition-colors font-inherit text-inherit {isSelected(commit) ? 'bg-accent' : 'hover:bg-accent/50'} {isHead ? 'font-medium' : ''}"
          style="height: {ROW_HEIGHT}px; position: relative;"
          onclick={() => selectCommit(commit)}
          oncontextmenu={(e) => handleCommitContextMenu(e, commit)}
        >
          <!-- Graph column -->
          <span class="graph-cell" style="height: {ROW_HEIGHT}px">
            {#if node}
              <svg width={graphColumnWidth} height={ROW_HEIGHT} class="block" style="overflow: visible;" filter={isTronEnhanced ? 'url(#tron-glow)' : null}>
                {#each [...rowLanes] as [lane, activity]}
                  {#if lane !== node.lane}
                    <line
                      x1={laneX(lane)} y1={0}
                      x2={laneX(lane)} y2={ROW_HEIGHT}
                      stroke={getGraphColor(activity.colorIndex)}
                      stroke-width={LINE_WIDTH}
                      stroke-linecap="round"
                    />
                  {:else}
                    {#if activity.hasTop}
                      <line
                        x1={laneX(lane)} y1={0}
                        x2={laneX(lane)} y2={ROW_HEIGHT / 2}
                        stroke={getGraphColor(activity.colorIndex)}
                        stroke-width={LINE_WIDTH}
                        stroke-linecap="round"
                      />
                    {/if}
                    {#if activity.hasBottom}
                      <line
                        x1={laneX(lane)} y1={ROW_HEIGHT / 2}
                        x2={laneX(lane)} y2={ROW_HEIGHT}
                        stroke={getGraphColor(activity.colorIndex)}
                        stroke-width={LINE_WIDTH}
                        stroke-linecap="round"
                      />
                    {/if}
                  {/if}
                {/each}

                {#each node.edges as edge}
                  {#if edge.from_lane !== edge.to_lane}
                    <path
                      d={getEdgePath(edge)}
                      stroke={getGraphColor(edge.color_index)}
                      stroke-width={LINE_WIDTH}
                      stroke-linecap="round"
                      fill="none"
                    />
                  {/if}
                {/each}

                {#if stashMap.has(commit.oid)}
                  {@const cx = laneX(node.lane)}
                  {@const cy = ROW_HEIGHT / 2}
                  {@const color = getGraphColor(node.color_index)}
                  <rect x={cx - 7} y={cy - 6} width="14" height="4" rx="1.5" fill={color} />
                  <rect x={cx - 8} y={cy - 2} width="16" height="9" rx="1.5" fill={color} />
                  <rect x={cx - 0.5} y={cy - 6} width="1" height="13" fill="var(--background)" opacity="0.5" />
                  <circle cx={cx} cy={cy + 2} r="1.8" fill="var(--background)" />
                {:else}
                  {#if isTronEnhanced && isHead}
                    <circle
                      class="tron-head-pulse"
                      cx={laneX(node.lane)}
                      cy={ROW_HEIGHT / 2}
                      r={CIRCLE_RADIUS}
                      fill={getGraphColor(node.color_index)}
                      stroke="none"
                    />
                  {/if}
                  <circle
                    cx={laneX(node.lane)}
                    cy={ROW_HEIGHT / 2}
                    r={CIRCLE_RADIUS}
                    fill={getGraphColor(node.color_index)}
                    stroke={isHead ? 'var(--foreground)' : 'var(--background)'}
                    stroke-width={isHead ? 2 : 1.5}
                  />
                {/if}
              </svg>
            {:else}
              <svg width="24" height="24" viewBox="0 0 24 24" class="block">
                <circle cx="12" cy="12" r="4" fill="#888" />
              </svg>
            {/if}
          </span>

          <!-- Message column -->
          <span class="min-w-0 overflow-hidden truncate text-left">{commit.summary}</span>

          <span class="author-cell text-muted-foreground text-center">
            <img
              class="avatar"
              src={gravatarUrl(commit.author.email, 40)}
              alt=""
              loading="lazy"
              onerror={(e) => { const img = e.currentTarget as HTMLImageElement; img.style.display = 'none'; (img.nextElementSibling as HTMLElement)?.style.setProperty('display', ''); }}
            />
            <!-- Fallback: person silhouette -->
            <svg class="avatar-fallback" style="display: none" viewBox="0 0 16 16" fill="currentColor">
              <path d="M8 8a3 3 0 1 0 0-6 3 3 0 0 0 0 6Zm5.72 4.72a.75.75 0 0 1-1.06 1.06A6.97 6.97 0 0 0 8 12a6.97 6.97 0 0 0-4.66 1.78.75.75 0 0 1-1.06-1.06A8.46 8.46 0 0 1 8 10.5c2.2 0 4.2.84 5.72 2.22Z"/>
            </svg>
            <span class="truncate">{commit.author.name}</span>
          </span>
          <span class="text-muted-foreground text-xs text-center">{formatDate(commit.timestamp)}</span>
          <span class="font-mono text-[11px] text-muted-foreground text-center">{commit.short_oid}</span>

          <!-- Branch tags: positioned absolutely over the row -->
          {#if node && branches.length > 0}
            {@const grouped = groupBranches(branches)}
            <span
              class="branch-tags"
              style="left: {laneX(node.lane) + CIRCLE_RADIUS + 6 + 8}px"
            >
              {#each grouped as group}
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <span
                  class="branch-tag"
                  style={getUnifiedBranchStyle(group)}
                  onclick={(e) => e.stopPropagation()}
                  ondblclick={(e) => handleBranchClick(e, group.primary)}
                  oncontextmenu={(e) => handleBranchContextMenu(e, group.primary)}
                >
                  {#if group.primary.is_head}
                    <svg class="branch-icon" viewBox="0 0 16 16" fill="currentColor"><path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.75.75 0 0 1 1.06-1.06L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"/></svg>
                  {/if}
                  {group.name}
                  {#if group.local}
                    <svg class="branch-icon" viewBox="0 0 16 16" fill="currentColor"><path d="M2 3.5A1.5 1.5 0 0 1 3.5 2h9A1.5 1.5 0 0 1 14 3.5v7a1.5 1.5 0 0 1-1.5 1.5H10v1h1.5a.5.5 0 0 1 0 1h-7a.5.5 0 0 1 0-1H6v-1H3.5A1.5 1.5 0 0 1 2 10.5v-7Zm1.5-.5a.5.5 0 0 0-.5.5v7a.5.5 0 0 0 .5.5h9a.5.5 0 0 0 .5-.5v-7a.5.5 0 0 0-.5-.5h-9ZM7 12v1h2v-1H7Z"/></svg>
                  {/if}
                  {#if group.remote}
                    <svg class="branch-icon" viewBox="0 0 16 16" fill="currentColor"><path d="M4.5 11a.5.5 0 0 1 .5-.5h6a.5.5 0 0 1 0 1H5a.5.5 0 0 1-.5-.5Zm-.4-3.8A3.5 3.5 0 0 1 11 5.5a.5.5 0 0 0 .5.5 2.5 2.5 0 0 1 0 5h-7a3 3 0 0 1-.4-5.8ZM8 3a4.5 4.5 0 0 0-4.38 3.48A4 4 0 0 0 4.5 14h7a3.5 3.5 0 0 0 .83-6.9A4.49 4.49 0 0 0 8 3Z"/></svg>
                  {/if}
                </span>
              {/each}
            </span>
          {/if}
        </button>
      {/each}
    </div>
  {:else}
    <div class="flex items-center justify-center h-full text-muted-foreground">
      <p>No commits to display</p>
    </div>
  {/if}
</div>

<!-- Context menu overlay -->
{#if contextMenu}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="fixed inset-0 z-40" onclick={closeContextMenu} oncontextmenu={(e) => { e.preventDefault(); closeContextMenu(); }}></div>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed z-50 min-w-[180px] rounded-lg border border-border bg-popover shadow-lg py-1"
    style="left: {contextMenu.x}px; top: {contextMenu.y}px"
    onclick={(e) => e.stopPropagation()}
  >
    {#each contextMenu.items as item}
      {#if item === 'separator'}
        <div class="my-1 h-px bg-border"></div>
      {:else if item.submenu}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="submenu-parent relative"
          onmouseenter={() => hoveredSubmenu = item.id}
          onmouseleave={() => hoveredSubmenu = null}
        >
          <button
            type="button"
            class="context-menu-item w-full text-left px-3 py-1.5 text-sm outline-none transition-colors text-popover-foreground hover:bg-accent cursor-pointer flex items-center justify-between"
          >
            <span>{item.label}</span>
            <span class="ml-2 text-muted-foreground">&rsaquo;</span>
          </button>
          {#if hoveredSubmenu === item.id}
            <div class="absolute left-full top-0 ml-0.5 min-w-[200px] rounded-lg border border-border bg-popover shadow-lg py-1 z-[60]">
              {#each item.submenu as sub}
                <button
                  type="button"
                  class="context-menu-item w-full text-left px-3 py-1.5 text-sm outline-none transition-colors
                    {sub.danger ? 'text-destructive hover:bg-destructive/10 cursor-pointer' : 'text-popover-foreground hover:bg-accent cursor-pointer'}"
                  onclick={() => executeMenuAction(sub.id)}
                >
                  {sub.label}
                </button>
              {/each}
            </div>
          {/if}
        </div>
      {:else}
        <button
          type="button"
          class="context-menu-item w-full text-left px-3 py-1.5 text-sm outline-none transition-colors
            {item.disabled ? 'text-muted-foreground/50 cursor-default pointer-events-none' : item.danger ? 'text-destructive hover:bg-destructive/10 cursor-pointer' : 'text-popover-foreground hover:bg-accent cursor-pointer'}"
          onclick={() => executeMenuAction(item.id)}
        >
          {item.label}
        </button>
      {/if}
    {/each}
  </div>
{/if}

<!-- Branch creation prompt -->
{#if branchPrompt}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="fixed inset-0 z-40" onclick={closeBranchPrompt}></div>
  <div
    class="fixed z-50 min-w-[240px] rounded-lg border border-border bg-popover shadow-lg p-3"
    style="left: {branchPrompt.x}px; top: {branchPrompt.y}px"
  >
    <div class="text-xs text-muted-foreground mb-2">
      Create branch at <span class="font-mono">{branchPrompt.shortOid}</span>
    </div>
    <input
      class="w-full px-2 py-1.5 text-sm bg-background border border-input rounded-md outline-none focus:border-primary transition-colors"
      placeholder="Branch name..."
      bind:value={newBranchName}
      onkeydown={handleBranchPromptKeydown}
      use:autoFocusAction
    />
  </div>
{/if}

<style>
  .commit-row {
    display: grid;
    grid-template-columns: var(--grid-cols);
    align-items: center;
    gap: 4px;
  }

  .graph-cell {
    position: relative;
    overflow: visible;
  }

  .branch-tags {
    position: absolute;
    top: 0;
    height: 100%;
    display: flex;
    align-items: center;
    gap: 4px;
    z-index: 10;
    pointer-events: auto;
  }

  .branch-tag {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    padding: 1px 6px;
    border-radius: 3px;
    font-size: 11px;
    font-weight: 600;
    white-space: nowrap;
    border-width: 1px;
    border-style: solid;
    cursor: pointer;
    transition: filter 150ms ease;
  }

  .branch-icon {
    width: 12px;
    height: 12px;
    flex-shrink: 0;
  }

  .author-cell {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 5px;
    min-width: 0;
    overflow: hidden;
  }

  .avatar {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    flex-shrink: 0;
    object-fit: cover;
  }

  .avatar-fallback {
    width: 18px;
    height: 18px;
    flex-shrink: 0;
    opacity: 0.5;
  }

  .branch-tag:hover {
    filter: brightness(1.25);
  }

  .header-cell {
    position: relative;
  }

  .resize-handle {
    position: absolute;
    top: 0;
    right: -5px;
    width: 6px;
    height: 100%;
    cursor: col-resize;
    z-index: 1;
    background: transparent;
  }

  .resize-handle::after {
    content: '';
    position: absolute;
    top: 15%;
    bottom: 15%;
    left: 50%;
    width: 2px;
    border-radius: 1px;
    background: var(--primary);
    opacity: 0.35;
    transform: translateX(-50%);
    transition: opacity 150ms ease;
  }

  .resize-handle:hover::after {
    opacity: 0.8;
  }

  .context-menu-item:disabled {
    pointer-events: none;
  }

  /* Tron: HEAD commit pulse ring */
  :global(.tron-enhanced) .tron-head-pulse {
    transform-box: fill-box;
    transform-origin: center;
    animation: tron-head-ring 2s ease-out infinite;
  }

  @keyframes tron-head-ring {
    0% {
      transform: scale(1);
      opacity: 0.7;
    }
    100% {
      transform: scale(3);
      opacity: 0;
    }
  }
</style>
