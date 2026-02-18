<script lang="ts">
  import { onMount } from 'svelte';
  import { hasRepo, currentBranch } from '$lib/stores/repo';
  import { CommandBar } from '$lib/components/ui/command';

  let commandBar: CommandBar | undefined = $state();

  onMount(() => {
    function handleKeydown(e: KeyboardEvent) {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        commandBar?.focus();
      }
    }
    document.addEventListener('keydown', handleKeydown);
    return () => document.removeEventListener('keydown', handleKeydown);
  });
</script>

<header class="flex items-center justify-between h-12 px-4 bg-card border-b border-border select-none" style="-webkit-app-region: drag;">
  <div class="flex items-center min-w-[200px]">
    <span class="font-bold text-sm tracking-wide text-primary">Gitron</span>
  </div>

  <div class="flex-1 flex justify-center" style="-webkit-app-region: no-drag;">
    <CommandBar bind:this={commandBar} />
  </div>

  <div class="flex items-center justify-end min-w-[200px]">
    {#if $hasRepo && $currentBranch}
      <span class="flex items-center gap-1.5 px-2.5 py-1 rounded bg-secondary text-xs font-medium text-muted-foreground">
        <svg class="shrink-0" viewBox="0 0 16 16" width="14" height="14">
          <path fill="currentColor" d="M9.5 3.25a2.25 2.25 0 1 1 3 2.122V6A2.5 2.5 0 0 1 10 8.5H6a1 1 0 0 0-1 1v1.128a2.251 2.251 0 1 1-1.5 0V5.372a2.25 2.25 0 1 1 1.5 0v1.836A2.5 2.5 0 0 1 6 7h4a1 1 0 0 0 1-1v-.628A2.25 2.25 0 0 1 9.5 3.25Z" />
        </svg>
        {$currentBranch}
      </span>
    {/if}
  </div>
</header>
