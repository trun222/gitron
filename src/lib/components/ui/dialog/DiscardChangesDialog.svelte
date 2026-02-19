<script lang="ts">
  import { Dialog } from 'bits-ui';
  import {
    discardConfirmOpen,
    discardAllChanges,
    stagedCount,
    unstagedCount,
  } from '$lib/stores/repo';
</script>

<Dialog.Root
  open={$discardConfirmOpen}
  onOpenChange={(open) => discardConfirmOpen.set(open)}
>
  <Dialog.Portal>
    <Dialog.Overlay class="fixed inset-0 bg-black/50 z-50" />
    <Dialog.Content
      class="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 z-50 w-full max-w-sm rounded-lg border border-border bg-card p-6 shadow-lg"
    >
      <Dialog.Title class="text-sm font-semibold text-foreground mb-1">
        Discard all changes?
      </Dialog.Title>
      <Dialog.Description class="text-sm text-muted-foreground mb-5">
        This will permanently discard <strong class="text-foreground">all</strong> staged, unstaged, and untracked changes.
        {#if $stagedCount + $unstagedCount > 0}
          <span class="block mt-2 text-destructive text-xs">
            {$stagedCount + $unstagedCount} file(s) will be lost. This cannot be undone.
          </span>
        {/if}
      </Dialog.Description>

      <div class="flex items-center gap-2 justify-end">
        <button
          type="button"
          class="px-3 py-1.5 text-sm rounded-md border border-border bg-background text-foreground hover:bg-accent transition-colors"
          onclick={() => discardConfirmOpen.set(false)}
        >
          Cancel
        </button>
        <button
          type="button"
          class="px-3 py-1.5 text-sm rounded-md bg-destructive text-destructive-foreground hover:bg-destructive/90 transition-colors font-medium"
          onclick={() => discardAllChanges()}
        >
          Discard All
        </button>
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
