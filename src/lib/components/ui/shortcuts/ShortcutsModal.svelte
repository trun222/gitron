<script lang="ts">
  let { open = $bindable(false) }: { open: boolean } = $props();

  const sections = [
    {
      title: 'General',
      shortcuts: [
        { keys: ['Cmd', 'K'], description: 'Open command bar' },
        { keys: ['Cmd', 'R'], description: 'Refresh repository' },
        { keys: ['Cmd', '`'], description: 'Toggle output panel' },
        { keys: ['?'], description: 'Show keyboard shortcuts' },
        { keys: ['Cmd', ','], description: 'Open settings' },
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
        { keys: ['Cmd', 'Shift', 'A'], description: 'Stage all files' },
        { keys: ['Cmd', 'Shift', 'U'], description: 'Unstage all files' },
        { keys: ['Cmd', 'Shift', 'D'], description: 'Discard all changes' },
      ],
    },
    {
      title: 'Commit & Sync',
      shortcuts: [
        { keys: ['Cmd', 'Enter'], description: 'Commit staged changes' },
        { keys: ['Cmd', 'Shift', 'P'], description: 'Push' },
        { keys: ['Cmd', 'Shift', 'L'], description: 'Pull' },
        { keys: ['Cmd', 'Shift', 'F'], description: 'Fetch' },
      ],
    },
  ];

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) open = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') open = false;
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="overlay"
    onclick={handleBackdropClick}
    onkeydown={handleKeydown}
  >
    <div class="modal" role="dialog" aria-label="Keyboard Shortcuts">
      <div class="header">
        <h2 class="title">Keyboard Shortcuts</h2>
        <button class="close-btn" onclick={() => open = false}>
          <svg viewBox="0 0 16 16" width="16" height="16">
            <path fill="currentColor" d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z" />
          </svg>
        </button>
      </div>

      <div class="scroll-area">
        {#each sections as section}
          <div class="section">
            <h3 class="section-title">{section.title}</h3>
            {#each section.shortcuts as shortcut}
              <div class="shortcut-row">
                <span class="description">{shortcut.description}</span>
                <div class="keys">
                  {#each shortcut.keys as key}
                    <kbd class="key">{key}</kbd>
                  {/each}
                </div>
              </div>
            {/each}
          </div>
        {/each}
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 50;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(4px);
  }

  .modal {
    width: 520px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    border-radius: 12px;
    border: 1px solid var(--border);
    background: var(--card);
    padding: 24px;
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
    flex-shrink: 0;
  }

  .title {
    font-size: 16px;
    font-weight: 600;
    color: var(--foreground);
  }

  .close-btn {
    padding: 4px;
    border-radius: 6px;
    color: var(--muted-foreground);
    cursor: pointer;
    transition: color 0.15s;
  }
  .close-btn:hover {
    color: var(--foreground);
  }

  .scroll-area {
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding-right: 8px;
  }

  .section-title {
    font-size: 11px;
    font-weight: 500;
    color: var(--muted-foreground);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 8px;
  }

  .shortcut-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 5px 0;
  }

  .description {
    font-size: 13px;
    color: var(--foreground);
    flex: 1;
    min-width: 0;
  }

  .keys {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .key {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 24px;
    height: 24px;
    padding: 0 6px;
    border-radius: 5px;
    border: 1px solid var(--border);
    background: var(--secondary);
    font-size: 11px;
    font-family: ui-monospace, monospace;
    color: var(--muted-foreground);
  }
</style>
