<script lang="ts">
  import { autoFetchInterval, setAutoFetchInterval } from '$lib/stores/settings';
  import type { AutoFetchInterval } from '$lib/api/types';

  const fetchOptions: { value: AutoFetchInterval; label: string }[] = [
    { value: 0, label: 'Off' },
    { value: 60, label: '1 minute' },
    { value: 300, label: '5 minutes' },
    { value: 900, label: '15 minutes' },
  ];
</script>

<div class="section">
  <h3 class="section-title">Fetch</h3>
  <div class="setting-row">
    <div class="setting-label">
      <span class="label-text">Auto-fetch interval</span>
      <span class="label-description">Automatically fetch from remotes at a regular interval</span>
    </div>
    <select
      class="select-input"
      value={$autoFetchInterval}
      onchange={(e) => setAutoFetchInterval(Number((e.target as HTMLSelectElement).value) as AutoFetchInterval)}
    >
      {#each fetchOptions as opt (opt.value)}
        <option value={opt.value}>{opt.label}</option>
      {/each}
    </select>
  </div>
</div>

<style>
  .section {
    margin-bottom: 24px;
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
</style>
