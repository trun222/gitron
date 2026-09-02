<script lang="ts">
  import { Dialog } from 'bits-ui';
  import { copyToClipboard } from '$lib/utils/clipboard';
  import { loginDialogOpen, deviceFlow, authLoading, authError, cancelLogin } from '$lib/stores/github';

  let copied = $state(false);

  async function copyCode() {
    const code = $deviceFlow?.user_code;
    if (!code) return;
    const ok = await copyToClipboard(code);
    if (!ok) return;
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }
</script>

<Dialog.Root
  open={$loginDialogOpen}
  onOpenChange={(open) => { if (!open) cancelLogin(); }}
>
  <Dialog.Portal>
    <Dialog.Overlay class="fixed inset-0 bg-black/50 z-50" />
    <Dialog.Content
      class="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 z-50 w-full max-w-sm rounded-lg border border-border bg-card p-6 shadow-lg"
    >
      <Dialog.Title class="text-sm font-semibold text-foreground mb-1">
        Sign in to GitHub
      </Dialog.Title>

      {#if $deviceFlow}
        <Dialog.Description class="text-sm text-muted-foreground mb-4">
          Enter this code at <strong class="text-foreground">{$deviceFlow.verification_uri}</strong>
        </Dialog.Description>

        <div class="flex items-center justify-center gap-3 mb-4">
          <code class="text-2xl font-mono font-bold tracking-[0.25em] text-foreground bg-background border border-border rounded-md px-4 py-2 select-all">
            {$deviceFlow.user_code}
          </code>
          <button
            type="button"
            class="px-3 py-2 text-xs rounded-md border border-border bg-background text-foreground hover:bg-accent transition-colors"
            onclick={copyCode}
          >
            {copied ? 'Copied!' : 'Copy'}
          </button>
        </div>

        {#if $authLoading}
          <div class="flex items-center justify-center gap-2 text-sm text-muted-foreground mb-4">
            <svg class="animate-spin" viewBox="0 0 16 16" width="14" height="14">
              <circle cx="8" cy="8" r="6" fill="none" stroke="currentColor" stroke-width="2" stroke-dasharray="28" stroke-dashoffset="8" stroke-linecap="round"/>
            </svg>
            Waiting for authorization...
          </div>
        {/if}

        {#if $authError}
          <p class="text-sm text-destructive mb-4">{$authError}</p>
        {/if}
      {:else}
        <Dialog.Description class="text-sm text-muted-foreground mb-4">
          Requesting device code...
        </Dialog.Description>
      {/if}

      <div class="flex justify-end">
        <button
          type="button"
          class="px-3 py-1.5 text-sm rounded-md border border-border bg-background text-foreground hover:bg-accent transition-colors"
          onclick={() => cancelLogin()}
        >
          Cancel
        </button>
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
