<script lang="ts">
  import { isAuthenticated, currentUser, authLoading, startLogin, logout } from '$lib/stores/github';

  let menuOpen = $state(false);

  function handleSignOut() {
    menuOpen = false;
    logout();
  }
</script>

<div class="relative" style="-webkit-app-region: no-drag;">
  {#if $isAuthenticated && $currentUser}
    <button
      type="button"
      class="flex items-center gap-1.5 px-2 py-1 rounded-md hover:bg-accent transition-colors text-sm text-muted-foreground"
      onclick={() => menuOpen = !menuOpen}
      title={$currentUser.login}
    >
      <img
        src={$currentUser.avatar_url}
        alt={$currentUser.login}
        class="w-5 h-5 rounded-full"
      />
      <span class="max-w-[100px] truncate text-xs">{$currentUser.login}</span>
    </button>

    {#if menuOpen}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="fixed inset-0 z-40"
        onclick={() => menuOpen = false}
        onkeydown={(e) => { if (e.key === 'Escape') menuOpen = false; }}
      ></div>
      <div class="absolute right-0 top-full mt-1 z-50 w-48 rounded-md border border-border bg-card shadow-lg py-1">
        <div class="px-3 py-2 border-b border-border">
          <p class="text-sm font-medium text-foreground truncate">{$currentUser.name ?? $currentUser.login}</p>
          <p class="text-xs text-muted-foreground truncate">{$currentUser.login}</p>
        </div>
        <button
          type="button"
          class="w-full text-left px-3 py-1.5 text-sm text-foreground hover:bg-accent transition-colors"
          onclick={handleSignOut}
        >
          Sign out
        </button>
      </div>
    {/if}
  {:else}
    <button
      type="button"
      class="flex items-center gap-1.5 px-2 py-1 rounded-md hover:bg-accent transition-colors text-sm text-muted-foreground"
      onclick={() => startLogin()}
      disabled={$authLoading}
      title="Sign in to GitHub"
    >
      {#if $authLoading}
        <svg class="animate-spin" viewBox="0 0 16 16" width="14" height="14">
          <circle cx="8" cy="8" r="6" fill="none" stroke="currentColor" stroke-width="2" stroke-dasharray="28" stroke-dashoffset="8" stroke-linecap="round"/>
        </svg>
      {:else}
        <svg viewBox="0 0 16 16" width="14" height="14">
          <path fill="currentColor" d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0 0 16 8c0-4.42-3.58-8-8-8Z"/>
        </svg>
      {/if}
      <span class="text-xs">Sign in</span>
    </button>
  {/if}
</div>
