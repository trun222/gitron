<script lang="ts">
  import { Dialog } from 'bits-ui';
  import {
    forcePushConfirmOpen,
    currentBranch,
    networkOperation,
    pushToRemote,
  } from '$lib/stores/repo';

  async function handleForcePush() {
    forcePushConfirmOpen.set(false);
    await pushToRemote(undefined, true);
  }
</script>

<Dialog.Root
  open={$forcePushConfirmOpen}
  onOpenChange={(open) => forcePushConfirmOpen.set(open)}
>
  <Dialog.Portal>
    <Dialog.Overlay class="fixed inset-0 bg-black/50 z-50" />
    <Dialog.Content
      class="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 z-50 w-full max-w-sm rounded-lg border border-border bg-card p-6 shadow-lg"
    >
      <Dialog.Title class="text-sm font-semibold text-foreground mb-1">
        Force push?
      </Dialog.Title>
      <Dialog.Description class="text-sm text-muted-foreground mb-5">
        This will force push <strong class="text-foreground">{$currentBranch ?? 'current branch'}</strong> to the remote using <code class="text-xs bg-muted px-1 py-0.5 rounded">--force-with-lease</code>.
        <span class="block mt-2 text-destructive text-xs">
          This may overwrite remote history. Only use this after rebasing or amending commits.
        </span>
      </Dialog.Description>

      <div class="flex items-center gap-2 justify-end">
        <button
          type="button"
          class="px-3 py-1.5 text-sm rounded-md border border-border bg-background text-foreground hover:bg-accent transition-colors"
          onclick={() => forcePushConfirmOpen.set(false)}
        >
          Cancel
        </button>
        <button
          type="button"
          class="px-3 py-1.5 text-sm rounded-md bg-destructive text-destructive-foreground hover:bg-destructive/90 transition-colors font-medium"
          disabled={!!$networkOperation}
          onclick={handleForcePush}
        >
          Force Push
        </button>
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
