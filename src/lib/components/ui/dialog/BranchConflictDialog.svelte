<script lang="ts">
  import { Dialog } from 'bits-ui';
  import {
    branchConflictPrompt,
    resetLocalToRemote,
    checkoutLocalInstead,
    dismissBranchConflict,
  } from '$lib/stores/repo';

  let isOpen = $derived($branchConflictPrompt !== null);
</script>

<Dialog.Root
  open={isOpen}
  onOpenChange={(open) => { if (!open) dismissBranchConflict(); }}
>
  <Dialog.Portal>
    <Dialog.Overlay class="fixed inset-0 bg-black/50 z-50" />
    <Dialog.Content
      class="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 z-50 w-full max-w-md rounded-lg border border-border bg-card p-6 shadow-lg"
    >
      {#if $branchConflictPrompt}
        <Dialog.Title class="text-sm font-semibold text-foreground mb-1">
          Local branch already exists
        </Dialog.Title>
        <Dialog.Description class="text-sm text-muted-foreground mb-5">
          A local <strong class="text-foreground">'{$branchConflictPrompt.localName}'</strong> already exists.
          It may differ from <strong class="text-foreground">{$branchConflictPrompt.remoteBranchName}</strong>.
        </Dialog.Description>

        <div class="flex items-center gap-2 justify-end">
          <button
            type="button"
            class="px-3 py-1.5 text-sm rounded-md border border-border bg-background text-foreground hover:bg-accent transition-colors"
            onclick={() => dismissBranchConflict()}
          >
            Cancel
          </button>
          <button
            type="button"
            class="px-3 py-1.5 text-sm rounded-md border border-border bg-background text-foreground hover:bg-accent transition-colors"
            onclick={() => checkoutLocalInstead($branchConflictPrompt!.localName)}
          >
            Checkout Local
          </button>
          <button
            type="button"
            class="px-3 py-1.5 text-sm rounded-md bg-destructive text-destructive-foreground hover:bg-destructive/90 transition-colors font-medium"
            onclick={() => resetLocalToRemote($branchConflictPrompt!.remoteBranchName)}
          >
            Reset Local to Remote
          </button>
        </div>
      {/if}
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
