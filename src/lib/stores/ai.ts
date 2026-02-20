import { writable, derived, get } from 'svelte/store';
import type { AIProvider, AISettings, GenerateResult } from '$lib/api/types';
import * as aiApi from '$lib/api/ai';
import { repoPath } from '$lib/stores/repo';

// State
export const aiProviders = writable<AIProvider[]>([]);
export const aiSettings = writable<AISettings>({
  selected_provider: null,
  selected_model: null,
  selected_models: {},
  custom_base_urls: {},
});
export const aiGenerating = writable(false);
export const aiError = writable<string | null>(null);
export const aiFetchingModels = writable(false);

// Derived
export const hasConfiguredProvider = derived(
  [aiSettings, aiProviders],
  ([$settings, $providers]) => {
    if (!$settings.selected_provider) return false;
    const provider = $providers.find((p) => p.id === $settings.selected_provider);
    return provider?.has_key ?? false;
  }
);

export const selectedProviderModels = derived(
  [aiSettings, aiProviders],
  ([$settings, $providers]) => {
    if (!$settings.selected_provider) return [];
    const provider = $providers.find((p) => p.id === $settings.selected_provider);
    return provider?.models ?? [];
  }
);

// Actions

export async function loadAIProviders() {
  try {
    const providers = await aiApi.getProviders();
    aiProviders.set(providers);
  } catch (e) {
    console.error('Failed to load AI providers:', e);
  }
}

export async function loadAISettings() {
  try {
    const settings = await aiApi.getSettings();
    aiSettings.set(settings);
  } catch (e) {
    console.error('Failed to load AI settings:', e);
  }
}

/// Load providers + settings, then fetch live models for the configured provider.
/// Call this once on mount to restore full state.
export async function initAI() {
  await loadAIProviders();
  await loadAISettings();

  const settings = get(aiSettings);
  if (settings.selected_provider) {
    const providers = get(aiProviders);
    const provider = providers.find((p) => p.id === settings.selected_provider);
    if (provider?.has_key) {
      await fetchModelsForProvider(settings.selected_provider);
    }
  }
}

export async function fetchModelsForProvider(providerId: string) {
  const settings = get(aiSettings);
  const baseUrl = settings.custom_base_urls[providerId] ?? null;
  aiFetchingModels.set(true);
  try {
    const models = await aiApi.fetchModels(providerId, baseUrl);
    // Update the provider's model list in place
    aiProviders.update((providers) =>
      providers.map((p) => (p.id === providerId ? { ...p, models } : p))
    );
  } catch (e) {
    console.error('Failed to fetch models for', providerId, e);
  } finally {
    aiFetchingModels.set(false);
  }
}

export async function saveAIKey(provider: string, key: string) {
  await aiApi.saveKey(provider, key);
  // Only update has_key for this provider, preserving other providers' fetched models
  aiProviders.update((providers) =>
    providers.map((p) => (p.id === provider ? { ...p, has_key: true } : p))
  );
  // Fetch live models for the provider whose key was just saved
  await fetchModelsForProvider(provider);
}

export async function deleteAIKey(provider: string) {
  await aiApi.deleteKey(provider);
  // Reload all providers (resets fallback state for the deleted provider)
  await loadAIProviders();
  // Re-fetch live models for the selected provider if it's a different one
  const settings = get(aiSettings);
  if (settings.selected_provider && settings.selected_provider !== provider) {
    const providers = get(aiProviders);
    const selected = providers.find((p) => p.id === settings.selected_provider);
    if (selected?.has_key) {
      await fetchModelsForProvider(settings.selected_provider);
    }
  }
}

export async function setSelectedProvider(providerId: string | null) {
  const settings = get(aiSettings);
  const models = { ...settings.selected_models };

  // Stash current model selection for the old provider
  if (settings.selected_provider && settings.selected_model) {
    models[settings.selected_provider] = settings.selected_model;
  }

  // Restore previously saved model for the new provider (if any)
  const restored = providerId ? models[providerId] ?? null : null;

  const updated: AISettings = {
    ...settings,
    selected_provider: providerId,
    selected_model: restored,
    selected_models: models,
  };
  aiSettings.set(updated);
  await aiApi.saveSettings(updated);

  // Fetch live models if provider has a key
  if (providerId) {
    const providers = get(aiProviders);
    const provider = providers.find((p) => p.id === providerId);
    if (provider?.has_key) {
      await fetchModelsForProvider(providerId);
    }

    // If no restored model, auto-select first from the (now potentially fetched) list
    if (!restored) {
      const updatedProviders = get(aiProviders);
      const updatedProvider = updatedProviders.find((p) => p.id === providerId);
      if (updatedProvider?.models.length) {
        await setSelectedModel(updatedProvider.models[0].id);
      }
    }
  }
}

export async function setSelectedModel(modelId: string | null) {
  const settings = get(aiSettings);
  const models = { ...settings.selected_models };

  // Keep per-provider map in sync
  if (settings.selected_provider) {
    if (modelId) {
      models[settings.selected_provider] = modelId;
    } else {
      delete models[settings.selected_provider];
    }
  }

  const updated: AISettings = { ...settings, selected_model: modelId, selected_models: models };
  aiSettings.set(updated);
  await aiApi.saveSettings(updated);
}

export async function setCustomBaseUrl(providerId: string, url: string) {
  const settings = get(aiSettings);
  const urls = { ...settings.custom_base_urls };
  if (url.trim()) {
    urls[providerId] = url.trim();
  } else {
    delete urls[providerId];
  }
  const updated: AISettings = { ...settings, custom_base_urls: urls };
  aiSettings.set(updated);
  await aiApi.saveSettings(updated);
}

export async function generateCommitMessage(): Promise<GenerateResult | null> {
  const path = get(repoPath);
  const settings = get(aiSettings);

  if (!path || !settings.selected_provider || !settings.selected_model) {
    aiError.set('No AI provider or model configured. Open Settings → AI to set up.');
    return null;
  }

  const providers = get(aiProviders);
  const provider = providers.find((p) => p.id === settings.selected_provider);
  if (!provider?.has_key) {
    aiError.set(`No API key configured for ${provider?.name ?? settings.selected_provider}. Open Settings → AI to add one.`);
    return null;
  }

  const baseUrl = settings.custom_base_urls[settings.selected_provider] ?? null;

  aiGenerating.set(true);
  aiError.set(null);

  try {
    const result = await aiApi.generateCommitMessage(
      path,
      settings.selected_provider,
      settings.selected_model,
      baseUrl
    );
    return result;
  } catch (e) {
    aiError.set(String(e));
    return null;
  } finally {
    aiGenerating.set(false);
  }
}
