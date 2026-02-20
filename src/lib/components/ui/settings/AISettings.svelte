<script lang="ts">
  import {
    aiProviders,
    aiSettings,
    aiFetchingModels,
    loadAIProviders,
    loadAISettings,
    saveAIKey,
    deleteAIKey,
    setSelectedProvider,
    setSelectedModel,
    setCustomBaseUrl,
    fetchModelsForProvider,
  } from '$lib/stores/ai';
  import type { AIProvider } from '$lib/api/types';

  let keyInputs = $state<Record<string, string>>({});
  let savingKey = $state<string | null>(null);
  let keyError = $state<string | null>(null);
  let showAdvanced = $state<Record<string, boolean>>({});

  // Load on mount
  $effect(() => {
    loadAIProviders();
    loadAISettings();
  });

  async function handleSaveKey(providerId: string) {
    const key = keyInputs[providerId]?.trim();
    if (!key) return;
    savingKey = providerId;
    keyError = null;
    try {
      await saveAIKey(providerId, key);
      keyInputs[providerId] = '';
    } catch (e) {
      keyError = String(e);
    } finally {
      savingKey = null;
    }
  }

  async function handleDeleteKey(providerId: string) {
    keyError = null;
    try {
      await deleteAIKey(providerId);
    } catch (e) {
      keyError = String(e);
    }
  }

  function handleProviderSelect(providerId: string) {
    const current = $aiSettings.selected_provider;
    if (current === providerId) {
      setSelectedProvider(null);
    } else {
      setSelectedProvider(providerId);
      // Auto-select first model
      const provider = $aiProviders.find((p: AIProvider) => p.id === providerId);
      if (provider?.models.length) {
        setSelectedModel(provider.models[0].id);
      }
    }
  }
</script>

