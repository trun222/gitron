<script lang="ts">
  import { commitGraph, selectedCommit, selectCommit } from '$lib/stores/repo';
  import { graphColumnWidths, saveGraphColumnWidths } from '$lib/stores/settings';
  import type { Commit, Branch, GraphColumnWidths, GraphEdge } from '$lib/api/types';

  const GRAPH_COLOR_COUNT = 14;
  const ROW_HEIGHT = 30;
  const LANE_WIDTH = 20;
  const LANE_PADDING = 8;
  const CIRCLE_RADIUS = 4;
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

  // Sync from store
  $effect(() => {
    const storeVal = $graphColumnWidths;
    colWidths = { ...storeVal };
  });

  // Cache graph colors from CSS custom properties
  let graphColors: string[] = $state([]);
  $effect(() => {
    const style = getComputedStyle(document.documentElement);
    const colors: string[] = [];
    for (let i = 0; i < GRAPH_COLOR_COUNT; i++) {
      colors.push(style.getPropertyValue(`--color-graph-${i}`).trim() || '#888');
    }
    graphColors = colors;
  });

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

  // Drag state for column resizing
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
          // Straight edge: vertical line at this lane from row to edge.to_row
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
          // Cross-lane edge: curve at this row, then vertical at to_lane
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
    // S-curve cubic bezier
    return `M ${x1} ${y1} C ${x1} ${y1 + ROW_HEIGHT * 0.35}, ${x2} ${y2 - ROW_HEIGHT * 0.35}, ${x2} ${y2}`;
  }

  // Branch color lookup from layout
  let branchColorMap = $derived.by(() => {
    const layout = $commitGraph?.layout;
    if (!layout) return new Map<string, number>();
    return new Map(layout.branch_colors.map((e) => [e.name, e.color_index]));
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
</script>

<div class="flex flex-col flex-1 overflow-hidden text-[13px]" style="--grid-cols: {getGridTemplate()}">
  {#if $commitGraph && $commitGraph.commits.length > 0}
    {@const layout = $commitGraph.layout}
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
          style="height: {ROW_HEIGHT}px"
          onclick={() => selectCommit(commit)}
        >
          <span class="flex items-center overflow-hidden" style="height: {ROW_HEIGHT}px">
            {#if node}
              <svg width={graphColumnWidth} height={ROW_HEIGHT} class="block shrink-0">
                <!-- Pass-through vertical lines and commit lane lines -->
                {#each [...rowLanes] as [lane, activity]}
                  {#if lane !== node.lane}
                    <!-- Pass-through: full vertical line -->
                    <line
                      x1={laneX(lane)} y1={0}
                      x2={laneX(lane)} y2={ROW_HEIGHT}
                      stroke={getGraphColor(activity.colorIndex)}
                      stroke-width={LINE_WIDTH}
                    />
                  {:else}
                    <!-- Commit's lane: split around circle -->
                    {#if activity.hasTop}
                      <line
                        x1={laneX(lane)} y1={0}
                        x2={laneX(lane)} y2={ROW_HEIGHT / 2}
                        stroke={getGraphColor(activity.colorIndex)}
                        stroke-width={LINE_WIDTH}
                      />
                    {/if}
                    {#if activity.hasBottom}
                      <line
                        x1={laneX(lane)} y1={ROW_HEIGHT / 2}
                        x2={laneX(lane)} y2={ROW_HEIGHT}
                        stroke={getGraphColor(activity.colorIndex)}
                        stroke-width={LINE_WIDTH}
                      />
                    {/if}
                  {/if}
                {/each}

                <!-- Cross-lane edge curves -->
                {#each node.edges as edge}
                  {#if edge.from_lane !== edge.to_lane}
                    <path
                      d={getEdgePath(edge)}
                      stroke={getGraphColor(edge.color_index)}
                      stroke-width={LINE_WIDTH}
                      fill="none"
                    />
                  {/if}
                {/each}

                <!-- Commit circle -->
                <circle
                  cx={laneX(node.lane)}
                  cy={ROW_HEIGHT / 2}
                  r={CIRCLE_RADIUS}
                  fill={getGraphColor(node.color_index)}
                  stroke={isHead ? '#fff' : 'none'}
                  stroke-width={isHead ? 2 : 0}
                />
              </svg>
            {:else}
              <svg width="24" height="24" viewBox="0 0 24 24" class="block shrink-0">
                <circle cx="12" cy="12" r="4" fill="#888" />
              </svg>
            {/if}
          </span>

          <span class="flex items-center gap-1.5 min-w-0 overflow-hidden">
            {#each branches as branch}
              {@const labelStyle = getBranchLabelStyle(branch)}
              <span
                class="inline-flex px-1.5 py-px rounded-sm text-[11px] font-semibold shrink-0 border"
                style={labelStyle}
              >
                {branch.name}
              </span>
            {/each}
            <span class="truncate">{commit.summary}</span>
          </span>

          <span class="text-muted-foreground truncate text-center">{commit.author.name}</span>
          <span class="text-muted-foreground text-xs text-center">{formatDate(commit.timestamp)}</span>
          <span class="font-mono text-[11px] text-muted-foreground text-center">{commit.short_oid}</span>
        </button>
      {/each}
    </div>
  {:else}
    <div class="flex items-center justify-center h-full text-muted-foreground">
      <p>No commits to display</p>
    </div>
  {/if}
</div>

<style>
  .commit-row {
    display: grid;
    grid-template-columns: var(--grid-cols);
    align-items: center;
    gap: 4px;
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
</style>
