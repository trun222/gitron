<script lang="ts">
  import { theme, setTheme, sidebarCollapsed, toggleSidebar } from '$lib/stores/settings';
  import type { ThemeMode } from '$lib/api/types';

  const themeOptions: { value: ThemeMode; label: string }[] = [
    { value: 'tron', label: 'Tron' },
    { value: 'tron-enhanced', label: 'Tron (Enhanced)' },
    { value: 'dark', label: 'Dark' },
    { value: 'light', label: 'Light' },
    { value: 'system', label: 'System' },
  ];
</script>

<div class="section">
  <h3 class="section-title">Appearance</h3>
  <div class="setting-row">
    <div class="setting-label">
      <span class="label-text">Theme</span>
      <span class="label-description">Choose the application color scheme</span>
    </div>
    <select
      class="select-input"
      value={$theme}
      onchange={(e) => setTheme((e.target as HTMLSelectElement).value as ThemeMode)}
    >
      {#each themeOptions as opt (opt.value)}
        <option value={opt.value}>{opt.label}</option>
      {/each}
    </select>
  </div>
</div>

<div class="section">
  <h3 class="section-title">Layout</h3>
  <div class="setting-row">
    <div class="setting-label">
      <span class="label-text">Sidebar collapsed by default</span>
      <span class="label-description">Start with the sidebar collapsed when opening the app</span>
    </div>
    <button
      class="toggle"
      class:on={$sidebarCollapsed}
      onclick={() => toggleSidebar()}
      role="switch"
      aria-checked={$sidebarCollapsed}
      aria-label="Toggle sidebar collapsed"
    >
      <span class="toggle-thumb"></span>
    </button>
  </div>
</div>

<style>
  .section {
    margin-bottom: 24px;
  }
  .section:last-child {
    margin-bottom: 0;
  }

  .section-title {
    font-size: 11px;
    font-weight: 500;
    color: var(--muted-foreground);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 12px;
  }

  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 8px 0;
  }

  .setting-label {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .label-text {
    font-size: 13px;
    color: var(--foreground);
  }

  .label-description {
    font-size: 11px;
    color: var(--muted-foreground);
  }

  .select-input {
    padding: 4px 8px;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--secondary);
    color: var(--foreground);
    font-size: 12px;
    cursor: pointer;
    flex-shrink: 0;
  }
  .select-input:focus {
    outline: none;
    border-color: var(--primary);
  }

  .toggle {
    position: relative;
    width: 36px;
    height: 20px;
    border-radius: 10px;
    background: var(--input);
    cursor: pointer;
    transition: background 0.2s;
    flex-shrink: 0;
  }
  .toggle.on {
    background: var(--primary);
  }

  .toggle-thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--foreground);
    transition: transform 0.2s;
  }
  .toggle.on .toggle-thumb {
    transform: translateX(16px);
    background: var(--primary-foreground);
  }
</style>