<div class="section">
  <h3 class="section-title">AI Provider</h3>
  <p class="section-description">
    Configure an AI provider to generate commit messages from staged diffs. API keys are stored securely in your OS keychain.
  </p>

  <div class="provider-list">
    {#each $aiProviders as provider (provider.id)}
      <div
        class="provider-card"
        class:active={$aiSettings.selected_provider === provider.id}
      >
        <div class="provider-header">
          <button
            class="provider-select"
            onclick={() => handleProviderSelect(provider.id)}
          >
            <span class="radio" class:checked={$aiSettings.selected_provider === provider.id}></span>
            <span class="provider-name">{provider.name}</span>
          </button>
          {#if provider.has_key}
            <span class="key-badge">Key saved</span>
          {/if}
        </div>

        <div class="provider-body">
          {#if provider.has_key}
            <div class="key-row">
              <span class="key-masked">••••••••••••</span>
              <button
                class="delete-btn"
                onclick={() => handleDeleteKey(provider.id)}
              >
                Remove Key
              </button>
            </div>
          {:else}
            <div class="key-row">
              <input
                type="password"
                class="key-input"
                placeholder="Paste API key..."
                bind:value={keyInputs[provider.id]}
                onkeydown={(e) => { if (e.key === 'Enter') handleSaveKey(provider.id); }}
              />
              <button
                class="save-btn"
                onclick={() => handleSaveKey(provider.id)}
                disabled={!keyInputs[provider.id]?.trim() || savingKey === provider.id}
              >
                {savingKey === provider.id ? 'Saving...' : 'Save'}
              </button>
            </div>
          {/if}

          {#if $aiSettings.selected_provider === provider.id}
            <div class="model-row">
              <!-- svelte-ignore a11y_label_has_associated_control -->
              <label class="model-label">Model</label>
              {#if $aiFetchingModels}
                <span class="fetching-text">Loading models...</span>
              {:else}
                <select
                  class="select-input"
                  value={$aiSettings.selected_model ?? ''}
                  onchange={(e) => setSelectedModel((e.target as HTMLSelectElement).value || null)}
                >
                  <option value="">Select a model...</option>
                  {#each provider.models as model (model.id)}
                    <option value={model.id}>{model.name}</option>
                  {/each}
                </select>
                {#if provider.has_key}
                  <button
                    class="refresh-btn"
                    onclick={() => fetchModelsForProvider(provider.id)}
                    aria-label="Refresh model list"
                    title="Refresh model list"
                  >
                    <svg viewBox="0 0 16 16" width="12" height="12">
                      <path fill="currentColor" d="M1.705 8.005a.75.75 0 0 1 .834.656 5.5 5.5 0 0 0 9.592 2.97l-1.204-1.204a.25.25 0 0 1 .177-.427h3.646a.25.25 0 0 1 .25.25v3.646a.25.25 0 0 1-.427.177l-1.38-1.38A7.002 7.002 0 0 1 1.05 8.84a.75.75 0 0 1 .656-.834ZM8 2.5a5.487 5.487 0 0 0-4.131 1.869l1.204 1.204A.25.25 0 0 1 4.896 6H1.25A.25.25 0 0 1 1 5.75V2.104a.25.25 0 0 1 .427-.177l1.38 1.38A7.002 7.002 0 0 1 14.95 7.16a.75.75 0 0 1-1.49.178A5.5 5.5 0 0 0 8 2.5Z" />
                    </svg>
                  </button>
                {/if}
              {/if}
            </div>

            <div class="advanced-toggle">
              <button
                class="advanced-btn"
                onclick={() => showAdvanced[provider.id] = !showAdvanced[provider.id]}
              >
                {showAdvanced[provider.id] ? 'Hide' : 'Show'} advanced
                <svg
                  class="chevron"
                  class:open={showAdvanced[provider.id]}
                  viewBox="0 0 16 16"
                  width="10"
                  height="10"
                >
                  <path fill="currentColor" d="M4.427 7.427l3.396 3.396a.25.25 0 00.354 0l3.396-3.396A.25.25 0 0011.396 7H4.604a.25.25 0 00-.177.427z" />
                </svg>
              </button>
            </div>

            {#if showAdvanced[provider.id]}
              <div class="advanced-section">
                <!-- svelte-ignore a11y_label_has_associated_control -->
                <label class="model-label">Custom Base URL</label>
                <input
                  type="text"
                  class="key-input"
                  placeholder={provider.base_url ?? ''}
                  value={$aiSettings.custom_base_urls[provider.id] ?? ''}
                  onchange={(e) => setCustomBaseUrl(provider.id, (e.target as HTMLInputElement).value)}
                />
                <span class="hint">Leave empty to use default endpoint</span>
              </div>
            {/if}
          {/if}
        </div>
      </div>
    {/each}
  </div>

  {#if keyError}
    <p class="error">{keyError}</p>
  {/if}
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
    margin-bottom: 8px;
  }

  .section-description {
    font-size: 12px;
    color: var(--muted-foreground);
    margin-bottom: 16px;
    line-height: 1.5;
  }

  .provider-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .provider-card {
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--secondary);
    overflow: hidden;
    transition: border-color 0.15s;
  }
  .provider-card.active {
    border-color: var(--primary);
  }

  .provider-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px;
  }

  .provider-select {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    color: var(--foreground);
    font-size: 13px;
    font-weight: 500;
  }

  .radio {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    border: 2px solid var(--border);
    position: relative;
    flex-shrink: 0;
    transition: border-color 0.15s;
  }
  .radio.checked {
    border-color: var(--primary);
  }
  .radio.checked::after {
    content: '';
    position: absolute;
    top: 2px;
    left: 2px;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--primary);
  }

  .provider-name {
    color: var(--foreground);
  }

  .key-badge {
    font-size: 10px;
    padding: 2px 6px;
    border-radius: 4px;
    background: var(--primary);
    color: var(--primary-foreground);
    font-weight: 500;
  }

  .provider-body {
    padding: 0 12px 10px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .key-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .key-input {
    flex: 1;
    padding: 5px 8px;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--input);
    color: var(--foreground);
    font-size: 12px;
  }
  .key-input:focus {
    outline: none;
    border-color: var(--primary);
  }
  .key-input::placeholder {
    color: var(--muted-foreground);
  }

  .key-masked {
    font-size: 12px;
    color: var(--muted-foreground);
    flex: 1;
    letter-spacing: 2px;
  }

  .save-btn {
    padding: 4px 12px;
    border-radius: 6px;
    background: var(--primary);
    color: var(--primary-foreground);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    flex-shrink: 0;
    transition: opacity 0.15s;
  }
  .save-btn:hover:not(:disabled) {
    opacity: 0.9;
  }
  .save-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .delete-btn {
    padding: 4px 12px;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--muted-foreground);
    font-size: 12px;
    cursor: pointer;
    flex-shrink: 0;
    transition: background 0.15s, color 0.15s;
  }
  .delete-btn:hover {
    background: var(--destructive);
    color: white;
    border-color: var(--destructive);
  }

  .model-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .model-label {
    font-size: 12px;
    color: var(--muted-foreground);
    flex-shrink: 0;
  }

  .select-input {
    flex: 1;
    padding: 4px 8px;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--secondary);
    color: var(--foreground);
    font-size: 12px;
    cursor: pointer;
  }
  .select-input:focus {
    outline: none;
    border-color: var(--primary);
  }

  .advanced-toggle {
    display: flex;
  }

  .advanced-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: var(--muted-foreground);
    cursor: pointer;
    transition: color 0.15s;
  }
  .advanced-btn:hover {
    color: var(--foreground);
  }

  .chevron {
    transition: transform 0.15s;
  }
  .chevron.open {
    transform: rotate(180deg);
  }

  .advanced-section {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .hint {
    font-size: 10px;
    color: var(--muted-foreground);
  }

  .fetching-text {
    font-size: 12px;
    color: var(--muted-foreground);
    font-style: italic;
  }

  .refresh-btn {
    padding: 4px;
    border-radius: 4px;
    color: var(--muted-foreground);
    cursor: pointer;
    flex-shrink: 0;
    transition: color 0.15s;
  }
  .refresh-btn:hover {
    color: var(--foreground);
  }

  .error {
    font-size: 12px;
    color: var(--destructive);
    margin-top: 8px;
  }
</style>
