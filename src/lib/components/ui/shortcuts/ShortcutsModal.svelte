<script lang="ts">
  import { Dialog } from 'bits-ui';

  let { open = $bindable(false) }: { open: boolean } = $props();

  const sections = [
    {
      title: 'General',
      shortcuts: [
        { keys: ['Cmd', 'K'], description: 'Open command bar' },
        { keys: ['?'], description: 'Show keyboard shortcuts' },
        { keys: ['Escape'], description: 'Close panel / clear selection' },
      ],
    },
    {
      title: 'Navigation',
      shortcuts: [
        { keys: ['\u2191'], description: 'Previous commit / file' },
        { keys: ['\u2193'], description: 'Next commit / file' },
      ],
    },
    {
      title: 'Staging',
      shortcuts: [
        { keys: ['S'], description: 'Stage selected file' },
        { keys: ['U'], description: 'Unstage selected file' },
      ],
    },
    {
      title: 'Commit',
      shortcuts: [
        { keys: ['Cmd', 'Enter'], description: 'Commit staged changes' },
      ],
    },
  ];
</script>

<Dialog.Root bind:open>
  <Dialog.Portal>
    <Dialog.Overlay
      class="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm"
    />
    <Dialog.Content
      class="fixed left-1/2 top-1/2 z-50 w-full max-w-md -translate-x-1/2 -translate-y-1/2 rounded-lg border border-border bg-card p-6 shadow-lg"
    >
      <div class="flex items-center justify-between mb-4">
        <Dialog.Title class="text-base font-semibold text-foreground">
          Keyboard Shortcuts
        </Dialog.Title>
        <Dialog.Close
          class="rounded-md p-1 text-muted-foreground hover:text-foreground transition-colors"
        >
          <svg viewBox="0 0 16 16" width="16" height="16">
            <path fill="currentColor" d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z" />
          </svg>
          <span class="sr-only">Close</span>
        </Dialog.Close>
      </div>

      <Dialog.Description class="sr-only">
        A list of all keyboard shortcuts available in Gitron.
      </Dialog.Description>

      <div class="space-y-4 max-h-[60vh] overflow-y-auto">
        {#each sections as section}
          <div>
            <h3 class="text-xs font-medium text-muted-foreground uppercase tracking-wide mb-2">
              {section.title}
            </h3>
            <div class="space-y-1.5">
              {#each section.shortcuts as shortcut}
                <div class="flex items-center justify-between py-1">
                  <span class="text-sm text-foreground">{shortcut.description}</span>
                  <div class="flex items-center gap-1">
                    {#each shortcut.keys as key}
                      <kbd class="inline-flex items-center justify-center min-w-[24px] h-6 px-1.5 rounded border border-border bg-secondary text-xs font-mono text-muted-foreground">
                        {key}
                      </kbd>
                    {/each}
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/each}
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
