import { invoke } from '@tauri-apps/api/core';
import type { AIModel, AIProvider, AISettings, GenerateResult } from './types';

export async function getProviders(): Promise<AIProvider[]> {
  return invoke('ai_get_providers');
}

export async function saveKey(provider: string, key: string): Promise<void> {
  return invoke('ai_save_key', { provider, key });
}

export async function deleteKey(provider: string): Promise<void> {
  return invoke('ai_delete_key', { provider });
}

export async function fetchModels(
  provider: string,
  baseUrl?: string | null
): Promise<AIModel[]> {
  return invoke('ai_fetch_models', {
    provider,
    baseUrl: baseUrl ?? null,
  });
}

export async function generateCommitMessage(
  path: string,
  provider: string,
  model: string,
  baseUrl?: string | null
): Promise<GenerateResult> {
  return invoke('ai_generate_commit_message', {
    path,
    provider,
    model,
    baseUrl: baseUrl ?? null,
  });
}

export async function getSettings(): Promise<AISettings> {
  return invoke('ai_get_settings');
}

export async function saveSettings(settings: AISettings): Promise<void> {
  return invoke('ai_save_settings', { settings });
}
