<script lang="ts">
  import { toasts, dismissToast } from '$lib/stores/toast';
</script>

{#if $toasts.length > 0}
  <div class="toast-container">
    {#each $toasts as toast (toast.id)}
      <div
        class="toast"
        class:toast-success={toast.type === 'success'}
        class:toast-error={toast.type === 'error'}
        class:toast-info={toast.type === 'info'}
        role="status"
      >
        <span class="toast-icon">
          {#if toast.type === 'success'}
            <svg viewBox="0 0 16 16" width="14" height="14">
              <path fill="currentColor" d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.75.75 0 0 1 1.06-1.06L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z" />
            </svg>
          {:else if toast.type === 'error'}
            <svg viewBox="0 0 16 16" width="14" height="14">
              <path fill="currentColor" d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z" />
            </svg>
          {:else}
            <svg viewBox="0 0 16 16" width="14" height="14">
              <path fill="currentColor" d="M0 8a8 8 0 1 1 16 0A8 8 0 0 1 0 8Zm8-6.5a6.5 6.5 0 1 0 0 13 6.5 6.5 0 0 0 0-13ZM6.5 7.75A.75.75 0 0 1 7.25 7h1a.75.75 0 0 1 .75.75v2.75h.25a.75.75 0 0 1 0 1.5h-2a.75.75 0 0 1 0-1.5h.25v-2h-.25a.75.75 0 0 1-.75-.75ZM8 6a1 1 0 1 1 0-2 1 1 0 0 1 0 2Z" />
            </svg>
          {/if}
        </span>
        <span class="toast-message">{toast.message}</span>
        <button
          type="button"
          class="toast-dismiss"
          onclick={() => dismissToast(toast.id)}
        >
          <svg viewBox="0 0 16 16" width="12" height="12">
            <path fill="currentColor" d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z" />
          </svg>
        </button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .toast-container {
    position: fixed;
    bottom: 36px;
    right: 12px;
    z-index: 1000;
    display: flex;
    flex-direction: column;
    gap: 8px;
    pointer-events: none;
  }

  .toast {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px;
    border-radius: 8px;
    font-size: 13px;
    font-weight: 500;
    line-height: 1.4;
    pointer-events: auto;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.6), 0 2px 8px rgba(0, 0, 0, 0.4);
    animation: toast-in 0.2s ease-out;
    max-width: 420px;
    border: 1px solid;
    backdrop-filter: blur(12px);
  }

  .toast-success {
    background-color: oklch(0.22 0.04 145);
    border-color: var(--color-git-added);
    color: var(--color-git-added);
  }

  .toast-error {
    background-color: oklch(0.22 0.04 25);
    border-color: var(--color-destructive);
    color: var(--color-destructive);
  }

  .toast-info {
    background-color: oklch(0.22 0.03 260);
    border-color: var(--color-primary);
    color: var(--color-foreground);
  }

  .toast-icon {
    display: flex;
    align-items: center;
    flex-shrink: 0;
  }

  .toast-message {
    flex: 1;
    min-width: 0;
  }

  .toast-dismiss {
    display: flex;
    align-items: center;
    flex-shrink: 0;
    opacity: 0.7;
    cursor: pointer;
    background: none;
    border: none;
    color: inherit;
    padding: 2px;
    border-radius: 4px;
  }

  .toast-dismiss:hover {
    opacity: 1;
  }

  @keyframes toast-in {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
