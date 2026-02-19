<script lang="ts">
  import { Dialog } from 'bits-ui';
  import {
    deleteBranchConfirm,
    deleteBranch,
    deleteRemoteBranch,
  } from '$lib/stores/repo';

  function handleConfirm() {
    const info = $deleteBranchConfirm;
    if (info.isRemote) {
      // Parse "origin/feature" into remote="origin", branch="feature"
      const slashIdx = info.branchName.indexOf('/');
      if (slashIdx > 0) {
        const remote = info.branchName.substring(0, slashIdx);
        const branch = info.branchName.substring(slashIdx + 1);
        deleteRemoteBranch(remote, branch);
      }
    } else {
      deleteBranch(info.branchName);
    }
  }
</script>

<Dialog.Root
  open={$deleteBranchConfirm.open}
  onOpenChange={(open) => {
    if (!open) deleteBranchConfirm.set({ open: false, branchName: '', isRemote: false });
  }}
>
  <Dialog.Portal>
    <Dialog.Overlay class="fixed inset-0 bg-black/50 z-50" />
    <Dialog.Content
      class="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 z-50 w-full max-w-sm rounded-lg border border-border bg-card p-6 shadow-lg"
    >
      {#if $deleteBranchConfirm.isRemote}
        <Dialog.Title class="text-sm font-semibold text-foreground mb-1">
          Delete remote branch?
        </Dialog.Title>
        <Dialog.Description class="text-sm text-muted-foreground mb-5">
          This will delete <strong class="text-foreground font-mono">{$deleteBranchConfirm.branchName}</strong> from the remote server.
          <span class="block mt-2 text-destructive text-xs">
            This will affect all collaborators. This cannot be undone.
          </span>
        </Dialog.Description>
      {:else}
        <Dialog.Title class="text-sm font-semibold text-foreground mb-1">
          Delete local branch?
        </Dialog.Title>
        <Dialog.Description class="text-sm text-muted-foreground mb-5">
          This will delete the local branch <strong class="text-foreground font-mono">{$deleteBranchConfirm.branchName}</strong>.
          <span class="block mt-2 text-muted-foreground text-xs">
            Unmerged commits on this branch may become unreachable.
          </span>
        </Dialog.Description>
      {/if}

      <div class="flex items-center gap-2 justify-end">
        <button
          type="button"
          class="px-3 py-1.5 text-sm rounded-md border border-border bg-background text-foreground hover:bg-accent transition-colors"
          onclick={() => deleteBranchConfirm.set({ open: false, branchName: '', isRemote: false })}
        >
          Cancel
        </button>
        <button
          type="button"
          class="px-3 py-1.5 text-sm rounded-md bg-destructive text-destructive-foreground hover:bg-destructive/90 transition-colors font-medium"
          onclick={handleConfirm}
        >
          Delete
        </button>
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
