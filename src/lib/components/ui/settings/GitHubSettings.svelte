<script lang="ts">
  import { isAuthenticated, currentUser, startLogin, logout, authLoading } from '$lib/stores/github';
</script>

<div class="section">
  <h3 class="section-title">Authentication</h3>

  {#if $isAuthenticated && $currentUser}
    <div class="auth-card">
      <img
        src={$currentUser.avatar_url}
        alt={$currentUser.login}
        class="avatar"
      />
      <div class="user-info">
        <span class="user-name">{$currentUser.name ?? $currentUser.login}</span>
        <span class="user-login">@{$currentUser.login}</span>
      </div>
      <button class="sign-out-btn" onclick={() => logout()}>Sign Out</button>
    </div>
  {:else}
    <div class="auth-card">
      <div class="github-placeholder">
        <svg viewBox="0 0 16 16" width="24" height="24">
          <path fill="currentColor" d="M8 0c4.42 0 8 3.58 8 8a8.013 8.013 0 0 1-5.45 7.59c-.4.08-.55-.17-.55-.38 0-.27.01-1.13.01-2.2 0-.75-.25-1.23-.54-1.48 1.78-.2 3.65-.88 3.65-3.95 0-.88-.31-1.59-.82-2.15.08-.2.36-1.02-.08-2.12 0 0-.67-.22-2.2.82-.64-.18-1.32-.27-2-.27-.68 0-1.36.09-2 .27-1.53-1.03-2.2-.82-2.2-.82-.44 1.1-.16 1.92-.08 2.12-.51.56-.82 1.28-.82 2.15 0 3.06 1.86 3.75 3.64 3.95-.23.2-.44.55-.51 1.07-.46.21-1.61.55-2.33-.66-.15-.24-.6-.83-1.23-.82-.67.01-.27.38.01.53.34.19.73.9.82 1.13.16.45.68 1.31 2.69.94 0 .67.01 1.3.01 1.49 0 .21-.15.45-.55.38A7.995 7.995 0 0 1 0 8c0-4.42 3.58-8 8-8Z" />
        </svg>
      </div>
      <div class="user-info">
        <span class="user-name not-connected">Not connected</span>
        <span class="user-login">Sign in with GitHub to enable integrations</span>
      </div>
      <button
        class="sign-in-btn"
        onclick={() => startLogin()}
        disabled={$authLoading}
      >
        {$authLoading ? 'Signing in...' : 'Sign In'}
      </button>
    </div>
  {/if}
</div>

<style>
  .section {
    margin-bottom: 24px;
  }

  .section-title {
    font-size: 11px;
    font-weight: 500;
    color: var(--muted-foreground);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 12px;
  }

  .auth-card {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--secondary);
  }

  .avatar {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .github-placeholder {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--accent);
    color: var(--muted-foreground);
    flex-shrink: 0;
  }

  .user-info {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
    flex: 1;
  }

  .user-name {
    font-size: 13px;
    font-weight: 500;
    color: var(--foreground);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .user-name.not-connected {
    color: var(--muted-foreground);
  }

  .user-login {
    font-size: 11px;
    color: var(--muted-foreground);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .sign-out-btn {
    padding: 4px 12px;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--muted-foreground);
    font-size: 12px;
    cursor: pointer;
    flex-shrink: 0;
    transition: background 0.15s, color 0.15s;
  }
  .sign-out-btn:hover {
    background: var(--destructive);
    color: white;
    border-color: var(--destructive);
  }

  .sign-in-btn {
    padding: 4px 12px;
    border-radius: 6px;
    background: var(--primary);
    color: var(--primary-foreground);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    flex-shrink: 0;
    transition: opacity 0.15s;
  }
  .sign-in-btn:hover:not(:disabled) {
    opacity: 0.9;
  }
  .sign-in-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
