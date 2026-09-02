import { writable, derived, get } from 'svelte/store';
import type { AIProvider, AISettings, CommitRangeSummary, GenerateResult, ReleaseNotesResult } from '$lib/api/types';
import * as aiApi from '$lib/api/ai';
import * as repoApi from '$lib/api/repo';
import { repoPath, commitGraph, error as repoError } from '$lib/stores/repo';

// State
export const aiProviders = writable<AIProvider[]>([]);
export const aiSettings = writable<AISettings>({
  selected_provider: null,
  selected_model: null,
  selected_models: {},
  custom_base_urls: {},
  max_tokens: 1500,
});
export const aiGenerating = writable(false);
export const aiError = writable<string | null>(null);
export const aiFetchingModels = writable(false);

// Release notes dialog
export interface ReleaseNotesDialogState {
  open: boolean;
  /** Exclusive start of the range (tag, branch, SHA, or any rev spec). */
  from: string;
  /** Inclusive end of the range. */
  to: string;
}
export const releaseNotesDialog = writable<ReleaseNotesDialogState>({ open: false, from: '', to: 'HEAD' });
export const releaseNotesRange = writable<CommitRangeSummary | null>(null);
export const releaseNotesRangeLoading = writable(false);
export const releaseNotesRangeError = writable<string | null>(null);
export const releaseNotesResult = writable<ReleaseNotesResult | null>(null);
export const releaseNotesGenerating = writable(false);
export const releaseNotesError = writable<string | null>(null);

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

export async function setMaxTokens(tokens: number) {
  const settings = get(aiSettings);
  const updated: AISettings = { ...settings, max_tokens: tokens };
  aiSettings.set(updated);
  await aiApi.saveSettings(updated);
}

/// Resolve the configured provider/model, or return a user-facing reason why generation can't run.
function resolveGenerationTarget():
  | { ok: true; path: string; provider: string; model: string; baseUrl: string | null; maxTokens: number }
  | { ok: false; reason: string } {
  const path = get(repoPath);
  const settings = get(aiSettings);

  if (!path || !settings.selected_provider || !settings.selected_model) {
    return { ok: false, reason: 'No AI provider or model configured. Open Settings → AI to set up.' };
  }

  const providers = get(aiProviders);
  const provider = providers.find((p) => p.id === settings.selected_provider);
  if (!provider?.has_key) {
    return {
      ok: false,
      reason: `No API key configured for ${provider?.name ?? settings.selected_provider}. Open Settings → AI to add one.`,
    };
  }

  return {
    ok: true,
    path,
    provider: settings.selected_provider,
    model: settings.selected_model,
    baseUrl: settings.custom_base_urls[settings.selected_provider] ?? null,
    maxTokens: settings.max_tokens,
  };
}

export async function generateCommitMessage(): Promise<GenerateResult | null> {
  const target = resolveGenerationTarget();
  if (!target.ok) {
    aiError.set(target.reason);
    return null;
  }

  aiGenerating.set(true);
  aiError.set(null);

  try {
    const result = await aiApi.generateCommitMessage(
      target.path,
      target.provider,
      target.model,
      target.baseUrl,
      target.maxTokens
    );
    return result;
  } catch (e) {
    repoError.set(`AI generation failed: ${String(e)}`);
    return null;
  } finally {
    aiGenerating.set(false);
  }
}

// --- Release notes ---

/// The most recent tag by graph position (tags on newer commits sort first).
function latestTagName(): string | null {
  const graph = get(commitGraph);
  if (!graph || graph.tags.length === 0) return null;
  const commitIndex = new Map(graph.commits.map((c, i) => [c.oid, i]));
  let best: { name: string; index: number } | null = null;
  for (const tag of graph.tags) {
    const index = commitIndex.get(tag.target_oid) ?? Infinity;
    if (!best || index < best.index) best = { name: tag.name, index };
  }
  return best?.name ?? null;
}

/// Open the release notes dialog. Defaults to latest tag → HEAD when no range is given.
export function openReleaseNotes(opts: { from?: string; to?: string } = {}) {
  const from = opts.from ?? latestTagName() ?? '';
  const to = opts.to ?? 'HEAD';
  releaseNotesResult.set(null);
  releaseNotesError.set(null);
  releaseNotesRange.set(null);
  releaseNotesRangeError.set(null);
  releaseNotesDialog.set({ open: true, from, to });
}

export function closeReleaseNotes() {
  releaseNotesDialog.update((d) => ({ ...d, open: false }));
}

let rangePreviewSeq = 0;

/// Resolve the range and show commit/diffstat counts before generating.
export async function previewReleaseNotesRange(from: string, to: string) {
  const path = get(repoPath);
  const seq = ++rangePreviewSeq;
  if (!path || !from.trim() || !to.trim()) {
    releaseNotesRange.set(null);
    releaseNotesRangeError.set(null);
    return;
  }
  releaseNotesRangeLoading.set(true);
  try {
    const summary = await repoApi.getCommitRange(path, from.trim(), to.trim());
    if (seq !== rangePreviewSeq) return; // superseded by a newer preview
    releaseNotesRange.set(summary);
    releaseNotesRangeError.set(null);
  } catch (e) {
    if (seq !== rangePreviewSeq) return;
    releaseNotesRange.set(null);
    releaseNotesRangeError.set(String(e));
  } finally {
    if (seq === rangePreviewSeq) releaseNotesRangeLoading.set(false);
  }
}

export async function generateReleaseNotes(from: string, to: string): Promise<ReleaseNotesResult | null> {
  const target = resolveGenerationTarget();
  if (!target.ok) {
    releaseNotesError.set(target.reason);
    return null;
  }

  releaseNotesGenerating.set(true);
  releaseNotesError.set(null);

  try {
    const result = await aiApi.generateReleaseNotes(
      target.path,
      from.trim(),
      to.trim(),
      target.provider,
      target.model,
      target.baseUrl,
      target.maxTokens
    );
    releaseNotesResult.set(result);
    releaseNotesRange.set(result.range);
    releaseNotesRangeError.set(null);
    return result;
  } catch (e) {
    releaseNotesError.set(`Release notes generation failed: ${String(e)}`);
    return null;
  } finally {
    releaseNotesGenerating.set(false);
  }
}
