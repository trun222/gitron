<script lang="ts">
  import { repoInfo, repoPath, currentBranch, stagedCount, unstagedCount, hasRepo, trackingStatus } from '$lib/stores/repo';
  import { hasEntries, toggleOutputPanel } from '$lib/stores/output';
  import { toggleTerminalPanel, bottomPanelOpen, activeBottomTab, terminalSessionId } from '$lib/stores/terminal';
</script>

<footer class="flex items-center justify-between h-6 px-3 bg-card border-t border-border text-muted-foreground text-[11px]">
  {#if $hasRepo}
    <div class="flex items-center gap-3">
      {#if $repoPath}
        <span class="flex items-center gap-1" title={$repoPath}>
          <svg class="shrink-0" viewBox="0 0 16 16" width="12" height="12">
            <path fill="currentColor" d="M1.75 1A1.75 1.75 0 0 0 0 2.75v10.5C0 14.216.784 15 1.75 15h12.5A1.75 1.75 0 0 0 16 13.25v-8.5A1.75 1.75 0 0 0 14.25 3H7.5a.25.25 0 0 1-.2-.1l-.9-1.2C6.07 1.26 5.55 1 5 1H1.75Z" />
          </svg>
          {$repoPath.split('/').pop()}
        </span>
      {/if}
      <span class="flex items-center gap-1">
        <svg class="shrink-0" viewBox="0 0 16 16" width="12" height="12">
          <path fill="currentColor" d="M9.5 3.25a2.25 2.25 0 1 1 3 2.122V6A2.5 2.5 0 0 1 10 8.5H6a1 1 0 0 0-1 1v1.128a2.251 2.251 0 1 1-1.5 0V5.372a2.25 2.25 0 1 1 1.5 0v1.836A2.5 2.5 0 0 1 6 7h4a1 1 0 0 0 1-1v-.628A2.25 2.25 0 0 1 9.5 3.25Z" />
        </svg>
        {$currentBranch ?? 'HEAD detached'}
      </span>
      {#if $trackingStatus?.upstream}
        {#if $trackingStatus.ahead > 0 || $trackingStatus.behind > 0}
          <span class="flex items-center gap-1 text-[10px]">
            {#if $trackingStatus.ahead > 0}
              <span title="Commits ahead of upstream">↑{$trackingStatus.ahead}</span>
            {/if}
            {#if $trackingStatus.behind > 0}
              <span title="Commits behind upstream">↓{$trackingStatus.behind}</span>
            {/if}
          </span>
        {:else}
          <span class="text-[10px] opacity-70" title="Up to date with upstream">✓</span>
        {/if}
      {/if}
    </div>
    <div class="flex items-center gap-3">
      {#if $stagedCount > 0}
        <span>{$stagedCount} staged</span>
      {/if}
      {#if $unstagedCount > 0}
        <span>{$unstagedCount} changed</span>
      {/if}
      {#if $hasEntries}
        <button
          class="flex items-center gap-1 hover:text-foreground transition-colors cursor-pointer {$bottomPanelOpen && $activeBottomTab === 'output' ? 'text-foreground' : ''}"
          onclick={toggleOutputPanel}
          title="Toggle output panel (Cmd+`)"
        >
          <svg viewBox="0 0 16 16" width="12" height="12">
            <path fill="currentColor" d="M0 2.75C0 1.784.784 1 1.75 1h12.5c.966 0 1.75.784 1.75 1.75v10.5A1.75 1.75 0 0 1 14.25 15H1.75A1.75 1.75 0 0 1 0 13.25Zm1.75-.25a.25.25 0 0 0-.25.25v10.5c0 .138.112.25.25.25h12.5a.25.25 0 0 0 .25-.25V2.75a.25.25 0 0 0-.25-.25Zm7.47 3.97a.75.75 0 0 1 1.06 0l2 2a.75.75 0 0 1 0 1.06l-2 2a.75.75 0 1 1-1.06-1.06L10.69 9l-1.47-1.47a.75.75 0 0 1 0-1.06ZM6.78 6.47a.75.75 0 0 1 0 1.06L5.31 9l1.47 1.47a.75.75 0 0 1-1.06 1.06l-2-2a.75.75 0 0 1 0-1.06l2-2a.75.75 0 0 1 1.06 0Z" />
          </svg>
          Output
        </button>
      {/if}
      <button
        class="flex items-center gap-1 hover:text-foreground transition-colors cursor-pointer {$bottomPanelOpen && $activeBottomTab === 'terminal' ? 'text-foreground' : ''}"
        onclick={toggleTerminalPanel}
        title="Toggle terminal (Ctrl+`)"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="4 17 10 11 4 5"></polyline><line x1="12" y1="19" x2="20" y2="19"></line></svg>
        Terminal
        {#if $terminalSessionId}
          <span class="w-1.5 h-1.5 rounded-full bg-primary"></span>
        {/if}
      </button>
    </div>
  {:else}
    <div class="flex items-center">
      <span class="opacity-70">No repository open</span>
    </div>
  {/if}
</footer>
