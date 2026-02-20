import { writable, derived, get } from 'svelte/store';
import type { AIProvider, AISettings, GenerateResult } from '$lib/api/types';
import * as aiApi from '$lib/api/ai';
import { repoPath } from '$lib/stores/repo';

// State
export const aiProviders = writable<AIProvider[]>([]);
export const aiSettings = writable<AISettings>({
  selected_provider: null,
  selected_model: null,
  custom_base_urls: {},
});
export const aiGenerating = writable(false);
export const aiError = writable<string | null>(null);

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

export async function saveAIKey(provider: string, key: string) {
  try {
    await aiApi.saveKey(provider, key);
    await loadAIProviders();
  } catch (e) {
    throw e;
  }
}

export async function deleteAIKey(provider: string) {
  try {
    await aiApi.deleteKey(provider);
    await loadAIProviders();
  } catch (e) {
    throw e;
  }
}

export async function setSelectedProvider(providerId: string | null) {
  const settings = get(aiSettings);
  const updated: AISettings = {
    ...settings,
    selected_provider: providerId,
    selected_model: null,
  };
  aiSettings.set(updated);
  await aiApi.saveSettings(updated);
}

export async function setSelectedModel(modelId: string | null) {
  const settings = get(aiSettings);
  const updated: AISettings = { ...settings, selected_model: modelId };
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
