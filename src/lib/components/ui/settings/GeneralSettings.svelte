<script lang="ts">
  import {
    theme, setTheme,
    sidebarCollapsed, toggleSidebar,
    autoShowOutput, setAutoShowOutput,
    zoomLevel, setZoomLevel,
    highContrast, setHighContrast,
    editorFontSize, setEditorFontSize,
    monoFont, setMonoFont,
    showTagsList, setShowTagsList,
    showWorktreesList, setShowWorktreesList,
    treeExpandedByDefault, setTreeExpandedByDefault,
  } from '$lib/stores/settings';
  import type { ThemeMode, ZoomLevel, EditorFontSize, MonoFont } from '$lib/api/types';
  import { Select } from '$lib/components/ui/select';

  const themeOptions: { value: ThemeMode; label: string }[] = [
    { value: 'tron', label: 'Tron' },
    { value: 'tron-enhanced', label: 'Tron (Enhanced)' },
    { value: 'synthwave', label: 'Synthwave' },
    { value: 'aurora', label: 'Aurora' },
    { value: 'dark', label: 'Dark' },
    { value: 'light', label: 'Light' },
    { value: 'system', label: 'System' },
  ];

  const zoomOptions: { value: ZoomLevel; label: string }[] = [
    { value: 0.8, label: '80%' },
    { value: 0.9, label: '90%' },
    { value: 1.0, label: '100%' },
    { value: 1.1, label: '110%' },
    { value: 1.25, label: '125%' },
    { value: 1.5, label: '150%' },
  ];

  const fontSizeOptions: { value: EditorFontSize; label: string }[] = [
    { value: 12, label: '12px' },
    { value: 13, label: '13px' },
    { value: 14, label: '14px' },
    { value: 16, label: '16px' },
  ];

  const monoFontOptions: { value: MonoFont; label: string }[] = [
    { value: 'default', label: 'Default' },
    { value: 'fira-code', label: 'Fira Code' },
    { value: 'jetbrains-mono', label: 'JetBrains Mono' },
    { value: 'cascadia-code', label: 'Cascadia Code' },
    { value: 'sf-mono', label: 'SF Mono' },
    { value: 'menlo', label: 'Menlo' },
  ];
</script>

<div class="section">
  <h3 class="section-title">Appearance</h3>
  <div class="setting-row">
    <div class="setting-label">
      <span class="label-text">Theme</span>
      <span class="label-description">Choose the application color scheme</span>
    </div>
    <Select
      value={$theme}
      options={themeOptions}
      onchange={(v) => setTheme(v)}
    />
  </div>
  <div class="setting-row">
    <div class="setting-label">
      <span class="label-text">UI Zoom</span>
      <span class="label-description">Scale the entire interface up or down</span>
    </div>
    <Select
      value={$zoomLevel}
      options={zoomOptions}
      onchange={(v) => setZoomLevel(v)}
    />
  </div>
  <div class="setting-row">
    <div class="setting-label">
      <span class="label-text">High Contrast</span>
      <span class="label-description">Increase contrast for muted text and borders</span>
    </div>
    <button
      class="toggle"
      class:on={$highContrast}
      onclick={() => setHighContrast(!$highContrast)}
      role="switch"
      aria-checked={$highContrast}
      aria-label="Toggle high contrast"
    >
      <span class="toggle-thumb"></span>
    </button>
  </div>
</div>

<div class="section">
  <h3 class="section-title">Editor</h3>
  <div class="setting-row">
    <div class="setting-label">
      <span class="label-text">Font Size</span>
      <span class="label-description">Font size for diff viewer and output panel</span>
    </div>
    <Select
      value={$editorFontSize}
      options={fontSizeOptions}
      onchange={(v) => setEditorFontSize(v)}
    />
  </div>
  <div class="setting-row">
    <div class="setting-label">
      <span class="label-text">Monospace Font</span>
      <span class="label-description">Font family for code and diffs</span>
    </div>
    <Select
      value={$monoFont}
      options={monoFontOptions}
      onchange={(v) => setMonoFont(v)}
    />
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
  <div class="setting-row">
    <div class="setting-label">
      <span class="label-text">Auto-show output panel</span>
      <span class="label-description">Automatically open the output panel when operations produce output</span>
    </div>
    <button
      class="toggle"
      class:on={$autoShowOutput}
      onclick={() => setAutoShowOutput(!$autoShowOutput)}
      role="switch"
      aria-checked={$autoShowOutput}
      aria-label="Toggle auto-show output panel"
    >
      <span class="toggle-thumb"></span>
    </button>
  </div>
  <div class="setting-row">
    <div class="setting-label">
      <span class="label-text">Show tags list</span>
      <span class="label-description">Show a tags section at the bottom of the changes panel</span>
    </div>
    <button
      class="toggle"
      class:on={$showTagsList}
      onclick={() => setShowTagsList(!$showTagsList)}
      role="switch"
      aria-checked={$showTagsList}
      aria-label="Toggle show tags list"
    >
      <span class="toggle-thumb"></span>
    </button>
  </div>
  <div class="setting-row">
    <div class="setting-label">
      <span class="label-text">Show worktrees panel</span>
      <span class="label-description">Show a worktrees section at the bottom of the sidebar</span>
    </div>
    <button
      class="toggle"
      class:on={$showWorktreesList}
      onclick={() => setShowWorktreesList(!$showWorktreesList)}
      role="switch"
      aria-checked={$showWorktreesList}
      aria-label="Toggle show worktrees panel"
    >
      <span class="toggle-thumb"></span>
    </button>
  </div>
  <div class="setting-row">
    <div class="setting-label">
      <span class="label-text">Expand tree by default</span>
      <span class="label-description">Automatically expand all folders in the changes tree view</span>
    </div>
    <button
      class="toggle"
      class:on={$treeExpandedByDefault}
      onclick={() => setTreeExpandedByDefault(!$treeExpandedByDefault)}
      role="switch"
      aria-checked={$treeExpandedByDefault}
      aria-label="Toggle tree expanded by default"
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
