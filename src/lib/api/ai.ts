import { getTransport } from '$lib/api';
import type { AIModel, AIProvider, AISettings, GenerateResult, ReleaseNotesResult } from './types';

export async function getProviders(): Promise<AIProvider[]> {
  return getTransport().invoke('ai_get_providers');
}

export async function saveKey(provider: string, key: string): Promise<void> {
  return getTransport().invoke('ai_save_key', { provider, key });
}

export async function deleteKey(provider: string): Promise<void> {
  return getTransport().invoke('ai_delete_key', { provider });
}

export async function fetchModels(
  provider: string,
  baseUrl?: string | null
): Promise<AIModel[]> {
  return getTransport().invoke('ai_fetch_models', {
    provider,
    baseUrl: baseUrl ?? null,
  });
}

export async function generateCommitMessage(
  path: string,
  provider: string,
  model: string,
  baseUrl?: string | null,
  maxTokens?: number
): Promise<GenerateResult> {
  return getTransport().invoke('ai_generate_commit_message', {
    path,
    provider,
    model,
    baseUrl: baseUrl ?? null,
    maxTokens: maxTokens ?? null,
  });
}

export async function generateReleaseNotes(
  path: string,
  from: string,
  to: string,
  provider: string,
  model: string,
  baseUrl?: string | null,
  maxTokens?: number
): Promise<ReleaseNotesResult> {
  return getTransport().invoke('ai_generate_release_notes', {
    path,
    from,
    to,
    provider,
    model,
    baseUrl: baseUrl ?? null,
    maxTokens: maxTokens ?? null,
  });
}

export async function getSettings(): Promise<AISettings> {
  return getTransport().invoke('ai_get_settings');
}

export async function saveSettings(settings: AISettings): Promise<void> {
  return getTransport().invoke('ai_save_settings', { settings });
}
