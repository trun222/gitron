<script lang="ts">
  import GeneralSettings from './GeneralSettings.svelte';
  import GitSettings from './GitSettings.svelte';
  import GitHubSettings from './GitHubSettings.svelte';
  import AISettings from './AISettings.svelte';

  let { open = $bindable(false) }: { open: boolean } = $props();

  type Section = 'general' | 'git' | 'github' | 'ai';
  let activeSection = $state<Section>('general');

  const sections: { id: Section; label: string; icon: string }[] = [
    { id: 'general', label: 'General', icon: 'general' },
    { id: 'git', label: 'Git', icon: 'git' },
    { id: 'github', label: 'GitHub', icon: 'github' },
    { id: 'ai', label: 'AI', icon: 'ai' },
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
    <div class="modal" role="dialog" aria-label="Settings">
      <div class="modal-layout">
        <nav class="sidebar">
          <h2 class="sidebar-title">Settings</h2>
          {#each sections as section (section.id)}
            <button
              class="nav-item"
              class:active={activeSection === section.id}
              onclick={() => activeSection = section.id}
            >
              {#if section.icon === 'general'}
                <svg viewBox="0 0 16 16" width="14" height="14">
                  <path fill="currentColor" d="M8 0a8.2 8.2 0 0 1 .701.031C9.444.095 9.99.645 10.16 1.29l.288 1.107c.018.066.079.158.212.224.231.114.454.243.668.386.123.082.233.09.3.071L12.727 2.8c.63-.186 1.345.018 1.737.631A8 8 0 0 1 15.63 5.57c.174.647-.134 1.318-.683 1.637l-.97.567a.4.4 0 0 0-.177.288 6.8 6.8 0 0 1 0 .776c.005.09.076.186.177.288l.97.567c.549.319.857.99.683 1.637a8 8 0 0 1-1.166 2.14c-.392.612-1.108.816-1.737.63l-1.1-.278c-.066-.018-.177-.011-.3.071a5.7 5.7 0 0 1-.668.386c-.133.066-.194.158-.212.224l-.288 1.107c-.17.645-.716 1.195-1.459 1.26a8.1 8.1 0 0 1-1.402 0c-.743-.065-1.289-.615-1.459-1.26l-.288-1.107a.37.37 0 0 0-.212-.224 5.7 5.7 0 0 1-.668-.386c-.123-.082-.233-.09-.3-.071l-1.1.278c-.63.186-1.345-.018-1.737-.631A8 8 0 0 1 .37 10.43c-.174-.647.134-1.318.683-1.637l.97-.567c.1-.102.171-.198.177-.288a6.8 6.8 0 0 1 0-.776.4.4 0 0 0-.177-.288l-.97-.567C.504 5.988.196 5.317.37 4.67a8 8 0 0 1 1.166-2.14c.392-.612 1.108-.816 1.737-.63l1.1.278c.066.018.177.011.3-.071.214-.143.437-.272.668-.386a.37.37 0 0 0 .212-.224l.288-1.107C6.01.645 6.556.095 7.299.03 7.53.01 7.764 0 8 0Zm-.571 1.525c-.036.003-.108.036-.137.146l-.289 1.105c-.147.56-.55.967-.997 1.189a4.2 4.2 0 0 0-.488.282c-.4.266-.881.395-1.437.223l-1.1-.278c-.11-.03-.175.016-.195.046a6.5 6.5 0 0 0-.9 1.652c-.03.11.004.186.077.228l.97.567c.477.278.79.754.816 1.297.015.315.015.635 0 .95-.027.543-.34 1.019-.816 1.297l-.97.567c-.073.042-.107.118-.077.228.203.724.508 1.305.9 1.652.02.03.085.076.195.046l1.1-.278c.556-.172 1.037-.043 1.437.223.155.104.318.197.488.283.448.222.85.629.997 1.189l.289 1.105c.029.109.101.143.137.146a6.6 6.6 0 0 0 1.142 0c.036-.003.108-.036.137-.146l.289-1.105c.147-.56.55-.967.997-1.189.17-.086.333-.179.488-.283.4-.266.881-.395 1.437-.223l1.1.278c.11.03.175-.016.195-.046.392-.347.697-.928.9-1.652.03-.11-.004-.186-.077-.228l-.97-.567c-.477-.278-.79-.754-.816-1.297a5.3 5.3 0 0 1 0-.95c.027-.543.34-1.019.816-1.297l.97-.567c.073-.042.107-.118.077-.228a6.5 6.5 0 0 0-.9-1.652c-.02-.03-.085-.076-.195-.046l-1.1.278c-.556.172-1.037.043-1.437-.223a4.2 4.2 0 0 0-.488-.282c-.448-.222-.85-.629-.997-1.189l-.289-1.105c-.029-.11-.101-.143-.137-.146a6.6 6.6 0 0 0-1.142 0ZM11 8a3 3 0 1 1-6 0 3 3 0 0 1 6 0ZM9.5 8a1.5 1.5 0 1 0-3.001.001A1.5 1.5 0 0 0 9.5 8Z" />
                </svg>
              {:else if section.icon === 'git'}
                <svg viewBox="0 0 16 16" width="14" height="14">
                  <path fill="currentColor" d="M9.5 3.25a2.25 2.25 0 1 1 3 2.122V6A2.5 2.5 0 0 1 10 8.5H6a1 1 0 0 0-1 1v1.128a2.251 2.251 0 1 1-1.5 0V5.372a2.25 2.25 0 1 1 1.5 0v1.836A2.5 2.5 0 0 1 6 7h4a1 1 0 0 0 1-1v-.628A2.25 2.25 0 0 1 9.5 3.25Z" />
                </svg>
              {:else if section.icon === 'github'}
                <svg viewBox="0 0 16 16" width="14" height="14">
                  <path fill="currentColor" d="M8 0c4.42 0 8 3.58 8 8a8.013 8.013 0 0 1-5.45 7.59c-.4.08-.55-.17-.55-.38 0-.27.01-1.13.01-2.2 0-.75-.25-1.23-.54-1.48 1.78-.2 3.65-.88 3.65-3.95 0-.88-.31-1.59-.82-2.15.08-.2.36-1.02-.08-2.12 0 0-.67-.22-2.2.82-.64-.18-1.32-.27-2-.27-.68 0-1.36.09-2 .27-1.53-1.03-2.2-.82-2.2-.82-.44 1.1-.16 1.92-.08 2.12-.51.56-.82 1.28-.82 2.15 0 3.06 1.86 3.75 3.64 3.95-.23.2-.44.55-.51 1.07-.46.21-1.61.55-2.33-.66-.15-.24-.6-.83-1.23-.82-.67.01-.27.38.01.53.34.19.73.9.82 1.13.16.45.68 1.31 2.69.94 0 .67.01 1.3.01 1.49 0 .21-.15.45-.55.38A7.995 7.995 0 0 1 0 8c0-4.42 3.58-8 8-8Z" />
                </svg>
              {:else}
                <!-- AI sparkle icon -->
                <svg viewBox="0 0 16 16" width="14" height="14">
                  <path fill="currentColor" d="M7.657 6.247c.11-.33.576-.33.686 0l.645 1.937a2.89 2.89 0 0 0 1.829 1.828l1.936.645c.33.11.33.576 0 .686l-1.937.645a2.89 2.89 0 0 0-1.828 1.829l-.645 1.936a.361.361 0 0 1-.686 0l-.645-1.937a2.89 2.89 0 0 0-1.828-1.828l-1.937-.645a.361.361 0 0 1 0-.686l1.937-.645a2.89 2.89 0 0 0 1.828-1.829l.645-1.936ZM3.794 1.148a.217.217 0 0 1 .412 0l.387 1.162c.173.518.579.924 1.097 1.097l1.162.387a.217.217 0 0 1 0 .412l-1.162.387A1.73 1.73 0 0 0 4.593 5.69l-.387 1.162a.217.217 0 0 1-.412 0L3.407 5.69a1.73 1.73 0 0 0-1.097-1.097l-1.162-.387a.217.217 0 0 1 0-.412l1.162-.387A1.73 1.73 0 0 0 3.407 2.31l.387-1.162Z" />
                </svg>
              {/if}
              {section.label}
            </button>
          {/each}
        </nav>

        <div class="content">
          <div class="content-header">
            <h2 class="content-title">
              {sections.find((s) => s.id === activeSection)?.label}
            </h2>
            <button class="close-btn" onclick={() => open = false} aria-label="Close settings">
              <svg viewBox="0 0 16 16" width="16" height="16">
                <path fill="currentColor" d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z" />
              </svg>
            </button>
          </div>
          <div class="scroll-area">
            {#if activeSection === 'general'}
              <GeneralSettings />
            {:else if activeSection === 'git'}
              <GitSettings />
            {:else if activeSection === 'github'}
              <GitHubSettings />
            {:else if activeSection === 'ai'}
              <AISettings />
            {/if}
          </div>
        </div>
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
    width: 720px;
    height: 520px;
    display: flex;
    flex-direction: column;
    border-radius: 12px;
    border: 1px solid var(--border);
    background: var(--card);
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
    overflow: hidden;
  }

  .modal-layout {
    display: flex;
    flex: 1;
    min-height: 0;
  }

  .sidebar {
    width: 180px;
    flex-shrink: 0;
    padding: 16px;
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .sidebar-title {
    font-size: 11px;
    font-weight: 500;
    color: var(--muted-foreground);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 8px;
    padding: 0 8px;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    border-radius: 6px;
    font-size: 13px;
    color: var(--muted-foreground);
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
    text-align: left;
  }
  .nav-item:hover {
    background: var(--accent);
    color: var(--foreground);
  }
  .nav-item.active {
    background: var(--accent);
    color: var(--foreground);
    font-weight: 500;
  }

  .content {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .content-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 24px;
    flex-shrink: 0;
    border-bottom: 1px solid var(--border);
  }

  .content-title {
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
    flex: 1;
    overflow-y: auto;
    padding: 24px;
  }
</style>
