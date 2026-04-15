<script lang="ts" generics="T extends string | number">
  type Option = { value: T; label: string };

  type Props = {
    value: T;
    options: Option[];
    onchange: (value: T) => void;
    placeholder?: string;
    disabled?: boolean;
    minWidth?: string;
    align?: 'left' | 'right';
  };

  let {
    value,
    options,
    onchange,
    placeholder = 'Select...',
    disabled = false,
    minWidth = '140px',
    align = 'right',
  }: Props = $props();

  let open = $state(false);
  let triggerEl: HTMLButtonElement | undefined = $state();
  let menuEl: HTMLDivElement | undefined = $state();
  let highlightIndex = $state(-1);

  let selected = $derived(options.find((o) => o.value === value));
  let displayLabel = $derived(selected?.label ?? placeholder);

  function toggle() {
    if (disabled) return;
    open = !open;
    if (open) {
      highlightIndex = options.findIndex((o) => o.value === value);
      queueMicrotask(() => menuEl?.focus());
    }
  }

  function selectOption(opt: Option) {
    onchange(opt.value);
    open = false;
    triggerEl?.focus();
  }

  function handleWindowClick(e: MouseEvent) {
    if (!open) return;
    const target = e.target as Node | null;
    if (target && (triggerEl?.contains(target) || menuEl?.contains(target))) return;
    open = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!open) {
      if (e.key === 'Enter' || e.key === ' ' || e.key === 'ArrowDown') {
        e.preventDefault();
        toggle();
      }
      return;
    }
    if (e.key === 'Escape') {
      e.preventDefault();
      open = false;
      triggerEl?.focus();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      highlightIndex = (highlightIndex + 1) % options.length;
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      highlightIndex = (highlightIndex - 1 + options.length) % options.length;
    } else if (e.key === 'Enter') {
      e.preventDefault();
      const opt = options[highlightIndex];
      if (opt) selectOption(opt);
    } else if (e.key === 'Home') {
      e.preventDefault();
      highlightIndex = 0;
    } else if (e.key === 'End') {
      e.preventDefault();
      highlightIndex = options.length - 1;
    }
  }
</script>

<svelte:window onclick={handleWindowClick} />

<div class="select-wrapper" style="min-width: {minWidth};">
  <button
    bind:this={triggerEl}
    type="button"
    class="select-trigger"
    class:open
    {disabled}
    aria-haspopup="listbox"
    aria-expanded={open}
    onclick={toggle}
    onkeydown={handleKeydown}
  >
    <span class="select-value" class:placeholder={!selected}>{displayLabel}</span>
    <svg
      class="select-chevron"
      class:open
      viewBox="0 0 16 16"
      width="10"
      height="10"
      aria-hidden="true"
    >
      <path
        fill="currentColor"
        d="M4.427 7.427l3.396 3.396a.25.25 0 0 0 .354 0l3.396-3.396A.25.25 0 0 0 11.396 7H4.604a.25.25 0 0 0-.177.427Z"
      />
    </svg>
  </button>

  {#if open}
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <div
      bind:this={menuEl}
      class="select-menu"
      class:align-left={align === 'left'}
      role="listbox"
      tabindex="-1"
      onkeydown={handleKeydown}
    >
      {#each options as opt, i (opt.value)}
        <button
          type="button"
          class="select-option"
          class:selected={opt.value === value}
          class:highlighted={i === highlightIndex}
          role="option"
          aria-selected={opt.value === value}
          onclick={() => selectOption(opt)}
          onmouseenter={() => (highlightIndex = i)}
        >
          <span class="option-label">{opt.label}</span>
          {#if opt.value === value}
            <svg viewBox="0 0 16 16" width="12" height="12" aria-hidden="true">
              <path
                fill="currentColor"
                d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.75.75 0 1 1 1.06-1.06L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"
              />
            </svg>
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .select-wrapper {
    position: relative;
    flex-shrink: 0;
  }

  .select-trigger {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    width: 100%;
    padding: 4px 8px;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--secondary);
    color: var(--foreground);
    font-size: 12px;
    cursor: pointer;
    transition: border-color 0.15s, background 0.15s;
  }
  .select-trigger:hover:not(:disabled) {
    border-color: var(--primary);
  }
  .select-trigger:focus-visible {
    outline: none;
    border-color: var(--primary);
    box-shadow: 0 0 0 2px var(--ring, var(--primary));
  }
  .select-trigger.open {
    border-color: var(--primary);
  }
  .select-trigger:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .select-value {
    flex: 1;
    text-align: left;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .select-value.placeholder {
    color: var(--muted-foreground);
  }

  .select-chevron {
    flex-shrink: 0;
    color: var(--muted-foreground);
    transition: transform 0.15s;
  }
  .select-chevron.open {
    transform: rotate(180deg);
  }

  .select-menu {
    position: absolute;
    top: calc(100% + 4px);
    right: 0;
    z-index: 50;
    min-width: 100%;
    max-height: 280px;
    overflow-y: auto;
    background: var(--popover, var(--secondary));
    color: var(--popover-foreground, var(--foreground));
    border: 1px solid var(--border);
    border-radius: 6px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.25);
    padding: 4px;
    outline: none;
  }
  .select-menu.align-left {
    right: auto;
    left: 0;
  }

  .select-option {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    width: 100%;
    padding: 6px 8px;
    border-radius: 4px;
    background: transparent;
    color: var(--foreground);
    font-size: 12px;
    cursor: pointer;
    text-align: left;
    border: none;
    white-space: nowrap;
  }
  .select-option.highlighted {
    background: var(--accent);
    color: var(--accent-foreground, var(--foreground));
  }
  .select-option.selected {
    color: var(--primary);
  }
  .select-option.selected.highlighted {
    color: var(--accent-foreground, var(--foreground));
  }

  .option-label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
